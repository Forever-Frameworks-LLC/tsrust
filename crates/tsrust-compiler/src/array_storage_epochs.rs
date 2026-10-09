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
        super::discarded_sequence::lower(body, &mut context.storage_exits);
        let mut results = HashMap::new();
        let eligible = super::array_result_storage::eligible_bindings(context, body);
        super::array_result_storage::reject_unplanned_replacements(context, body, &eligible)?;
        self.plan_array_storage_body(context, body, &mut std::collections::BTreeMap::new(), &mut results, &eligible)?;
        Ok(())
    }

    #[inline(never)]
    fn plan_array_storage_body(&mut self, context: &mut Context, body: &mut [Stmt], storage: &mut std::collections::BTreeMap<Place, Storage>, results: &mut HashMap<usize, Type>, eligible: &HashSet<usize>) -> Result<(), Diagnostic> {
        for statement in body {
            if statement.labels.is_empty() && let StmtKind::Block(inner) = &mut statement.kind
                && array_normal_block(inner)
            {
                let mut incoming_results = results.keys().copied().collect::<Vec<_>>();
                let incoming = storage.keys().cloned().collect::<Vec<_>>();
                self.plan_array_storage_body(context, inner, storage, results, eligible)?;
                if let Some(binding)=context.storage_exits.sequence_export(statement.span.start) { incoming_results.push(binding); }
                let roots = self.array_epoch_root_types(context, storage);
                let mut epochs = incoming.iter().filter_map(|place| {
                    let state = storage.get(place)?;
                    if context.storage_exits.transfer_at_exit(inner, place.binding, &place.fields).is_some() { return None; }
                    Some(MapEntryEpoch { place: place.clone(), input: roots[&place.binding], output: roots[&place.binding], empty: state.slots.is_empty(), before: false })
                }).collect::<Vec<_>>();
                for &binding in &incoming_results {
                    if context.storage_exits.transfer_at_exit(inner, binding, &[]).is_none() && let Some(&ty) = results.get(&binding) {
                        epochs.push(MapEntryEpoch { place: Place { binding, fields: vec![] }, input: ty, output: ty, empty: false, before: false });
                    }
                }
                results.retain(|binding, _| incoming_results.contains(binding) && context.storage_exits.transfer_at_exit(inner, *binding, &[]).is_none());
                context.storage_exits.record_array_block_epochs(statement.span.start, epochs);
                storage.retain(|place, _| incoming.contains(place));
                continue;
            }
            self.retag_array_result_uses(std::slice::from_mut(statement), results)?;
            if let StmtKind::Let { value, .. } | StmtKind::Assign { value, .. } = &mut statement.kind {
                super::array_result_storage::unwrap_removal(value);
            }
            let mut moved_slots = Vec::new();
            if let StmtKind::Let { binding: Some(destination), value, .. } = &statement.kind
                && context.bindings[*destination].borrowed_place.is_none()
                && let Some((_, fields)) = Self::direct_fields(payload(value))
                && let Some(binding) = Self::root_binding(payload(value))
            {
                for (place, state) in storage.iter() {
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
            let removal_value = match &statement.kind { StmtKind::Expr(value) | StmtKind::Let { value, .. } => Some(value), StmtKind::Assign { target, value, operator: None } if target.binding.is_some_and(|id| results.contains_key(&id)) => Some(value), _ => None };
            let removal = if let Some(value) = removal_value
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
            let retaining = removal.is_none() && (matches!(statement.kind, StmtKind::Let { .. } | StmtKind::Return(_) | StmtKind::Throw(_))
                || matches!(statement.kind, StmtKind::Assign { .. }) && replacement.is_none());
            let mut mentions_storage = false;
            let mut effect = false;
            crate::source_walk::body(std::slice::from_mut(statement), &mut |_| Ok(()), &mut |value| {
                if matches!(value.kind, ExprKind::Name(_)) && value.binding.is_some_and(|id| storage.keys().any(|place| place.binding == id)) { mentions_storage = true; }
                if retaining && matches!(value.kind, ExprKind::Name(_))
                    && value.binding.is_some_and(|id| storage.keys().any(|place| place.binding == id))
                { opaque = true; }
                if matches!(value.kind, ExprKind::Call(_, _))
                    && !matches!(value.target, Some(CallTarget::Log | CallTarget::StoredCallableField(_) | CallTarget::MapMethod(MapMethod::Clear)))
                    && !removal.as_ref().is_some_and(|(_, _, span)| *span == value.span)
                    && !(matches!(value.target, Some(CallTarget::ConditionalClosure(_)))
                        && matches!(&value.kind, ExprKind::Call(callee, _) if self.planned_record_array_type(callee, results).is_some()))
                { effect = true; }
                if matches!(value.kind, ExprKind::Assign { .. } | ExprKind::CompoundAssign { .. } | ExprKind::CoalesceAssign { .. } | ExprKind::LogicalAssign { .. }) {
                    opaque = true;
                }
                Ok(())
            })?;
            opaque = effect || opaque && mentions_storage;
            if opaque {
                self.restore_array_storage_baselines(context, storage, statement.span, None)?;
                storage.clear();
                replacement = None;
            }
            if let Some((place, _, _)) = &replacement {
                self.restore_array_storage_baselines(context, storage, statement.span, Some(place))?;
            }
            let plans = self.array_epoch_root_types(context, storage);
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
            if let StmtKind::Let { binding: Some(binding), value, .. } = &mut statement.kind
                && (!context.bindings[*binding].rebindable || eligible.contains(binding))
            {
                let selected = removal.as_ref().and_then(|(place, front, _)| {
                    let state = storage.get(place)?;
                    if *front { state.slots.first() } else { state.slots.last() }.copied()
                });
                let result = if let Some(selected) = selected {
                    Some(self.unions.intern([selected, Type::Undefined], value.span)?)
                } else if removal.is_some() {
                    value.ty
                } else if matches!(value.kind, ExprKind::Name(_)) {
                    value.binding.and_then(|source| results.get(&source).copied())
                } else { None };
                if let Some(result) = result {
                    value.ty = Some(result);
                    context.bindings[*binding].ty = result;
                    results.insert(*binding, result);
                }
            }
            if let StmtKind::Assign { target, value, operator: None } = &mut statement.kind
                && matches!(target.kind, ExprKind::Name(_))
                && let Some(binding) = target.binding
                && let Some(previous) = results.get(&binding).copied()
                && eligible.contains(&binding)
            {
                let selected = removal.as_ref().and_then(|(place, front, _)| {
                    let state = storage.get(place)?;
                    if *front { state.slots.first() } else { state.slots.last() }.copied()
                });
                if let Some(result) = self.array_result_replacement_type(value, selected, previous, results)? {
                    if result != previous {
                        context.storage_exits.record_array_result_write(statement.span, result);
                    }
                    results.insert(binding, result);
                }
            }
            let update = replacement.map(|(place, index, ty)| (place, Some((index, ty)), false))
                .or_else(|| removal.map(|(place, front, _)| (place, None, front)));
            if let Some((place, replacement, front)) = update && storage.contains_key(&place) {
                let roots = self.array_epoch_root_types(context, storage);
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

/// Lexical blocks without abrupt completion can return changed owners normally.
/// Branches are allowed only when their array effects remain opaque below.
fn array_normal_block(body: &[Stmt]) -> bool {
    let mut pending = body.iter().collect::<Vec<_>>();
    while let Some(statement) = pending.pop() {
        if !statement.labels.is_empty() { return false; }
        match &statement.kind {
            StmtKind::Block(body) => pending.extend(body),
            StmtKind::If { then_body, else_body, .. } => { pending.extend(then_body); pending.extend(else_body); }
            StmtKind::Let { .. } | StmtKind::Assign { .. } | StmtKind::Expr(_) => {},
            _ => return false,
        }
    }
    true
}
