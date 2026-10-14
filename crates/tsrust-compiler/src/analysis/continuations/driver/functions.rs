//! Named bodies suspend on the same stack as their callers. Capture contexts
//! and dependency schedules remain live until body finalization completes.
use super::*;
use super::super::super::function_scheduling::{PreparedFunction, Schedule};

pub(super) struct FunctionTask {
    schedule: Schedule,
    active: Option<(usize, ContextId, Type)>,
}

impl FunctionTask {
    pub(super) fn new(index: usize) -> Self {
        Self { schedule: Schedule::new(index), active: None }
    }
}

pub(super) fn step<'a>(analyzer: &mut Analyzer, contexts: &mut Contexts, mut task: Box<FunctionTask>, completion: Option<Completion<'a>>) -> Step<'a> {
    let next = (|| {
        if let Some((index, child, result_type)) = task.active.take() {
            let Completion::Block(body, returned) = completion.expect("named function body") else { unreachable!("named function completion") };
            let mut context = *contexts.take(child);
            let returned = returned.and_then(|returned| { analyzer.validate_native_returns(&mut context)?; Ok(returned) });
            let uncalled = analyzer.finish_function_body(PreparedFunction { index, body: body.into_child(), context, result_type }, returned)?;
            task.schedule.extend(uncalled);
        }
        while let Some(index) = task.schedule.next(analyzer)? {
            let Some(mut prepared) = analyzer.prepare_function_body(index)? else { continue; };
            analyzer.enter_source_callable_body(&mut prepared.context)
                .map_err(|error| analyzer.filter_ownership_fixes(&prepared.context, error))?;
            let start = prepared.body.len() - Analyzer::uninitialized_body(&mut prepared.body).len();
            let exceptions = has_exception_syntax(&prepared.body[start..]);
            let context = contexts.push(Box::new(prepared.context));
            task.active = Some((index, context, prepared.result_type));
            let body = Body::Child { body: prepared.body, start };
            let child = if exceptions {
                Task::Callable(Box::new(CallableTask { body: Some(body), context, result_type: prepared.result_type, boundary: None }))
            } else {
                Task::Block(BlockTask { body, context, frame: None, pending: None, result_type: prepared.result_type, keep_names: false, label_span: None })
            };
            return Ok(Some(child));
        }
        Ok(None)
    })();
    match next {
        Ok(Some(child)) => Step::Push(Task::Function(task), child),
        result => {
            task.schedule.release(analyzer);
            Step::Complete(Completion::Function(result.map(|_| ())))
        }
    }
}

#[cfg(test)]
mod tests {
    fn captured_source(depth: usize, defaults: bool, leaf: &str) -> String {
        let parameters = if defaults { "value:int=captured" } else { "" };
        format!("function probe():int{{const captured=7;{}return {leaf};{}}}function main(){{console.log(probe());}}",
            format!("function nested({parameters}):int{{").repeat(depth), "}return nested();".repeat(depth))
    }

    fn parse(source: &str) -> crate::syntax::Program {
        let mut program = crate::syntax::parse_unbounded(source).unwrap();
        crate::callables::module_state::lower(&mut program).unwrap();
        crate::local_functions::resolve(&mut program).unwrap();
        program
    }

    #[test]
    fn captured_bodies_suspend_on_shared_checker_stack() {
        for defaults in [false, true] {
            let source = captured_source(512, defaults, if defaults { "value" } else { "captured" });
            let checked = crate::analysis::analyze(parse(&source)).unwrap();
            assert_eq!(checked.functions.len(), 514);
            assert!(checked.functions.iter().all(|function| !function.body.is_empty()));
            drop(checked);
        }
    }

    #[test]
    fn captured_body_error_keeps_original_location() {
        let source = captured_source(64, false, "true");
        let error = crate::analysis::analyze(parse(&source)).err().expect("invalid return");
        assert_eq!(error.code, "type");
        assert_eq!(&source[error.span.start..error.span.end], "return");
    }
}
