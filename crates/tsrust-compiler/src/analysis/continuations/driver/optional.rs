//! Optional branches and their receiver expressions share the checker scheduler.
use super::*;
use super::super::optional::{Base,Branch};
pub(super) struct OptionalTask<'a>{
    pub(super) expression:Expression<'a>,pub(super) context:ContextId,
    pub(super) mode:Mode,pub(super) loans:ActiveLoans<'a>,pub(super) expected:Option<Type>,
    body:Option<Box<Expr>>,parents:Vec<Branch>,pending:Option<Pending>,
}
enum Pending{Base(Base),Body}
impl<'a> OptionalTask<'a>{
    pub(super) fn new(expression:Expression<'a>,context:ContextId,mode:Mode,loans:ActiveLoans<'a>,expected:Option<Type>)->Self{
        Self{expression,context,mode,loans,expected,body:None,parents:Vec::new(),pending:None}
    }
}
fn finish<'a>(analyzer:&mut Analyzer,context:&mut Context,mut task:Box<OptionalTask<'a>>,mut checked:Result<Type,Diagnostic>)->Step<'a>{
    let mut body=task.body.take().expect("optional body");
    while let Some(frame)=task.parents.pop(){
        checked=match checked{
            Ok(ty)=>analyzer.finish_optional_branch(context,&mut body,frame,ty),
            Err(error)=>{analyzer.abort_optional_branch(context,frame);Err(error)},
        };
    }
    let result=checked.map(|ty|{
        let expr=task.expression.value();let span=expr.span;
        expr.kind=body.take_kind();expr.ty=Some(ty);expr.binding=body.binding;expr.target=body.target;
        if !matches!(expr.kind,ExprKind::Optional{..}){
            let mut value=Expr::checked(std::mem::replace(&mut expr.kind,ExprKind::Null),span,Some(ty));
            value.binding=expr.binding;value.target=expr.target;
            expr.kind=ExprKind::Unary{op:UnaryOp::OptionalSelected,operator:span,value:Box::new(value)};
        }
        ty
    });
    Step::Complete(Completion::Expression(task.expression,result))
}
pub(super) fn step<'a>(analyzer:&mut Analyzer,context:&mut Context,mut task:Box<OptionalTask<'a>>,completion:Option<Completion<'a>>)->Step<'a>{
    if let Some(pending)=task.pending.take(){
        let Completion::Expression(value,checked)=completion.expect("optional child completed") else{unreachable!("optional child result")};
        match pending{
            Pending::Body=>{task.body=Some(value.into_child());return finish(analyzer,context,task,checked);},
            Pending::Base(mut frame)=>{
                frame.value=*value.into_child();
                let branch=analyzer.prepare_optional_branch(context,task.body.as_mut().expect("optional body"),frame,task.expected,checked);
                match branch{
                    Ok(frame)=>{task.parents.push(frame);task.expected=task.expected.map(|ty|analyzer.unions.payload(ty).unwrap_or(ty));},
                    Err(error)=>return finish(analyzer,context,task,Err(error)),
                }
            },
        }
    }else{
        let expr=task.expression.value();let span=expr.span;
        let mut old=std::mem::replace(expr,Expr::checked(ExprKind::Null,span,None));
        let ExprKind::OptionalChain(mut body)=old.take_kind() else{unreachable!("optional chain task")};
        if let Err(error)=analyzer.prepare_generic_call(context,&mut body,task.expected){task.body=Some(body);return finish(analyzer,context,task,Err(error));}
        analyzer.optional_static(context,&mut body);analyzer.optional_present_storage(context,&mut body);
        task.body=Some(body);
    }
    let body=task.body.as_mut().expect("optional body");
    let child=if let Some(mut frame)=analyzer.prepare_optional_base(context,body){
        let span=frame.value.span;
        let value=Box::new(std::mem::replace(&mut frame.value,Expr::checked(ExprKind::Null,span,None)));
        let child=ExpressionTask::new(Expression::Child(value),task.context,frame.mode,task.loans.clone(),None);
        task.pending=Some(Pending::Base(frame));child
    }else{
        let value=task.body.take().expect("optional terminal body");task.pending=Some(Pending::Body);
        ExpressionTask::new(Expression::Child(value),task.context,task.mode,task.loans.clone(),task.expected)
    };
    Step::Push(Task::Optional(task),Task::Expression(child))
}

#[cfg(test)]
mod tests{
    use super::*;
    fn chain(layers:usize,fail:bool)->Expr{
        let mut body=Expr::new(ExprKind::Integer("1".into()),Span::new(0,1));
        for index in 0..layers{
            let operator=Span::new(index+1,index+2);
            body=Expr::new(ExprKind::Unary{op:UnaryOp::OptionalBoundary,operator,value:Box::new(body)},Span::new(0,index+2));
        }
        if fail{body=Expr::new(ExprKind::Field(Box::new(body),"missing".into()),Span::new(90000,90007));}
        let span=body.span;Expr::new(ExprKind::OptionalChain(Box::new(body)),span)
    }
    #[test]
    fn optional_receiver_spines_and_branch_completion_use_normal_stacks(){
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());let mut value=chain(4096,false);
        assert_eq!(check_optional(&mut analyzer,&mut context,&mut value,Mode::Copy,&Loans::new(),None).unwrap(),Type::Int);
        assert!(context.suppressed_flow.is_empty());assert!(context.selected_access.is_empty());
        let mut cursor=&value;let mut count=0;
        while let ExprKind::Optional{body,..}=&cursor.kind{count+=1;cursor=body;}
        assert_eq!(count,4096);drop(value);
    }
    #[test]
    fn optional_failure_unwinds_all_suspended_selection_scopes(){
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());let mut value=chain(4096,true);
        let error=check_optional(&mut analyzer,&mut context,&mut value,Mode::Copy,&Loans::new(),None).unwrap_err();
        assert_eq!(error.span,Span::new(90000,90007));
        assert!(context.suppressed_flow.is_empty());assert!(context.selected_access.is_empty());
        assert!(context.capture_loans.is_empty());assert!(context.keep_flow.is_empty());assert_eq!(analyzer.depth,0);drop(value);
    }
}
