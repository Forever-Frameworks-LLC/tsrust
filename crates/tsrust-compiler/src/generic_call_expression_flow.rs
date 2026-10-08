//! Iterative expression evaluation keeps callable values separate from bindings.
use super::*;
type Facts = HashMap<(usize, usize), Option<Vec<Contract>>>;
#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) struct Value {
    pub(super) alternatives: Option<Vec<environment_store::Alternative>>,
    pub(super) origins: Vec<body_origins::Origin>,
    pub(super) contracts: Option<Vec<Contract>>,
    truth: Option<bool>,
    nullish: Option<bool>,
    undefined: Option<bool>,
    pub(super) elements: Option<Vec<Element>>,
    pub(super) for_each: bool,
    pub(super) console: bool,
}
impl Value {
    #[cfg(test)]
    pub(super) fn from_origins(origins: Vec<body_origins::Origin>) -> Self { Self { origins: body_origins::canonical(origins), ..Self::default() } }
    pub(super) fn selection(&self) -> Selection { Selection { truth: self.truth, nullish: self.nullish, undefined: self.undefined } }
    pub(super) fn undefined() -> Self { Self { truth: Some(false), nullish: Some(true), undefined: Some(true), ..Self::default() } }
    pub(super) fn index(position: usize) -> Self { Self { truth: Some(position != 0), nullish: Some(false), undefined: Some(false), ..Self::default() } }
    pub(super) fn known_truth(&self) -> Option<bool> { self.truth }
}

impl Default for Value {
    fn default() -> Self {
        Self { alternatives: None, origins: vec![body_origins::Origin::Unknown], contracts: None, truth: None, nullish: None, undefined: None, elements: None, for_each: false, console: false }
    }
}

pub(super) fn merge_elements(left: Option<Vec<Element>>, right: Option<Vec<Element>>, declarations: &HashMap<String, Contract>) -> Option<Vec<Element>> {
    let (left, right) = (left?, right?);
    if left.len() != right.len() { return None; }
    Some(left.into_iter().zip(right).map(|(left,right)| {
        let value = merge_value(Value::element(&left), Value::element(&right), declarations);
        Element { alternatives: value.alternatives.clone(), origins: value.origins.clone(), contracts: value.contracts.clone(), selection: value.selection() }
    }).collect())
}
impl Value {
    pub(super) fn element(element: &Element) -> Self {
        Self { alternatives: element.alternatives.clone(), origins: element.origins.clone(), contracts: element.contracts.clone(), truth: element.selection.truth, nullish: element.selection.nullish, undefined: element.selection.undefined, elements: None, for_each: false, console: false }
    }
    pub(super) fn array(values: &[Value]) -> Self {
        Self { truth: Some(true), nullish: Some(false), undefined: Some(false), elements: Some(values.iter().map(|value| Element { alternatives: value.alternatives.clone(), origins: value.origins.clone(), contracts: value.contracts.clone(), selection: value.selection() }).collect()), ..Self::default() }
    }
}
pub(super) fn merge_value(left: Value, right: Value, declarations: &HashMap<String, Contract>) -> Value {
    let alternatives = environment_store::merge_alternatives(left.alternatives, &left.origins, right.alternatives, &right.origins);
    let contracts = match (left.contracts, right.contracts) {
        (Some(left), Some(right)) => resolve_all(left.into_iter().chain(right).map(Target::Contract).collect(), &[], declarations),
        _ => None,
    };
    Value { alternatives, origins: body_origins::canonical(left.origins.into_iter().chain(right.origins).collect()), contracts, truth: (left.truth == right.truth).then_some(left.truth).flatten(),
        nullish: (left.nullish == right.nullish).then_some(left.nullish).flatten(),
        undefined: (left.undefined == right.undefined).then_some(left.undefined).flatten(), elements: merge_elements(left.elements, right.elements, declarations), for_each: left.for_each && right.for_each, console: left.console && right.console }
}
fn snapshot(expression: &Expr, state: &[Binding], declarations: &HashMap<String, Contract>, effects: &effect_bodies::Index) -> Value {
    let contracts = if let Some((name, span)) = name(expression) && effects.binding(name, span).is_some() {
        binding_at(state, name, span, effects).and_then(|index| {
            state[index].contract.clone().map(|contract| vec![contract]).or_else(|| resolve_indexed(state[index].targets.clone(), state, declarations, effects))
        })
    } else { resolve_indexed(targets::value(expression), state, declarations, effects) };
    let (mut truth, mut nullish) = match &expression.kind {
        ExprKind::Boolean(value) => (Some(*value), Some(false)),
        ExprKind::Null | ExprKind::Undefined => (Some(false), Some(true)),
        ExprKind::Name(value) if value == "undefined" && effects.binding(value, expression.span).is_none() => (Some(false), Some(true)),
        ExprKind::String(value) => (Some(!value.is_empty()), Some(false)),
        ExprKind::Integer(value) => (value.parse::<i64>().ok().map(|value| value != 0), Some(false)),
        ExprKind::Number(value) => (Some(*value != 0.0 && !value.is_nan()), Some(false)),
        ExprKind::Array(_) | ExprKind::Record(_) => (Some(true), Some(false)),
        _ if contracts.is_some() && name(expression).is_some_and(|(name, span)| {
            binding_at(state, name, span, effects).is_none_or(|index| state[index].contract.is_none())
        }) => (Some(true), Some(false)),
        _ => (None, None),
    };
    let mut undefined = if matches!(expression.kind, ExprKind::Undefined) || matches!(&expression.kind, ExprKind::Name(value) if value == "undefined" && effects.binding(value, expression.span).is_none()) { Some(true) }
        else if nullish == Some(false) || matches!(expression.kind, ExprKind::Null) { Some(false) } else { None };
    if let Some((name, span)) = name(expression) && let Some(index) = binding_at(state, name, span, effects) {
        let selection = state[index].selection;
        truth = selection.truth.or(truth);
        nullish = selection.nullish.or(nullish);
        undefined = selection.undefined.or(undefined);
    }
    Value { alternatives: name(expression).and_then(|(name,span)| binding_at(state,name,span,effects)).and_then(|index|state[index].alternatives.clone()), origins: body_origins::resolve(body_origins::value(expression), state, declarations, effects), contracts, truth, nullish, undefined, elements: name(expression).and_then(|(name,span)| binding_at(state,name,span,effects)).and_then(|index|state[index].elements.clone()), for_each: false, console: false }
}
fn commit(target: &Expr, value: &Value, state: &mut [Binding], effects: &effect_bodies::Index) {
    if !matches!(target.kind, ExprKind::Name(_)) {
        // Array aliases have no storage identity yet; a write invalidates all
        // element snapshots rather than retaining stale callable provenance.
        for binding in state.iter_mut() { binding.elements = None; }
    }
    if let ExprKind::Name(name) = &target.kind
        && let Some(index) = binding_at(state, name, target.span, effects)
    {
        state[index].alternatives = value.alternatives.clone();
        state[index].origins = body_origins::canonical(value.origins.clone());
        state[index].selection = value.selection();
        state[index].elements = value.elements.clone();
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

pub(super) struct Invocation {
    pub normal: Option<FlowState>,
    pub throwing: Vec<FlowState>,
    pub result: Value,
    pub facts: Facts,
}
pub(super) trait Dispatch {
    fn reject(&mut self) {}
    fn capture(&mut self, _: &Expr, _: &mut Value, _: &mut FlowState) {}
    fn validate_escape(&mut self, _: &Value, _: &FlowState) -> bool {true}
    fn invoke(&mut self, expression: &Expr, callee: &Value, arguments: &[Value], state: &FlowState) -> Option<Invocation>;
}
pub(super) struct UnknownDispatch;
impl Dispatch for UnknownDispatch {
    fn invoke(&mut self, _: &Expr, _: &Value, _: &[Value], _: &FlowState) -> Option<Invocation> { None }
}
pub(super) struct Evaluation {
    pub value: Value,
    pub failures: Vec<FlowState>,
    pub normal: bool,
}

enum Task<'a> {
    Eval(&'a Expr),
    Finish(&'a Expr, usize),
    Call(&'a Expr, usize),
    Commit(&'a Expr),
    Kill(&'a Expr),
    Select(&'a Expr, &'a Expr),
    Lazy(&'a Expr, BinaryOp, Option<&'a Expr>),
    Between(&'a Expr, FlowState),
    Join(FlowState, Value),
    Discard,
}

/// Returns the evaluated value snapshot and environments of possible failures.
#[cfg(test)]
pub(super) fn evaluate(expression: &Expr, state: &mut FlowState, facts: &mut Facts,
    declarations: &HashMap<String, Contract>, observe_failures: bool, effects: &effect_bodies::Index) -> (Value, Vec<FlowState>) {
    let evaluation = evaluate_with_dispatch(expression, state, facts, declarations, observe_failures, effects, &mut UnknownDispatch);
    (evaluation.value, evaluation.failures)
}
pub(super) fn evaluate_with_dispatch(expression: &Expr, state: &mut FlowState, facts: &mut Facts, declarations: &HashMap<String, Contract>, observe_failures: bool, effects: &effect_bodies::Index, dispatch: &mut dyn Dispatch) -> Evaluation {
    let mut normal = true;
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
                _ => {
                    let mut value = snapshot(expression, state, declarations, effects);
                    dispatch.capture(expression, &mut value, state);
                    values.push(value);
                }
            },
            Task::Discard => { values.pop().unwrap(); }
            Task::Commit(target) => { commit(target, values.last().unwrap(), state, effects); }
            Task::Kill(target) => { commit(target, &Value::default(), state, effects); }
            Task::Finish(expression, count) => {
                let preserved = if count == 1 && matches!(expression.kind, ExprKind::TypeAssertion { .. } | ExprKind::Unary { op: UnaryOp::NonNull | UnaryOp::OptionalSelected, .. }) {
                    values.last().map(|value| value.origins.clone())
                } else { None };
                let operands = values.split_off(values.len() - count);
                if matches!(expression.kind, ExprKind::Index(..) | ExprKind::Binary { .. } | ExprKind::Unary { .. } | ExprKind::Update { .. } | ExprKind::CompoundAssign { .. }) {
                    if observe_failures { failures.push(state.clone()); }
                }
                let mut value = snapshot(expression, state, declarations, effects);
                if let Some(origins) = preserved { value.origins = origins; value.alternatives = operands[0].alternatives.clone(); value.elements = operands[0].elements.clone(); }
                value.console = leaf_effects::console(expression, effects);
                match &expression.kind {
                    ExprKind::Field(_, method) if method == "forEach" && operands[0].elements.is_some() => { value.elements = operands[0].elements.clone(); value.for_each = true; }
                    ExprKind::Array(_) => value = Value::array(&operands),
                    ExprKind::Index(_, index) => {
                        value = if let ExprKind::Integer(index) = &index.kind { index.parse::<usize>().ok().and_then(|index| operands[0].elements.as_ref()?.get(index)).map(Value::element).unwrap_or_default() } else { Value::default() };
                    }
                    _ => {}
                }
                values.push(value);
            }
            Task::Call(expression, count) => {
                let arguments = values.split_off(values.len() - count);
                let callee = values.pop().unwrap();
                fact(expression, callee.clone(), facts, declarations);
                if let Some(invocation) = dispatch.invoke(expression, &callee, &arguments, state) {
                    for (key, candidates) in invocation.facts {
                        let merged = if let Some(previous) = facts.get(&key) {
                            match (previous, candidates) {
                                (Some(previous), Some(candidates)) => resolve_all(previous.iter().cloned().chain(candidates).map(Target::Contract).collect(), &[], declarations),
                                _ => None,
                            }
                        } else { candidates };
                        facts.insert(key, merged);
                    }
                    if observe_failures { failures.extend(invocation.throwing); }
                    if let Some(outgoing) = invocation.normal {
                        *state = outgoing;
                        values.push(invocation.result);
                    } else {
                        normal = false;
                        tasks.clear();
                        values.clear();
                        values.push(Value::default());
                    }
                } else {
                    for binding in state.iter_mut() { binding.elements = None; if binding.mutable { binding.selection = Selection::default(); } }
                    if observe_failures { failures.push(state.clone()); }
                    values.push(Value::default());
                }
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
                if state.join(&left_state,declarations).is_none() {dispatch.reject();for fact in facts.values_mut(){*fact=None;}}
                values.push(merge_value(left, right, declarations));
            }
        }
    }
    Evaluation { value: values.pop().unwrap(), failures, normal }
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
        let mut bindings=Vec::new();collect(&function.body,&mut bindings,&effects);let mut bindings:FlowState=bindings.into();
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

#[cfg(test)]
mod dispatch_tests {
    use super::*;
    struct ThrowingWriter { saved: Span, replacement: Span, seen: bool }
    impl Dispatch for ThrowingWriter {
        fn invoke(&mut self, _: &Expr, callee: &Value, arguments: &[Value], state: &FlowState) -> Option<Invocation> {
            assert_eq!(callee.origins,vec![body_origins::Origin::Body(self.saved)]);
            assert_eq!(arguments.len(),1);
            self.seen=true;
            let mut outgoing=state.clone();
            let shared=outgoing.iter_mut().find(|binding|binding.name=="shared").unwrap();
            shared.origins=vec![body_origins::Origin::Body(self.replacement)];
            Some(Invocation { normal:None, throwing:vec![outgoing], result:Value::default(), facts:HashMap::new() })
        }
    }
    #[test]
    fn dispatcher_receives_saved_callee_after_arguments_and_throw_stops_remaining_expression() {
        let source=crate::format("function one(value:int):int{return value;}function two(value:int):int{return value+1;}function main(){let action=one;let shared=one;const result=(action((action=two,0)),action=one);}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let program=crate::syntax::parse(&source).unwrap();let effects=effect_bodies::Index::new(&program).unwrap();
        let declarations=program.functions.iter().map(|function|(function.name.text.clone(),contract(0,&function.parameter_initializers,function.rest_parameter,function.name.span))).collect::<HashMap<_,_>>();
        let main=program.functions.iter().find(|function|function.name.text=="main").unwrap();
        let one=program.functions.iter().find(|function|function.name.text=="one").unwrap().name.span;
        let two=program.functions.iter().find(|function|function.name.text=="two").unwrap().name.span;
        let mut state=Vec::new();collect(&main.body,&mut state,&effects);let mut state:FlowState=state.into();
        let StmtKind::Let {value,..}=&main.body[2].kind else {panic!("result")};
        let mut dispatch=ThrowingWriter { saved:one,replacement:two,seen:false };
        let result=evaluate_with_dispatch(value,&mut state,&mut HashMap::new(),&declarations,true,&effects,&mut dispatch);
        assert!(dispatch.seen);assert!(!result.normal);assert_eq!(result.failures.len(),1);
        assert_eq!(result.failures[0].iter().find(|binding|binding.name=="shared").unwrap().origins,vec![body_origins::Origin::Body(two)]);
        assert_eq!(state.iter().find(|binding|binding.name=="action").unwrap().origins,vec![body_origins::Origin::Body(two)]);
    }
}

#[cfg(test)]
mod opaque_selection_tests {
    use super::*;
    #[test]
    fn opaque_call_does_not_preserve_mutable_guard_assumptions() {
        let source=crate::format("function opaque(value:boolean){}function main(){let flag=false;opaque(flag);}").unwrap();
        let program=crate::syntax::parse(&source).unwrap();let effects=effect_bodies::Index::new(&program).unwrap();
        let declarations=program.functions.iter().map(|function|(function.name.text.clone(),contract(0,&function.parameter_initializers,function.rest_parameter,function.name.span))).collect::<HashMap<_,_>>();
        let main=program.functions.iter().find(|function|function.name.text=="main").unwrap();
        let mut state=Vec::new();collect(&main.body,&mut state,&effects);let mut state:FlowState=state.into();
        state[0].selection.truth=Some(false);
        let StmtKind::Expr(call)=&main.body[1].kind else {panic!("call")};
        evaluate(call,&mut state,&mut HashMap::new(),&declarations,false,&effects);
        assert_eq!(state[0].selection.truth,None);
    }
}

#[cfg(test)]
mod environment_transport_tests {
    use super::*;
    #[test]
    fn correlated_environments_survive_aliases_assertions_arrays_and_value_joins() {
        let source = crate::format("function one():int{return 1;}function main(){let action=one;const saved=action;const asserted=action as Function<int>;const list=[action];const indexed=list[0];}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let program=crate::syntax::parse(&source).unwrap();
        let effects=effect_bodies::Index::new(&program).unwrap();
        let declarations=program.functions.iter().map(|function|(function.name.text.clone(),contract(0,&function.parameter_initializers,function.rest_parameter,function.name.span))).collect::<HashMap<_,_>>();
        let main=program.functions.iter().find(|function|function.name.text=="main").unwrap();
        let one=program.functions.iter().find(|function|function.name.text=="one").unwrap().name.span;
        let mut state=Vec::new();collect(&main.body,&mut state,&effects);let mut state:FlowState=state.into();
        let mut store=environment_store::Store::default();
        let first=store.allocate(Span::new(900,901),Span::new(902,903),&[]).unwrap();
        let second=store.allocate(Span::new(904,905),Span::new(902,903),&[]).unwrap();
        let pairs=vec![environment_store::Alternative::Body{declaration:one,environment:Some(first)}];
        state[0].alternatives=Some(pairs.clone());
        let mut facts=HashMap::new();
        for statement in &main.body[1..] {
            let StmtKind::Let{name,value,..}=&statement.kind else {panic!("declaration")};
            let (value,_)=evaluate(value,&mut state,&mut facts,&declarations,false,&effects);
            if name.text=="list" {assert_eq!(value.elements.as_ref().unwrap()[0].alternatives,Some(pairs.clone()));}
            else {assert_eq!(value.alternatives,Some(pairs.clone()),"{}",name.text);}
            let destination=state.iter().position(|binding|binding.name==name.text).unwrap();
            state[destination].origins=value.origins;state[destination].alternatives=value.alternatives;state[destination].elements=value.elements;
        }
        let left=Value{alternatives:Some(pairs.clone()),origins:vec![body_origins::Origin::Body(one)],..Value::default()};
        let right=Value{alternatives:Some(vec![environment_store::Alternative::Body{declaration:one,environment:Some(second)}]),origins:vec![body_origins::Origin::Body(one)],..Value::default()};
        let merged=merge_value(left.clone(),right,&declarations);
        assert_eq!(merged.alternatives.as_ref().unwrap().len(),2);
        assert!(merged.alternatives.as_ref().unwrap().contains(&pairs[0]));
        let unknown=merge_value(left,Value::default(),&declarations);
        assert!(unknown.alternatives.as_ref().unwrap().contains(&environment_store::Alternative::Unknown));
    }
}
