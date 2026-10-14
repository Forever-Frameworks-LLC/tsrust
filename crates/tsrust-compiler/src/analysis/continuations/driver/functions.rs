//! Named bodies suspend on the same stack as their callers. Capture contexts
//! and dependency schedules remain live until body finalization completes.
use super::*;
use super::super::super::function_scheduling::{PreparedFunction, Schedule};

enum Active { Parameters { index: usize, child: ContextId }, Body { index: usize, child: ContextId, result_type: Type } }

pub(super) struct FunctionTask {
    schedule: Schedule,
    active: Option<Active>,
    boundary: Option<(usize,usize)>,
}

impl FunctionTask {
    pub(super) fn new(index: usize) -> Self {
        Self { schedule: Schedule::new(index), active: None, boundary: None }
    }
}

pub(super) fn step<'a>(analyzer: &mut Analyzer, contexts: &mut Contexts, mut task: Box<FunctionTask>, completion: Option<Completion<'a>>) -> Step<'a> {
    let boundary=*task.boundary.get_or_insert((analyzer.depth,analyzer.function_stack.len()));
    let next = (|| {
        if let Some(active) = task.active.take() {
            match (active, completion.expect("named function child")) {
                (Active::Body { index, child, result_type }, Completion::Block(body, returned)) => {
                    let mut context = *contexts.take(child);
                    let returned = returned.and_then(|returned| { analyzer.validate_native_returns(&mut context)?; Ok(returned) });
                    let uncalled = analyzer.finish_function_body(PreparedFunction { index, body: body.into_child(), context, result_type }, returned)?;
                    task.schedule.extend(uncalled);
                }
                (Active::Parameters { index, child }, Completion::Parameters(mut body, result)) => {
                    if let Err(error) = result { contexts.take(child); return Err(error); }
                    let context = contexts.get(child);
                    let result_type = analyzer.complete_named_parameters(index, context, &body);
                    if let Err(error) = analyzer.enter_source_callable_body(context) {
                        let error = analyzer.filter_ownership_fixes(context, error);
                        contexts.take(child);
                        return Err(error);
                    }
                    let start = body.len() - Analyzer::uninitialized_body(&mut body).len();
                    let exceptions = has_exception_syntax(&body[start..]);
                    task.active = Some(Active::Body { index, child, result_type });
                    let body = Body::Child { body, start };
                    let child = if exceptions {
                        Task::Callable(Box::new(CallableTask { body: Some(body), context: child, result_type, boundary: None }))
                    } else {
                        Task::Block(BlockTask { body, context: child, frame: None, pending: None, result_type, keep_names: false, label_span: None })
                    };
                    return Ok(Some(child));
                }
                _ => unreachable!("named function completion"),
            }
        }
        while let Some(index) = task.schedule.next(analyzer)? {
            let Some((prepared, parameters)) = analyzer.prepare_function_body(index)? else { continue; };
            let context = contexts.push(Box::new(prepared.context));
            task.active = Some(Active::Parameters { index, child: context });
            return Ok(Some(Task::Parameters(Box::new(parameters::ParametersTask::new(parameters, context, prepared.body)))));
        }
        Ok(None)
    })();
    match next {
        Ok(Some(child)) => Step::Push(Task::Function(task), child),
        result => {
            if result.is_err() {
                for index in analyzer.function_stack.drain(boundary.1..) { analyzer.visiting[index]=false; }
                analyzer.depth=boundary.0;
            }
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
    fn default_call_chain(count: usize, leaf: &str) -> String {
        let mut source = String::from("function probe():int{const captured=1;");
        for index in 0..count {
            let default = if index + 1 == count { leaf.into() } else { format!("step{}()", index + 1) };
            source.push_str(&format!("function step{index}(value:int={default}):int{{return value+captured;}}"));
        }
        source.push_str("return step0();}function main(){console.log(probe());}");
        source
    }

    #[test]
    fn captured_default_calls_suspend_before_named_body_entry() {
        let source = default_call_chain(512, "7");
        let checked = crate::analysis::analyze(parse(&source)).unwrap();
        assert_eq!(checked.functions.len(), 514);
        for function in &checked.functions {
            if function.parameter_count == 1 {
                assert!(function.bindings[0].initializer.is_some());
                assert_eq!(function.bindings[0].omission, crate::ParameterOmission::Defaulted);
            }
        }
        drop(checked);
    }

    #[test]
    fn failed_default_call_chain_preserves_leaf_diagnostic() {
        let source = default_call_chain(256, "missing");
        let error = crate::analysis::analyze(parse(&source)).err().expect("unknown default");
        assert_eq!(error.code, "name");
        assert_eq!(&source[error.span.start..error.span.end], "missing");
    }

}
