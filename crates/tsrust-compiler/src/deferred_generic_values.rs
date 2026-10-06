//! Explicit source origins for generic values whose invocation types are deferred.
use super::*;

pub(super) struct Source {
    pub declaration: String,
    pub span: Span,
}

impl Analyzer {
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
        &mut self, outer: &Context, expression: &mut Expr,
    ) -> Result<Type, Diagnostic> {
        let ExprKind::Name(name) = &expression.kind else { unreachable!() };
        let declaration = name.clone();
        let span = expression.span;
        let origin = self.deferred_generic_values.iter()
            .find_map(|(&origin, source)| (source.declaration == declaration && source.span == span && self.closures[origin].function.template == outer.template).then_some(origin))
            .unwrap_or_else(|| {
                let origin = self.closures.len();
                self.closures.push(CheckedClosure {
                    polymorphic: true, methods: Vec::new(), forward: None,
                    borrowed: false, lifetime_count: 0, depth: outer.closure_depth + 1,
                    parameters: Vec::new(),
                    // Deferred source metadata, not this empty prototype, supplies
                    // every invocation contract. No unresolved data binding exists.
                    function: CheckedFunction {
                        template: outer.template, specialization: false, name: outer.function_name.clone(),
                        bindings: Vec::new(), parameter_count: 0, result: Type::Void, body: Vec::new(),
                    },
                    captures: Vec::new(), span, escapes: false, invocation: Mode::Read,
                });
                self.deferred_generic_values.insert(origin, Source { declaration, span });
                origin
            });
        if !self.discovering_results { self.validated_closures.insert(origin); }
        expression.kind = ExprKind::Closure {
            id: usize::MAX - span.start, params: Vec::new(),
            result: ResultAnnotation::Inferred(span), body: Vec::new(),
        };
        Ok(Type::Closure(origin))
    }

    #[allow(clippy::too_many_arguments)]
    #[inline(never)]
    pub(super) fn deferred_generic_call(
        &mut self, context: &mut Context, binding: usize, origin: usize,
        callee: &mut Expr, args: &mut Vec<Expr>, loans: &Loans, span: Span, once: bool,
    ) -> Result<(usize, Type), Diagnostic> {
        let declaration = self.deferred_generic_values[&origin].declaration.clone();
        let mut source_call = Expr::new(ExprKind::Call(
            Box::new(Expr::new(ExprKind::Name(declaration), span)), args.clone()), span);
        // Original template preparation solves constraints without evaluating
        // source arguments. It also retains omission and homogeneous rest packs.
        self.prepare_generic_call(context, &mut source_call, None)?;
        let ExprKind::Call(target, prepared) = source_call.kind else { unreachable!() };
        let ExprKind::Name(name) = target.kind else { unreachable!() };
        let function = self.function_names[&name];
        self.function(function)?;
        let checked = self.checked[function].as_ref().ok_or_else(|| Self::pending_result(span))?;
        debug_assert_eq!(checked.parameter_count, self.program.functions[function].params.len());
        let contracts = checked.bindings[..checked.parameter_count].iter().map(ArgumentContract::from).collect::<Vec<_>>();
        *args = prepared;
        Self::prepare_arguments(args, &contracts, span)?;
        let invocation = if once { Mode::Consume } else { Mode::Read };
        Self::access(context, binding, invocation, callee.span, loans)?;
        callee.binding = Some(binding);
        callee.ty = Some(context.bindings[binding].ty);
        let first_capture = context.capture_loans.len();
        let mut active = loans.clone();
        active.entry(binding).or_default().push((invocation, callee.span));
        for (argument, contract) in args.iter_mut().zip(contracts) {
            let actual = self.parameter_argument(context, argument, contract, &active)?;
            self.require(actual, Some(contract.ty), argument.span)?;
            if !actual.is_copy() {
                for root in Self::loan_roots(argument, contract.mode) {
                    active.entry(root).or_default().push((contract.mode, argument.span));
                }
            }
        }
        context.capture_loans.truncate(first_capture);
        let (function, result) = self.resolved_call(context, function, args, span)?;
        let types = args.iter().map(|argument| argument.ty.expect("checked generic value argument")).collect::<Vec<_>>();
        let key = (origin, types, self.effects_only, self.discovering_results);
        if let Some(&method) = self.closure_specializations.get(&key) { return Ok((method, result)); }
        if self.specializations.len() + self.closure_specializations.keys()
            .filter(|(_, _, effects, discovery)| !effects && !discovery).count() >= 128 {
            return Err(Diagnostic::new("unsupported", "Callback origin specialization count exceeds the prototype limit of 128.", span));
        }
        let checked = self.checked[function].as_ref().ok_or_else(|| Self::pending_result(span))?;
        let method = self.closures[origin].methods.len();
        self.closures[origin].methods.push(CheckedClosureMethod {
            invocation: Mode::Read,
            parameters: (0..checked.parameter_count).collect(),
            function: CheckedFunction {
                template: checked.template, specialization: true, name: checked.name.clone(),
                bindings: checked.bindings.clone(), parameter_count: 0,
                result: checked.result, body: checked.body.clone(),
            },
        });
        self.closure_specializations.insert(key, method);
        Ok((method, result))
    }
}

#[cfg(test)]
mod tests {
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
