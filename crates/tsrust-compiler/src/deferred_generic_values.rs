//! Explicit source origins for generic values whose invocation types are deferred.
use super::*;

pub(super) struct Source {
    pub declaration: String,
    pub span: Span,
    pub context: Option<Context>,
    pub identities: Vec<crate::local_capture_plan::BindingId>,
}

impl Analyzer {
    #[inline(never)]
    fn retain_generic_invocation(&mut self, context: &Context, origin: usize, method: usize, result: Type, span: Span) -> Result<(usize, Type), Diagnostic> {
        if !self.effects_only && !self.discovering_results && !context.template {
            self.polymorphic_invocations.publish(
                [crate::polymorphic_invocation_plans::Pair { origin, method }], &self.closures, span,
            )?;
        }
        Ok((method, result))
    }

    /// Fresh generic declarations retain independent construction
    /// origins while sharing the binding's registered storage family.
    #[inline(never)]
    pub(super) fn fresh_generic_replacement_type(
        &self, expected: Type, actual: Type, value: &mut Expr,
    ) -> Type {
        let (Type::Closure(previous), Type::Closure(next)) = (expected, actual) else { return actual; };
        if previous == next || !matches!(value.kind, ExprKind::Closure { .. }) { return actual; }
        let (Some(_previous_source), Some(_next_source)) = (
            self.deferred_generic_values.get(&previous), self.deferred_generic_values.get(&next),
        ) else { return actual; };
        value.callable_origin = Some(CallableOrigin::Closure(next));
        value.ty = Some(expected);
        expected
    }

    #[inline(never)]
    pub(super) fn deferred_generic_requirement(
        &self, actual: Type, expected: Option<Type>, span: Span,
    ) -> Result<(), Diagnostic> {
        if let Some(expected) = expected && actual != expected {
            let actual_payload = self.unions.payload(actual).unwrap_or(actual);
            let expected_payload = self.unions.payload(expected).unwrap_or(expected);
            let deferred = |ty| matches!(ty, Type::Closure(origin) if self.deferred_generic_values.contains_key(&origin));
            let callable = |ty| matches!(ty, Type::Closure(_) | Type::Callback(_) | Type::Callable(_) | Type::StoredCallable(_));
            if actual_payload != expected_payload && (deferred(actual_payload) || deferred(expected_payload))
                && callable(actual_payload) && callable(expected_payload) {
                return Err(Diagnostic::new("unsupported", "Adapting an unannotated polymorphic value to a different callable contract is not yet supported. Assign the original generic declaration directly to a concrete callable annotation.", span));
            }
        }
        Ok(())
    }

    pub(super) fn deferred_generic_value(
        &mut self, outer: &mut Context, expression: &mut Expr, loans: &Loans,
    ) -> Result<Type, Diagnostic> {
        let ExprKind::Name(name) = &expression.kind else { unreachable!() };
        let span = expression.span;
        let ty = self.deferred_captured_generic_value(outer, name, usize::MAX - span.start, span, loans)?;
        expression.kind = ExprKind::Closure {
            id: usize::MAX - span.start, params: Vec::new(),
            result: ResultAnnotation::Inferred(span), body: Vec::new(),
        };
        let Type::Closure(origin) = ty else { unreachable!("deferred generic environment") };
        expression.callable_origin = Some(CallableOrigin::Closure(origin));
        Ok(ty)
    }

    #[inline(never)]
    pub(super) fn deferred_captured_generic_value(
        &mut self, outer: &mut Context, declaration: &str, source_origin: usize,
        span: Span, loans: &Loans,
    ) -> Result<Type, Diagnostic> {
        let projected = self.named_captures.source_context(declaration, outer, &self.program)?;
        let (mut context, identities) = projected.map_or_else(
            || (capture_context::project(outer, &[]), Vec::new()), |value| value);
        if !identities.is_empty() && self.assumed_modes.is_none() {
            // The public single-pass checker must request the same finite access
            // inference as the project checker before exposing a capture layout.
            return Err(Diagnostic::new("recursion-inference", "Generic captured values require capture-access inference.", span));
        }
        let escapes = self.escaping.contains(&source_origin);
        let mut captures = Vec::new();
        let mut invocation = Mode::Read;
        let mut lifetime_count = 0;
        for (inner, &identity) in identities.iter().enumerate() {
            let outer_id = outer.source_bindings[&identity.declaration_start];
            let binding = &context.bindings[inner];
            let initial = if binding.ty.is_copy() { Mode::Copy } else { Mode::Read };
            let access = self.assumed_capture_access.declaration_mode(declaration, identity, initial);
            invocation = invocation.max(access);
            let storage = if binding.ty.is_copy() {
                if access == Mode::Mutate && !escapes { Mode::Mutate } else { Mode::Copy }
            } else if escapes { Mode::Consume } else { access };
            if !self.effects_only && escapes && let Type::Closure(id) = binding.ty && self.closures[id].borrowed {
                return Err(Diagnostic::new("closure-escape", "An escaping closure cannot retain a borrowed callable environment.", span).related(self.closures[id].span, "Borrowed callable is created here."));
            }
            Self::access(outer, outer_id, storage, span, loans)?;
            if matches!(storage, Mode::Read | Mode::Mutate) {
                outer.capture_loans.push((outer_id, storage, span));
                lifetime_count += 1;
            }
            if let Type::Closure(id) = binding.ty { lifetime_count += self.closures[id].lifetime_count; }
            captures.push(Capture { restored:false, outer: outer_id, inner, storage, access, span });
        }
        if !self.effects_only && lifetime_count > 128 {
            return Err(Diagnostic::new("unsupported", "Closure environment lifetime complexity exceeds the prototype limit of 128.", span));
        }
        for capture in &captures {
            let binding = &mut context.bindings[capture.inner];
            binding.capture_reference = match capture.storage {
                Mode::Read if matches!(binding.ty, Type::BorrowedUnion(_) | Type::Union(_)) => false,
                Mode::Read | Mode::Mutate => true,
                Mode::Consume => invocation != Mode::Consume,
                Mode::Copy => capture.access == Mode::Mutate && invocation == Mode::Mutate,
            };
            binding.mode = if capture.storage == Mode::Consume || (capture.storage == Mode::Copy && binding.capture_reference) { invocation } else { capture.storage };
        }
        let origin = self.deferred_generic_values.iter().find_map(|(&origin, source)| {
            let closure = &self.closures[origin];
            (source.declaration == declaration && source.span == span
                && closure.function.template == outer.template && closure.escapes == escapes
                && closure.function.bindings.iter().map(|binding| binding.ty).eq(context.bindings.iter().map(|binding| binding.ty)))
                .then_some(origin)
        }).unwrap_or_else(|| {
            let origin = self.closures.len();
            self.closures.push(CheckedClosure {
                polymorphic: true, methods: Vec::new(), forward: None,
                borrowed: lifetime_count != 0, lifetime_count: lifetime_count.min(129), depth: outer.closure_depth + 1,
                parameters: Vec::new(),
                function: CheckedFunction {
                storage_exits: context.storage_exits.clone(), native_returns: context.native_returns.clone(),
                    template: outer.template, specialization: false, name: outer.function_name.clone(),
                    bindings: context.bindings.clone(), parameter_count: 0, result: Type::Void, body: Vec::new(),
                }, captures, span, escapes, invocation,
            });
            self.deferred_generic_values.insert(origin, Source {
                declaration: declaration.to_owned(), span,
                context: (!identities.is_empty()).then_some(context), identities,
            });
            origin
        });
        if !self.discovering_results { self.validated_closures.insert(origin); }
        Ok(Type::Closure(origin))
    }

    /// Check the original generic body with supplied type hints only. No source
    /// argument or default initializer is executed and no environment is rebuilt.
    #[inline(never)]
    pub(super) fn deferred_generic_result_hint(&mut self, context: &Context, origin: usize, hints: &[Option<Type>], span: Span) -> Result<Type, Diagnostic> {
        let source = &self.deferred_generic_values[&origin];
        let declaration = source.declaration.clone();
        let preparation = source.context.as_ref().map(|source| source.clone()).unwrap_or_else(|| context.clone());
        let arguments = hints.iter().map(|hint| Expr::checked(ExprKind::Undefined, span, *hint)).collect();
        let mut call = Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name(declaration), span)), arguments), span);
        self.prepare_generic_call(&preparation, &mut call, None)?;
        let ExprKind::Call(callee, _) = call.kind else { unreachable!() };
        let ExprKind::Name(name) = callee.kind else { unreachable!() };
        let function = self.function_names[&name];
        self.named_captures.prepare(function, &preparation, &self.program)?;
        self.function(function)?;
        Ok(self.checked[function].as_ref().ok_or_else(|| Self::pending_result(span))?.result)
    }

    #[allow(clippy::too_many_arguments)]
    #[inline(never)]
    pub(super) fn deferred_generic_call(
        &mut self, context: &mut Context, binding: Option<usize>, origin: usize,
        callee: &mut Expr, args: &mut Vec<Expr>, loans: &Loans, span: Span, once: bool, expected: Option<Type>, checked_receiver: bool,
    ) -> Result<(usize, Type), Diagnostic> {
        let declaration = self.deferred_generic_values[&origin].declaration.clone();
        let reachable_origins = if !context.template
            && let Some(reachable) = self.generic_functions.reachable_declarations(&context.function_name, span, callee.span)
        {
            self.polymorphic_storage.reachable_origins(origin, reachable, |member| {
                self.deferred_generic_values.get(&member)
                    .and_then(|source| self.generic_functions.template(&source.declaration))
                    .map(|template| template.name.span)
            }, span)?
        } else if self.polymorphic_storage.contains(origin) {
            // Generated contextual adapters have no original source call proof.
            // Retain every matching environment instead of guessing the first.
            let source = self.generic_functions.template(&declaration).expect("deferred source declaration").name.span;
            self.polymorphic_storage.reachable_origins(origin, &[source], |member| {
                self.deferred_generic_values.get(&member)
                    .and_then(|source| self.generic_functions.template(&source.declaration))
                    .map(|template| template.name.span)
            }, span)?
        } else { vec![origin] };
        let preparation_origin=if reachable_origins.contains(&origin) {origin} else {
            *reachable_origins.first().ok_or_else(||Diagnostic::new("callable-layout","Generic invocation has no reachable environment.",span))?
        };
        let prepared = self.prepare_deferred_generic_body(context, preparation_origin, args.clone(), expected, span)?;
        let function = prepared.function;
        let mut contracts = prepared.contracts;
        *args = prepared.arguments;
        self.join_deferred_argument_contracts(context,preparation_origin,&reachable_origins,args,&mut contracts,expected,span)?;
        let accesses = prepared.accesses;
        let method_invocation = prepared.invocation;
        let invocation = if once { Mode::Consume } else { method_invocation };
        if let Some(binding) = binding {
            Self::access(context, binding, invocation, callee.span, loans)?;
            callee.binding = Some(binding);
            callee.ty = Some(context.bindings[binding].ty);
        } else {
            let receiver = self.expression_type_hint(context, callee).unwrap_or(Type::Closure(origin));
            let actual = if checked_receiver { callee.ty.expect("checked generic receiver") }
                else { self.expression(context, callee, invocation, loans, Some(receiver))? };
            self.require(actual, Some(receiver), callee.span)?;
        }
        let capture_boundary = crate::capture_loan_identity::Boundary::capture(&context.capture_loans);
        let mut active = loans.clone();
        if let Some(binding) = binding { active.entry(binding).or_default().push((invocation, callee.span)); }
        else { for root in Self::loan_roots(callee, invocation) { active.entry(root).or_default().push((invocation, callee.span)); } }
        for (argument, contract) in args.iter_mut().zip(contracts) {
            let actual = self.parameter_argument(context, argument, contract, &active)?;
            self.require(actual, Some(contract.ty), argument.span)?;
            if !actual.is_copy() {
                for root in Self::loan_roots(argument, contract.mode) {
                    active.entry(root).or_default().push((contract.mode, argument.span));
                }
            }
        }
        capture_boundary.unwind(&mut context.capture_loans);
        let (function, result) = self.resolved_call(context, function, args, span)?;
        // Callable arguments can select a further concrete body specialization.
        // Check that body before retaining its invocation method.
        self.function(function)?;
        let method = self.materialize_deferred_method(preparation_origin, function, result, args, &accesses, method_invocation, span)?;
        if !self.effects_only && !self.discovering_results && !context.template && (reachable_origins.len() > 1 || preparation_origin!=origin) {
            callee.generic_invocation = Some(self.materialize_selected_generic_methods(context, preparation_origin, function, result, args, &accesses, method_invocation, &reachable_origins, span)?);
        }
        self.retain_generic_invocation(context, preparation_origin, method, result, span)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn constructed_environment_cannot_silently_lose_compacted_identity() {
        let source = crate::format("function identity<T>(value:T):T{return value;}function main(){const action=identity;action<int>(7);}").unwrap();
        assert_eq!(crate::format(&source).unwrap(), source);
        let mut checked = crate::check(&source).unwrap();
        let ids = (0..checked.program.closures.len()).map(Some).collect::<Vec<_>>();
        let function = checked.program.functions.iter_mut().find(|function| function.name == "main").unwrap();
        let value = function.body.iter_mut().find_map(|statement| match &mut statement.kind {
            crate::syntax::StmtKind::Let { value, .. } => Some(value), _ => None,
        }).unwrap();
        assert!(matches!(value.kind, crate::syntax::ExprKind::Closure { .. }));
        value.callable_origin = Some(crate::syntax::CallableOrigin::Closure(usize::MAX));
        let span = value.span;
        let error = super::remap_function_closures(function, &ids).unwrap_err();
        assert_eq!(error.code, "callable-layout");
        assert_eq!(error.span, span);
    }

    #[test]
    fn explicit_unused_type_arguments_keep_distinct_methods_and_reuse_equivalents() {
        let checked=crate::check("function action<T>():int{return 7;}function main(){const alias=action;alias<int>();alias<string>();alias<int>();}").unwrap();
        let origin=*checked.program.deferred_generic_values.iter().next().unwrap();
        assert_eq!(checked.program.closures[origin].methods.len(),2);
    }

    #[test]
    fn result_only_methods_reuse_each_concrete_result_in_one_environment() {
        let checked=crate::check("function empty<T>():T[]{return [];}function main(){const action=empty;const one:int[]=action();const text:string[]=action();const two:int[]=action();}").unwrap();
        let origin=*checked.program.deferred_generic_values.iter().next().unwrap();
        let closure=&checked.program.closures[origin];
        assert_eq!(closure.methods.len(),2);
        assert!(closure.methods.iter().all(|method|method.parameters.is_empty()));
        assert_ne!(closure.methods[0].function.result,closure.methods[1].function.result);
    }
    #[test]
    fn captured_data_methods_keep_one_typed_layout_with_separate_binding_slots() {
        let checked=crate::check("function main(){let calls=0;function select<T>(value:T):T{calls+=1;return value;}{const action=select;const one=action(1);const text=action('x');}console.log(calls);}").unwrap();
        assert_eq!(checked.program.deferred_generic_values.len(),1);
        let origin=*checked.program.deferred_generic_values.iter().next().unwrap();
        let closure=&checked.program.closures[origin];
        assert_eq!(closure.captures.len(),1);
        assert_eq!(closure.captures[0].storage,crate::Mode::Mutate);
        assert_eq!(closure.function.bindings.len(),1);
        assert_eq!(closure.function.bindings[0].ty,crate::syntax::Type::Int);
        assert_eq!(closure.methods.len(),2);
        for method in &closure.methods {
            assert_eq!(method.invocation,crate::Mode::Mutate);
            assert_eq!(method.parameters,vec![0]);
            assert_eq!(method.captures,vec![1]);
            assert!(method.function.bindings[1].capture_reference);
        }
    }
    #[test]
    fn data_methods_share_one_empty_origin_and_reuse_checked_types() {
        let checked=crate::check("function identity<T>(value:T):T{return value;}function main(){const action=identity;const one=action(1);const two=action(2);const text=action('a');const values=action([3,4]);}").unwrap();
        assert_eq!(checked.program.deferred_generic_values.len(),1);
        let origin=*checked.program.deferred_generic_values.iter().next().unwrap();
        let closure=&checked.program.closures[origin];
        assert!(closure.captures.is_empty());
        assert!(closure.function.bindings.is_empty());
        assert_eq!(closure.methods.len(),3);
        assert!(closure.methods.iter().all(|method|method.parameters.len()==1));
    }
}
