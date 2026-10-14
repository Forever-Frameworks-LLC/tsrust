//! Deferred generic calls suspend body preparation before accessing real operands.
use super::*;
use crate::analysis::deferred_generic_call_work::{Request, Selection};
use crate::analysis::deferred_generic_preparation::{Pending, Prepared};

enum Purpose { Primary, Join, Selected(usize) }
enum Target { Primary, Selected { origin: usize, prepared: Prepared } }
enum Phase {
    Start, Preparation { context: std::rc::Rc<Context>, purpose: Purpose }, Pending { body: Pending, purpose: Purpose }, Join, Receiver(Type), Arguments,
    Argument { index: usize, frame: super::super::arguments::ArgumentFrame },
    Resolve(Target), Resolved { target: Target, function: usize, result: Option<Type> },
    Selected, Done,
}
struct Final { function: usize, result: Type, method: usize }
pub(super) struct DeferredTask<'a> {
    expression: Box<Expr>, request: Request, pub(super) context: ContextId, loans: ActiveLoans<'a>,
    phase: Phase, selection: Option<Selection>, primary: Option<Prepared>, next_join: usize,
    invocation: Mode, arguments: Option<super::super::arguments::Arguments>,
    final_call: Option<Final>, next_selected: usize, pairs: Vec<crate::polymorphic_invocation_plans::Pair>,
}
impl<'a> DeferredTask<'a> {
    pub(super) fn new(expression: Box<Expr>, request: Request, context: ContextId, loans: ActiveLoans<'a>) -> Self {
        Self { expression, request, context, loans, phase: Phase::Start, selection: None, primary: None,
            next_join: 0, invocation: Mode::Read, arguments: None, final_call: None, next_selected: 0, pairs: Vec::new() }
    }
    fn operands(&mut self) -> (&mut Expr, &mut Vec<Expr>) {
        let ExprKind::Call(callee, args) = &mut self.expression.kind else { unreachable!("deferred invocation") };
        (callee, args)
    }
}
enum Action<'a> { Child(Task<'a>), Done((usize, Type)) }
fn function_child<'a>(analyzer: &mut Analyzer, index: usize) -> Result<Option<Task<'a>>, Diagnostic> {
    if analyzer.function_request_ready(index)? { Ok(None) }
    else { Ok(Some(Task::Function(Box::new(functions::FunctionTask::new(index))))) }
}
fn prepare<'a>(analyzer: &mut Analyzer, context: &Context, task: &mut DeferredTask<'a>, origin: usize, expected: Option<Type>, purpose: Purpose) -> Result<Option<Task<'a>>, Diagnostic> {
    let span = task.expression.span;
    let args = task.operands().1.clone();
    let (preparation, call) = analyzer.start_deferred_generic_body(context, origin, args, span)?;
    let child = Task::Generic(Box::new(generic::GenericTask::new(call, preparation.clone(), expected)));
    task.phase = Phase::Preparation { context: preparation, purpose };
    Ok(Some(child))
}
pub(super) fn step<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: Box<DeferredTask<'a>>, mut completion: Option<Completion<'a>>) -> Step<'a> {
    let action = (|| -> Result<Action<'a>, Diagnostic> {
        loop {
            match std::mem::replace(&mut task.phase, Phase::Start) {
                Phase::Start => {
                    let span = task.expression.span;
                    let origin = task.request.origin;
                    let (callee, _) = task.operands();
                    task.selection = Some(analyzer.deferred_call_selection(context, callee, origin, span)?);
                    let primary = task.selection.as_ref().unwrap().primary;
                    let expected = task.request.expected;
                    if let Some(child) = prepare(analyzer, context, &mut task, primary, expected, Purpose::Primary)? { return Ok(Action::Child(child)); }
                }
                Phase::Preparation { context: preparation, purpose } => {
                    let Completion::Generic(call, result) = completion.take().expect("deferred generic preparation") else { unreachable!() };
                    result?;
                    let body = analyzer.finish_deferred_generic_request(&preparation, *call)?;
                    let child = function_child(analyzer, body.function)?;
                    task.phase = Phase::Pending { body, purpose };
                    if let Some(child) = child { return Ok(Action::Child(child)); }
                }
                Phase::Pending { body, purpose } => {
                    if let Some(completion) = completion.take() {
                        let Completion::Function(result) = completion else { unreachable!("deferred body completion") };
                        result?;
                    }
                    let mut prepared = analyzer.complete_deferred_generic_body(body)?;
                    match purpose {
                        Purpose::Primary => {
                            *task.operands().1 = std::mem::take(&mut prepared.arguments);
                            task.primary = Some(prepared);
                            task.phase = Phase::Join;
                        }
                        Purpose::Join => {
                            analyzer.join_prepared_deferred_contracts(&mut task.primary.as_mut().unwrap().contracts, &prepared.contracts, task.expression.span)?;
                            task.phase = Phase::Join;
                        }
                        Purpose::Selected(origin) => task.phase = Phase::Resolve(Target::Selected { origin, prepared }),
                    }
                }
                Phase::Join => {
                    let selection = task.selection.as_ref().unwrap();
                    if let Some(&origin) = selection.origins.get(task.next_join) {
                        task.next_join += 1;
                        if origin == selection.primary { task.phase = Phase::Join; continue; }
                        let expected = task.request.expected;
                        if let Some(child) = prepare(analyzer, context, &mut task, origin, expected, Purpose::Join)? { return Ok(Action::Child(child)); }
                        continue;
                    }
                    task.invocation = if task.request.once { Mode::Consume } else { task.primary.as_ref().unwrap().invocation };
                    let binding = task.request.binding;
                    let origin = task.request.origin;
                    let checked_receiver = task.request.checked_receiver;
                    let invocation = task.invocation;
                    let loans = task.loans.clone();
                    let (callee, _) = task.operands();
                    if let Some(binding) = binding {
                        Analyzer::access(context, binding, invocation, callee.span, &loans)?;
                        callee.binding = Some(binding);
                        callee.ty = Some(context.bindings[binding].ty);
                    } else {
                        let receiver = analyzer.expression_type_hint(context, callee).unwrap_or(Type::Closure(origin));
                        if checked_receiver { analyzer.require(callee.ty.expect("checked generic receiver"), Some(receiver), callee.span)?; }
                        else {
                            let span = callee.span;
                            let value = Box::new(std::mem::replace(callee, Expr::new(ExprKind::Undefined, span)));
                            let child = Task::Expression(ExpressionTask::new(Expression::Child(value), task.context, invocation, loans, Some(receiver)));
                            task.phase = Phase::Receiver(receiver);
                            return Ok(Action::Child(child));
                        }
                    }
                    task.phase = Phase::Arguments;
                }
                Phase::Receiver(expected) => {
                    let Completion::Expression(value, result) = completion.take().expect("deferred receiver") else { unreachable!() };
                    *task.operands().0 = *value.into_child();
                    analyzer.require(result?, Some(expected), task.operands().0.span)?;
                    task.phase = Phase::Arguments;
                }
                Phase::Argument { index, frame } => {
                    let Completion::Expression(value, result) = completion.take().expect("deferred argument") else { unreachable!() };
                    let ExprKind::Call(_, args) = &mut task.expression.kind else { unreachable!() };
                    args[index] = *value.into_child();
                    analyzer.finish_argument(&mut args[index], task.arguments.as_mut().unwrap(), frame, result?)?;
                    task.phase = Phase::Arguments;
                }
                Phase::Arguments => {
                    if task.arguments.is_none() {
                        let mut active = (*task.loans).clone();
                        let binding = task.request.binding;
                        let invocation = task.invocation;
                        let (callee, _) = task.operands();
                        if let Some(binding) = binding { active.entry(binding).or_default().push((invocation, callee.span)); }
                        else { for root in Analyzer::loan_roots(callee, invocation) { active.entry(root).or_default().push((invocation, callee.span)); } }
                        let contracts = std::mem::take(&mut task.primary.as_mut().unwrap().contracts);
                        task.arguments = Some(super::super::arguments::Arguments::new(contracts, active, context, true, false, false));
                    }
                    let ExprKind::Call(_, args) = &mut task.expression.kind else { unreachable!() };
                    let work = task.arguments.as_mut().unwrap();
                    if let Some((index, frame)) = analyzer.next_argument(context, args, work) {
                        let span = args[index].span;
                        let value = Box::new(std::mem::replace(&mut args[index], Expr::new(ExprKind::Undefined, span)));
                        let child = Task::Expression(ExpressionTask::new(Expression::Child(value), task.context, frame.mode, ActiveLoans::Shared(work.active.clone()), frame.expected));
                        task.phase = Phase::Argument { index, frame };
                        return Ok(Action::Child(child));
                    }
                    work.finish(context);
                    task.phase = Phase::Resolve(Target::Primary);
                }
                Phase::Resolve(target) => {
                    let span = task.expression.span;
                    let (function, args) = match &target {
                        Target::Primary => (task.primary.as_ref().unwrap().function, task.expression_args()),
                        Target::Selected { prepared, .. } => (prepared.function, &prepared.arguments),
                    };
                    let (function, immediate) = analyzer.prepare_specialization(function, args, span)?;
                    let result = if immediate { None } else { Some(analyzer.resolved_prepared_call(context, function, args, span)?.1) };
                    let child = function_child(analyzer, function)?;
                    task.phase = Phase::Resolved { target, function, result };
                    if let Some(child) = child { return Ok(Action::Child(child)); }
                }
                Phase::Resolved { target, function, result } => {
                    if let Some(completion) = completion.take() {
                        let Completion::Function(result) = completion else { unreachable!() };
                        result?;
                    }
                    let span = task.expression.span;
                    match target {
                        Target::Primary => {
                            let args = task.expression_args();
                            let result = result.map_or_else(|| analyzer.resolved_prepared_call(context, function, args, span).map(|(_, result)| result), Ok)?;
                            let prepared = task.primary.as_ref().unwrap();
                            let primary = task.selection.as_ref().unwrap().primary;
                            let method = analyzer.materialize_deferred_method(primary, function, result, args, &prepared.accesses, prepared.invocation, span)?;
                            task.final_call = Some(Final { function, result, method });
                            task.phase = if !analyzer.effects_only && !analyzer.discovering_results && !context.template
                                && (task.selection.as_ref().unwrap().origins.len() > 1 || primary != task.request.origin) { Phase::Selected } else { Phase::Done };
                        }
                        Target::Selected { origin, prepared } => {
                            let result = result.map_or_else(|| analyzer.resolved_prepared_call(context, function, &prepared.arguments, span).map(|(_, result)| result), Ok)?;
                            let method = analyzer.materialize_deferred_method(origin, function, result, &prepared.arguments, &prepared.accesses, prepared.invocation, span)?;
                            task.pairs.push(crate::polymorphic_invocation_plans::Pair { origin, method });
                            task.phase = Phase::Selected;
                        }
                    }
                }
                Phase::Selected => {
                    let selection = task.selection.as_ref().unwrap();
                    if let Some(&origin) = selection.origins.get(task.next_selected) {
                        task.next_selected += 1;
                        let primary = selection.primary;
                        let final_call = task.final_call.as_ref().unwrap();
                        if analyzer.deferred_alternative_needs_body(primary, origin) {
                            let result = final_call.result;
                            if let Some(child) = prepare(analyzer, context, &mut task, origin, Some(result), Purpose::Selected(origin))? { return Ok(Action::Child(child)); }
                        } else {
                            let prepared = task.primary.as_ref().unwrap();
                            let method = analyzer.materialize_deferred_method(origin, final_call.function, final_call.result, task.expression_args(), &prepared.accesses, prepared.invocation, task.expression.span)?;
                            task.pairs.push(crate::polymorphic_invocation_plans::Pair { origin, method });
                            task.phase = Phase::Selected;
                        }
                    } else {
                        let plan = analyzer.polymorphic_invocations.publish(std::mem::take(&mut task.pairs), &analyzer.closures, task.expression.span)?;
                        task.operands().0.generic_invocation = Some(plan);
                        task.phase = Phase::Done;
                    }
                }
                Phase::Done => {
                    let call = task.final_call.as_ref().unwrap();
                    let result = analyzer.retain_generic_invocation(context, task.selection.as_ref().unwrap().primary, call.method, call.result, task.expression.span)?;
                    return Ok(Action::Done(result));
                }
            }
        }
    })();
    match action {
        Ok(Action::Child(child)) => Step::Push(Task::Deferred(task), child),
        result => Step::Complete(Completion::Deferred(task.expression, result.map(|action| { let Action::Done(result) = action else { unreachable!() }; result }))),
    }
}
impl DeferredTask<'_> {
    fn expression_args(&self) -> &Vec<Expr> {
        let ExprKind::Call(_, args) = &self.expression.kind else { unreachable!() };
        args
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn named_generic_adapter_bodies_share_checker_stack() {
        let count = 64;
        let mut source = "function step0<T>(value:T):T{return value;}".to_owned();
        for index in 1..=count {
            source.push_str(&format!("function step{index}<T>(value:T):T{{const action=step{};return [value].map(action)[0];}}", index-1));
        }
        source.push_str(&format!("function main(){{const action=step{count};console.log([7].map(action)[0]);}}"));
        let mut program = crate::syntax::parse_unbounded(&source).unwrap();
        crate::callables::module_state::lower(&mut program).unwrap();
        crate::local_functions::resolve(&mut program).unwrap();
        let checked = crate::analysis::analyze_recursive(program).unwrap();
        assert!(checked.closures.len() > count);
        drop(checked);
    }
    #[test]
    fn deferred_argument_failure_preserves_source_location() {
        let source = crate::format("function identity<T>(value:T):T{return value;}function main(){const action=identity;action<int>(true);}").unwrap();
        let error = crate::check(&source).err().expect("invalid deferred argument");
        assert_eq!(error.code, "type");
        assert_eq!(&source[error.span.start..error.span.end], "true");
    }
}
