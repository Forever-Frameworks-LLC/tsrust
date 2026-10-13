//! Closure defaults and parameter patterns use the enclosing checker work stack.
use super::*;
use super::super::closure::{Parameters,ParameterWork};

fn abort<'a>(analyzer:&mut Analyzer,contexts:&mut Contexts<'_>,task:ExpressionTask<'a>,child:ContextId,error:Diagnostic)->Step<'a>{
    contexts.take(child);
    finish_expression(analyzer,contexts.get(task.context),task,Err(error))
}
pub(super) fn resume_default<'a>(analyzer:&mut Analyzer,contexts:&mut Contexts<'_>,mut task:ExpressionTask<'a>,mut frame:Box<Parameters>,child:ContextId,mut work:Box<ParameterWork>,completion:Completion<'a>)->Step<'a>{
    let Completion::Expression(value,actual)=completion else{unreachable!("closure default result")};
    work.value=Some(value.into_child());
    let ExprKind::Closure{params,..}=&task.expression.value().kind else{unreachable!()};
    let position=frame.next;
    let checked=actual.and_then(|actual|analyzer.finish_closure_parameter(contexts.get(child),&mut frame,&params[position],*work,Some(actual)));
    match checked{Ok(())=>next(analyzer,contexts,task,frame,child,true),Err(error)=>abort(analyzer,contexts,task,child,error)}
}
pub(super) fn resume_pattern<'a>(analyzer:&mut Analyzer,contexts:&mut Contexts<'_>,mut task:ExpressionTask<'a>,frame:Box<Parameters>,child:ContextId,start:usize,completion:Completion<'a>)->Step<'a>{
    let Completion::Block(checked,result)=completion else{unreachable!("closure pattern result")};
    let ExprKind::Closure{body,..}=&mut task.expression.value().kind else{unreachable!()};
    body.splice(start..start,checked.into_child());
    match result{Ok(_)=>next(analyzer,contexts,task,frame,child,false),Err(error)=>abort(analyzer,contexts,task,child,error)}
}
pub(super) fn next<'a>(analyzer:&mut Analyzer,contexts:&mut Contexts<'_>,mut task:ExpressionTask<'a>,mut frame:Box<Parameters>,child:ContextId,mut pattern_pending:bool)->Step<'a>{
    loop{
        let ExprKind::Closure{params,body,..}=&mut task.expression.value().kind else{unreachable!("closure parameter task")};
        if pattern_pending{
            let param=&params[frame.next-1];
            if let Some(range)=analyzer.prepare_parameter_pattern(contexts.get(child),&param.name,body,param.access.as_ref().is_some_and(|access|access.text=="consume")){
                let start=range.start;let pattern=body.drain(range).collect();
                task.state=ExpressionState::ClosurePattern{frame,child,start};
                return Step::Push(Task::Expression(task),Task::Block(BlockTask{body:Body::Child{body:pattern,start:0},context:child,frame:None,pending:None,result_type:Type::Void,keep_names:true,label_span:None}));
            }
        }
        if frame.next==params.len(){
            let frame=match analyzer.complete_closure_parameters(frame,contexts.get(child),params,body){Ok(frame)=>frame,Err(error)=>return abort(analyzer,contexts,task,child,error)};
            if let Err(error)=analyzer.enter_source_callable_body(contexts.get(child)){
                let child_context=contexts.take(child);
                let result=analyzer.finish_closure(contexts.get(task.context),frame,child_context,body,&task.loans,Err(error));
                return finish_expression(analyzer,contexts.get(task.context),task,result);
            }
            let exceptions=has_exception_syntax(Analyzer::uninitialized_body(body));
            let start=body.len()-Analyzer::uninitialized_body(body).len();
            let body=Body::Child{body:std::mem::take(body),start};let result_type=frame.result;
            task.state=ExpressionState::Closure{frame,child};
            let work=if exceptions{Task::Callable(Box::new(CallableTask{body:Some(body),context:child,result_type,boundary:None}))}
                else{Task::Block(BlockTask{body,context:child,frame:None,pending:None,result_type,keep_names:false,label_span:None})};
            return Step::Push(Task::Expression(task),work);
        }
        let position=frame.next;
        let mut work=match analyzer.prepare_closure_default(contexts.get(child),&frame,&params[position]){Ok(work)=>work,Err(error)=>return abort(analyzer,contexts,task,child,error)};
        if let Some(value)=work.value.take(){
            let expression=ExpressionTask::new(Expression::Child(value),child,work.mode,ActiveLoans::Shared(std::rc::Rc::new(Loans::new())),work.expected);
            task.state=ExpressionState::ClosureDefault{frame,child,work:Box::new(work)};
            return Step::Push(Task::Expression(task),Task::Expression(expression));
        }
        if let Err(error)=analyzer.finish_closure_parameter(contexts.get(child),&mut frame,&params[position],work,None){return abort(analyzer,contexts,task,child,error);}
        pattern_pending=true;
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    fn chain(layers:usize,fail:bool)->Expr{
        typed_chain(layers,fail,"int")
    }
    pub(super) fn typed_chain(layers:usize,fail:bool,ty:&str)->Expr{
        let mut parsed=crate::syntax::parse(&format!("function main(){{const f=(value:{ty}=1):{ty}=>value;}}")).unwrap();
        let StmtKind::Let{value,..}=&mut parsed.functions[0].body[0].kind else{unreachable!()};
        let template=value.clone();let mut current=template.clone();
        let ExprKind::Closure{id,params,..}=&mut current.kind else{unreachable!()};*id=0;
        if fail{
            let ParameterInitializer::Default{value,..}=&mut params[0].initializer else{unreachable!()};
            *value=Box::new(Expr::new(ExprKind::Name("missing".into()),Span::new(100,107)));
        }
        for index in 1..=layers{
            let mut next=template.clone();
            let ExprKind::Closure{id,params,..}=&mut next.kind else{unreachable!()};*id=index;
            let ParameterInitializer::Default{value,..}=&mut params[0].initializer else{unreachable!()};
            *value=Box::new(Expr::new(ExprKind::Call(Box::new(current),Vec::new()),Span::default()));
            current=next;
        }
        current
    }
    #[test]
    fn nested_closure_defaults_share_the_expression_work_stack(){
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());let mut value=chain(256,false);
        let ty=analyzer.expression(&mut context,&mut value,Mode::Consume,&Loans::new(),None).unwrap();
        assert!(matches!(ty,Type::Closure(_)));assert_eq!(analyzer.closures.len(),257);assert!(context.names.is_empty());assert_eq!(analyzer.depth,0);
        drop(value);
    }
    #[test]
    fn nested_default_error_releases_child_contexts_and_preserves_source(){
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());let mut value=chain(256,true);
        let error=analyzer.expression(&mut context,&mut value,Mode::Consume,&Loans::new(),None).unwrap_err();
        assert_eq!(error.span,Span::new(100,107));assert!(context.names.is_empty());assert!(context.bindings.is_empty());
        assert!(matches!(value.kind,ExprKind::Closure{..}));assert!(analyzer.closures.is_empty());assert_eq!(analyzer.depth,0);drop(value);
    }
}

#[cfg(test)]
mod contextual_tests{
    use super::*;
    #[test]
    fn nullable_defaults_and_closure_bodies_share_the_driver(){
        for fail in [false,true]{
            let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
            let mut context=Context::new("main".into());
            let mut value=super::tests::typed_chain(256,fail,"int|undefined");
            let checked=analyzer.expression(&mut context,&mut value,Mode::Consume,&Loans::new(),None);
            if fail{assert_eq!(checked.unwrap_err().span,Span::new(100,107));}
            else{assert!(matches!(checked.unwrap(),Type::Closure(_)));assert_eq!(analyzer.closures.len(),257);}
            assert!(context.mutation_place.is_none());assert!(context.suppressed_flow.is_empty());
            assert!(context.keep_flow.is_empty());assert!(context.names.is_empty());assert_eq!(analyzer.depth,0);
            drop(value);
        }
    }
}
