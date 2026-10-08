//! Concrete local array storage shares one checked record environment carrier.
use super::*;

impl Analyzer {
    pub(super) fn scoped_array_element_contract(&self, base: &Expr, element: ArrayElement) -> Type {
        if matches!(base.kind, ExprKind::Name(_))
            && let ArrayElement::Record(id, 0) = element
            && self.aggregate_callable_facts(Type::Record(id)).scoped
        { return Type::Record(self.scoped_records.declaration(id)); }
        element.ty()
    }

    #[inline(never)]
    pub(super) fn widen_scoped_array_storage(&mut self, context: &mut Context, base: &Expr, incoming: Type, span: Span) -> Result<(), Diagnostic> {
        if !matches!(base.kind, ExprKind::Name(_)) { return Ok(()); }
        let binding = base.binding.expect("checked named array");
        let (Type::Array(ArrayElement::Record(previous, 0)), Type::Record(incoming)) = (context.bindings[binding].ty, incoming) else { return Ok(()); };
        if !self.aggregate_callable_facts(Type::Record(previous)).scoped || previous == incoming { return Ok(()); }
        let joined = self.join_record_types(previous, incoming, span)?.ok_or_else(|| Diagnostic::new("type", "Array callback field environments have incompatible contracts.", span))?;
        let Type::Record(record) = joined else { unreachable!() };
        context.bindings[binding].ty = Type::Array(ArrayElement::Record(record, 0));
        Ok(())
    }

    #[inline(never)]
    pub(super) fn record_array_carriers(&mut self, context: &mut Context, body: &mut [Stmt]) -> Result<(), Diagnostic> {
        if !context.bindings.iter().any(|binding| matches!(binding.ty, Type::Array(ArrayElement::Record(_, 0)))) { return Ok(()); }
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
            let Type::Array(ArrayElement::Record(record, 0)) = binding.ty else { continue; };
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
                        if let ExprKind::Field(base, _) = &callee.kind { base.binding } else { None }
                    }
                    _ => None,
                };
                let Some(source) = source else { continue; };
                let (Some(Type::Array(ArrayElement::Record(left, 0))), Some(Type::Array(ArrayElement::Record(right, 0)))) = (plans.get(&source).copied(), plans.get(&write.binding).copied()) else { continue; };
                if left == right { continue; }
                let Some(Type::Record(joined)) = self.join_record_types(left, right, write.span)? else { continue; };
                let ty = Type::Array(ArrayElement::Record(joined, 0));
                for binding in [source, write.binding] {
                    if plans.insert(binding, ty) != Some(ty) { changed = true; }
                }
            }
            if !changed { break; }
        }
        for (&binding, &ty) in &plans { context.bindings[binding].ty = ty; }
        for write in &writes {
            if !write.path.is_empty() { continue; }
            if let Some(&ty) = plans.get(&write.binding) {
                let Type::Array(element) = ty else { continue; };
                destinations.insert((write.span.start, write.span.end), if write.array_element { element.ty() } else { ty });
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
            if let Some(destination) = destinations.remove(&(value.span.start, value.span.end)) {
                if let (ExprKind::Array(elements), Type::Array(ArrayElement::Record(record, 0))) = (&mut value.kind, destination) {
                    for element in elements {
                        if element.ty != Some(Type::Record(record)) { element.wrap_record_carrier(Type::Record(record)); }
                    }
                    value.ty = Some(destination);
                } else if matches!(destination, Type::Record(_)) && value.ty != Some(destination) {
                    value.wrap_record_carrier(destination);
                }
            }
            if let Some(ty) = self.planned_record_array_type(value, &plans) { value.ty = Some(ty); }
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
            _ => None,
        }
    }
}
