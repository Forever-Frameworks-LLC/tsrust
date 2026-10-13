//! Source-body return discovery preserves scope and exception order explicitly.
use super::*;
use std::collections::VecDeque;
struct Body {
    statements:std::vec::IntoIter<Stmt>,
    // Only the root uses the caller's mutable context; nested scopes own clones.
    context:Option<Box<Context>>,
    absent:Option<Type>,
}
impl Body {
    fn child(statements:Vec<Stmt>,context:Context)->Self{Self{statements:statements.into_iter(),context:Some(Box::new(context)),absent:None}}
}
struct Branches { pending:VecDeque<(Vec<Stmt>,Context)>,absent:Option<Type> }
struct Region { source:Box<ExceptionRegion>,context:Box<Context>,cleanup:Option<Type>,body:Option<Type> }
enum Task {
    Body(Body),AfterBody(Body,bool),
    Branches(Branches),AfterBranch(Branches),
    Region(Region),AfterCleanup(Region),AfterTry(Region),AfterCatch(Region),
}
impl Analyzer {
    pub(super) fn discover_body_work(&mut self,context:&mut Context,body:&[Stmt],expected:Type)->Result<Option<Type>,Diagnostic>{
        self.discovery_work(context,Task::Body(Body{statements:body.to_vec().into_iter(),context:None,absent:None}),expected)
    }
    #[cfg(test)]
    pub(super) fn discover_region_work(&mut self,context:&Context,region:ExceptionRegion,expected:Type)->Result<Option<Type>,Diagnostic>{
        self.discovery_work(&mut context.clone(),Task::Region(Region{source:Box::new(region),context:Box::new(context.clone()),cleanup:None,body:None}),expected)
    }
    fn discovery_work(&mut self,root:&mut Context,first:Task,expected:Type)->Result<Option<Type>,Diagnostic>{
        let mut work=vec![first];let mut found=None;
        while let Some(task)=work.pop(){match task{
            Task::Body(mut frame)=>{
                let Some(mut statement)=frame.statements.next()else{found=frame.absent;continue;};
                let context=frame.context.as_deref_mut().unwrap_or(&mut *root);
                let abrupt=matches!(statement.kind,StmtKind::Try(_)) && crate::completion::definitely_abrupt(std::slice::from_ref(&statement));
                match &mut statement.kind{
                    StmtKind::Block(body)|StmtKind::While{body,..}=>{
                        let child=Body::child(std::mem::take(body),context.clone());
                        work.push(Task::AfterBody(frame,false));work.push(Task::Body(child));
                    }
                    StmtKind::Loop{body,update,..}=>{
                        let pending=VecDeque::from([(std::mem::take(body),context.clone()),(std::mem::take(update),context.clone())]);
                        work.push(Task::AfterBody(frame,false));work.push(Task::Branches(Branches{pending,absent:None}));
                    }
                    StmtKind::If{condition,then_body,else_body}=>{
                        let mut probe=context.clone();
                        if let Some((actual,_))=self.discovery_expression(&mut probe,condition,Some(Type::Boolean))?{
                            if actual==Type::Boolean{self.require(actual,Some(Type::Boolean),condition.span)?;}
                        }
                        let(then_flow,else_flow)=Self::condition_outcomes(&probe,condition);
                        let mut then_context=probe.clone();then_context.flow=then_flow;
                        let mut else_context=probe.clone();else_context.flow=else_flow;
                        let pending=VecDeque::from([(std::mem::take(then_body),then_context),(std::mem::take(else_body),else_context)]);
                        work.push(Task::AfterBody(frame,false));work.push(Task::Branches(Branches{pending,absent:None}));
                    }
                    StmtKind::ForOf{name,rebindable,iterable,body,..}=>{
                        let mut inner=context.clone();
                        if let Some((actual,_))=self.discovery_expression(&mut inner,iterable,None)? && let Some(element)=self.iterable_element(actual){
                            let id=Self::bind(&mut inner,name,element.ty(),false)?;
                            inner.bindings[id].rebindable=*rebindable;
                            if !element.ty().is_copy(){inner.bindings[id].borrowed_element=Some(iterable.span);}
                            let child=Body::child(std::mem::take(body),inner);
                            work.push(Task::AfterBody(frame,false));work.push(Task::Body(child));
                        }else{work.push(Task::Body(frame));}
                    }
                    StmtKind::Try(_)=>{
                        let StmtKind::Try(region)=std::mem::replace(&mut statement.kind,StmtKind::Block(Vec::new()))else{unreachable!()};
                        let region=Region{source:region,context:Box::new(context.clone()),cleanup:None,body:None};
                        work.push(Task::AfterBody(frame,abrupt));work.push(Task::Region(region));
                    }
                    _=>{
                        let stop=matches!(statement.kind,StmtKind::Throw(_));
                        let value=self.discover_leaf_results(context,std::slice::from_ref(&statement),expected)?;
                        if let Some(ty)=value{
                            if !self.absent_callable_type(ty){found=Some(ty);continue;}
                            frame.absent=Some(ty);
                        }
                        if stop{found=frame.absent;}else{work.push(Task::Body(frame));}
                    }
                }
            }
            Task::AfterBody(mut frame,stop)=>{
                if let Some(ty)=found{
                    if !self.absent_callable_type(ty){continue;}
                    frame.absent=Some(ty);
                }
                if stop{found=frame.absent;}else{work.push(Task::Body(frame));}
            }
            Task::Branches(mut frame)=>{
                if let Some((body,context))=frame.pending.pop_front(){work.push(Task::AfterBranch(frame));work.push(Task::Body(Body::child(body,context)));}
                else{found=frame.absent;}
            }
            Task::AfterBranch(mut frame)=>{
                if let Some(ty)=found{if !self.absent_callable_type(ty){continue;}frame.absent=Some(ty);}
                work.push(Task::Branches(frame));
            }
            Task::Region(frame)=>{
                if let Some(cleanup)=&frame.source.finally{
                    let body=Body::child(cleanup.clone(),(*frame.context).clone());
                    work.push(Task::AfterCleanup(frame));work.push(Task::Body(body));
                }else{found=None;work.push(Task::AfterCleanup(frame));}
            }
            Task::AfterCleanup(mut frame)=>{
                frame.cleanup=found;
                if frame.source.finally.as_ref().is_some_and(|body|crate::completion::definitely_abrupt(body)){continue;}
                if frame.source.catch.as_ref().is_some_and(|handler|handler.name.is_some() && handler.error_type.is_none() && handler.error_contract.is_none()){
                    // Refinement historically restarts discovery, including cleanup.
                    frame.source=Box::new(self.prepare_discovery_exception(&frame.context,&frame.source,expected,None)?);
                    frame.cleanup=None;work.push(Task::Region(frame));continue;
                }
                let body=Body::child(std::mem::take(&mut frame.source.body),(*frame.context).clone());
                work.push(Task::AfterTry(frame));work.push(Task::Body(body));
            }
            Task::AfterTry(mut frame)=>{
                frame.body=found;
                if let Some(handler)=&mut frame.source.catch && handler.error_contract.as_ref().is_none_or(|contract|contract.runtime || contract.opaque || !contract.explicit.is_empty()){
                    let mut inner=(*frame.context).clone();
                    if let Some(name)=&handler.name{
                        let ty=handler.error_type.ok_or_else(||Diagnostic::new("error-effect","Catch result discovery requires a resolved payload type.",name.span))?;
                        inner.names.remove(&name.text);Self::bind(&mut inner,name,ty,false)?;
                    }
                    let body=Body::child(std::mem::take(&mut handler.body),inner);
                    work.push(Task::AfterCatch(frame));work.push(Task::Body(body));
                }else{found=None;work.push(Task::AfterCatch(frame));}
            }
            Task::AfterCatch(frame)=>{
                let candidates=[frame.body,found,frame.cleanup];
                found=candidates.into_iter().flatten().find(|ty|!self.absent_callable_type(*ty)).or_else(||candidates.into_iter().flatten().next());
            }
        }}
        Ok(found)
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    fn nest(mut body:Vec<Stmt>,count:usize)->Vec<Stmt>{
        for _ in 0..count{body=vec![Stmt{labels:Vec::new(),control_target:None,kind:StmtKind::Block(body),span:Span::default()}];}
        body
    }
    #[test]
    fn deep_discovery_scopes_use_work_frames_on_normal_stack(){
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut parsed=crate::syntax::parse("function main():int{return 7;}").unwrap();
        let body=nest(std::mem::take(&mut parsed.functions[0].body),16384);
        let mut context=Context::new("main".into());
        assert_eq!(analyzer.discover_body(&mut context,&body,Type::Int).unwrap(),Some(Type::Int));
        assert_eq!(analyzer.depth,0);assert!(context.names.is_empty());drop(body);
    }
    #[test]
    fn discovery_error_keeps_root_mutations_and_restores_return_permission(){
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut root=crate::syntax::parse("function main(){const outer=1;}").unwrap();
        let mut inner=crate::syntax::parse("function main():int{const nested=2;return missing;}").unwrap();
        let missing=match &inner.functions[0].body[1].kind{StmtKind::Return(Some(value))=>value.span,_=>unreachable!()};
        let mut body=std::mem::take(&mut root.functions[0].body);
        body.extend(nest(std::mem::take(&mut inner.functions[0].body),4096));
        let mut context=Context::new("main".into());
        assert_eq!(analyzer.discover_body(&mut context,&body,Type::Int).unwrap_err().span,missing);
        assert!(context.names.contains_key("outer"));assert!(!context.names.contains_key("nested"));
        assert!(!context.returning_closure);assert_eq!(analyzer.depth,0);drop(body);
    }
}
