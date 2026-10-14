//! Concat checks each child once while retaining earlier receiver/argument loans.
use super::*;
use std::rc::Rc;

pub(super) enum State {
    Receiver,
    Argument { receiver: Type, active: Rc<Loans>, index: usize },
}

fn receiver_hint(analyzer: &Analyzer, context: &Context, base: &Expr, args: &[Expr], expected: Option<Type>) -> Option<Type> {
    let ExprKind::Array(values) = &base.kind else { return None; };
    expected.filter(|ty| matches!(ty, Type::Array(_))).or_else(|| {
        if !values.is_empty() { return None; }
        let mut inferred = None;
        for arg in args {
            let Some(ty) = analyzer.expression_type_hint(context, arg) else { continue; };
            let hint = match ty {
                Type::Array(_) => Some(ty),
                Type::Int | Type::Number | Type::Boolean | Type::String | Type::Record(_) => Analyzer::transform_array_type(ty, arg.span).ok(),
                _ => None,
            };
            // Number supplies literal context without converting typed integers.
            if matches!(hint, Some(Type::Array(ArrayElement::Number(_)))) { return hint; }
            if inferred.is_none() { inferred = hint; }
        }
        inferred
    })
}

pub(super) fn start<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>) -> Step<'a> {
    let expected = task.checked.as_ref().expect("checked concat").expected;
    let ExprKind::Call(callee, args) = &mut task.expression.value().kind else { unreachable!() };
    let ExprKind::Field(base, _) = &mut callee.kind else { unreachable!() };
    let hint = receiver_hint(analyzer, context, base, args, expected);
    let span = base.span;
    let receiver = std::mem::replace(base, Box::new(Expr::new(ExprKind::Undefined, span)));
    let child = Task::Expression(ExpressionTask::new(Expression::Child(receiver), task.context, Mode::Read, task.loans.clone(), hint));
    task.state = ExpressionState::Concat(Box::new(State::Receiver));
    Step::Push(Task::Expression(task), child)
}

pub(super) fn resume<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, state: Box<State>, completion: Completion<'a>) -> Step<'a> {
    let Completion::Expression(child, result) = completion else { unreachable!("concat expression child") };
    match *state {
        State::Receiver => {
            let ExprKind::Call(callee, _) = &mut task.expression.value().kind else { unreachable!() };
            let ExprKind::Field(base, _) = &mut callee.kind else { unreachable!() };
            *base = child.into_child();
            let receiver = match result {
                Ok(ty) if ty.scalar_base() == Type::String => Type::String,
                Ok(ty @ Type::Array(_)) => ty,
                Ok(_) => { let error = Diagnostic::new("type", "concat() requires a string or array.", base.span); return finish_expression(analyzer, context, task, Err(error)); },
                Err(error) => return finish_expression(analyzer, context, task, Err(error)),
            };
            let mut active = (*task.loans).clone();
            for root in Analyzer::loan_roots(base, Mode::Read) {
                active.entry(root).or_default().push((Mode::Read, base.span));
            }
            next(analyzer, context, task, receiver, Rc::new(active), 0)
        }
        State::Argument { receiver, mut active, index } => {
            let ExprKind::Call(_, args) = &mut task.expression.value().kind else { unreachable!() };
            args[index] = *child.into_child();
            let argument = &args[index];
            let checked = result.and_then(|actual| {
                if let Type::Array(element) = receiver {
                    if matches!(actual, Type::Array(_)) {
                        if actual != receiver {
                            return Err(Diagnostic::new("type", "concat() spreads array arguments one level; use an array at the receiver's full rank (wrap a nested element in an outer array).", argument.span));
                        }
                    } else {
                        if matches!(element.ty(), Type::Array(_)) {
                            return Err(Diagnostic::new("type", "Nested concat() requires array arguments at the receiver's full rank.", argument.span));
                        }
                        analyzer.require(actual, Some(element.ty()), argument.span)?;
                    }
                } else { analyzer.require(actual, Some(Type::String), argument.span)?; }
                if receiver == Type::String || !actual.is_copy() {
                    for root in Analyzer::loan_roots(argument, Mode::Read) {
                        Rc::make_mut(&mut active).entry(root).or_default().push((Mode::Read, argument.span));
                    }
                }
                Ok(())
            });
            if let Err(error) = checked { return finish_expression(analyzer, context, task, Err(error)); }
            next(analyzer, context, task, receiver, active, index + 1)
        }
    }
}

fn next<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, receiver: Type, active: Rc<Loans>, index: usize) -> Step<'a> {
    let expr = task.expression.value();
    let ExprKind::Call(_, args) = &mut expr.kind else { unreachable!() };
    if let Some(argument) = args.get_mut(index) {
        let target = if let Type::Array(element) = receiver {
            if matches!(analyzer.expression_type_hint(context, argument), Some(Type::Array(_))) || matches!(argument.kind, ExprKind::Array(_)) { receiver } else { element.ty() }
        } else { Type::String };
        let mode = if target.is_copy() { Mode::Copy } else { Mode::Read };
        let span = argument.span;
        let argument = std::mem::replace(argument, Expr::new(ExprKind::Undefined, span));
        let child = Task::Expression(ExpressionTask::new(Expression::Child(Box::new(argument)), task.context, mode, ActiveLoans::Shared(active.clone()), Some(target)));
        task.state = ExpressionState::Concat(Box::new(State::Argument { receiver, active, index }));
        return Step::Push(Task::Expression(task), child);
    }
    expr.target = Some(if receiver == Type::String { CallTarget::StringConcat } else { CallTarget::ArrayConcat });
    let result = analyzer.finish_builtin_result(context, expr, receiver);
    if let Ok(ty) = result { analyzer.resolve_completed_capture_invocation(context, expr, ty); }
    finish_expression(analyzer, context, task, result)
}
