//! Iterative expression evaluation keeps callable values separate from bindings.
use super::*;
type Facts = HashMap<(usize, usize), Option<Vec<Contract>>>;
#[derive(Clone, Default)]
struct Value {
    contracts: Option<Vec<Contract>>,
    truth: Option<bool>,
    nullish: Option<bool>,
    undefined: Option<bool>,
}
fn merge_value(left: Value, right: Value, declarations: &HashMap<String, Contract>) -> Value {
    let contracts = match (left.contracts, right.contracts) {
        (Some(left), Some(right)) => resolve_all(left.into_iter().chain(right).map(Target::Contract).collect(), &[], declarations),
        _ => None,
    };
    Value { contracts, truth: (left.truth == right.truth).then_some(left.truth).flatten(),
        nullish: (left.nullish == right.nullish).then_some(left.nullish).flatten(),
        undefined: (left.undefined == right.undefined).then_some(left.undefined).flatten() }
}
fn snapshot(expression: &Expr, state: &[Binding], declarations: &HashMap<String, Contract>) -> Value {
    let contracts = resolve_all(targets::value(expression), state, declarations);
    let (truth, nullish) = match &expression.kind {
        ExprKind::Boolean(value) => (Some(*value), Some(false)),
        ExprKind::Null | ExprKind::Undefined => (Some(false), Some(true)),
        ExprKind::String(value) => (Some(!value.is_empty()), Some(false)),
        ExprKind::Integer(value) => (value.parse::<i64>().ok().map(|value| value != 0), Some(false)),
        ExprKind::Number(value) => (Some(*value != 0.0 && !value.is_nan()), Some(false)),
        ExprKind::Array(_) | ExprKind::Record(_) => (Some(true), Some(false)),
        _ if contracts.is_some() && name(expression).is_some_and(|(name, span)| {
            binding_at(state, name, span).is_none_or(|index| state[index].contract.is_none())
        }) => (Some(true), Some(false)),
        _ => (None, None),
    };
    let undefined = if matches!(expression.kind, ExprKind::Undefined) { Some(true) }
        else if nullish == Some(false) || matches!(expression.kind, ExprKind::Null) { Some(false) } else { None };
    Value { contracts, truth, nullish, undefined }
}
fn commit(target: &Expr, value: &Value, state: &mut [Binding]) {
    if let ExprKind::Name(name) = &target.kind
        && let Some(index) = binding_at(state, name, target.span)
        && state[index].contract.is_none()
    {
        state[index].targets = value.contracts.clone().map_or_else(|| vec![Target::Unknown], |values| values.into_iter().map(Target::Contract).collect());
    }
}
fn fact(expression: &Expr, value: Value, facts: &mut Facts, declarations: &HashMap<String, Contract>) {
    let key = (expression.span.start, expression.span.end);
    let value = if let Some(previous) = facts.get(&key) {
        merge_value(Value { contracts: previous.clone(), ..Value::default() }, value, declarations)
    } else { value };
    facts.insert(key, value.contracts);
}

enum Task<'a> {
    Eval(&'a Expr),
    Finish(&'a Expr, usize),
    Call(&'a Expr, usize),
    Commit(&'a Expr),
    Kill(&'a Expr),
    Select(&'a Expr, &'a Expr),
    Lazy(&'a Expr, BinaryOp, Option<&'a Expr>),
    Between(&'a Expr, Vec<Binding>),
    Join(Vec<Binding>, Value),
    Discard,
}

/// Returns the evaluated value snapshot and environments of possible failures.
pub(super) fn evaluate(expression: &Expr, state: &mut Vec<Binding>, facts: &mut Facts,
    declarations: &HashMap<String, Contract>, observe_failures: bool) -> (Option<Vec<Contract>>, Vec<Vec<Binding>>) {
    let mut tasks = vec![Task::Eval(expression)];
    let mut values = Vec::new();
    let mut failures = Vec::new();
    while let Some(task) = tasks.pop() {
        match task {
            Task::Eval(expression) => match &expression.kind {
                ExprKind::Assign { target, value, .. } => {
                    tasks.extend([Task::Commit(target), Task::Eval(value), Task::Discard, Task::Eval(target)]);
                }
                ExprKind::CompoundAssign { target, value, .. } => {
                    tasks.extend([Task::Kill(target), Task::Finish(expression, 2), Task::Eval(value), Task::Eval(target)]);
                }
                ExprKind::LogicalAssign { target, value, and, .. } => {
                    tasks.extend([Task::Lazy(value, if *and { BinaryOp::And } else { BinaryOp::Or }, Some(target)), Task::Eval(target)]);
                }
                ExprKind::CoalesceAssign { target, value, .. } => {
                    tasks.extend([Task::Lazy(value, BinaryOp::Coalesce, Some(target)), Task::Eval(target)]);
                }
                ExprKind::Update { target, .. } => {
                    tasks.extend([Task::Kill(target), Task::Finish(expression, 1), Task::Eval(target)]);
                }
                ExprKind::Conditional { predicate, consequent, alternate } => {
                    tasks.extend([Task::Select(consequent, alternate), Task::Eval(predicate)]);
                }
                ExprKind::Binary { op, left, right, .. } if matches!(op, BinaryOp::And | BinaryOp::Or | BinaryOp::Coalesce | BinaryOp::PatternDefault) => {
                    tasks.extend([Task::Lazy(right, *op, None), Task::Eval(left)]);
                }
                ExprKind::Sequence(children) => {
                    for (index, child) in children.iter().enumerate().rev() {
                        if index + 1 != children.len() { tasks.push(Task::Discard); }
                        tasks.push(Task::Eval(child));
                    }
                    if children.is_empty() { values.push(Value::default()); }
                }
                ExprKind::Call(callee, arguments) => {
                    tasks.push(Task::Call(expression, arguments.len()));
                    for argument in arguments.iter().rev() { tasks.push(Task::Eval(argument)); }
                    tasks.push(Task::Eval(callee));
                }
                ExprKind::Binary { left, right, .. } | ExprKind::Index(left, right) => {
                    tasks.extend([Task::Finish(expression, 2), Task::Eval(right), Task::Eval(left)]);
                }
                ExprKind::Unary { op: UnaryOp::OptionalBoundary, value, .. } => tasks.push(Task::Eval(value)),
                ExprKind::Unary { value, .. } | ExprKind::TypeAssertion { value, .. } | ExprKind::Field(value, _) => {
                    tasks.extend([Task::Finish(expression, 1), Task::Eval(value)]);
                }
                ExprKind::Array(children) | ExprKind::Template(children) | ExprKind::ArrayOf(children, _)
                | ExprKind::SetConstructor(children, _) | ExprKind::MapConstructor(children, _) => {
                    tasks.push(Task::Finish(expression, children.len()));
                    for child in children.iter().rev() { tasks.push(Task::Eval(child)); }
                }
                ExprKind::Record(children) => {
                    tasks.push(Task::Finish(expression, children.len()));
                    for (_, child) in children.iter().rev() { tasks.push(Task::Eval(child)); }
                }
                _ => values.push(snapshot(expression, state, declarations)),
            },
            Task::Discard => { values.pop().unwrap(); }
            Task::Commit(target) => { commit(target, values.last().unwrap(), state); }
            Task::Kill(target) => { commit(target, &Value::default(), state); }
            Task::Finish(expression, count) => {
                values.truncate(values.len() - count);
                if matches!(expression.kind, ExprKind::Index(..) | ExprKind::Binary { .. } | ExprKind::Unary { .. } | ExprKind::Update { .. } | ExprKind::CompoundAssign { .. }) {
                    if observe_failures { failures.push(state.clone()); }
                }
                values.push(snapshot(expression, state, declarations));
            }
            Task::Call(expression, count) => {
                values.truncate(values.len() - count);
                let callee = values.pop().unwrap();
                fact(expression, callee, facts, declarations);
                if observe_failures { failures.push(state.clone()); }
                values.push(Value::default());
            }
            Task::Select(consequent, alternate) => {
                let predicate = values.pop().unwrap();
                match predicate.truth {
                    Some(true) => tasks.push(Task::Eval(consequent)),
                    Some(false) => tasks.push(Task::Eval(alternate)),
                    None => tasks.extend([Task::Between(alternate, state.clone()), Task::Eval(consequent)]),
                }
            }
            Task::Lazy(right, operator, destination) => {
                let left = values.pop().unwrap();
                let select = match operator {
                    BinaryOp::And => left.truth,
                    BinaryOp::Or => left.truth.map(|value| !value),
                    BinaryOp::Coalesce => left.nullish,
                    BinaryOp::PatternDefault => left.undefined,
                    _ => unreachable!(),
                };
                if select == Some(false) { values.push(left); continue; }
                if select.is_none() { tasks.push(Task::Join(state.clone(), left)); }
                if let Some(destination) = destination { tasks.push(Task::Commit(destination)); }
                tasks.push(Task::Eval(right));
            }
            Task::Between(alternate, saved) => {
                let left = values.pop().unwrap();
                let left_state = std::mem::replace(state, saved);
                tasks.extend([Task::Join(left_state, left), Task::Eval(alternate)]);
            }
            Task::Join(left_state, left) => {
                let right = values.pop().unwrap();
                join(state, &left_state, declarations);
                values.push(merge_value(left, right, declarations));
            }
        }
    }
    (values.pop().unwrap().contracts, failures)
}
