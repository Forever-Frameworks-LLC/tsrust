//! Array literal and Array.of spreads use the shared expression scheduler.
use super::*;
use crate::analysis::array_spread_work::{Work,Input};
pub(super) struct ArraySpreadTask<'a>{
    expression:Box<Expr>,pub context:ContextId,loans:ActiveLoans<'a>,expected:Option<Type>,work:Option<Work>,pending:Option<Input>,
}
impl<'a> ArraySpreadTask<'a>{
    pub fn new(expression:Box<Expr>,context:ContextId,loans:ActiveLoans<'a>,expected:Option<Type>)->Self{
        Self{expression,context,loans,expected,work:None,pending:None}
    }
}
pub(super) fn step<'a>(analyzer:&mut Analyzer,context:&mut Context,mut task:Box<ArraySpreadTask<'a>>,completion:Option<Completion<'a>>)->Step<'a>{
    let next=(||->Result<Option<Task<'a>>,Diagnostic>{
        if let Some(mut input)=task.pending.take(){
            let Completion::Expression(value,result)=completion.expect("array spread source") else{unreachable!()};
            input.value=value.into_child();
            analyzer.finish_array_spread_value(task.work.as_mut().unwrap(),input,result?)?;
        }else{
            task.work=Some(analyzer.start_array_spread_work(context,&task.expression,task.expected)?);
        }
        let work=task.work.as_mut().unwrap();
        if let Some(mut input)=analyzer.next_array_spread_value(context,&task.expression,work)?{
            let span=input.value.span;
            let value=std::mem::replace(&mut input.value,Box::new(Expr::new(ExprKind::Undefined,span)));
            let child=Task::Expression(ExpressionTask::new(Expression::Child(value),task.context,input.mode,task.loans.clone(),input.expected));
            task.pending=Some(input);return Ok(Some(child));
        }
        analyzer.finish_array_spread_work(context,&mut task.expression,task.work.take().unwrap(),task.expected)?;
        Ok(None)
    })();
    match next{
        Ok(Some(child))=>Step::Push(Task::ArraySpread(task),child),
        result=>{let result=result.map(|_|task.expression.ty.expect("completed array spread"));Step::Complete(Completion::Spread(task.expression,result))},
    }
}
#[cfg(test)]
mod tests{
    #[test]
    fn array_literal_and_array_of_spread_sources_share_the_driver(){
        for literal in [true,false]{
            let count=512;
            let value=if literal{format!("{}[7]{}","[...".repeat(count),"]".repeat(count))}else{format!("{}[7]{}","Array.of(...".repeat(count),")".repeat(count))};
            let source=format!("function main(){{console.log(({value}).length);}}");
            let mut program=crate::syntax::parse_unbounded(&source).unwrap();
            crate::callables::module_state::lower(&mut program).unwrap();crate::local_functions::resolve(&mut program).unwrap();
            let checked=crate::Analysis{program:crate::analysis::analyze_recursive(program).unwrap()};
            drop(checked.emit_rust());drop(checked);
        }
    }
    #[test]
    fn dense_array_of_keeps_empty_and_inferred_construction(){
        let source="function main(){const empty=Array.of<int>();const values=Array.of(7,9);console.log(empty.length,values.length,values[0]);}";
        let checked=crate::check(source).unwrap();drop(checked.emit_rust());drop(checked);
    }
    #[test]
    fn array_spread_failures_keep_original_constructor_and_leaf_span(){
        use super::*;
        for expression in ["[...[...missing]]","Array.of(...Array.of(...missing))","[1,...[true]]"]{
            let source=crate::format(&format!("function main(){{const value={expression};}}")).unwrap();
            let mut program=crate::syntax::parse(&source).unwrap();
            let StmtKind::Let{mut value,..}=program.functions[0].body.remove(0).into_kind() else{panic!("constructor source")};
            let before=format!("{:?}",value.kind);
            let mut analyzer=Analyzer::new(program).unwrap();let mut context=Context::new("main".into());
            let error=analyzer.expression(&mut context,&mut value,Mode::Consume,&Loans::new(),None).expect_err("invalid constructor child");
            let expected=if source.contains("true"){ "true" }else{ "missing" };
            assert_eq!(&source[error.span.start..error.span.end],expected);
            assert_eq!(format!("{:?}",value.kind),before);
        }
    }
}
