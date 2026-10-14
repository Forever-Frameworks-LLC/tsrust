//! Reduction receiver, callback, initial value and method checks share one stack.
use super::*;
use super::super::reduction::{Start, Callback, Initial, CompletionFrame};

pub(super) enum State {
    Receiver(Start), Callback(Callback), Initial(Initial), Method(CompletionFrame),
}
pub(super) fn start<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>) -> Step<'a> {
    let expr = task.expression.value();
    let ExprKind::Call(callee, args) = &mut expr.kind else { unreachable!() };
    let ExprKind::Field(base, _) = &mut callee.kind else { unreachable!() };
    let frame = match analyzer.prepare_reduction(context, base, args, expr.span) { Ok(frame) => frame, Err(error) => return finish_expression(analyzer, context, task, Err(error)) };
    let span = base.span;
    let value = std::mem::replace(base, Box::new(Expr::new(ExprKind::Undefined, span)));
    let child = Task::Expression(ExpressionTask::new(Expression::Child(value), task.context, Mode::Read, task.loans.clone(), frame.receiver_hint));
    task.state = ExpressionState::Reduction(Box::new(State::Receiver(frame)));
    Step::Push(Task::Expression(task), child)
}
pub(super) fn resume<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, state: Box<State>, completion: Completion<'a>) -> Step<'a> {
    match (*state, completion) {
        (State::Receiver(frame), Completion::Expression(receiver, result)) => {
            let expr = task.expression.value();
            let ExprKind::Call(callee, args) = &mut expr.kind else { unreachable!() };
            let ExprKind::Field(base, _) = &mut callee.kind else { unreachable!() };
            *base = receiver.into_child();
            let frame = match result.and_then(|receiver| analyzer.receive_reduction(context, base, args, &task.loans, frame, receiver)) { Ok(frame) => frame, Err(error) => return finish_expression(analyzer, context, task, Err(error)) };
            match analyzer.prepare_generic_traversal(context, &mut args[0], &frame.supplied) {
                Ok(Some(adapter)) => {
                    let span = args[0].span;
                    let original = Box::new(std::mem::replace(&mut args[0], Expr::new(ExprKind::Undefined, span)));
                    let child = Task::Adapter(Box::new(adapters::AdapterTask::new(original, adapter, task.context, ActiveLoans::Shared(std::rc::Rc::new(frame.active.clone())))));
                    task.state = ExpressionState::Reduction(Box::new(State::Callback(frame)));
                    Step::Push(Task::Expression(task), child)
                }
                Err(error) => finish_expression(analyzer, context, task, Err(error)),
                Ok(None) => {
                    let span = args[0].span;
                    let value = std::mem::replace(&mut args[0], Expr::new(ExprKind::Undefined, span));
                    let child = Task::Expression(ExpressionTask::new(Expression::Child(Box::new(value)), task.context, Mode::Mutate, ActiveLoans::Shared(std::rc::Rc::new(frame.active.clone())), frame.checked_hint));
                    task.state = ExpressionState::Reduction(Box::new(State::Callback(frame)));
                    Step::Push(Task::Expression(task), child)
                }
            }
        }
        (State::Callback(frame), Completion::Expression(callback, result)) => {
            let ExprKind::Call(_, args) = &mut task.expression.value().kind else { unreachable!() };
            args[0] = *callback.into_child();
            checked_callback(analyzer, context, task, frame, result)
        }
        (State::Initial(frame), Completion::Expression(initial, result)) => {
            let ExprKind::Call(_, args) = &mut task.expression.value().kind else { unreachable!() };
            args[1] = *initial.into_child();
            checked_initial(analyzer, context, task, frame, result.map(Some))
        }
        (State::Method(frame), Completion::Method(result)) => finish(analyzer, context, task, frame, result.map(Some)),
        _ => unreachable!("reduction completion"),
    }
}
fn checked_callback<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, frame: Callback, actual: Result<Type, Diagnostic>) -> Step<'a> {
    let ExprKind::Call(_, args) = &mut task.expression.value().kind else { unreachable!() };
    let frame = match actual.and_then(|ty| analyzer.check_reduction_callback(context, args, frame, ty)) { Ok(frame) => frame, Err(error) => return finish_expression(analyzer, context, task, Err(error)) };
    if let Some(initial) = args.get_mut(1) {
        let span = initial.span;
        let value = std::mem::replace(initial, Expr::new(ExprKind::Undefined, span));
        let child = Task::Expression(ExpressionTask::new(Expression::Child(Box::new(value)), task.context, Mode::Consume, ActiveLoans::Shared(std::rc::Rc::new(frame.active.clone())), Some(frame.result)));
        task.state = ExpressionState::Reduction(Box::new(State::Initial(frame)));
        Step::Push(Task::Expression(task), child)
    } else { checked_initial(analyzer, context, task, frame, Ok(None)) }
}
fn checked_initial<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, frame: Initial, actual: Result<Option<Type>, Diagnostic>) -> Step<'a> {
    let expr = task.expression.value();
    let ExprKind::Call(_, args) = &mut expr.kind else { unreachable!() };
    let mut frame = match actual.and_then(|actual| analyzer.complete_reduction_arguments(args, expr.span, frame, actual)) { Ok(frame) => frame, Err(error) => return finish_expression(analyzer, context, task, Err(error)) };
    if let Some(method) = frame.method.take() {
        let child = Task::Method(Box::new(methods::MethodTask::Start { origin: method.origin, types: method.types, span: args[0].span }));
        task.state = ExpressionState::Reduction(Box::new(State::Method(frame)));
        Step::Push(Task::Expression(task), child)
    } else { finish(analyzer, context, task, frame, Ok(None)) }
}
fn finish<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, frame: CompletionFrame, specialized: Result<Option<(usize, Type)>, Diagnostic>) -> Step<'a> {
    let expr = task.expression.value();
    let result = (|| {
        let specialized = specialized?;
        let ExprKind::Call(callee, args) = &mut expr.kind else { unreachable!() };
        let ExprKind::Field(base, name) = &mut callee.kind else { unreachable!() };
        let result = analyzer.finish_reduction(context, base, args, &task.loans, expr.span, frame, specialized)?;
        expr.target = Some(if name == "reduceRight" { CallTarget::ArrayReduceRight } else { CallTarget::ArrayReduce });
        let result = analyzer.finish_builtin_result(context, expr, result)?;
        analyzer.resolve_completed_capture_invocation(context, expr, result);
        Ok(result)
    })();
    finish_expression(analyzer, context, task, result)
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
    fn nested_reductions_and_comparators_share_checker_stack() {
        let count = 512;
        for (opening, closing) in [("[7].reduce((acc:int,value:int):int=>", ",0)"), ("[7,8].toSorted((left:int,right:int):int=>(", ")-7)[0]")] {
            let source = format!("function main(){{console.log({}7{});}}", opening.repeat(count), closing.repeat(count));
            let checked = crate::analysis::analyze(parse(&source)).unwrap();
            assert_eq!(checked.closures.len(), count);
            drop(checked);
        }
    }
    #[test]
    fn failed_nested_initial_value_preserves_source_location() {
        let source = format!("function main(){{console.log({}7,missing){});}}", "[7].reduce((acc:int,value:int):int=>".repeat(128), ",0)".repeat(127));
        let error = crate::analysis::analyze(parse(&source)).err().expect("missing initial value");
        assert_eq!(error.code, "name");
        assert_eq!(&source[error.span.start..error.span.end], "missing");
    }
    #[test]
    fn failed_nested_sorting_restores_literal_hint() {
        let source = format!("function main(){{console.log({}missing{});}}", "[7,8].toSorted((left:int,right:int):int=>(".repeat(128), ")-7)[0]".repeat(128));
        let mut analyzer = Analyzer::new(parse(&source)).unwrap();
        let error = analyzer.function(0).expect_err("missing comparator result");
        assert_eq!(error.code, "name");
        assert_eq!(&source[error.span.start..error.span.end], "missing");
        assert!(!analyzer.builtin_literals);
    }
}
