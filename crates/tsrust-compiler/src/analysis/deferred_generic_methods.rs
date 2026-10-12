//! Materialize checked generic bodies without checking source arguments again.
use super::*;
type BodyProfile<'a> = (usize, Option<&'a str>, &'a [Type]);

impl Analyzer {
    fn deferred_body_profile<'a>(&'a self, origin: usize, types: &'a [Type]) -> BodyProfile<'a> {
        let source = self.deferred_generic_values.get(&origin);
        let empty = source.is_some_and(|source| source.identities.is_empty()) && self.closures[origin].captures.is_empty();
        let group = if empty { self.polymorphic_storage.canonical_origin(origin) } else { origin };
        (group, source.map(|source| source.declaration.as_str()), types)
    }

    fn deferred_body_profiles(&self) -> HashSet<BodyProfile<'_>> {
        self.closure_specializations.keys().filter(|(_, _, effects, discovery)| !effects && !discovery)
            .map(|(origin, types, _, _)| self.deferred_body_profile(*origin, types)).collect()
    }

    #[inline(never)]
    pub(super) fn callable_body_specialization_count(&self) -> usize {
        if self.polymorphic_storage.is_empty() {
            return self.specializations.len() + self.closure_specializations.keys()
                .filter(|(_, _, effects, discovery)| !effects && !discovery).count();
        }
        self.specializations.len() + self.deferred_body_profiles().len()
    }

    fn deferred_method_fits_budget(&self, origin: usize, types: &[Type]) -> bool {
        if self.polymorphic_storage.is_empty() { return self.callable_body_specialization_count() < 128; }
        let mut profiles = self.deferred_body_profiles();
        profiles.insert(self.deferred_body_profile(origin, types));
        self.specializations.len() + profiles.len() <= 128
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn materialize_selected_generic_methods(
        &mut self, context:&mut Context, original: usize, function: usize, result: Type, args: &[Expr],
        accesses: &[named_capture_access::Access], invocation: Mode, origins: &[usize], span: Span,
    ) -> Result<usize, Diagnostic> {
        let mut pairs = Vec::with_capacity(origins.len());
        for &origin in origins {
            if origin != original {
                let previous = &self.deferred_generic_values[&original];
                let next = &self.deferred_generic_values[&origin];
                if previous.declaration != next.declaration
                    || !previous.identities.is_empty() || !next.identities.is_empty()
                    || !self.closures[original].captures.is_empty() || !self.closures[origin].captures.is_empty()
                {
                    pairs.push(self.prepare_deferred_alternative_method(context,origin,args.to_vec(),Some(result),span)?);
                    continue;
                }
            }
            let method = self.materialize_deferred_method(origin, function, result, args, accesses, invocation, span)?;
            pairs.push(crate::polymorphic_invocation_plans::Pair { origin, method });
        }
        self.polymorphic_invocations.publish(pairs, &self.closures, span)
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn materialize_deferred_method(
        &mut self, origin: usize, function: usize, result: Type, args: &[Expr],
        accesses: &[named_capture_access::Access], method_invocation: Mode, span: Span,
    ) -> Result<usize, Diagnostic> {
        let mut types = args.iter().map(|argument| argument.ty.expect("checked generic value argument")).collect::<Vec<_>>();
        // Zero-argument generic calls can select different concrete results.
        // Their environments remain shared; only invocation methods differ.
        types.push(result);
        // Explicit type arguments can differ even when runtime arguments and
        // result types do not. Keep those checked methods distinct as well.
        if let Some(application) = self.program.functions[function].application.clone() {
            for argument in &application.arguments {
                types.push(self.callable_resolver().resolve_value_annotation(argument, false)?);
            }
        }
        let key = (origin, types, self.effects_only, self.discovering_results);
        if let Some(&method) = self.closure_specializations.get(&key) {
            return Ok(method);
        }
        if !self.deferred_method_fits_budget(origin, &key.1) {
            return Err(Diagnostic::new("unsupported", "Callback origin specialization count exceeds the prototype limit of 128.", span));
        }
        let checked = self.checked[function].as_ref().ok_or_else(|| Self::pending_result(span))?;
        let method = self.closures[origin].methods.len();
        let mut bindings = checked.bindings.clone();
        let method_captures = self.deferred_generic_values[&origin].identities.iter().map(|identity| {
            accesses.iter().find(|access| access.binding == *identity).expect("retained lexical capture").slot
        }).collect::<Vec<_>>();
        for (capture, &slot) in self.closures[origin].captures.iter().zip(&method_captures) {
            let binding = &mut bindings[slot];
            binding.capture_reference = match capture.storage {
                Mode::Read if matches!(binding.ty, Type::BorrowedUnion(_) | Type::Union(_)) => false,
                Mode::Read | Mode::Mutate => true,
                Mode::Consume => method_invocation != Mode::Consume,
                Mode::Copy => binding.mode == Mode::Mutate && method_invocation == Mode::Mutate,
            };
            binding.mode = if capture.storage == Mode::Consume || (capture.storage == Mode::Copy && binding.capture_reference) { method_invocation } else { capture.storage };
        }
        self.closures[origin].methods.push(CheckedClosureMethod {
            invocation: method_invocation,
            captures: method_captures,
            parameters: (0..checked.parameter_count).collect(),
            parameter_callbacks:bindings[..checked.parameter_count].iter().map(|binding|match binding.ty {Type::Callback(id)=>Some(self.callables.callbacks[id].structural),_=>None}).collect(),
            function: CheckedFunction {
                storage_exits: checked.storage_exits.clone(), native_returns: checked.native_returns.clone(), native_mutations: checked.native_mutations.clone(),
                template: checked.template, specialization: true, name: checked.name.clone(),
                bindings, parameter_count: 0,
                result: checked.result, body: checked.body.clone(),
            },
        });
        self.closure_specializations.insert(key, method);
        Ok(method)
    }
}
