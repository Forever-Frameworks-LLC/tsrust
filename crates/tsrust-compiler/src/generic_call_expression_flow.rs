//! Iterative expression evaluation keeps callable values separate from bindings.
use super::*;
type Facts = HashMap<(usize, usize), Option<Vec<Contract>>>;
#[derive(Clone)]
pub(super) struct Value {
    pub(super) origins: Vec<body_origins::Origin>,
    pub(super) contracts: Option<Vec<Contract>>,
    truth: Option<bool>,
    nullish: Option<bool>,
    undefined: Option<bool>,
}
impl Value {
    pub(super) fn known_truth(&self) -> Option<bool> { self.truth }
}

impl Default for Value {
    fn default() -> Self {
        Self { origins: vec![body_origins::Origin::Unknown], contracts: None, truth: None, nullish: None, undefined: None }
    }
}

fn merge_value(left: Value, right: Value, declarations: &HashMap<String, Contract>) -> Value {
    let contracts = match (left.contracts, right.contracts) {
        (Some(left), Some(right)) => resolve_all(left.into_iter().chain(right).map(Target::Contract).collect(), &[], declarations),
        _ => None,
    };
    Value { origins: body_origins::canonical(left.origins.into_iter().chain(right.origins).collect()), contracts, truth: (left.truth == right.truth).then_some(left.truth).flatten(),
        nullish: (left.nullish == right.nullish).then_some(left.nullish).flatten(),
        undefined: (left.undefined == right.undefined).then_some(left.undefined).flatten() }
}
fn snapshot(expression: &Expr, state: &[Binding], declarations: &HashMap<String, Contract>, effects: &effect_bodies::Index) -> Value {
    let contracts = if let Some((name, span)) = name(expression) && effects.binding(name, span).is_some() {
        binding_at(state, name, span, effects).and_then(|index| {
            state[index].contract.clone().map(|contract| vec![contract]).or_else(|| resolve_indexed(state[index].targets.clone(), state, declarations, effects))
        })
    } else { resolve_indexed(targets::value(expression), state, declarations, effects) };
    let (truth, nullish) = match &expression.kind {
        ExprKind::Boolean(value) => (Some(*value), Some(false)),
        ExprKind::Null | ExprKind::Undefined => (Some(false), Some(true)),
        ExprKind::String(value) => (Some(!value.is_empty()), Some(false)),
        ExprKind::Integer(value) => (value.parse::<i64>().ok().map(|value| value != 0), Some(false)),
        ExprKind::Number(value) => (Some(*value != 0.0 && !value.is_nan()), Some(false)),
        ExprKind::Array(_) | ExprKind::Record(_) => (Some(true), Some(false)),
        _ if contracts.is_some() && name(expression).is_some_and(|(name, span)| {
            binding_at(state, name, span, effects).is_none_or(|index| state[index].contract.is_none())
        }) => (Some(true), Some(false)),
        _ => (None, None),
    };
    let undefined = if matches!(expression.kind, ExprKind::Undefined) { Some(true) }
        else if nullish == Some(false) || matches!(expression.kind, ExprKind::Null) { Some(false) } else { None };
    Value { origins: body_origins::resolve(body_origins::value(expression), state, declarations, effects), contracts, truth, nullish, undefined }
}
fn commit(target: &Expr, value: &Value, state: &mut [Binding], effects: &effect_bodies::Index) {
    if let ExprKind::Name(name) = &target.kind
        && let Some(index) = binding_at(state, name, target.span, effects)
    {
        state[index].origins = body_origins::canonical(value.origins.clone());
        if state[index].contract.is_some() { return; }
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
    declarations: &HashMap<String, Contract>, observe_failures: bool, effects: &effect_bodies::Index) -> (Value, Vec<Vec<Binding>>) {
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
                _ => values.push(snapshot(expression, state, declarations, effects)),
            },
            Task::Discard => { values.pop().unwrap(); }
            Task::Commit(target) => { commit(target, values.last().unwrap(), state, effects); }
            Task::Kill(target) => { commit(target, &Value::default(), state, effects); }
            Task::Finish(expression, count) => {
                let preserved = if count == 1 && matches!(expression.kind, ExprKind::TypeAssertion { .. } | ExprKind::Unary { op: UnaryOp::NonNull | UnaryOp::OptionalSelected, .. }) {
                    values.last().map(|value| value.origins.clone())
                } else { None };
                values.truncate(values.len() - count);
                if matches!(expression.kind, ExprKind::Index(..) | ExprKind::Binary { .. } | ExprKind::Unary { .. } | ExprKind::Update { .. } | ExprKind::CompoundAssign { .. }) {
                    if observe_failures { failures.push(state.clone()); }
                }
                let mut value = snapshot(expression, state, declarations, effects);
                if let Some(origins) = preserved { value.origins = origins; }
                values.push(value);
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
    (values.pop().unwrap(), failures)
}

#[cfg(test)]
mod provenance_tests {
    use super::*;
    #[test]
    fn annotated_assignment_and_assertion_preserve_saved_body_snapshots() {
        let source=crate::format("function one():int{return 1;}function two():int{return 2;}function inspect<T>(flag:boolean){let action:Function<int>=one;const saved=action;const change=(action=two) as Function<int>;const selected=flag?saved:action;}function main(){}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let program=crate::syntax::parse(&source).unwrap();
        let effects=effect_bodies::Index::new(&program).unwrap();
        let declarations=program.functions.iter().map(|function|(function.name.text.clone(),contract(0,&function.parameter_initializers,function.rest_parameter,function.name.span))).collect::<HashMap<_,_>>();
        let function=program.functions.iter().find(|function|function.name.text=="inspect").unwrap();
        let one=program.functions.iter().find(|function|function.name.text=="one").unwrap().name.span;
        let two=program.functions.iter().find(|function|function.name.text=="two").unwrap().name.span;
        let mut bindings=Vec::new();collect(&function.body,&mut bindings,&effects);
        let mut facts=HashMap::new();
        for statement in &function.body {
            let StmtKind::Let {name,value,..}=&statement.kind else {continue};
            let (snapshot,_) = evaluate(value,&mut bindings,&mut facts,&declarations,false,&effects);
            let expected = match name.text.as_str() {
                "action" | "saved" => vec![body_origins::Origin::Body(one)],
                "change" => vec![body_origins::Origin::Body(two)],
                "selected" => vec![body_origins::Origin::Body(one),body_origins::Origin::Body(two)],
                _=>unreachable!(),
            };
            assert_eq!(snapshot.origins,body_origins::canonical(expected),"{}",name.text);
            let destination=bindings.iter().position(|binding|binding.name==name.text).unwrap();
            bindings[destination].origins=snapshot.origins;
        }
        assert_eq!(bindings.iter().find(|binding|binding.name=="action").unwrap().origins,vec![body_origins::Origin::Body(two)]);
        assert_eq!(bindings.iter().find(|binding|binding.name=="saved").unwrap().origins,vec![body_origins::Origin::Body(one)]);
    }
    #[test]
    fn uncertain_value_join_retains_known_body_and_unknown_alternative() {
        let body=body_origins::Origin::Body(Span::new(4,9));
        let left=Value {origins:vec![body.clone()],..Value::default()};
        let right=Value::default();
        let joined=merge_value(left,right,&HashMap::new());
        assert_eq!(joined.origins,vec![body_origins::Origin::Unknown,body]);
    }
}
