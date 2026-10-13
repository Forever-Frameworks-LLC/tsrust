//! Named and field stores suspend before checking their replacement values.
use super::super::*;

pub(super) struct FieldTarget {
    mutation: super::assignment::Mutation,
    suppressed: Option<crate::flow::Place>,
}

pub(super) enum Finish {
    Named { id: usize, ty: Type, removal: Option<Box<crate::capture_loan_owners::Owners>> },
    Field { contract: Type },
}

pub(super) struct Value {
    pub(super) finish: Finish,
    pub(super) stored: Option<super::StoredValueFrame>,
    pub(super) expected: Option<Type>,
    pub(super) loans: Loans,
}

impl Analyzer {
    pub(in crate::analysis) fn store_work(&mut self, context: &mut Context, target: &mut Expr, value: &mut Expr, loans: &Loans, precise: bool) -> Result<(), Diagnostic> {
        super::driver::check_store(self, context, target, value, loans, precise)
    }

    pub(super) fn prepare_named_store_work(&mut self, context: &mut Context, target: &Expr, value: &Expr, loans: &Loans) -> Result<Value, Diagnostic> {
        let id = self.rebind_target(context, target)?;
        let ty = context.bindings[id].ty;
        if !context.effects_only && let Some(active)=loans.get(&id) && let Some((_,earlier))=active.first(){
            return Err(Diagnostic::new("conflicting-access",format!("Cannot replace {:?} through overlapping access.",context.bindings[id].name),target.span).related(*earlier,"Earlier access starts here."));
        }
        // Check the RHS against the current availability, allowing x = take(x)
        // but rejecting a read of x after an earlier transfer. Commit only after
        // evaluation and exact type checking complete.
        let removal = Self::array_removal_snapshot(context, value);
        Ok(Value { finish: Finish::Named { id, ty, removal }, stored: None, expected: Some(ty), loans: loans.clone() })
    }

    pub(super) fn prepare_field_store_work(&mut self, context: &mut Context, target: &Expr) -> Result<FieldTarget, Diagnostic> {
        if !Self::named_storage(target)
            || !matches!(&target.kind, ExprKind::Field(base, _) | ExprKind::Index(base, _) if matches!(base.kind, ExprKind::Name(_) | ExprKind::Index(_, _) | ExprKind::Field(_, _) | ExprKind::Project { .. }))
        {
            return Err(Diagnostic::new(
                "unsupported",
                "Only a named record field or array element can be replaced here.",
                target.span,
            ));
        }
        context.replacement = Some(target.span);
        let suppressed = crate::flow::place(target, &context.names);
        if let Some(place) = &suppressed {
            context.suppressed_flow.insert(place.clone());
        }
        let mutation = self.prepare_mutation_work(context, target, true);
        Ok(FieldTarget { mutation, suppressed })
    }

    pub(super) fn field_store_selected(&mut self, context: &mut Context, target: &mut Expr, value: &mut Expr, frame: FieldTarget, loans: &Loans, checked: Result<Type, Diagnostic>) -> Result<Value, Diagnostic> {
        self.finish_mutation_work(context, frame.mutation);
        if let Some(place) = frame.suppressed { context.suppressed_flow.remove(&place); }
        context.replacement = None;
        let ty = checked?;
        let selected = match &target.kind {
            ExprKind::Index(base, _) => base.as_ref(),
            ExprKind::Field(base, _) => match &base.kind {
                ExprKind::Index(parent, _) => parent.as_ref(),
                _ => base.as_ref(),
            },
            _ => unreachable!("checked assignment target"),
        };
        let active = Self::selection_loans(selected, loans);
        if self.local_record_nullable_callback(context,target) {
            return Err(Diagnostic::new("unsupported","A callback field that can be null or undefined and borrows local variables cannot be replaced yet. Declare a new record with the new value.",target.span));
        }
        let contract=if let ExprKind::Index(base, _) = &target.kind && let Some(Type::Array(element)) = base.ty {
            self.scoped_array_element_contract(base, element)
        } else {self.local_record_field_contract(context,target).or_else(||self.native_view_field_contract(context,target,ty)).unwrap_or(ty)};
        self.prepare_direct_callback_store(context, target, value)?;
        // A redeclared scoped field accepts any value of its written contract. Only a
        // new closure is checked against the symbolic family; other values keep the
        // concrete family they already have.
        let redeclared_native = contract != ty && self.native_view_field_contract(context,target,ty) == Some(contract);
        let (expected, stored) = if redeclared_native && !matches!(value.kind, ExprKind::Closure { .. } | ExprKind::Conditional { .. }) {
            (matches!(self.expression_type_hint(context, value), Some(Type::Null | Type::Undefined)).then_some(ty), None)
        } else if let Type::StoredCallable(family) = contract {
            let stored = self.prepare_stored_value(context, value, family)?;
            (stored.expected, Some(stored))
        } else { (Some(contract), None) };
        Ok(Value { finish: Finish::Field { contract }, stored, expected, loans: active })
    }

    pub(super) fn finish_store_work(&mut self, context: &mut Context, target: &mut Expr, value: &mut Expr, frame: Value, loans: &Loans, actual: Type) -> Result<(), Diagnostic> {
        let actual = if let Some(stored) = frame.stored { self.finish_stored_value(context, value, stored, actual)? } else { actual };
        match frame.finish {
            Finish::Named { id, ty, removal } => self.finish_named_store_work(context, target, value, id, ty, removal, actual),
            Finish::Field { contract } => self.finish_field_store_work(context, target, value, contract, loans, actual),
        }
    }

    fn finish_named_store_work(&mut self, context: &mut Context, target: &mut Expr, value: &mut Expr, id: usize, ty: Type, removal: Option<Box<crate::capture_loan_owners::Owners>>, actual: Type) -> Result<(), Diagnostic> {
        if !context.template
            && let (Type::Closure(previous), Type::Closure(next)) = (ty, actual)
            && self.deferred_generic_values.contains_key(&previous)
            && self.deferred_generic_values.contains_key(&next)
        {
            self.polymorphic_storage.observe(previous, next);
        }
        let actual = self.fresh_generic_replacement_type(ty, actual, value);
        let nullable_callback = matches!(self.unions.payload(ty), Some(Type::Callback(_)));
        if matches!(ty, Type::Callback(_)) && actual != ty {
            return Err(Diagnostic::new(
                "callable-origin",
                "Callback replacement requires a value derived from the same parameter; matching signatures alone are insufficient.",
                value.span,
            ));
        }
        if let (Type::Closure(previous), Type::Closure(next)) = (ty, actual)
            && previous != next
        {
            return Err(Diagnostic::new("callable-origin",
                    "Callable replacement requires the same concrete environment origin; matching signatures alone are insufficient.", value.span)
                    .related(self.closures[previous].span, "Original environment is defined here.")
                    .related(self.closures[next].span, "Replacement environment is defined here."));
        }
        let replacement_contract=if let (Type::Record(a),Type::Record(b))=(actual,ty) && self.scoped_records.declaration(a)==self.scoped_records.declaration(b){Type::Record(self.scoped_records.declaration(b))}else{ty};
        self.require(actual, Some(replacement_contract), value.span)?;
        let source_kind = value
            .binding
            .and_then(|source| context.bindings[source].callback_kind);
        if !nullable_callback && context.bindings[id].callback_kind != source_kind {
            return Err(Diagnostic::new(
                "callback-contract",
                "Callback replacement must preserve the source ownership and invocation contract.",
                value.span,
            ));
        }
        if context.is_capture(id) {
            context.bindings[id].used.get_or_insert(target.span);
            context.bindings[id].mode = context.bindings[id].mode.max(Mode::Mutate);
        }
        crate::flow::invalidate(&mut context.flow, id);
        if matches!((actual,ty),(Type::Record(a),Type::Record(b)) if self.scoped_records.declaration(a)==self.scoped_records.declaration(b)){context.bindings[id].ty=actual;}
        if matches!((actual,ty),(Type::Array(ArrayElement::Record(a,0)),Type::Array(ArrayElement::Record(b,0))) if self.scoped_records.declaration(a)==self.scoped_records.declaration(b)){context.bindings[id].ty=actual;}
        if self.scoped_record_nullable_contract(actual, ty) { context.bindings[id].ty = actual; }
        let mut native_loans = Self::native_view_value_loans(context, value)?;
        Self::retain_unified_native_view_loans(context, id, &mut native_loans);
        Self::set_native_view_loans(context, id, native_loans);
        context.bindings[id].transferred = None;
        context.bindings[id].moved_fields.clear();
        context.bindings[id].mutable = true;
        target.binding = Some(id);
        target.ty = Some(ty);
        self.replace_target_capture_loans(context, target, value);
        Self::finish_bound_array_removal(context, value, removal, id);
        Ok(())
    }

    fn finish_field_store_work(&mut self, context: &mut Context, target: &mut Expr, value: &mut Expr, contract: Type, loans: &Loans, actual: Type) -> Result<(), Diagnostic> {
        if !self.scoped_native_storage_contract(actual, contract, value.span)? {
            self.require(actual, Some(contract), value.span)?;
        }
        let actual = self.recursive_native_field_value(context, target, value, actual)?;
        let binding = Self::root_binding(target).expect("checked field has a binding");
        if let Some((_, path)) = Self::direct_fields(target) {
            Self::availability(
                &context.bindings[binding],
                Some((&path, true)),
                target.span,
                context.effects_only,
            )?;
            context.bindings[binding]
                .moved_fields
                .retain(|missing, _| !missing.starts_with(&path));
        } else if !context.effects_only && context.bindings[binding].transferred.is_some() {
            return Err(Self::transferred(&context.bindings[binding], target.span));
        }
        Self::storage_access(context, target, Mode::Mutate, target.span, loans)?;
        if let ExprKind::Index(base, _) = &target.kind {
            self.widen_scoped_array_storage(context, base, actual, value.span)?;
        }
        self.update_local_record_field_type(context,target,actual)?;
        self.update_native_view_field_type(context,target,actual);
        self.refresh_native_recursive_shapes(context);
        Self::replace_native_view_field(context,target,value)?;
        self.record_native_parameter_store(context,target,value)?;
        self.replace_target_capture_loans(context, target, value);
        Ok(())
    }
}
