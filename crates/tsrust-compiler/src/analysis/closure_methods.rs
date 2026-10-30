//! Invocation specialization keeps one capture environment for every method.
use super::*;

impl Analyzer {
    // Calls retain argument, contract, and diagnostic temporaries in their own
    // frame, rather than in every recursive unary or binary expression.
    // Source replay specializes only invocation bodies. The existing environment
    // retains the same owners, capture field layout, access and invocation mode.
    pub(super) fn closure_method_argument_types(&self, origin: usize, args: &[Expr]) -> Vec<Type> {
        let closure=&self.closures[origin];
        args.iter().zip(&closure.parameters).map(|(arg,&parameter)| {
            if closure.function.bindings[parameter].omission.is_omittable()
                && let ExprKind::Inject{value,..}=&arg.kind {value.ty.expect("checked supplied member")}
            else {arg.ty.expect("checked method argument")}
        }).collect::<Vec<_>>()
    }

    pub(super) fn prepare_closure_method(&mut self, origin: usize, types: Vec<Type>, span: Span) -> Result<PreparedMethod, Diagnostic> {
        let source = self.closure_method_sources.get(&origin).cloned().ok_or_else(|| Diagnostic::new("unsupported", "Callable traversal visitor requires an arrow environment with a checked specialization body.", span))?;
        let key = (
            origin,
            types.clone(),
            self.effects_only,
            self.discovering_results,
        );
        if let Some(&method) = self.closure_specializations.get(&key) {
            let result = self.closures[origin].methods[method].function.result;
            if matches!(result, Type::Callable(_)) {
                return Err(Self::pending_result(span));
            }
            return Ok(PreparedMethod::Ready(method, result));
        }
        let mut context = source.context.clone();
        context.effects_only = self.effects_only;
        context.template |= self.discovering_results;
        for (&parameter, actual) in source.parameters.iter().zip(&types) {
            let input=context.bindings[parameter].input_ty.unwrap_or(context.bindings[parameter].ty);
            if context.bindings[parameter].omission.is_omittable()
                && let Some(union)=input.union()
                && let Some(Type::Callback(callback))=self.unions.payload(input) {
                let payload=if matches!(actual,Type::Null|Type::Undefined) {self.absent_callback(callback)}
                    else {self.unions.payload(*actual).unwrap_or(*actual)};
                let specialized=self.unions.intern(self.unions.members(union).into_iter().map(|ty|if matches!(ty,Type::Callback(_)){payload}else{ty}),span)?;
                context.bindings[parameter].input_ty=Some(specialized);
                if context.bindings[parameter].omission==crate::ParameterOmission::Defaulted {
                    let body=context.bindings[parameter].ty;
                    let mut initializer=context.bindings[parameter].initializer.clone();
                    let concrete=self.owning_default_family(specialized,body,&mut initializer,span)?;
                    context.bindings[parameter].ty=concrete;
                    context.bindings[parameter].initializer=initializer;
                } else {context.bindings[parameter].ty=specialized;}
                continue;
            }
            if context.bindings[parameter].ty.union().is_some()
                && matches!(
                    self.unions.payload(context.bindings[parameter].ty),
                    Some(Type::Callback(_))
                )
            {
                context.bindings[parameter].ty = actual.owning();
            }

            if matches!(context.bindings[parameter].ty, Type::Callback(_)) {
                context.bindings[parameter].ty = if let Type::Callback(id) = *actual {
                    // Distinct method argument paths keep distinct owner IDs even
                    // when a caller passes the same witness at two positions.
                    let contract = &self.callables.callbacks[id];
                    let copy = CallbackContract {
                        structural: contract.structural,
                        parameter_names: contract.parameter_names.clone(),
                        params: contract.params.clone(),
                        result: contract.result,
                        kind: contract.kind,
                    };
                    let id = self.callables.callbacks.len();
                    self.callables.callbacks.push(copy);
                    context.template |= self.callback_needs_context(id);
                    Type::Callback(id)
                } else {
                    *actual
                };
            }
            if !context.bindings[parameter].omission.is_omittable() {
                context.bindings[parameter].input_ty=Some(context.bindings[parameter].ty);
            }
        }
        let method = self.closures[origin].methods.len();
        let invocation = self.closures[origin].invocation;
        let captures = self.closures[origin].captures.iter().map(|capture| capture.inner).collect();
        self.closures[origin].methods.push(CheckedClosureMethod {
            invocation,
            captures,
            parameters: source.parameters.clone(),
            parameter_callbacks:source.parameters.iter().map(|&parameter|match context.bindings[parameter].ty {Type::Callback(id)=>Some(self.callables.callbacks[id].structural),_=>None}).collect(),
            function: CheckedFunction {
                storage_exits: context.storage_exits.clone(), native_returns: context.native_returns.clone(), native_mutations: context.native_mutations.clone(),
                template: context.template,
                specialization: true,
                name: context.function_name.clone(),
                bindings: context.bindings.clone(),
                parameter_count: 0,
                result: source.result,
                body: Vec::new(),
            },
        });
        self.closure_specializations.insert(key, method);
        self.depth += 1;
        Ok(PreparedMethod::Body(MethodBody { origin, method, context, body: source.body, result: source.result, span }))
    }

    pub(super) fn finish_closure_method(&mut self, frame: MethodBody, checked: Result<bool, Diagnostic>) -> Result<(usize, Type), Diagnostic> {
        let MethodBody { origin, method, mut context, mut body, result: source_result, span } = frame;
        self.depth -= 1;
        let returned = checked.map_err(|error| self.filter_ownership_fixes(&context, error))?;
        if source_result != Type::Void && !returned {
            return Err(Diagnostic::new(
                "type",
                "Closure must return a value on every path.",
                span,
            ));
        }
        let result = if context.inferred_body_result {
            self.inferred_body_result(&body)?
        } else if self.callable_result_type(source_result) {
            context
                .returned_callable
                .ok_or_else(|| Self::pending_result(span))?
        } else {
            source_result
        };
        if !context.template && self.has_pending_result(result) {
            return Err(Self::pending_result(span));
        }
        self.finalize_nullable_locals(&mut context.bindings, &mut body)?;
        let captures = self.closures[origin]
            .captures
            .iter()
            .map(|c| (c.inner, c.access))
            .collect::<HashMap<_, _>>();
        for (id, binding) in context.bindings[..context.capture_boundary]
            .iter_mut()
            .enumerate()
        {
            if binding.used.is_some() && !captures.contains_key(&id) {
                return Err(Diagnostic::new(
                    "unsupported",
                    "Callable method specialization cannot change its captured owner layout.",
                    span,
                ));
            }
            if let Some(&access) = captures.get(&id) {
                if binding.used.is_some() && binding.mode > access {
                    return Err(Diagnostic::new(
                        "callback-access",
                        "Callable method specialization cannot strengthen captured storage access.",
                        span,
                    ));
                }
                let original = &self.closures[origin].function.bindings[id];
                binding.mode = original.mode;
                binding.capture_reference = original.capture_reference;
            }
        }
        self.closures[origin].methods[method].function = CheckedFunction {
                storage_exits: context.storage_exits.clone(), native_returns: context.native_returns.clone(), native_mutations: context.native_mutations.clone(),
            template: context.template,
            specialization: true,
            name: context.function_name,
            bindings: context.bindings,
            parameter_count: 0,
            result,
            body,
        };
        Ok((method, result))
    }

}

pub(super) enum PreparedMethod { Ready(usize, Type), Body(MethodBody) }

pub(super) struct MethodBody {
    pub(super) origin: usize,
    pub(super) method: usize,
    pub(super) context: Context,
    pub(super) body: Vec<Stmt>,
    pub(super) result: Type,
    pub(super) span: Span,
}
