//! Object spread preserves sequential inference and literal policy across child work.
use super::*;
use crate::analysis::object_spread::{Work,Input};
pub(super) struct ObjectSpreadTask<'a>{
    expression:Box<Expr>,pub context:ContextId,loans:ActiveLoans<'a>,expected:Option<Type>,work:Option<Work>,pending:Option<Input>,
}
impl<'a> ObjectSpreadTask<'a>{
    pub fn new(expression:Box<Expr>,context:ContextId,loans:ActiveLoans<'a>,expected:Option<Type>)->Self{
        Self{expression,context,loans,expected,work:None,pending:None}
    }
}
pub(super) fn step<'a>(analyzer:&mut Analyzer,context:&mut Context,mut task:Box<ObjectSpreadTask<'a>>,completion:Option<Completion<'a>>)->Step<'a>{
    let next=(||->Result<Option<Task<'a>>,Diagnostic>{
        if let Some(mut input)=task.pending.take(){
            let Completion::Expression(value,result)=completion.expect("object spread source") else{unreachable!()};
            input.value=value.into_child();
            analyzer.finish_object_spread_value(task.work.as_mut().unwrap(),input,result)?;
        }else{task.work=Some(analyzer.start_object_spread_work(&mut task.expression,task.expected)?);}
        if let Some(mut input)=analyzer.next_object_spread_value(context,task.work.as_mut().unwrap(),task.expected){
            let span=input.value.span;
            let value=std::mem::replace(&mut input.value,Box::new(Expr::new(ExprKind::Undefined,span)));
            let child=Task::Expression(ExpressionTask::new(Expression::Child(value),task.context,if input.spread && !input.consume{Mode::Read}else{Mode::Consume},task.loans.clone(),input.expected));
            task.pending=Some(input);return Ok(Some(child));
        }
        analyzer.finish_object_spread_work(&mut task.expression,task.work.take().unwrap(),task.expected).map(|ty|task.expression.ty=Some(ty))?;Ok(None)
    })();
    match next{
        Ok(Some(child))=>Step::Push(Task::ObjectSpread(task),child),
        result=>{let result=result.map(|_|task.expression.ty.expect("completed object spread"));Step::Complete(Completion::Spread(task.expression,result))},
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    fn pipeline(source:&str){
        let mut program=crate::syntax::parse_unbounded(source).unwrap();
        crate::callables::module_state::lower(&mut program).unwrap();crate::local_functions::resolve(&mut program).unwrap();
        let checked=crate::Analysis{program:crate::analysis::analyze_recursive(program).unwrap()};
        drop(checked.emit_rust());drop(checked);
    }
    #[test]
    fn object_spread_sources_share_checker_and_emitter_stacks(){
        let count=512;
        let value=format!("{}{{value:7}}{}","{...".repeat(count),"}".repeat(count));
        pipeline(&format!("function main(){{console.log(({value}).value);}}"));
    }
    #[test]
    fn overwrite_proof_traverses_all_finite_spread_sources(){
        let value=format!("{}{{value:7}}{}","{...".repeat(64),"}".repeat(64));
        pipeline(&format!("type Item={{value:int;}};function main(){{const item:Item={{value:'overwritten',...{value}}};console.log(item.value);}}"));
    }
    #[test]
    fn failed_spread_restores_literal_policy_and_diagnostic_span(){
        for prior in [false,true]{
            let source="function main(){const item={...{...{value:missing}}};}";
            let mut program=crate::syntax::parse(source).unwrap();
            let StmtKind::Let{mut value,..}=program.functions[0].body.remove(0).into_kind()else{panic!("record source")};
            let mut analyzer=Analyzer::new(program).unwrap();analyzer.builtin_literals=prior;
            let mut context=Context::new("main".into());
            let error=analyzer.expression(&mut context,&mut value,Mode::Consume,&Loans::new(),None).expect_err("missing leaf");
            assert_eq!(&source[error.span.start..error.span.end],"missing");
            assert_eq!(analyzer.builtin_literals,prior);
            assert!(matches!(&value.kind,ExprKind::Record(fields) if fields.is_empty()));
        }
    }
}
