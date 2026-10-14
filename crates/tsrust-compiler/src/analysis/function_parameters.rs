//! Named parameter initialization separates declaration reservation, defaults,
//! name publication and destructuring so each can suspend without host recursion.
use super::*;

pub(super) struct FunctionParameters {
    pub(super) index: usize,
    pub(super) parameters: Vec<(Token, crate::type_syntax::ValueAnnotation, Option<CallbackAnnotation>)>,
    pub(super) initializers: Vec<ParameterInitializer>,
    signature: Signature,
    pub(super) next: usize,
}

impl Analyzer {
    pub(super) fn reserve_function_parameters(&mut self, index: usize) -> Result<(Context, FunctionParameters), Diagnostic> {
        // Discovery has collected concrete recursive components. Parameter
        // bindings must carry that same callback family and shared lifetime.
        let signature = &mut self.signatures[index];
        for ty in signature.params.iter_mut().chain(&mut signature.body_params) {
            *ty = self.scoped_records.recursive_parameter(*ty);
        }
        let mut context = Context::new(self.program.functions[index].name.text.clone());
        context.effects_only = self.effects_only;
        context.template = self.templates.contains_key(&index);
        let parameters = self.program.functions[index].params.clone();
        let initializers = self.program.functions[index].parameter_initializers.clone();
        let signature = self.signatures[index].clone();
        // Reserve the ordinary contiguous parameter ABI before defaults create
        // any local temporaries; expose names only after their initialization.
        for (position, (param, _, _)) in parameters.iter().enumerate() {
            let body_ty = if matches!(initializers[position], ParameterInitializer::Optional(_)) { signature.params[position] } else { signature.body_params[position] };
            Self::bind(&mut context, param, body_ty, true)?;
        }
        context.names.clear();
        self.named_captures.attach(index, &mut context);
        Ok((context, FunctionParameters { index, parameters, initializers, signature, next: 0 }))
    }

    pub(super) fn named_default_mode(&self, context: &Context, frame: &FunctionParameters) -> Result<Mode, Diagnostic> {
        let index = frame.index;
        let position = frame.next;
        let initializer = &frame.initializers[position];
        let body_ty = context.bindings[position].ty;
        if self.assumed_modes.is_none() && matches!(initializer, ParameterInitializer::Default { .. })
            && !body_ty.is_copy() && !matches!(self.unions.payload(body_ty).unwrap_or(body_ty), Type::Callback(_) | Type::Closure(_) | Type::StoredCallable(_)) {
            return Err(Diagnostic::new("recursion-inference", "Default owner access requires initialized parameter access inference.", frame.parameters[position].0.span));
        }
        Ok(self.assumed_modes.as_ref().and_then(|modes| modes.get(index)).and_then(|modes| modes.get(position)).copied().unwrap_or(Mode::Consume))
    }

    pub(super) fn finish_named_parameter(&mut self, context: &mut Context, frame: &mut FunctionParameters, mut checked_initializer: Option<Box<Expr>>, default_mode: Mode) -> Result<(), Diagnostic> {
        let index = frame.index;
        let position = frame.next;
        let parameters = &frame.parameters;
        let (param, _, annotation) = &parameters[position];
        let input_ty = &frame.signature.params[position];
        let initializer = &frame.initializers[position];
        let binding = position;
        let mut body_ty = context.bindings[binding].ty;
        body_ty=self.owning_default_family(*input_ty,body_ty,&mut checked_initializer,param.span)?;
        context.bindings[binding].ty=body_ty;
        context.names.insert(param.text.clone(), binding);
        let ty = &body_ty;
        context.bindings[binding].input_ty = Some(*input_ty);
        context.bindings[binding].omission = initializer.omission();
        context.bindings[binding].rest=self.program.functions[index].rest_parameter && position+1==parameters.len();
        if context.bindings[binding].rest {
            if !matches!(body_ty,Type::Array(_)) {return Err(Diagnostic::new("rest-type","A rest parameter requires a supported homogeneous array type.",param.span));}
            context.bindings[binding].mode=Mode::Consume;
        }
        context.bindings[binding].declaration_span = Some(param.span);
        context.bindings[binding].initializer = checked_initializer;
        if matches!(default_mode,Mode::Read|Mode::Mutate)
            && let Some(value)=context.bindings[binding].initializer.as_deref()
            && let Some(source)=Self::root_binding(value)
            && source!=binding {
            // Pattern locals follow reserved argument IDs; visibility, not
            // numeric binding order, proves the source is initialized.
            context.bindings[binding].view_roots=Self::view_sources(&context,value);
        }
        self.initialized_parameter_fact(context,binding);
        let source_kind = crate::callable_annotations::nullable_kind(
            &self.program.functions[index].params[binding].1,
        );
        let nullable_kind = match self.unions.payload(*ty).unwrap_or(*ty) {
            Type::Callback(id) => Some(self.callables.callbacks[id].kind),
            Type::StoredCallable(id) => self
                .callables
                .storage
                .declaration(id)
                .map(|id| self.callables.callbacks[id].kind),
            _ => source_kind,
        };
        if let Some(kind) = nullable_kind {
            context.bindings[binding].callback_kind = Some(kind);
            context.bindings[binding].callback_reference =
                matches!(kind, CallbackKind::Reusable | CallbackKind::Read);
            context.bindings[binding].callback_invocation = Some(kind.invocation());
            if kind.is_once() || kind.is_owned() {
                context.bindings[binding].mode = Mode::Consume;
            }
        }
        if let Some(annotation) = annotation {
            context.bindings[binding].callback_kind = Some(annotation.kind);
            context.bindings[binding].callback_reference =
                matches!(annotation.kind, CallbackKind::Reusable | CallbackKind::Read);
            context.bindings[binding].callback_invocation = Some(annotation.kind.invocation());
        }
        if annotation
            .as_ref()
            .is_some_and(|c| c.kind.is_once() || c.kind.is_owned())
        {
            // The annotation transfers ownership even when never invoked.
            context.bindings[binding].mode = Mode::Consume;
        }
        if matches!(self.program.functions[index].parameter_access[position],Some((Mode::Read,_))) && !body_ty.is_copy() {
            context.readonly_parameters.insert(binding);
        }
        if matches!(self.program.functions[index].parameter_access[position], Some((Mode::Mutate, _))) { context.mutable_parameters.insert(binding); }
        frame.next += 1;
        Ok(())
    }
}
