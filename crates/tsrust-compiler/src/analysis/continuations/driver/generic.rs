//! Generic argument inference and inferred bodies share the checker work stack.
use super::*;
use crate::analysis::generic_call_work::{Arguments, Prepared, call};
use std::rc::Rc;
struct Pass { frame: Arguments, contexts: Vec<Option<Type>>, next: usize, progress: bool, deferred: Option<Diagnostic> }
enum Phase { Start, Spread, Pass(Pass), Argument { pass: Pass, index: usize }, Function { restore_name: Option<String> }, Done }
pub(super) struct GenericTask { expression: Box<Expr>, context: Rc<Context>, expected: Option<Type>, phase: Phase }
impl GenericTask {
    pub(super) fn new(expression: Box<Expr>, context: Rc<Context>, expected: Option<Type>) -> Self {
        Self { expression, context, expected, phase: Phase::Start }
    }
}
fn prepared<'a>(analyzer: &mut Analyzer, task: &mut GenericTask, value: Prepared) -> Result<Option<Task<'a>>, Diagnostic> {
    match value {
        Prepared::Spread(prototype) => {
            task.phase = Phase::Spread;
            return Ok(Some(Task::Generic(Box::new(GenericTask::new(prototype, task.context.clone(), task.expected)))));
        }
        Prepared::Done => task.phase = Phase::Done,
        Prepared::Function { index, restore_name } => {
            let ready = analyzer.function_request_ready(index);
            if ready.is_err() && let Some(original) = &restore_name {
                let ExprKind::Call(callee, _) = &mut call(&mut task.expression).kind else { unreachable!() };
                *Analyzer::generic_call_name(callee).expect("published generic name").0 = original.clone();
            }
            task.phase = Phase::Function { restore_name };
            if !ready? { return Ok(Some(Task::Function(Box::new(functions::FunctionTask::new(index))))); }
        }
        Prepared::Arguments(frame) => {
            let contexts = analyzer.generic_preparation_contexts(&task.context, &mut task.expression, task.expected, &frame)?;
            task.phase = Phase::Pass(Pass { frame, contexts, next: 0, progress: false, deferred: None });
        }
    }
    Ok(None)
}
pub(super) fn step<'a>(analyzer: &mut Analyzer, mut task: Box<GenericTask>, mut completion: Option<Completion<'a>>) -> Step<'a> {
    let action = (|| -> Result<Option<Task<'a>>, Diagnostic> {
        loop {
            match std::mem::replace(&mut task.phase, Phase::Start) {
                Phase::Start => {
                    let value = analyzer.start_generic_preparation(&task.context, &mut task.expression, task.expected)?;
                    if let Some(child) = prepared(analyzer, &mut task, value)? { return Ok(Some(child)); }
                }
                Phase::Spread => {
                    let Completion::Generic(prototype, result) = completion.take().expect("spread generic prototype") else { unreachable!() };
                    result?;
                    let ExprKind::Call(prepared, _) = prototype.into_kind() else { unreachable!() };
                    let ExprKind::Call(callee, _) = &mut call(&mut task.expression).kind else { unreachable!() };
                    *callee = prepared;
                    task.phase = Phase::Done;
                }
                Phase::Function { restore_name } => {
                    if let Some(value) = completion.take() {
                        let Completion::Function(result) = value else { unreachable!("generic inferred body") };
                        if let Err(error) = result {
                            if let Some(original) = restore_name {
                                let ExprKind::Call(callee, _) = &mut call(&mut task.expression).kind else { unreachable!() };
                                *Analyzer::generic_call_name(callee).expect("published generic name").0 = original;
                            }
                            return Err(error);
                        }
                    }
                    task.phase = Phase::Done;
                }
                Phase::Argument { mut pass, index } => {
                    let Completion::Generic(value, result) = completion.take().expect("generic argument preparation") else { unreachable!() };
                    let ExprKind::Call(_, args) = &mut call(&mut task.expression).kind else { unreachable!() };
                    args[index] = *value;
                    match result {
                        Ok(()) => { pass.frame.prepared[index] = true; pass.progress = true; }
                        Err(error) if error.code == "generic-inference" => { if pass.deferred.is_none() { pass.deferred = Some(error); } }
                        Err(error) => return Err(error),
                    }
                    task.phase = Phase::Pass(pass);
                }
                Phase::Pass(mut pass) => {
                    let ExprKind::Call(_, args) = &mut call(&mut task.expression).kind else { unreachable!() };
                    let next = (pass.next..pass.contexts.len().min(args.len())).find(|&index| !pass.frame.prepared[index]);
                    if let Some(index) = next {
                        pass.next = index + 1;
                        let span = args[index].span;
                        let value = Box::new(std::mem::replace(&mut args[index], Expr::new(ExprKind::Undefined, span)));
                        let child = Task::Generic(Box::new(GenericTask::new(value, task.context.clone(), pass.contexts[index])));
                        task.phase = Phase::Argument { pass, index };
                        return Ok(Some(child));
                    }
                    if pass.frame.prepared.iter().all(|value| *value) {
                        let value = analyzer.finish_generic_preparation(&task.context, &mut task.expression, task.expected, &pass.frame)?;
                        if let Some(child) = prepared(analyzer, &mut task, value)? { return Ok(Some(child)); }
                    } else {
                        if !pass.progress { return Err(pass.deferred.expect("unprepared generic argument")); }
                        pass.contexts = analyzer.generic_preparation_contexts(&task.context, &mut task.expression, task.expected, &pass.frame)?;
                        pass.next = 0; pass.progress = false; pass.deferred = None;
                        task.phase = Phase::Pass(pass);
                    }
                }
                Phase::Done => return Ok(None),
            }
        }
    })();
    match action {
        Ok(Some(child)) => Step::Push(Task::Generic(task), child),
        result => Step::Complete(Completion::Generic(task.expression, result.map(|_| ()))),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn inferred_generic_adapter_bodies_share_the_preparation_stack() {
        let count = 64;
        let mut source = "function step0<T>(value:T){return value;}".to_owned();
        for index in 1..=count {
            source.push_str(&format!("function step{index}<T>(value:T){{const action=step{};return [value].map(action)[0];}}", index-1));
        }
        source.push_str(&format!("function main(){{const action=step{count};console.log([7].map(action)[0]);}}"));
        let checked = crate::check(&source).unwrap();
        assert!(checked.program.closures.len() > count);
        drop(checked.emit_rust());
        drop(checked);
    }
    #[test]
    fn generic_argument_preparation_restores_failed_nested_call_names() {
        let source = crate::format("function fail<T>(value:T){return missing;}function outer<T>(value:T):T{return value;}function main(){outer(fail(7));}").unwrap();
        let mut program = crate::syntax::parse(&source).unwrap();
        let crate::syntax::StmtKind::Expr(mut value) = program.functions[2].body.remove(0).into_kind() else { panic!("outer call"); };
        let mut analyzer = super::Analyzer::new(program).unwrap();
        let context = super::Context::new("main".into());
        let error = analyzer.prepare_generic_call(&context, &mut value, None).expect_err("inferred nested body failure");
        assert_eq!(&source[error.span.start..error.span.end], "missing");
        let crate::syntax::ExprKind::Call(callee, args) = &value.kind else { panic!("restored outer call"); };
        assert!(matches!(&callee.kind, crate::syntax::ExprKind::Name(name) if name == "outer"));
        let crate::syntax::ExprKind::Call(callee, _) = &args[0].kind else { panic!("restored inner call"); };
        assert!(matches!(&callee.kind, crate::syntax::ExprKind::Name(name) if name == "fail"));
    }
}

#[cfg(test)]
mod chain_tests {
    #[test]
    fn inferred_direct_generic_calls_share_the_driver() {
        let count = 256;
        let mut source = "function step0<T>(value:T){return value;}".to_owned();
        for index in 1..count {
            source.push_str(&format!("function step{index}<T>(value:T){{return step{}(value);}}", index-1));
        }
        source.push_str(&format!("function main(){{console.log(step{}(7));}}", count-1));
        drop(crate::check(&source).unwrap().emit_rust());
    }
}

#[cfg(test)]
mod entry_tests {
    fn chain(kind: &str, count: usize, fail: bool) -> String {
        let leaf = if fail { "missing" } else if kind == "receiver" { "[value]" } else { "value" };
        let mut source = format!("function step0<T>(value:T){{return {leaf};}}");
        for index in 1..count {
            let previous = index-1;
            let value = match kind {
                "spread" => format!("step{previous}(...[value])"),
                "receiver" => format!("step{previous}(value).slice()"),
                "optional" => format!("step{previous}?.(value)"),
                _ => unreachable!(),
            };
            source.push_str(&format!("function step{index}<T>(value:T){{return {value};}}"));
        }
        source.push_str(&format!("function main(){{console.log(step{}(7));}}", count-1));
        source
    }
    #[test]
    fn generic_spread_optional_and_receiver_preparation_share_the_driver() {
        for (kind, count) in [("spread",128),("receiver",64),("optional",128)] {
            let source = chain(kind,count,false);
            let checked = crate::check(&source).unwrap_or_else(|error|panic!("{kind}: {}",error.render("input.tsr",&source)));
            drop(checked.emit_rust());
            drop(checked);
        }
    }
    #[test]
    fn failed_generic_entry_preparation_keeps_leaf_source_location() {
        for kind in ["spread","receiver","optional"] {
            let source = chain(kind,32,true);
            let error = crate::check(&source).err().expect("invalid leaf result");
            assert_eq!(&source[error.span.start..error.span.end],"missing","{kind}: {error:?}");
        }
    }
}
