//! Symbolic original payload ownership proves which call inputs are discarded.
use super::*;
use crate::capture_loan_owners::{Owners, Place};
use std::collections::BTreeSet;
type Payload = Vec<(Vec<String>, BTreeSet<usize>)>;
#[derive(Clone, Default)]
struct State {
    owners: Owners,
    escaped: BTreeSet<usize>,
}
impl State {
    fn join(&mut self, other: &Self) {
        self.owners.join(&other.owners);
        self.escaped.extend(&other.escaped);
    }
    fn store(&mut self, place: Place, payload: Payload) {
        self.owners.discard(&place);
        for (path, loans) in payload {
            let mut selected = place.clone();
            selected.fields.extend(path);
            self.owners.attach(selected, loans);
        }
    }
}
fn place(value: &Expr) -> Option<Place> {
    let binding = Analyzer::root_binding(value)?;
    let (_, fields) = Analyzer::direct_fields(value)?;
    Some(Place { binding, fields })
}
fn tokens(payload: &Payload) -> BTreeSet<usize> {
    payload
        .iter()
        .flat_map(|(_, loans)| loans.iter().copied())
        .collect()
}
struct Replay<'a> {
    unions: &'a crate::unions::UnionArena,
    functions: &'a [Option<CheckedFunction>],
    records: &'a [Record],
    closures: &'a [CheckedClosure],
    active: HashSet<usize>,
}
impl Replay<'_> {
    fn inputs(&self, function: &CheckedFunction) -> Vec<Place> {
        let mut result = Vec::new();
        for (id, binding) in function
            .bindings
            .iter()
            .take(function.parameter_count)
            .enumerate()
        {
            if binding.mode != Mode::Mutate {
                continue;
            }
            let mut pending = vec![(binding.input_ty.unwrap_or(binding.ty), Vec::new())];
            while let Some((ty, path)) = pending.pop() {
                match ty {
                    Type::Record(record) => {
                        for field in &self.records[record].fields {
                            let mut path = path.clone();
                            path.push(field.name.text.clone());
                            pending.push((field.ty.unwrap(), path));
                        }
                    }
                    Type::Union(id) | Type::BorrowedUnion(id) => pending.extend(
                        self.unions
                            .members(id)
                            .into_iter()
                            .map(|member| (member, path.clone())),
                    ),
                    Type::Callback(_)
                    | Type::Closure(_)
                    | Type::StoredCallable(_)
                    | Type::BorrowedCallable(_) => result.push(Place {
                        binding: id,
                        fields: path,
                    }),
                    _ => {}
                }
            }
        }
        result
    }
    fn released(&mut self, id: usize) -> Vec<Place> {
        let Some(function) = self.functions.get(id).and_then(Option::as_ref) else {
            return Vec::new();
        };
        // Exception payload and captured-owner transport have separate summaries.
        if function.template || has_exception_syntax(&function.body) || !self.active.insert(id) {
            return Vec::new();
        }
        let inputs = self.inputs(function);
        if inputs.is_empty() {
            self.active.remove(&id);
            return Vec::new();
        }
        // Defaults and new borrowed captures need argument-origin transport.
        if function.bindings[..function.parameter_count]
            .iter()
            .any(|binding| binding.omission.is_omittable())
        {
            self.active.remove(&id);
            return Vec::new();
        }
        let mut creates_borrow = false;
        crate::fixed_storage::visit_function_expressions(function, &mut |expr| {
            if matches!(expr.kind, ExprKind::Closure { .. })
                && let Some(Type::Closure(id)) = expr.ty
            {
                creates_borrow |= self.closures.get(id).is_none_or(|closure| {
                    closure.captures.iter().any(|capture| {
                        capture.storage != Mode::Consume
                            && function
                                .bindings
                                .get(capture.outer)
                                .is_some_and(|binding| binding.parameter && !binding.ty.is_copy())
                    })
                });
            }
        });
        if creates_borrow {
            self.active.remove(&id);
            return Vec::new();
        }
        let mut initial = State::default();
        for (token, place) in inputs.iter().enumerate() {
            initial.owners.attach(place.clone(), [token]);
        }
        let mut exits = Vec::new();
        if let Some(state) = self.body(function, &function.body, initial, &mut exits) {
            exits.push(state);
        }
        let mut retained = BTreeSet::new();
        for state in exits {
            retained.extend(state.escaped);
            for (id, binding) in function.bindings.iter().enumerate() {
                if id < function.parameter_count || binding.capture_reference {
                    retained.extend(state.owners.loans_under(&Place {
                        binding: id,
                        fields: Vec::new(),
                    }));
                }
            }
        }
        self.active.remove(&id);
        inputs
            .into_iter()
            .enumerate()
            .filter_map(|(token, place)| (!retained.contains(&token)).then_some(place))
            .collect()
    }
    fn body(
        &mut self,
        function: &CheckedFunction,
        body: &[Stmt],
        mut state: State,
        exits: &mut Vec<State>,
    ) -> Option<State> {
        for stmt in body {
            match &stmt.kind {
                StmtKind::Let {
                    binding: Some(id),
                    value,
                    ..
                } => {
                    let borrowed = function.bindings[*id].borrowed_place.is_some();
                    let payload = self.expression(function, value, &mut state, !borrowed);
                    state.store(
                        Place {
                            binding: *id,
                            fields: Vec::new(),
                        },
                        payload,
                    );
                }
                StmtKind::Assign {
                    target,
                    value,
                    operator: None,
                } => {
                    let payload = self.expression(function, value, &mut state, true);
                    if let Some(place) = place(target) {
                        state.store(place, payload);
                    } else {
                        state.escaped.extend(tokens(&payload));
                    }
                }
                StmtKind::Expr(value) => {
                    self.expression(function, value, &mut state, true);
                }
                StmtKind::Return(value) => {
                    if let Some(value) = value {
                        let payload = self.expression(function, value, &mut state, true);
                        state.escaped.extend(tokens(&payload));
                    }
                    exits.push(state);
                    return None;
                }
                StmtKind::Throw(_) => return None,
                StmtKind::Block(body) => {
                    state = self.body(function, body, state, exits)?;
                }
                StmtKind::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    self.expression(function, condition, &mut state, false);
                    let left = self.body(function, then_body, state.clone(), exits);
                    let right = self.body(function, else_body, state, exits);
                    state = match (left, right) {
                        (Some(mut left), Some(right)) => {
                            left.join(&right);
                            left
                        }
                        (Some(state), None) | (None, Some(state)) => state,
                        (None, None) => return None,
                    };
                }
                StmtKind::While { .. }
                | StmtKind::Loop { .. }
                | StmtKind::ForOf { .. }
                | StmtKind::Try(_) => {
                    // Loop/region completion requires the dedicated completion model.
                    state.escaped.extend(state.owners.live());
                }
                _ => {}
            }
        }
        Some(state)
    }
    fn expression(
        &mut self,
        function: &CheckedFunction,
        expr: &Expr,
        state: &mut State,
        consume: bool,
    ) -> Payload {
        if let Some(place) = place(expr) {
            if let ExprKind::Sequence(values) = &expr.kind {
                for value in &values[..values.len() - 1] {
                    self.expression(function, value, state, false);
                }
            }
            let payload = state.owners.payload(&place);
            if consume {
                state.owners.discard(&place);
            }
            return payload;
        }
        match &expr.kind {
            ExprKind::Record(fields) => fields
                .iter()
                .flat_map(|(key, value)| {
                    self.expression(function, value, state, consume)
                        .into_iter()
                        .map(|(mut path, loans)| {
                            path.insert(0, key.text.clone());
                            (path, loans)
                        })
                })
                .collect(),
            ExprKind::Closure { .. } => {
                let Some(Type::Closure(id)) = expr.ty else {
                    state.escaped.extend(state.owners.live());
                    return Vec::new();
                };
                let mut retained = BTreeSet::new();
                for capture in &self.closures[id].captures {
                    let place = Place {
                        binding: capture.outer,
                        fields: Vec::new(),
                    };
                    retained.extend(state.owners.loans_under(&place));
                    if capture.storage == Mode::Consume {
                        state.owners.discard(&place);
                    }
                }
                vec![(Vec::new(), retained)]
            }
            ExprKind::Unary { value, .. }
            | ExprKind::TypeAssertion { value, .. }
            | ExprKind::Project { value, .. }
            | ExprKind::Inject { value, .. }
            | ExprKind::OptionalChain(value) => self.expression(function, value, state, consume),
            ExprKind::Sequence(values) => {
                let (last, prefix) = values.split_last().expect("nonempty sequence");
                for value in prefix {
                    self.expression(function, value, state, false);
                }
                self.expression(function, last, state, consume)
            }
            ExprKind::Assign { target, value, .. } => {
                let payload = self.expression(function, value, state, true);
                if let Some(place) = place(target) {
                    state.store(place, payload);
                } else {
                    state.escaped.extend(tokens(&payload));
                }
                Vec::new()
            }
            ExprKind::Conditional {
                predicate,
                consequent,
                alternate,
            } => {
                self.expression(function, predicate, state, false);
                let mut other = state.clone();
                let mut left = self.expression(function, consequent, state, consume);
                let right = self.expression(function, alternate, &mut other, consume);
                state.join(&other);
                left.extend(right);
                left
            }
            ExprKind::Call(callee, args) => {
                let mut payload = self.expression(function, callee, state, false);
                let target = match expr.target {
                    Some(CallTarget::Function(id)) => Some(id),
                    _ => None,
                };
                for (index, arg) in args.iter().enumerate() {
                    let consumes = target
                        .and_then(|id| self.functions.get(id).and_then(Option::as_ref))
                        .and_then(|callee| callee.bindings.get(index))
                        .is_some_and(|binding| binding.mode == Mode::Consume);
                    payload.extend(self.expression(function, arg, state, consumes));
                }
                if let Some(id) = target {
                    for released in self.released(id) {
                        if let Some(Some(mut selected)) = args.get(released.binding).map(place) {
                            selected.fields.extend(released.fields);
                            state.owners.discard(&selected);
                        }
                    }
                } else {
                    // Unknown calls can retain or relocate an owned argument.
                    state.escaped.extend(tokens(&payload));
                }
                if expr.ty.is_some_and(|ty| {
                    matches!(
                        ty,
                        Type::Callback(_)
                            | Type::Closure(_)
                            | Type::StoredCallable(_)
                            | Type::BorrowedCallable(_)
                            | Type::Record(_)
                            | Type::Array(_)
                    )
                }) {
                    payload
                } else {
                    Vec::new()
                }
            }
            ExprKind::Binary {
                op: BinaryOp::Or | BinaryOp::And | BinaryOp::Coalesce | BinaryOp::PatternDefault,
                left,
                right,
                ..
            }
            | ExprKind::CoalesceAssign {
                target: left,
                value: right,
                ..
            }
            | ExprKind::LogicalAssign {
                target: left,
                value: right,
                ..
            }
            | ExprKind::Optional {
                base: left,
                body: right,
                ..
            } => {
                let mut payload = self.expression(function, left, state, false);
                let skipped = state.clone();
                payload.extend(self.expression(function, right, state, false));
                state.join(&skipped);
                state.escaped.extend(tokens(&payload));
                Vec::new()
            }
            ExprKind::Field(base, name) => self
                .expression(function, base, state, consume)
                .into_iter()
                .filter_map(|(mut path, loans)| {
                    if path.first() == Some(name) {
                        path.remove(0);
                        Some((path, loans))
                    } else if path.is_empty() {
                        Some((path, loans))
                    } else {
                        None
                    }
                })
                .collect(),
            ExprKind::SpreadCall(plan) => {
                let payload = plan
                    .expressions()
                    .flat_map(|value| self.expression(function, value, state, false))
                    .collect();
                state.escaped.extend(tokens(&payload));
                payload
            }
            ExprKind::Selected { source, .. } => {
                let mut payload = Vec::new();
                for root in &source.roots {
                    payload.extend(state.owners.payload(&Place {
                        binding: *root,
                        fields: Vec::new(),
                    }));
                }
                state.escaped.extend(tokens(&payload));
                payload
            }
            ExprKind::SpreadValue(selected) => {
                let mut payload = Vec::new();
                for root in &selected.source.roots {
                    payload.extend(state.owners.payload(&Place {
                        binding: *root,
                        fields: Vec::new(),
                    }));
                }
                state.escaped.extend(tokens(&payload));
                payload
            }
            ExprKind::Binary { left, right, .. }
            | ExprKind::Index(left, right)
            | ExprKind::CompoundAssign {
                target: left,
                value: right,
                ..
            } => {
                let mut payload = self.expression(function, left, state, false);
                payload.extend(self.expression(function, right, state, false));
                state.escaped.extend(tokens(&payload));
                Vec::new()
            }
            ExprKind::Array(values)
            | ExprKind::Template(values)
            | ExprKind::ArrayOf(values, _)
            | ExprKind::MapConstructor(values, _)
            | ExprKind::SetConstructor(values, _) => {
                let payload = values
                    .iter()
                    .flat_map(|value| self.expression(function, value, state, consume))
                    .collect();
                state.escaped.extend(tokens(&payload));
                payload
            }
            _ => Vec::new(),
        }
    }
}
pub(super) fn summaries(
    unions: &crate::unions::UnionArena,
    functions: &[Option<CheckedFunction>],
    records: &[Record],
    closures: &[CheckedClosure],
) -> HashMap<usize, Vec<(usize, Vec<String>)>> {
    let mut replay = Replay {
        unions,
        functions,
        records,
        closures,
        active: HashSet::new(),
    };
    (0..functions.len())
        .filter_map(|id| {
            let fields = replay
                .released(id)
                .into_iter()
                .map(|place| (place.binding, place.fields))
                .collect::<Vec<_>>();
            (!fields.is_empty()).then_some((id, fields))
        })
        .collect()
}
impl Analyzer {
    pub(super) fn release_record_call_captures(
        &self,
        context: &mut Context,
        id: usize,
        args: &[Expr],
    ) {
        let mut replay = Replay {
            unions: &self.unions,
            functions: &self.checked,
            records: &self.program.records,
            closures: &self.closures,
            active: HashSet::new(),
        };
        let released = replay.released(id);
        for released in released {
            let Some(Some(mut selected)) = args.get(released.binding).map(place) else {
                continue;
            };
            selected.fields.extend(released.fields);
            context.capture_loan_owners.discard(&selected);
        }
        Self::restore_live_capture_loans(context);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capture_free_field_store_discards_only_its_original_payload() {
        let checked=crate::check("type Holder={run:ReadOnly<Function<int>>;};function replace(holder:Mutable<Holder>):void{holder.run=():int=>11;}function main(){}").unwrap();
        let program = checked.program;
        let functions = program.functions.into_iter().map(Some).collect::<Vec<_>>();
        let unions = crate::unions::UnionArena::from_snapshot(program.unions.clone());
        let mut replay = Replay {
            unions: &unions,
            functions: &functions,
            records: &program.records,
            closures: &program.closures,
            active: HashSet::new(),
        };
        let id = functions
            .iter()
            .position(|function| function.as_ref().unwrap().name == "replace")
            .unwrap();
        assert_eq!(
            replay.released(id),
            vec![Place {
                binding: 0,
                fields: vec!["run".into()]
            }]
        );
    }
}
