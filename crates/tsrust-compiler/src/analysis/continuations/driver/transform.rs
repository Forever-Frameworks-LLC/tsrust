//! Transform parents suspend receivers, callbacks and specialized method bodies.
use super::*;
use super::super::transform::{Start, Callback, CompletionFrame, Request, ResultKind};

pub(super) enum State {
    Receiver { frame: Start, previous: bool, request: Request },
    Callback { frame: Callback, request: Request },
    Method { frame: CompletionFrame, request: Request },
}

fn failed<'a>(analyzer: &mut Analyzer, context: &mut Context, task: ExpressionTask<'a>, request: Request, mut error: Diagnostic) -> Step<'a> {
    if matches!(request.kind, ResultKind::ArrayFrom) && error.code == "arity" { error.message = format!("Array.from: {}", error.message); }
    finish_expression(analyzer, context, task, Err(error))
}

pub(super) fn start<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, request: Request) -> Step<'a> {
    let expr = task.expression.value();
    let ExprKind::Call(callee, args) = &mut expr.kind else { unreachable!("transform call") };
    let ExprKind::Field(base, _) = &mut callee.kind else { unreachable!("transform receiver") };
    let frame = match analyzer.prepare_transform(context, base, args, expr.span, request.options) {
        Ok(frame) => frame,
        Err(error) => return failed(analyzer, context, task, request, error),
    };
    let previous = analyzer.builtin_literals;
    analyzer.builtin_literals |= request.options.3;
    let span = base.span;
    let receiver = std::mem::replace(base, Box::new(Expr::new(ExprKind::Undefined, span)));
    let child = Task::Expression(ExpressionTask::new(Expression::Child(receiver), task.context, request.options.2, task.loans.clone(), frame.receiver_hint));
    task.state = ExpressionState::Transform(Box::new(State::Receiver { frame, previous, request }));
    Step::Push(Task::Expression(task), child)
}

pub(super) fn resume<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, state: Box<State>, completion: Completion<'a>) -> Step<'a> {
    match (*state, completion) {
        (State::Receiver { frame, previous, request }, Completion::Expression(receiver, result)) => {
            analyzer.builtin_literals = previous;
            let expr = task.expression.value();
            let ExprKind::Call(callee, args) = &mut expr.kind else { unreachable!() };
            let ExprKind::Field(base, _) = &mut callee.kind else { unreachable!() };
            *base = receiver.into_child();
            let frame = match result.and_then(|receiver| analyzer.receive_transform(context, base, args, &task.loans, receiver, frame)) {
                Ok(frame) => frame,
                Err(error) => return failed(analyzer, context, task, request, error),
            };
            match analyzer.prepare_generic_traversal(context, &mut args[0], &frame.supplied_types) {
                Ok(Some(adapter)) => {
                    let span = args[0].span;
                    let original = Box::new(std::mem::replace(&mut args[0], Expr::new(ExprKind::Undefined, span)));
                    let child = Task::Adapter(Box::new(adapters::AdapterTask::new(original, adapter, task.context, ActiveLoans::Shared(std::rc::Rc::new(frame.active.clone())))));
                    task.state = ExpressionState::Transform(Box::new(State::Callback { frame, request }));
                    Step::Push(Task::Expression(task), child)
                }
                Err(error) => failed(analyzer, context, task, request, error),
                Ok(None) => {
                    let span = args[0].span;
                    let callback = std::mem::replace(&mut args[0], Expr::new(ExprKind::Undefined, span));
                    let child = Task::Expression(ExpressionTask::new(Expression::Child(Box::new(callback)), task.context, Mode::Mutate,
                        ActiveLoans::Shared(std::rc::Rc::new(frame.active.clone())), frame.checked_hint));
                    task.state = ExpressionState::Transform(Box::new(State::Callback { frame, request }));
                    Step::Push(Task::Expression(task), child)
                }
            }
        }
        (State::Callback { frame, request }, Completion::Expression(callback, result)) => {
            let ExprKind::Call(_, args) = &mut task.expression.value().kind else { unreachable!() };
            args[0] = *callback.into_child();
            checked(analyzer, context, task, frame, request, result)
        }
        (State::Method { frame, request }, Completion::Method(result)) => finish(analyzer, context, task, frame, request, result.map(Some)),
        _ => unreachable!("transform completion"),
    }
}

fn checked<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, frame: Callback, request: Request, actual: Result<Type, Diagnostic>) -> Step<'a> {
    let expr = task.expression.value();
    let ExprKind::Call(_, args) = &mut expr.kind else { unreachable!() };
    let mut frame = match actual.and_then(|ty| analyzer.check_transform_callback(context, args, expr.span, frame, ty)) {
        Ok(frame) => frame,
        Err(error) => return failed(analyzer, context, task, request, error),
    };
    if let Some(method) = frame.method.take() {
        let child = Task::Method(Box::new(methods::MethodTask::Start { origin: method.origin, types: method.types, span: args[0].span }));
        task.state = ExpressionState::Transform(Box::new(State::Method { frame, request }));
        Step::Push(Task::Expression(task), child)
    } else { finish(analyzer, context, task, frame, request, Ok(None)) }
}

fn finish<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, frame: CompletionFrame, request: Request, specialized: Result<Option<(usize, Type)>, Diagnostic>) -> Step<'a> {
    let expr = task.expression.value();
    let result = (|| {
        let specialized = specialized?;
        let ExprKind::Call(callee, args) = &mut expr.kind else { unreachable!() };
        let ExprKind::Field(base, _) = &mut callee.kind else { unreachable!() };
        analyzer.finish_transform(context, base, args, &task.loans, expr.span, frame, specialized)
    })();
    match result {
        Ok(result) => {
            let result = analyzer.finish_transform_request(context, expr, request, result);
            finish_expression(analyzer, context, task, result)
        }
        Err(error) => failed(analyzer, context, task, request, error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(source: &str) -> crate::syntax::Program {
        let mut program = crate::syntax::parse_unbounded(source).unwrap();
        crate::callables::module_state::lower(&mut program).unwrap();
        crate::local_functions::resolve(&mut program).unwrap();
        program
    }

    #[test]
    fn nested_transform_and_for_each_callbacks_share_checker_stack() {
        let count = 512;
        let map = format!("{}7{}", "[7].map((value:int):int=>".repeat(count), ")[0]".repeat(count));
        let foreach = format!("{}console.log(7);{}", "[7].forEach((value:int):void=>{".repeat(count), "});".repeat(count));
        for body in [format!("console.log({map});"), foreach] {
            let source = format!("function main(){{{body}}}");
            let checked = crate::analysis::analyze(parse(&source)).unwrap();
            assert_eq!(checked.closures.len(), count);
            drop(checked);
        }
    }

    #[test]
    fn nested_group_by_size_receivers_share_checker_stack() {
        let count = 512;
        let source = format!("function main(){{console.log({}7{});}}", "Map.groupBy([7],(value:int):int=>".repeat(count), ").size".repeat(count));
        let checked = crate::analysis::analyze(parse(&source)).unwrap();
        assert_eq!(checked.closures.len(), count);
        drop(checked);
    }

    #[test]
    fn failed_nested_flat_map_restores_literal_hint_and_diagnostic_location() {
        let source = format!("function main(){{console.log({}missing{});}}", "[7].flatMap((value:int):int[]=>[".repeat(128), "])[0]".repeat(128));
        let mut analyzer = Analyzer::new(parse(&source)).unwrap();
        assert!(!analyzer.builtin_literals);
        let error = analyzer.function(0).expect_err("missing callback result");
        assert_eq!(error.code, "name");
        assert_eq!(&source[error.span.start..error.span.end], "missing");
        assert!(!analyzer.builtin_literals);
    }
}
