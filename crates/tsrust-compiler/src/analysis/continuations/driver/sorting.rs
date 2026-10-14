//! Comparator checking suspends before final sorting mutation and copy validation.
use super::*;
use super::super::sorting::{Start, Callback, CompletionFrame, Received};

pub(super) enum State { Receiver { frame: Start, previous: bool }, Callback(Callback), Method(CompletionFrame) }
pub(super) fn start<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, copying: bool, expected: Option<Type>) -> Step<'a> {
    let expr = task.expression.value();
    let ExprKind::Call(callee, args) = &mut expr.kind else { unreachable!() };
    let ExprKind::Field(base, _) = &mut callee.kind else { unreachable!() };
    let frame = match analyzer.prepare_sorting(context, base, args, expr.span, (copying, expected)) { Ok(frame) => frame, Err(error) => return finish_expression(analyzer, context, task, Err(error)) };
    let previous = analyzer.builtin_literals;
    analyzer.builtin_literals = true;
    let span = base.span;
    let receiver = std::mem::replace(base, Box::new(Expr::new(ExprKind::Undefined, span)));
    let child = Task::Expression(ExpressionTask::new(Expression::Child(receiver), task.context, if copying || frame.named { Mode::Read } else { Mode::Consume }, task.loans.clone(), frame.hint));
    task.state = ExpressionState::Sorting(Box::new(State::Receiver { frame, previous }));
    Step::Push(Task::Expression(task), child)
}
pub(super) fn resume<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, state: Box<State>, completion: Completion<'a>) -> Step<'a> {
    match (*state, completion) {
        (State::Receiver { frame, previous }, Completion::Expression(receiver, result)) => {
            analyzer.builtin_literals = previous;
            let expr = task.expression.value();
            let ExprKind::Call(callee, args) = &mut expr.kind else { unreachable!() };
            let ExprKind::Field(base, _) = &mut callee.kind else { unreachable!() };
            *base = receiver.into_child();
            let frame = match result.and_then(|receiver| analyzer.receive_sorting(context, base, args, &task.loans, expr.span, frame, receiver)) {
                Ok(Received::Ready(frame)) => return finish(analyzer, context, task, frame, Ok(None)),
                Ok(Received::Callback(frame)) => frame,
                Err(error) => return finish_expression(analyzer, context, task, Err(error)),
            };
            match analyzer.prepare_generic_traversal(context, &mut args[0], &frame.supplied) {
                Ok(Some(adapter)) => {
                    let span = args[0].span;
                    let original = Box::new(std::mem::replace(&mut args[0], Expr::new(ExprKind::Undefined, span)));
                    let child = Task::Adapter(Box::new(adapters::AdapterTask::new(original, adapter, task.context, ActiveLoans::Shared(std::rc::Rc::new(frame.active.clone())))));
                    task.state = ExpressionState::Sorting(Box::new(State::Callback(frame)));
                    Step::Push(Task::Expression(task), child)
                }
                Err(error) => finish_expression(analyzer, context, task, Err(error)),
                Ok(None) => {
                    let span = args[0].span;
                    let callback = std::mem::replace(&mut args[0], Expr::new(ExprKind::Undefined, span));
                    let child = Task::Expression(ExpressionTask::new(Expression::Child(Box::new(callback)), task.context, Mode::Mutate, ActiveLoans::Shared(std::rc::Rc::new(frame.active.clone())), frame.hint));
                    task.state = ExpressionState::Sorting(Box::new(State::Callback(frame)));
                    Step::Push(Task::Expression(task), child)
                }
            }
        }
        (State::Callback(frame), Completion::Expression(callback, result)) => {
            let ExprKind::Call(_, args) = &mut task.expression.value().kind else { unreachable!() };
            args[0] = *callback.into_child();
            checked(analyzer, context, task, frame, result)
        }
        (State::Method(frame), Completion::Method(result)) => finish(analyzer, context, task, frame, result.map(Some)),
        _ => unreachable!("sorting completion"),
    }
}
fn checked<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, frame: Callback, actual: Result<Type, Diagnostic>) -> Step<'a> {
    let ExprKind::Call(_, args) = &mut task.expression.value().kind else { unreachable!() };
    let mut frame = match actual.and_then(|ty| analyzer.check_sorting_callback(context, &mut args[0], frame, ty)) { Ok(frame) => frame, Err(error) => return finish_expression(analyzer, context, task, Err(error)) };
    if let Some(method) = frame.method.take() {
        let child = Task::Method(Box::new(methods::MethodTask::Start { origin: method.origin, types: method.types, span: args[0].span }));
        task.state = ExpressionState::Sorting(Box::new(State::Method(frame)));
        Step::Push(Task::Expression(task), child)
    } else { finish(analyzer, context, task, frame, Ok(None)) }
}
fn finish<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, frame: CompletionFrame, specialized: Result<Option<(usize, Type)>, Diagnostic>) -> Step<'a> {
    let expr = task.expression.value();
    let result = (|| {
        let specialized = specialized?;
        let ExprKind::Call(callee, args) = &mut expr.kind else { unreachable!() };
        let ExprKind::Field(base, name) = &mut callee.kind else { unreachable!() };
        let result = analyzer.finish_sorting(context, base, args, &task.loans, expr.span, frame, specialized)?;
        expr.target = Some(if name == "toSorted" { CallTarget::ArrayToSorted } else { CallTarget::ArraySort });
        let result = analyzer.finish_builtin_result(context, expr, result)?;
        analyzer.resolve_completed_capture_invocation(context, expr, result);
        Ok(result)
    })();
    finish_expression(analyzer, context, task, result)
}
