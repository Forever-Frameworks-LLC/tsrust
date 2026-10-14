//! Set and Map constructor sources run on the shared expression scheduler.
use super::*;
use crate::analysis::collection_constructors::Work;
pub(super) struct CollectionConstructorTask<'a>{
    expression:Box<Expr>,pub context:ContextId,loans:ActiveLoans<'a>,expected:Option<Type>,work:Option<Work>,
}
impl<'a> CollectionConstructorTask<'a>{
    pub fn new(expression:Box<Expr>,context:ContextId,loans:ActiveLoans<'a>,expected:Option<Type>)->Self{
        Self{expression,context,loans,expected,work:None}
    }
}
pub(super) fn step<'a>(analyzer:&mut Analyzer,context:&mut Context,mut task:Box<CollectionConstructorTask<'a>>,completion:Option<Completion<'a>>)->Step<'a>{
    let result=if let Some(work)=task.work.take(){
        let Completion::Expression(value,result)=completion.expect("collection constructor source") else{unreachable!()};
        analyzer.finish_collection_constructor(&mut task.expression,work,Some((value.into_child(),result)))
    }else{
        match analyzer.start_collection_constructor(context,&mut task.expression,task.expected){
            Ok((work,Some(input)))=>{
                let child=Task::Expression(ExpressionTask::new(Expression::Child(input.value),task.context,Mode::Read,task.loans.clone(),input.expected));
                task.work=Some(work);return Step::Push(Task::CollectionConstructor(task),child);
            },
            Ok((work,None))=>analyzer.finish_collection_constructor(&mut task.expression,work,None),
            Err(error)=>Err(error),
        }
    };
    Step::Complete(Completion::Spread(task.expression,result))
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
    fn nested_set_and_map_sources_share_checker_and_emitter_stacks(){
        let count=512;
        for (open,leaf) in [("new Set<int>(","[7]"),("new Map<int,int>(","[[7,9]]"),("new Set<int>([...","[7]"),("new Map<int,int>([...","[[7,9]]")]{
            let close=if open.ends_with("...") {"])"} else {")"};
            pipeline(&format!("function main(){{console.log(({}{leaf}{}).size);}}",open.repeat(count),close.repeat(count)));
        }
    }
    #[test]
    fn failed_constructor_source_restores_child_and_diagnostic_span(){
        for (source,leaf) in [("function main(){const values=new Set(new Set(missing));}","missing"),("function main(){const values=new Map(new Map(absent));}","absent")]{
            let mut program=crate::syntax::parse(source).unwrap();
            crate::callables::module_state::lower(&mut program).unwrap();crate::local_functions::resolve(&mut program).unwrap();
            let StmtKind::Let{mut value,..}=program.functions[0].body.remove(0).into_kind()else{panic!("constructor source")};
            let mut analyzer=Analyzer::new(program).unwrap();
            let error=analyzer.expression(&mut Context::new("main".into()),&mut value,Mode::Consume,&Loans::new(),None).expect_err("missing leaf");
            assert_eq!(&source[error.span.start..error.span.end],leaf,"{}",error.message);
            let (ExprKind::SetConstructor(outer,_)|ExprKind::MapConstructor(outer,_))=&value.kind else{panic!("constructor restored")};
            let (ExprKind::SetConstructor(inner,_)|ExprKind::MapConstructor(inner,_))=&outer[0].kind else{panic!("inner constructor restored")};
            assert!(matches!(&inner[0].kind,ExprKind::Name(name) if name==leaf));
        }
    }
}
