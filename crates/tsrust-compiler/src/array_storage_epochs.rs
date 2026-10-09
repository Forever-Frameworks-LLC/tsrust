//! Point-sensitive native carriers for checked dense local array slots.
use super::*;
use crate::capture_loan_owners::Place;

struct Storage {
    slots: Vec<Type>,
    current: Type,
    baseline: Type,
}

fn payload(mut value: &Expr) -> &Expr {
    while let ExprKind::Unary { op: UnaryOp::RecordCarrier, value: inner, .. } = &value.kind {
        value = inner;
    }
    value
}

impl Analyzer {
    #[inline(never)]
    pub(super) fn scoped_array_storage_epochs(&mut self, context: &mut Context, body: &mut [Stmt]) -> Result<(), Diagnostic> {
        if !context.bindings.iter().any(|binding|
            matches!(binding.ty, Type::Array(ArrayElement::Record(_, 0)))
                && self.aggregate_callable_facts(binding.ty).scoped)
        { return Ok(()); }
        let mut storage = HashMap::<usize, Storage>::new();
        for statement in body {
            let mut replacement = None;
            if let StmtKind::Assign { target, value, operator: None } = &statement.kind
                && let ExprKind::Index(base, index) = &target.kind
                && matches!(base.kind, ExprKind::Name(_))
                && let Some(binding) = base.binding
                && let ExprKind::Integer(index) = &index.kind
                && let Ok(index) = index.parse::<usize>()
                && let Some(state) = storage.get(&binding)
                && index < state.slots.len()
                && let Some(ty @ Type::Record(_)) = payload(value).ty
            {
                replacement = Some((binding, index, ty));
            }

            // Until control-flow slot joins are implemented, an opaque use or
            // nonlocal store restores the original carrier before leaving this
            // straight-line epoch. Existing global layouts remain authoritative.
            let mut opaque = !matches!(statement.kind, StmtKind::Let { .. } | StmtKind::Assign { .. } | StmtKind::Expr(_) | StmtKind::Return(_) | StmtKind::Throw(_));
            if let StmtKind::Assign { target, .. } = &statement.kind
                && Self::root_binding(target).is_some_and(|id| storage.contains_key(&id))
                && replacement.is_none()
            { opaque = true; }
            if let StmtKind::Let { value, .. } | StmtKind::Return(Some(value)) | StmtKind::Throw(value) = &statement.kind
                && matches!(value.kind, ExprKind::Name(_))
                && value.binding.is_some_and(|id| storage.contains_key(&id))
            { opaque = true; }
            let retaining = matches!(statement.kind, StmtKind::Let { .. } | StmtKind::Return(_) | StmtKind::Throw(_))
                || matches!(statement.kind, StmtKind::Assign { .. }) && replacement.is_none();
            crate::source_walk::body(std::slice::from_mut(statement), &mut |_| Ok(()), &mut |value| {
                if retaining && matches!(value.kind, ExprKind::Name(_))
                    && value.binding.is_some_and(|id| storage.contains_key(&id))
                { opaque = true; }
                if matches!(value.kind, ExprKind::Call(_, _))
                    && !matches!(value.target, Some(CallTarget::Log | CallTarget::StoredCallableField(_)))
                { opaque = true; }
                if matches!(value.kind, ExprKind::Assign { .. } | ExprKind::CompoundAssign { .. } | ExprKind::CoalesceAssign { .. } | ExprKind::LogicalAssign { .. }) {
                    opaque = true;
                }
                Ok(())
            })?;
            if opaque {
                for (&binding, state) in &storage {
                    if state.current != state.baseline {
                        context.storage_exits.record_array_epoch(statement.span, MapEntryEpoch {
                            place: Place { binding, fields: vec![] }, input: state.current,
                            output: state.baseline, empty: false, before: true,
                        });
                    }
                }
                storage.clear();
                replacement = None;
            }
            if let Some((binding, _, _)) = replacement {
                let state = storage.get_mut(&binding).unwrap();
                if state.current != state.baseline {
                    context.storage_exits.record_array_epoch(statement.span, MapEntryEpoch {
                        place: Place { binding, fields: vec![] }, input: state.current,
                        output: state.baseline, empty: false, before: true,
                    });
                    state.current = state.baseline;
                }
            }
            let plans = storage.iter().map(|(&id, state)| (id, state.current)).collect::<HashMap<_, _>>();
            crate::source_walk::body(std::slice::from_mut(statement), &mut |_| Ok(()), &mut |value| {
                if let Some(ty) = self.planned_record_array_type(value, &plans) { value.ty = Some(ty); }
                if let ExprKind::Call(callee, _) = &mut value.kind
                    && let Some(Type::BorrowedCallable(family)) = self.planned_record_array_type(callee, &plans)
                {
                    callee.ty = Some(Type::BorrowedCallable(family));
                    value.target = Some(CallTarget::StoredCallableField(family));
                }
                Ok(())
            })?;
            if let Some((binding, index, ty)) = replacement {
                let state = storage.get_mut(&binding).unwrap();
                state.slots[index] = ty;
                let mut records = state.slots.iter().copied();
                let mut joined = records.next().unwrap();
                for ty in records {
                    let (Type::Record(left), Type::Record(right)) = (joined, ty) else { unreachable!() };
                    joined = self.join_record_types(left, right, statement.span)?.expect("checked compatible array slots");
                }
                let output = Type::Array(ArrayElement::from_value_type(joined).expect("record array element"));
                if output != state.current {
                    context.storage_exits.record_array_epoch(statement.span, MapEntryEpoch {
                        place: Place { binding, fields: vec![] }, input: state.current,
                        output, empty: false, before: false,
                    });
                    state.current = output;
                }
            }
            if let StmtKind::Let { binding: Some(binding), value, .. } = &statement.kind
                && let ExprKind::Array(elements) = &payload(value).kind
                && !elements.is_empty()
                && matches!(context.bindings[*binding].ty, Type::Array(ArrayElement::Record(_, 0)))
                && self.aggregate_callable_facts(context.bindings[*binding].ty).scoped
                && !context.bindings[*binding].parameter && !context.bindings[*binding].capture_reference
            {
                let slots = elements.iter().map(|value| payload(value).ty).collect::<Option<Vec<_>>>();
                if let Some(slots) = slots && slots.iter().all(|ty| matches!(ty, Type::Record(_))) {
                    let baseline = context.bindings[*binding].ty;
                    storage.insert(*binding, Storage { slots, current: baseline, baseline });
                }
            }
        }
        Ok(())
    }
}
