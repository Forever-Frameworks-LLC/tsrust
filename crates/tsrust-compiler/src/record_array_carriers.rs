//! Concrete local array storage shares one checked record environment carrier.
use super::*;

impl Analyzer {
    pub(super) fn indexed_array_root(value: &Expr) -> Option<usize> {
        let mut value = value;
        while let ExprKind::Index(base, _) = &value.kind { value = base; }
        matches!(value.kind, ExprKind::Name(_)).then_some(value.binding).flatten()
    }
    pub(super) fn scoped_record_nullable_contract(&self, actual: Type, expected: Type) -> bool {
        let (Some(actual_id), Some(expected_id)) = (actual.union(), expected.union()) else { return false; };
        let (Some(Type::Record(actual_record)), Some(Type::Record(expected_record))) = (self.unions.payload(actual), self.unions.payload(expected)) else { return false; };
        if !self.aggregate_callable_facts(Type::Record(actual_record)).scoped
            || self.scoped_records.declaration(actual_record) != self.scoped_records.declaration(expected_record) { return false; }
        let expected_members = self.unions.members(expected_id);
        self.unions.members(actual_id).iter().all(|member| matches!(member, Type::Record(_)) || expected_members.contains(member))
    }
    pub(super) fn scoped_array_element_contract(&self, base: &Expr, element: ArrayElement) -> Type {
        if Self::indexed_array_root(base).is_some()
            && let ArrayElement::Record(id, depth) = element
            && self.aggregate_callable_facts(Type::Record(id)).scoped
        { return ArrayElement::Record(self.scoped_records.declaration(id), depth).ty(); }
        element.ty()
    }

    #[inline(never)]
    pub(super) fn widen_scoped_array_storage(&mut self, context: &mut Context, base: &Expr, incoming: Type, span: Span) -> Result<(), Diagnostic> {
        let Some(binding) = Self::indexed_array_root(base) else { return Ok(()); };
        let Type::Array(ArrayElement::Record(previous, depth)) = context.bindings[binding].ty else { return Ok(()); };
        let incoming = match incoming {
            Type::Record(record) | Type::Array(ArrayElement::Record(record, _)) => record,
            _ => return Ok(()),
        };
        if !self.aggregate_callable_facts(Type::Record(previous)).scoped || previous == incoming { return Ok(()); }
        let joined = self.join_record_types(previous, incoming, span)?.ok_or_else(|| Diagnostic::new("type", "Array callback field environments have incompatible contracts.", span))?;
        let Type::Record(record) = joined else { unreachable!() };
        context.bindings[binding].ty = Type::Array(ArrayElement::Record(record, depth));
        Ok(())
    }

    #[inline(never)]
    pub(super) fn record_array_carriers(&mut self, context: &mut Context, body: &mut [Stmt]) -> Result<(), Diagnostic> {
        if !context.bindings.iter().any(|binding| matches!(binding.ty, Type::Array(ArrayElement::Record(_, _)))) { return Ok(()); }
        self.record_array_carriers_impl(context, body)
    }

    #[inline(never)]
    fn record_array_carriers_impl(&mut self, context: &mut Context, body: &mut [Stmt]) -> Result<(), Diagnostic> {
        let mut destinations = HashMap::new();
        let mut plans = HashMap::new();
        let writes = record_storage_writes::collect(body);
        for write in &writes {
            if !write.path.is_empty() { continue; }
            let binding = &context.bindings[write.binding];
            if binding.parameter || binding.capture_reference { continue; }
            let Type::Array(ArrayElement::Record(record, _)) = binding.ty else { continue; };
            if !self.aggregate_callable_facts(Type::Record(record)).scoped { continue; }
            plans.insert(write.binding, binding.ty);
            destinations.insert((write.span.start, write.span.end), if write.array_element { Type::Record(record) } else { binding.ty });
        }
        if plans.is_empty() { return Ok(()); }
        loop {
            let mut changed = false;
            for write in &writes {
                if write.array_element || !write.path.is_empty() { continue; }
                let source = match &write.value.kind {
                    ExprKind::Name(_) => write.value.binding,
                    ExprKind::Call(callee, _) if matches!(write.value.target, Some(CallTarget::ArraySplice)) => {
                        if let ExprKind::Field(base, _) = &callee.kind { Self::indexed_array_root(base) } else { None }
                    }
                    _ => None,
                };
                let Some(source) = source else { continue; };
                let (Some(Type::Array(ArrayElement::Record(left, left_depth))), Some(Type::Array(ArrayElement::Record(right, right_depth)))) = (plans.get(&source).copied(), plans.get(&write.binding).copied()) else { continue; };
                if left == right { continue; }
                let Some(Type::Record(joined)) = self.join_record_types(left, right, write.span)? else { continue; };
                for (binding, depth) in [(source, left_depth), (write.binding, right_depth)] {
                    let ty = Type::Array(ArrayElement::Record(joined, depth));
                    if plans.insert(binding, ty) != Some(ty) { changed = true; }
                }
            }
            if !changed { break; }
        }
        for write in &writes {
            if !write.path.is_empty() || write.array_element { continue; }
            let mut incoming = write.value;
            while let ExprKind::Inject { value, .. } = &incoming.kind { incoming = value; }
            let ExprKind::Call(callee, _) = &incoming.kind else { continue; };
            if !matches!(incoming.target, Some(CallTarget::ArrayPop | CallTarget::ArrayShift)) { continue; }
            let ExprKind::Field(base, _) = &callee.kind else { continue; };
            let Some(Type::Array(element)) = self.planned_record_array_type(base, &plans) else { continue; };
            let ty = self.unions.intern([element.ty(), Type::Undefined], write.span)?;
            plans.insert(write.binding, ty);
        }
        loop {
            let mut changed = false;
            for write in &writes {
                if !write.path.is_empty() || write.array_element || !matches!(write.value.kind, ExprKind::Name(_)) { continue; }
                let Some(mut ty) = write.value.binding.and_then(|id| plans.get(&id).copied()) else { continue; };
                if !matches!(ty, Type::Union(_)) { continue; }
                if let Some(previous) = plans.get(&write.binding).copied()
                    && previous != ty
                    && let (Some(Type::Record(left)), Some(Type::Record(right))) = (self.unions.payload(previous), self.unions.payload(ty))
                    && let Some(record) = self.join_record_types(left, right, write.span)?
                {
                    let mut members = vec![record];
                    for id in [previous.union(), ty.union()].into_iter().flatten() {
                        members.extend(self.unions.members(id).into_iter().filter(|member| matches!(member, Type::Null | Type::Undefined)));
                    }
                    ty = self.unions.intern(members, write.span)?;
                }
                if plans.get(&write.binding).copied() != Some(ty) {
                    plans.insert(write.binding, ty);
                    changed = true;
                }
            }
            if !changed { break; }
        }
        for (&binding, &ty) in &plans { context.bindings[binding].ty = ty; }
        for write in &writes {
            if !write.path.is_empty() { continue; }
            if let Some(&ty) = plans.get(&write.binding) {
                let destination = if write.array_element {
                    let Type::Array(ArrayElement::Record(record, _)) = ty else { continue; };
                    match write.value.ty {
                        Some(Type::Array(ArrayElement::Record(_, depth))) => Type::Array(ArrayElement::Record(record, depth)),
                        _ => Type::Record(record),
                    }
                } else { ty };
                destinations.insert((write.span.start, write.span.end), destination);
            }
        }
        drop(writes);
        for statement in body.iter_mut() {
            if let StmtKind::Let { binding: Some(binding), value, .. } = &mut statement.kind
                && value.is_deferred()
                && let Some(ty) = plans.get(binding)
            { value.ty = Some(*ty); }
        }
        crate::source_walk::body(body, &mut |_| Ok(()), &mut |value| {
            if let Some(ty) = self.planned_record_array_type(value, &plans) { value.ty = Some(ty); }
            if let Some(destination) = destinations.remove(&(value.span.start, value.span.end)) {
                if let (ExprKind::Array(elements), Type::Array(ArrayElement::Record(record, depth))) = (&mut value.kind, destination) {
                    Self::apply_scoped_array_carrier(elements, record, depth);
                    value.ty = Some(destination);
                } else if matches!(destination, Type::Record(_) | Type::Array(ArrayElement::Record(_, _))) && value.ty != Some(destination) {
                    value.wrap_record_carrier(destination);
                } else if let (Type::Union(id), ExprKind::Inject { value: inner, union }) = (destination, &mut value.kind)
                    && (matches!(inner.ty, Some(Type::Null | Type::Undefined)) || matches!(self.planned_record_array_type(inner, &plans), Some(Type::Union(_))))
                {
                    *union = id;
                    value.ty = Some(destination);
                }
            }
            if let Some(ty) = self.planned_record_array_type(value, &plans) { value.ty = Some(ty); }
            if let ExprKind::Project { value: inner, union, narrowed } = &mut value.kind
                && matches!(narrowed, Type::Record(_))
                && let Some(Type::Union(id)) = self.planned_record_array_type(inner, &plans)
                && let Some(payload) = self.unions.payload(Type::Union(id))
            {
                *union = id;
                *narrowed = payload;
                value.ty = Some(payload);
            }
            if let ExprKind::Call(callee, _) = &mut value.kind {
                if let Some(Type::BorrowedCallable(family)) = self.planned_record_array_type(callee, &plans) {
                    callee.ty = Some(Type::BorrowedCallable(family));
                    value.target = Some(CallTarget::StoredCallableField(family));
                }
            }
            Ok(())
        })
    }

    fn planned_record_array_type(&self, value: &Expr, plans: &HashMap<usize, Type>) -> Option<Type> {
        match &value.kind {
            ExprKind::Name(_) => plans.get(&value.binding?).copied(),
            ExprKind::Index(base, _) => match self.planned_record_array_type(base, plans)? { Type::Array(element) => Some(element.ty()), _ => None },
            ExprKind::Field(base, name) => {
                let Type::Record(record) = self.planned_record_array_type(base, plans)? else { return None; };
                self.program.records[record].fields.iter().find(|field| field.name.text == *name)?.ty
            }
            ExprKind::Call(callee, _) if matches!(value.target, Some(CallTarget::ArraySplice)) => {
                if let ExprKind::Field(base, _) = &callee.kind { self.planned_record_array_type(base, plans) } else { None }
            }
            ExprKind::Call(callee, _) if matches!(value.target, Some(CallTarget::ArrayPop | CallTarget::ArrayShift)) => {
                let ExprKind::Field(base, _) = &callee.kind else { return None; };
                let Type::Array(element) = self.planned_record_array_type(base, plans)? else { return None; };
                self.unions.intern([element.ty(), Type::Undefined], value.span).ok()
            }
            ExprKind::Project { value, narrowed, .. } if matches!(narrowed, Type::Record(_)) => self.unions.payload(self.planned_record_array_type(value, plans)?),
            _ => None,
        }
    }
}
