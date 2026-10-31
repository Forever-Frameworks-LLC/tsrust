//! Closure method bodies share the caller's checker stack and real environment.
use super::*;
use crate::analysis::closure_methods::{MethodBody, PreparedMethod};

pub(super) enum MethodTask {
    Start { origin: usize, types: Vec<Type>, span: Span },
    Body { origin: usize, method: usize, child: ContextId, result: Type, span: Span },
}

pub(super) fn step<'a>(analyzer: &mut Analyzer, contexts: &mut Contexts, task: Box<MethodTask>, completion: Option<Completion<'a>>) -> Step<'a> {
    match *task {
        MethodTask::Start { origin, types, span } => match analyzer.prepare_closure_method(origin, types, span) {
            Ok(PreparedMethod::Ready(method, result)) => Step::Complete(Completion::Method(Ok((method, result)))),
            Err(error) => Step::Complete(Completion::Method(Err(error))),
            Ok(PreparedMethod::Body(mut frame)) => {
                if let Err(error) = analyzer.enter_source_callable_body(&mut frame.context) {
                    return Step::Complete(Completion::Method(analyzer.finish_closure_method(frame, Err(error))));
                }
                let start = frame.body.len() - Analyzer::uninitialized_body(&mut frame.body).len();
                let exceptions = has_exception_syntax(&frame.body[start..]);
                let child = contexts.push(Box::new(frame.context));
                let result_type = frame.result;
                let parent = Task::Method(Box::new(MethodTask::Body { origin, method: frame.method, child, result: result_type, span }));
                let body = Body::Child { body: frame.body, start };
                let work = if exceptions {
                    Task::Callable(Box::new(CallableTask { body: Some(body), context: child, result_type, boundary: None }))
                } else {
                    Task::Block(BlockTask { body, context: child, frame: None, pending: None, result_type, keep_names: false, label_span: None })
                };
                Step::Push(parent, work)
            }
        },
        MethodTask::Body { origin, method, child, result, span } => {
            let Completion::Block(body, checked) = completion.expect("closure method body") else { unreachable!("closure method result") };
            let mut context = *contexts.take(child);
            let checked = checked.and_then(|returned| { analyzer.validate_native_returns(&mut context)?; Ok(returned) });
            let frame = MethodBody { origin, method, context, body: body.into_child(), result, span };
            Step::Complete(Completion::Method(analyzer.finish_closure_method(frame, checked)))
        }
    }
}

#[cfg(test)]
mod tests {
    fn chain(count: usize, conditional: bool) -> String {
        let mut source = String::from("function main(){const step0=(f:Function<int,[x:int]>):int=>f(0);");
        for index in 1..count {
            let previous = format!("step{}", index - 1);
            let callee = if conditional { format!("(true?{previous}:{previous})") } else { previous };
            source.push_str(&format!("const step{index}=(f:Function<int,[x:int]>):int=>{callee}(f);"));
        }
        source.push_str(&format!("console.log(step{}((x:int):int=>x+7));}}", count - 1));
        source
    }

    #[test]
    fn method_invocations_share_work_for_named_and_expression_callees() {
        for conditional in [false, true] {
            let source = chain(12, conditional);
            let formatted = crate::format(&source).unwrap();
            assert_eq!(crate::format(&formatted).unwrap(), formatted);
            let checked = crate::check(&formatted).unwrap();
            assert!(!checked.emit_rust().is_empty());
        }
    }

    #[test]
    fn long_callback_chains_specialize_without_a_count_limit() {
        // Like rustc, each distinct callback origin is specialized; there is no
        // count limit. A 40-step chain needs several hundred specializations.
        let source = chain(40, false);
        crate::check(&source).unwrap_or_else(|error| panic!("{}", error.message)).emit_rust();
    }
}
