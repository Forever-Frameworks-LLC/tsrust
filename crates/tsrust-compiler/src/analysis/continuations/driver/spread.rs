//! One owned spread plan suspends body contracts, sources and expanded variants.
use super::*;
use crate::analysis::spread_call_work::{Work,Stage,Segment,Variant};
enum Phase { Start, Generic, Access, Fresh, Contract, ContractBody, Stage, Callee, Receiver(Mode), Segments, Segment(Segment), Variants, Variant{selection:SpreadVariantSelection,count:usize}, Done }
pub(super) struct SpreadTask<'a> {
    expression:Box<Expr>,pub context:ContextId,loans:ActiveLoans<'a>,expected:Option<Type>,
    work:Option<Work>,phase:Phase,next_segment:usize,
}
impl<'a> SpreadTask<'a> {
    pub fn new(expression:Box<Expr>,context:ContextId,loans:ActiveLoans<'a>,expected:Option<Type>)->Self {
        Self{expression,context,loans,expected,work:None,phase:Phase::Start,next_segment:0}
    }
}
fn take(value:&mut Box<Expr>)->Box<Expr>{let span=value.span;std::mem::replace(value,Box::new(Expr::new(ExprKind::Undefined,span)))}
pub(super) fn step<'a>(analyzer:&mut Analyzer,context:&mut Context,mut task:Box<SpreadTask<'a>>,mut completion:Option<Completion<'a>>)->Step<'a>{
    let action=(||->Result<Option<Task<'a>>,Diagnostic>{
        loop {
            match std::mem::replace(&mut task.phase,Phase::Start){
                Phase::Start=>{
                    let work=analyzer.start_spread_work(context,&task.expression,&task.loans)?;
                    let prototype=analyzer.spread_work_prototype(context,&work);
                    task.work=Some(work);
                    if let Some(prototype)=prototype {
                        task.phase=Phase::Generic;
                        return Ok(Some(Task::Generic(Box::new(generic::GenericTask::new(prototype,std::rc::Rc::new(context.clone()),task.expected)))));
                    }
                    task.phase=Phase::Access;
                }
                Phase::Generic=>{
                    let Completion::Generic(prototype,result)=completion.take().expect("spread prototype") else{unreachable!()};result?;
                    let ExprKind::Call(callee,_)=prototype.into_kind() else{unreachable!()};
                    task.work.as_mut().unwrap().callee=callee;task.phase=Phase::Access;
                }
                Phase::Access=>{
                    let work=task.work.as_mut().unwrap();
                    if analyzer.begin_spread_access(context,work)? {
                        let child=Task::Expression(ExpressionTask::new(Expression::Child(take(&mut work.callee)),task.context,Mode::Consume,task.loans.clone(),None));
                        task.phase=Phase::Fresh;return Ok(Some(child));
                    }
                    task.phase=Phase::Contract;
                }
                phase @ (Phase::Fresh|Phase::Callee)=>{
                    let fresh=matches!(phase,Phase::Fresh);
                    let work=task.work.as_mut().unwrap();
                    let Completion::Expression(value,result)=completion.take().expect("spread callee") else{unreachable!()};
                    work.callee=value.into_child();let ty=result?;
                    if fresh{task.phase=Phase::Contract;}else{analyzer.finish_spread_callee(context,work,ty)?;task.phase=Phase::Segments;}
                }
                Phase::Contract=>{
                    let work=task.work.as_ref().unwrap();
                    if let Some(index)=analyzer.spread_contract_request(context,work)? && !analyzer.function_request_ready(index)? {
                        task.phase=Phase::ContractBody;
                        return Ok(Some(Task::Function(Box::new(functions::FunctionTask::new(index)))));
                    }
                    analyzer.prepare_spread_contracts(context,task.work.as_mut().unwrap())?;task.phase=Phase::Stage;
                }
                Phase::ContractBody=>{
                    let Completion::Function(result)=completion.take().expect("spread target body") else{unreachable!()};result?;
                    analyzer.prepare_spread_contracts(context,task.work.as_mut().unwrap())?;task.phase=Phase::Stage;
                }
                Phase::Stage=>{
                    let work=task.work.as_mut().unwrap();
                    match analyzer.prepare_spread_stage(context,work)? {
                        Stage::Ready=>task.phase=Phase::Segments,
                        Stage::Callee{mode,expected}=>{
                            let child=Task::Expression(ExpressionTask::new(Expression::Child(take(&mut work.callee)),task.context,mode,ActiveLoans::Shared(work.active.clone()),expected));
                            task.phase=Phase::Callee;return Ok(Some(child));
                        }
                        Stage::Receiver(mode)=>{
                            let value=take(work.receiver());
                            let child=Task::Expression(ExpressionTask::new(Expression::Child(value),task.context,mode,ActiveLoans::Shared(work.active.clone()),None));
                            task.phase=Phase::Receiver(mode);return Ok(Some(child));
                        }
                    }
                }
                Phase::Receiver(mode)=>{
                    let Completion::Expression(value,result)=completion.take().expect("spread receiver") else{unreachable!()};
                    let work=task.work.as_mut().unwrap();*work.receiver()=value.into_child();
                    analyzer.finish_spread_receiver(context,work,result?,mode)?;task.phase=Phase::Segments;
                }
                Phase::Segments=>{
                    let work=task.work.as_mut().unwrap();
                    if task.next_segment<work.segments.len(){
                        let frame=analyzer.prepare_spread_segment(context,work,task.next_segment)?;
                        task.next_segment+=1;
                        let child=Task::Expression(ExpressionTask::new(Expression::Child(take(&mut work.segments[frame.index].value)),task.context,frame.mode,ActiveLoans::Shared(work.active.clone()),frame.expected));
                        task.phase=Phase::Segment(frame);return Ok(Some(child));
                    }
                    analyzer.prepare_spread_variants(work)?;task.phase=Phase::Variants;
                }
                Phase::Segment(frame)=>{
                    let Completion::Expression(value,result)=completion.take().expect("spread source") else{unreachable!()};
                    let work=task.work.as_mut().unwrap();work.segments[frame.index].value=value.into_child();
                    analyzer.finish_spread_segment(work,frame,result?)?;task.phase=Phase::Segments;
                }
                Phase::Variants=>{
                    if let Some(variant)=analyzer.next_spread_variant(context,task.work.as_mut().unwrap()){
                        task.phase=Phase::Variant{selection:variant.selection,count:variant.count};
                        return Ok(Some(Task::Expression(ExpressionTask::new(Expression::Child(variant.call),task.context,Mode::Consume,task.loans.clone(),task.expected))));
                    }
                    task.phase=Phase::Done;
                }
                Phase::Variant{selection,count}=>{
                    let Completion::Expression(value,result)=completion.take().expect("spread expanded variant") else{unreachable!()};
                    analyzer.finish_spread_variant(task.work.as_mut().unwrap(),Variant{call:value.into_child(),selection,count},result?)?;task.phase=Phase::Variants;
                }
                Phase::Done=>{analyzer.finish_spread_work(context,&mut task.expression,task.work.take().unwrap());return Ok(None);}
            }
        }
    })();
    match action{
        Ok(Some(child))=>Step::Push(Task::Spread(task),child),
        Ok(None)=>{let ty=task.expression.ty.expect("completed spread type");Step::Complete(Completion::Spread(task.expression,Ok(ty)))},
        Err(error)=>Step::Complete(Completion::Spread(task.expression,Err(error))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn check_unbounded(source:&str)->crate::Analysis {
        let mut program=crate::syntax::parse_unbounded(source).unwrap();
        crate::callables::module_state::lower(&mut program).unwrap();
        crate::local_functions::resolve(&mut program).unwrap();
        crate::Analysis{program:crate::analysis::analyze_recursive(program).unwrap()}
    }
    #[test]
    fn spread_operands_and_annotated_body_contracts_share_the_driver(){
        let count=512;
        let source=format!("function identity(value:int):int{{return value;}}function main(){{console.log({}7{});}}","identity(...[".repeat(count),"])".repeat(count));
        let checked=check_unbounded(&source);drop(checked.emit_rust());drop(checked);
        let mut source="function step0<T>(value:T):T{return value;}".to_owned();
        for index in 1..128 {source.push_str(&format!("function step{index}<T>(value:T):T{{return step{}(...[value]);}}",index-1));}
        source.push_str("function main(){console.log(step127(7));}");
        let checked=crate::check(&source).unwrap();drop(checked.emit_rust());drop(checked);
    }
    #[test]
    fn spread_failures_preserve_original_call_tree_and_leaf_location(){
        for source in [
            "function pair(left:int,right:int):int{return left;}function main(){pair(...[1,true]);}",
            "function identity(value:int):int{return value;}function main(){identity(...[identity(...[missing])]);}",
            "function fail(value:int):int{return missing;}function main(){fail(...[7]);}",
            "function main(){((value:int):int=>missing)(...[7]);}",
        ] {
            let source=crate::format(source).unwrap();
            let mut program=crate::syntax::parse(&source).unwrap();
            let statement=program.functions.last_mut().unwrap().body.remove(0);
            let StmtKind::Expr(mut value)=statement.into_kind() else{panic!("call expression")};
            let before=format!("{:?}",value.kind);
            let mut analyzer=Analyzer::new(program).unwrap();
            let mut context=Context::new("main".into());
            let error=analyzer.expression(&mut context,&mut value,Mode::Consume,&Loans::new(),None).expect_err("invalid spread child");
            let expected=if source.contains("true") {"true"}else{"missing"};
            assert_eq!(&source[error.span.start..error.span.end],expected);
            assert!(matches!(value.kind,ExprKind::Call(..)));
            assert_eq!(format!("{:?}",value.kind),before);
            assert_eq!(analyzer.depth,0);
        }
    }
}
