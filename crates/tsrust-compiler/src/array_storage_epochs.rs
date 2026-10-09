//! Point-sensitive native carriers for checked dense local array slots.
use super::*;
use crate::capture_loan_owners::Place;

#[derive(Clone)]
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
            matches!(binding.ty, Type::Record(_) | Type::Array(ArrayElement::Record(_, 0)))
                && self.aggregate_callable_facts(binding.ty).scoped)
        { return Ok(()); }
        let mut storage = std::collections::BTreeMap::<Place, Storage>::new();
        for statement in body {
            let mut moved_slots = Vec::new();
            if let StmtKind::Let { binding: Some(destination), value, .. } = &statement.kind
                && context.bindings[*destination].borrowed_place.is_none()
                && let Some((_, fields)) = Self::direct_fields(payload(value))
                && let Some(binding) = Self::root_binding(payload(value))
            {
                for (place, state) in &storage {
                    if place.binding == binding && place.fields.starts_with(&fields) {
                        let path = place.fields[fields.len()..].to_vec();
                        if let Some(baseline @ Type::Array(ArrayElement::Record(_, 0))) = self.scoped_array_field_type(context.bindings[*destination].ty, &path) {
                            moved_slots.push((Place { binding: *destination, fields: path }, Storage {
                                slots: state.slots.clone(), current: baseline, baseline,
                            }));
                        }
                    }
                }
            }
            let mut replacement = None;
            let removal = if let StmtKind::Expr(value) = &statement.kind
                && matches!(value.target, Some(CallTarget::ArrayPop | CallTarget::ArrayShift))
                && let ExprKind::Call(callee, arguments) = &value.kind
                && arguments.is_empty()
                && let ExprKind::Field(base, _) = &callee.kind
                && let Some((_, fields)) = Self::direct_fields(base)
                && let Some(binding) = Self::root_binding(base)
                && storage.contains_key(&Place { binding, fields: fields.clone() })
            { Some((Place { binding, fields }, value.target == Some(CallTarget::ArrayShift), value.span)) }
            else { None };
            if let StmtKind::Assign { target, value, operator: None } = &statement.kind
                && let ExprKind::Index(base, index) = &target.kind
                && let Some((_, fields)) = Self::direct_fields(base)
                && let Some(binding) = Self::root_binding(base)
                && let place = (Place { binding, fields })
                && let ExprKind::Integer(index) = &index.kind
                && let Ok(index) = index.parse::<usize>()
                && let Some(state) = storage.get(&place)
                && index < state.slots.len()
                && let Some(ty @ Type::Record(_)) = payload(value).ty
            {
                replacement = Some((place, index, ty));
            }

            // Until control-flow slot joins are implemented, an opaque use or
            // nonlocal store restores the original carrier before leaving this
            // straight-line epoch. Existing global layouts remain authoritative.
            let mut opaque = !matches!(statement.kind, StmtKind::Let { .. } | StmtKind::Assign { .. } | StmtKind::Expr(_) | StmtKind::Return(_) | StmtKind::Throw(_));
            if let StmtKind::Assign { target, .. } = &statement.kind
                && Self::root_binding(target).is_some_and(|id| storage.keys().any(|place| place.binding == id))
                && replacement.is_none()
            { opaque = true; }
            if let StmtKind::Let { value, .. } | StmtKind::Return(Some(value)) | StmtKind::Throw(value) = &statement.kind
                && matches!(value.kind, ExprKind::Name(_))
                && value.binding.is_some_and(|id| storage.keys().any(|place| place.binding == id))
            { opaque = true; }
            let retaining = matches!(statement.kind, StmtKind::Let { .. } | StmtKind::Return(_) | StmtKind::Throw(_))
                || matches!(statement.kind, StmtKind::Assign { .. }) && replacement.is_none();
            crate::source_walk::body(std::slice::from_mut(statement), &mut |_| Ok(()), &mut |value| {
                if retaining && matches!(value.kind, ExprKind::Name(_))
                    && value.binding.is_some_and(|id| storage.keys().any(|place| place.binding == id))
                { opaque = true; }
                if matches!(value.kind, ExprKind::Call(_, _))
                    && !matches!(value.target, Some(CallTarget::Log | CallTarget::StoredCallableField(_)))
                    && !removal.as_ref().is_some_and(|(_, _, span)| *span == value.span)
                { opaque = true; }
                if matches!(value.kind, ExprKind::Assign { .. } | ExprKind::CompoundAssign { .. } | ExprKind::CoalesceAssign { .. } | ExprKind::LogicalAssign { .. }) {
                    opaque = true;
                }
                Ok(())
            })?;
            if opaque {
                self.restore_array_storage_baselines(context, &mut storage, statement.span, None)?;
                storage.clear();
                replacement = None;
            }
            if let Some((place, _, _)) = &replacement {
                self.restore_array_storage_baselines(context, &mut storage, statement.span, Some(place))?;
            }
            let plans = self.array_epoch_root_types(context, &storage);
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
            let update = replacement.map(|(place, index, ty)| (place, Some((index, ty)), false))
                .or_else(|| removal.map(|(place, front, _)| (place, None, front)));
            if let Some((place, replacement, front)) = update && storage.contains_key(&place) {
                let roots = self.array_epoch_root_types(context, &storage);
                let state = storage.get_mut(&place).unwrap();
                if let Some((index, ty)) = replacement { state.slots[index] = ty; }
                else if !state.slots.is_empty() {
                    if front { state.slots.remove(0); } else { state.slots.pop(); }
                }
                let mut records = state.slots.iter().copied();
                let mut joined = if let Some(record) = records.next() { record } else {
                    let Type::Array(ArrayElement::Record(record, _)) = state.baseline else { unreachable!() };
                    self.empty_scoped_record_environment(self.scoped_records.declaration(record))?
                };
                for ty in records {
                    let (Type::Record(left), Type::Record(right)) = (joined, ty) else { unreachable!() };
                    joined = self.join_record_types(left, right, statement.span)?.expect("checked compatible array slots");
                }
                let output = Type::Array(ArrayElement::from_value_type(joined).expect("record array element"));
                if output != state.current {
                    context.storage_exits.record_array_epoch(statement.span, MapEntryEpoch {
                        place: place.clone(), input: roots[&place.binding],
                        output: self.scoped_array_replace_field(roots[&place.binding], &place.fields, output).expect("checked array field"), empty: state.slots.is_empty(), before: false,
                    });
                    state.current = output;
                }
            }
            if let StmtKind::Let { binding: Some(binding), value, .. } = &statement.kind
                && !context.bindings[*binding].parameter && !context.bindings[*binding].capture_reference
            {
                let mut pending = vec![(value, Vec::<String>::new())];
                while let Some((value, fields)) = pending.pop() {
                    match &payload(value).kind {
                        ExprKind::Record(values) => for (name, value) in values {
                            let mut path = fields.clone(); path.push(name.text.clone()); pending.push((value, path));
                        },
                        ExprKind::Array(elements) if !elements.is_empty() => {
                            let Some(baseline @ Type::Array(ArrayElement::Record(_, 0))) = self.scoped_array_field_type(context.bindings[*binding].ty, &fields) else { continue; };
                            if !self.aggregate_callable_facts(baseline).scoped { continue; }
                            let slots = elements.iter().map(|value| payload(value).ty).collect::<Option<Vec<_>>>();
                            if let Some(slots) = slots && slots.iter().all(|ty| matches!(ty, Type::Record(_))) {
                                storage.insert(Place { binding: *binding, fields }, Storage { slots, current: baseline, baseline });
                            }
                        }
                        _ => {},
                    }
                }
            }
            storage.extend(moved_slots);
        }
        Ok(())
    }
    fn array_epoch_root_types(&mut self, context: &Context, storage: &std::collections::BTreeMap<Place, Storage>) -> HashMap<usize, Type> {
        let mut roots = HashMap::new();
        for (place, state) in storage {
            let root = roots.entry(place.binding).or_insert(context.bindings[place.binding].ty);
            *root = self.scoped_array_replace_field(*root, &place.fields, state.current).expect("checked array storage path");
        }
        roots
    }

    fn restore_array_storage_baselines(&mut self, context: &mut Context, storage: &mut std::collections::BTreeMap<Place, Storage>, span: Span, selected: Option<&Place>) -> Result<(), Diagnostic> {
        let places = storage.keys().filter(|place| selected.is_none_or(|selected| selected == *place)).cloned().collect::<Vec<_>>();
        for place in places {
            let roots = self.array_epoch_root_types(context, storage);
            let state = storage.get_mut(&place).unwrap();
            if state.current != state.baseline {
                let input = roots[&place.binding];
                let output = self.scoped_array_replace_field(input, &place.fields, state.baseline).expect("checked array field");
                context.storage_exits.record_array_epoch(span, MapEntryEpoch { place, input, output, empty: state.slots.is_empty(), before: true });
                state.current = state.baseline;
            }
        }
        Ok(())
    }

}
