//! Match bodies and yielded values suspend on the shared checker stack.
use super::*;
pub(super) struct ValueFrame {
    environment: Option<std::rc::Rc<exception_state::CheckEnvironment>>,
    exits: Option<super::super::exceptions::ExitBoundary>,
}

pub(super) fn schedule_value<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, expected: Option<Type>) -> Step<'a> {
    let ExprKind::Match { id, body } = &mut task.expression.value().kind else { unreachable!() };
    context.match_values.push((*id, expected));
    context.match_loans.push((*task.loans).clone());
    let environment = context.exception_environment.clone();
    if environment.is_none() && has_exception_syntax(body) {
        context.exception_environment = Some(std::rc::Rc::new(exception_state::CheckEnvironment {
            effects: analyzer.discovery_error_effects(), runtime: None, refresh_contracts: true, partial_contracts: true,
        }));
    }
    let exits = if context.completion_frame.is_none() && has_exception_syntax(body) { Some(Analyzer::prepare_exit_frames(context)) } else { None };
    let child = Task::Block(BlockTask { body: Body::Child { body: std::mem::take(body), start: 0 }, context: task.context,
        frame: None, pending: None, result_type: context.block_result_type, keep_names: false, label_span: None });
    task.state = ExpressionState::MatchValue(Box::new(ValueFrame { environment, exits }));
    Step::Push(Task::Expression(task), child)
}

pub(super) fn resume_value<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, frame: Box<ValueFrame>, completion: Completion<'a>) -> Step<'a> {
    let Completion::Block(body, checked) = completion else { unreachable!() };
    let expr = task.expression.value();
    let ExprKind::Match { body: destination, .. } = &mut expr.kind else { unreachable!() };
    *destination = body.into_child();
    context.exception_environment = frame.environment;
    let checked = if let Some(boundary) = frame.exits {
        Analyzer::finish_exit_frames(context, boundary, checked).map(|(normal, _, _)| normal.is_none())
    } else { checked };
    context.match_loans.pop();
    let (_, output) = context.match_values.pop().expect("match output context");
    let result = checked.and_then(|_| output.ok_or_else(|| Diagnostic::new("match-value", "A value-producing match needs a value arm or a contextual result type.", expr.span)));
    finish_expression(analyzer, context, task, result)
}

pub(super) fn schedule_yield<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>) -> Step<'a> {
    let (id, expected) = *context.match_values.last().expect("yield inside match");
    let ExprKind::Call(_, arguments) = &mut task.expression.value().kind else { unreachable!() };
    let returning = context.returning_closure;
    context.returning_closure = matches!(arguments[0].kind, ExprKind::Closure { .. }) || expected.is_some_and(|ty| analyzer.callable_result_type(ty));
    let span = arguments[0].span;
    let value = Box::new(std::mem::replace(&mut arguments[0], Expr::new(ExprKind::Undefined, span)));
    let child = Task::Expression(ExpressionTask::new(Expression::Child(value), task.context, Mode::Consume, task.loans.clone(), expected));
    task.state = ExpressionState::MatchYield { id, expected, returning };
    Step::Push(Task::Expression(task), child)
}

pub(super) fn resume_yield<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, id: usize, expected: Option<Type>, returning: bool, completion: Completion<'a>) -> Step<'a> {
    let Completion::Expression(value, checked) = completion else { unreachable!() };
    let expr = task.expression.value();
    let ExprKind::Call(_, arguments) = &mut expr.kind else { unreachable!() };
    arguments[0] = *value.into_child();
    context.returning_closure = returning;
    let result = checked.and_then(|actual| {
        analyzer.require(actual, expected, arguments[0].span)?;
        if id>=usize::MAX/2 {
            analyzer.attach_payload_capture_loans(context,&arguments[0],crate::capture_loan_owners::Place{binding:usize::MAX,fields:vec![format!("future:{id}")]},true);
        }
        context.match_values.last_mut().expect("match output context").1 = Some(actual);
        expr.target = Some(CallTarget::MatchYield(id));
        expr.ty = Some(Type::Void);
        Ok(Type::Void)
    });
    finish_expression(analyzer, context, task, result)
}
