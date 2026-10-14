//! Saved closure state across body checking. The shared driver will own suspension.
//! Parameter defaults and patterns can suspend before the body starts.
use super::super::*;

pub(super) struct Frame {
    program_scoped: bool,
    origin: usize,
    span: Span,
    pub(super) result: Type,
    inferred_result: bool,
    escapes: bool,
    visible: Vec<usize>,
    parameters: Vec<usize>,
    method_template: bool,
    declared_result: Type,
    method_context: Option<Context>,
    source_body: Option<Vec<Stmt>>,
    restoration: capture_restoration::Tracker,
    access_checks: Vec<(usize, String, Mode, Span)>,
}
pub(super) struct Parameters {
    program_scoped: bool,
    origin:usize,span:Span,result:Type,inferred_result:bool,escapes:bool,
    visible:Vec<usize>,parameters:Vec<usize>,declared:Vec<Type>,
    pub(super) next:usize,
}
pub(super) struct ParameterWork {
    pub(super) value:Option<Box<Expr>>,
    pub(super) mode:Mode,
    pub(super) expected:Option<Type>,
    default:Option<super::defaults::DefaultFrame>,
    inferred:bool,body_ty:Type,default_mode:Mode,
}
pub(super) enum Started { Ready(Type), Parameters(Box<Parameters>,Box<Context>) }
pub(super) enum Prepared { Ready(Type), Body(Box<Frame>, Box<Context>) }

impl Analyzer {
    pub(in crate::analysis) fn closure(
        &mut self,
        outer: &mut Context,
        syntax: (usize, &[CallableParameter], &ResultAnnotation),
        contextual: Option<Type>,
        body: &mut Vec<Stmt>,
        span: Span,
        loans: &Loans,
    ) -> Result<Type, Diagnostic> {
        let (frame, mut context) = match self.prepare_closure(outer, syntax, contextual, body, span, loans)? {
            Prepared::Ready(ty) => return Ok(ty),
            Prepared::Body(frame, context) => (frame, context),
        };
        let returned = self.check_source_callable_body(&mut context, Self::uninitialized_body(body), frame.result);
        self.finish_closure(outer, frame, context, body, loans, returned)
    }

    pub(super) fn start_closure(
        &mut self,
        outer: &mut Context,
        syntax: (usize, &[CallableParameter], &ResultAnnotation),
        contextual: Option<Type>,
        body: &mut Vec<Stmt>,
        span: Span,
        loans: &Loans,
    ) -> Result<Started, Diagnostic> {
        let (origin, params, result) = syntax;
        let generic_wrapper = if let [statement] = body.as_slice()
            && let StmtKind::Return(Some(call)) = &statement.kind
            && let ExprKind::Call(callee, _) = &call.kind
            && call.span == span && callee.span == span
            && let ExprKind::Name(name) = &callee.kind
            && self.generic_functions.template(name).is_some() { Some(name.as_str()) } else { None };
        if let Some(name) = generic_wrapper
            && (contextual.is_none() || matches!(contextual, Some(Type::Closure(id)) if self.deferred_generic_values.contains_key(&id)))
            && (params.iter().any(|parameter| matches!(parameter.ty, ParameterAnnotation::Inferred(_)))
                || (params.is_empty() && matches!(result, ResultAnnotation::Inferred(_)))) {
            return self.deferred_captured_generic_value(outer, name, origin, span, loans).map(Started::Ready);
        }
        let contextual_result = generic_wrapper.and_then(|_| contextual)
            .and_then(|ty| self.checked_generic_callable_shape(self.unions.payload(ty).unwrap_or(ty)))
            .map(|(_, result)| result);
        let inferred_result = matches!(result, ResultAnnotation::Inferred(_)) && contextual_result.is_none();
        let result = if let Some(result) = contextual_result { result }
            else if inferred_result { Type::Void } else { self.resolve_result(result)? };
        let escapes = self.escaping.contains(&origin) || self.program_callback_sites.get(&span.start) == Some(&CallbackKind::Program);
        let mut visible = outer.names.values().copied().collect::<Vec<_>>();
        // A nested function called in the body can need a variable that has no
        // visible name here: a shadowed one, or one that only a called function uses.
        let hidden_captures = !self.program.local_capture_plans.is_empty();
        if hidden_captures {
            visible.extend(outer.source_bindings.values().copied());
        }
        visible.sort_unstable();
        visible.dedup();
        let mut context = capture_context::project(outer, &visible);
        capture_context::retain_module_references(&mut context, &self.program.module_owner_bindings);
        // Preserve lexical aliases as well as binding declaration names. Adapter
        // wrappers introduce a private alias to avoid parameter-name shadowing.
        context.names = outer.names.iter().map(|(name, slot)| (name.clone(), visible.binary_search(slot).expect("visible name"))).collect();
        context.caller_error_origins = outer.caller_error_origins.clone();
        context.inferred_body_result = inferred_result;
        if self.discovering_results {
            context.pending_names = outer.pending_names.clone();
        }
        let program_scoped = self.program_callback_sites.contains_key(&span.start) || matches!(contextual, Some(Type::Callback(id)) if matches!(self.callables.callbacks[id].kind, CallbackKind::Program | CallbackKind::ProgramReusable));
        let contextual = contextual.and_then(|ty| match self.unions.payload(ty).unwrap_or(ty) {
            Type::Callback(id) => Some(self.callables.callbacks[id].params.clone()),
            Type::Callable(id) => Some(self.callables.results[id].params.clone()),
            Type::StoredCallable(id) => self.callables.storage.declaration(id).map(|id|self.callables.callbacks[id].params.clone()),
            _ => None,
        });
        let mut parameters = Vec::new();
        let mut declared = Vec::new();
        let inherited_names = context.names.clone();
        // Parameter names shadow captures, but must remain unique among themselves.
        context.names.clear();
        for (position, param) in params.iter().enumerate() {
            let ty = match &param.ty {
                ParameterAnnotation::Named(annotation) => self.callable_resolver().resolve_value_annotation(annotation,true)?,
                ParameterAnnotation::Callback(annotation) => self.resolve_callback(annotation)?,
                ParameterAnnotation::Inferred(_) => contextual.as_ref().and_then(|params|params.get(position)).map_or(Type::Void, |parameter| {
                    // The compiler's accepted input type supplies context; its
                    // undefined admission is not a body type inference rule.
                    if matches!(param.initializer,ParameterInitializer::Default{..}) {
                        parameter.ty.union().and_then(|id|self.unions.intern(self.unions.members(id).into_iter().filter(|ty|*ty!=Type::Undefined),param.name.span).ok()).unwrap_or(parameter.ty)
                    } else {parameter.ty}
                }),
            };
            let ty = self.scoped_records.recursive_parameter(ty);
            self.arrow_parameter_contract(param, ty)?;
            let binding = Self::bind(&mut context,&param.name,ty,true)?;
            parameters.push(binding);
            declared.push(ty);
        }
        context.names = inherited_names;
        Ok(Started::Parameters(Box::new(Parameters{program_scoped,origin,span,result,inferred_result,escapes,visible,parameters,declared,next:0}),Box::new(context)))
    }

    pub(super) fn prepare_closure(&mut self,outer:&mut Context,syntax:(usize,&[CallableParameter],&ResultAnnotation),contextual:Option<Type>,body:&mut Vec<Stmt>,span:Span,loans:&Loans)->Result<Prepared,Diagnostic>{
        let (mut frame,mut context)=match self.start_closure(outer,syntax,contextual,body,span,loans)?{
            Started::Ready(ty)=>return Ok(Prepared::Ready(ty)),Started::Parameters(frame,context)=>(frame,context),
        };
        while frame.next<syntax.1.len(){
            let param=&syntax.1[frame.next];
            let mut work=self.prepare_closure_default(&context,&frame,param)?;
            let actual=if let Some(value)=work.value.as_mut(){Some(self.expression(&mut context,value,work.mode,&Loans::new(),work.expected)?)}else{None};
            self.finish_closure_parameter(&mut context,&mut frame,param,work,actual)?;
            self.initialize_parameter_pattern(&mut context,&param.name,body,param.access.as_ref().is_some_and(|access|access.text=="consume"))?;
        }
        let prepared=self.complete_closure_parameters(frame,&mut context,syntax.1,body)?;
        Ok(Prepared::Body(prepared,context))
    }

    pub(super) fn prepare_closure_default(&self,context:&Context,frame:&Parameters,param:&CallableParameter)->Result<ParameterWork,Diagnostic>{
        let body_ty=frame.declared[frame.next];
        let default_mode=if body_ty.is_copy(){Mode::Copy}else{match param.access.as_ref().map(|access|access.text.as_str()){Some("read")=>Mode::Read,Some("mutate")=>Mode::Mutate,_=>Mode::Consume}};
        let inferred=body_ty==Type::Void && matches!(param.ty,ParameterAnnotation::Inferred(_));
        let value=match &param.initializer{ParameterInitializer::Default{value,..}=>Some(value.clone()),_=>None};
        if inferred && value.is_none(){return Err(Diagnostic::new("parameter-inference",format!("Cannot infer the type of arrow parameter `{}`. Add a type annotation, such as `({}: string) => ...`, or pass this arrow where a callback parameter type is known.",param.name.text,param.name.text),param.name.span));}
        let default=if inferred{None}else{value.as_ref().map(|value|self.prepare_default_work(context,value,body_ty,default_mode))};
        let(mode,expected)=if let Some(default)=&default{(default.mode,default.expected)}else{(Mode::Consume,value.as_ref().and_then(|value|self.expression_type_hint(context,value)))};
        Ok(ParameterWork{value,mode,expected,default,inferred,body_ty,default_mode})
    }

    pub(super) fn finish_closure_parameter(&mut self,context:&mut Context,frame:&mut Parameters,param:&CallableParameter,mut work:ParameterWork,actual:Option<Type>)->Result<(),Diagnostic>{
        if let Some(actual)=actual{
            if work.inferred{work.body_ty=actual;}
            else if let Some(default)=work.default.take(){self.finish_default_work(context,work.value.as_mut().expect("default value"),default,actual)?;}
        }
        let binding=frame.parameters[frame.next];
        let mut body_ty=work.body_ty;let default_mode=work.default_mode;let checked_initializer=work.value;
        let input_ty=if param.initializer.omission().is_omittable() {self.unions.intern([body_ty,Type::Undefined],param.name.span)?} else {body_ty};
        if matches!(param.initializer,ParameterInitializer::Optional(_)) {body_ty=input_ty;}
        context.bindings[binding].ty=body_ty;
        context.bindings[binding].input_ty=Some(input_ty);
        context.bindings[binding].omission=param.initializer.omission();
        context.bindings[binding].rest=param.rest;
        if param.rest {
            if !matches!(body_ty,Type::Array(_)) {return Err(Diagnostic::new("rest-type","A rest parameter requires a supported homogeneous array type.",param.name.span));}
            if param.access.as_ref().is_some_and(|access|access.text!="consume") {return Err(Diagnostic::new("rest-access","A rest parameter owns its pack; only explicit consume access is supported.",param.name.span));}
            context.bindings[binding].mode=Mode::Consume;
        }
        context.bindings[binding].declaration_span=Some(param.name.span);
        context.bindings[binding].initializer=checked_initializer;
        if matches!(default_mode,Mode::Read|Mode::Mutate)
            && let Some(value)=context.bindings[binding].initializer.as_deref()
            && Self::root_binding(value).is_some() {
            context.bindings[binding].view_roots=Self::view_sources(&context,value);
        }
        context.names.insert(param.name.text.clone(),binding);
        self.initialized_parameter_fact(context,binding);
        if body_ty.is_copy() {context.bindings[binding].mode=Mode::Copy;}
        if let Type::Callback(id)=self.unions.payload(body_ty).unwrap_or(body_ty) {
            let kind=self.callables.callbacks[id].kind;
            context.bindings[binding].callback_kind=Some(kind);
            context.bindings[binding].callback_reference=matches!(kind,CallbackKind::Reusable|CallbackKind::Read);
            context.bindings[binding].callback_invocation=Some(kind.invocation());
            if kind.is_owned()||kind.is_once() {context.bindings[binding].mode=Mode::Consume;}
        }
        if param.access.as_ref().is_some_and(|access|access.text=="read") && !body_ty.is_copy() {
            context.readonly_parameters.insert(binding);
        }
        if param.access.as_ref().is_some_and(|access|access.text=="mutate") && !body_ty.is_copy() {
            context.mutable_parameters.insert(binding);
        }
        frame.next+=1;
        Ok(())
    }

    pub(super) fn complete_closure_parameters(&mut self,frame:Box<Parameters>,context:&mut Context,params:&[CallableParameter],body:&mut Vec<Stmt>)->Result<Box<Frame>,Diagnostic>{
        let Parameters{program_scoped,origin,span,result,inferred_result,escapes,visible,parameters,..}=*frame;
        let method_template = parameters.iter().any(|&parameter| matches!(self.unions.payload(context.bindings[parameter].ty).unwrap_or(context.bindings[parameter].ty),Type::Callback(_)));
        let source_body=method_template.then(||body.clone());
        let declared_result=result;
        let method_context = method_template.then(|| context.clone());
        if method_template {
            context.template = true;
        }
        for (param, &binding) in params.iter().zip(&parameters) {
            if param.access.as_ref().is_some_and(|access| access.text == "read") && !param.rest && !context.bindings[binding].ty.is_copy() {
                context.readonly_parameters.insert(binding);
            }
        }
        let restoration=capture_restoration::Tracker::new((0..visible.len()).collect());
        context.capture_restoration=Some(restoration.clone());
        let access_checks = params.iter().zip(&parameters).filter_map(|(param, &binding)| {
            let access = param.access.as_ref().filter(|_| !param.rest)?;
            let mode = match access.text.as_str() { "read" => Mode::Read, "mutate" => Mode::Mutate, _ => Mode::Consume };
            Some((binding, param.name.text.clone(), mode, access.span))
        }).collect();
        Ok(Box::new(Frame {
            program_scoped,
            origin, span, result, inferred_result, escapes, visible, parameters,
            method_template, declared_result, method_context, source_body, restoration, access_checks,
        }))
    }

    pub(super) fn finish_closure(&mut self, outer: &mut Context, frame: Box<Frame>, mut context: Box<Context>, body: &mut Vec<Stmt>, loans: &Loans, returned: Result<bool, Diagnostic>) -> Result<Type, Diagnostic> {
        let Frame { program_scoped, origin, span, mut result, inferred_result, escapes, visible, parameters,
            method_template, declared_result, method_context, source_body, restoration, access_checks } = *frame;
        let returned = returned.map_err(|error| self.filter_ownership_fixes(&context, error))?;
        for (binding, name, declared, access_span) in access_checks {
            Self::stated_parameter_access(&name, &mut context.bindings[binding], declared, access_span)?;
        }
        if inferred_result { result = self.inferred_body_result(body)?; }
        if result != Type::Void && !returned {
            return Err(Diagnostic::new(
                "type",
                "Closure must return a value on every path.",
                span,
            ));
        }
        let result = if self.callable_result_type(result) {
            if self.discovering_results && context.returned_callable.is_none() {
                return Err(Self::pending_result(span));
            }
            context
                .returned_callable
                .expect("checked callable closure return")
        } else {
            result
        };
        let result = if method_template && !inferred_result {
            declared_result
        } else {
            result
        };
        self.finalize_nullable_locals(&mut context.bindings, body)?;
        let restoration_bounded=capture_restoration::bounded_body(body);
        let mut captures = Vec::new();
        let mut program_captures = Vec::new();
        let mut invocation = Mode::Read;
        for (inner, &outer_id) in visible.iter().enumerate() {
            let binding = &context.bindings[inner];
            if let Some(usage) = binding.used {
                if self.discovering_results && self.has_pending_result(binding.ty) {
                    return Err(Self::pending_result(span));
                }
                let access = binding.mode;
                let restored=access==Mode::Consume && restoration.permits(inner) && restoration_bounded;
                invocation = invocation.max(if restored {Mode::Mutate}else{access});
                if escapes && binding.callback_kind.is_some_and(|kind| !kind.is_owned()) {
                    return Err(Diagnostic::new("callback-escape",
                        "An owned escaping closure cannot retain a borrowed callback input. Use an owned callback parameter or keep the borrowing closure in the caller's scope.", span));
                }
                let module_capture = self.program.module_owner_bindings.iter()
                    .any(|declaration| context.source_bindings.get(&declaration.start) == Some(&inner));
                if program_scoped && escapes && !module_capture && !binding.ty.is_copy()
                    && (context.readonly_parameters.contains(&inner) || context.mutable_parameters.contains(&inner)) {
                    return Err(Diagnostic::new("callback-escape",
                        "A registered callback cannot retain a borrowed local or parameter. Move owned data into the callback, or use program-owned module storage.", span));
                }
                let storage = if module_capture {
                    if access == Mode::Consume {
                        return Err(Diagnostic::new("module-state-consume", "A callback cannot consume program-owned module storage; use explicit clone().", usage));
                    }
                    access.max(Mode::Read)
                } else if binding.ty.is_copy() {
                    if access == Mode::Mutate && !escapes {
                        Mode::Mutate
                    } else {
                        Mode::Copy
                    }
                } else if escapes {
                    Mode::Consume
                } else {
                    access
                };
                if !self.effects_only
                    && escapes
                    && let Type::Closure(id) = binding.ty
                    && self.closures[id].borrowed
                    && self.borrows_local_storage(binding.ty)
                {
                    return Err(Diagnostic::new(
                        "closure-escape",
                        "An escaping closure cannot retain a borrowed callable environment.",
                        span,
                    )
                    .related(self.closures[id].span, "Borrowed callable is created here."));
                }
                if program_scoped && module_capture {
                    let declaration = *self.program.module_owner_bindings.iter()
                        .find(|declaration| context.source_bindings.get(&declaration.start) == Some(&inner))
                        .expect("program binding identity");
                    program_captures.push(ProgramCapture { declaration, inner, access: storage, span: usage });
                    continue;
                }
                Self::access(outer, outer_id, storage, span, loans)?;
                if matches!(storage, Mode::Read | Mode::Mutate) {
                    outer.capture_loans.push((outer_id, storage, span));
                }
                captures.push(Capture {
                    restored,
                    outer: outer_id,
                    inner,
                    storage,
                    access,
                    span: usage,
                });
            }
        }
        // Captured scalar writes borrow local storage, or mutate a private copied
        // field in an escaping environment. Record the actual representation so
        // scalar reads and whole-binding stores lower through references correctly.
        for capture in &captures {
            let binding = &mut context.bindings[capture.inner];
            binding.capture_reference = match capture.storage {
                Mode::Read if matches!(binding.ty, Type::BorrowedUnion(_) | Type::Union(_)) => {
                    false
                }
                Mode::Read | Mode::Mutate => true,
                Mode::Consume => invocation != Mode::Consume,
                Mode::Copy => capture.access == Mode::Mutate && invocation == Mode::Mutate,
            };
            binding.mode = if capture.storage == Mode::Consume
                || (capture.storage == Mode::Copy && binding.capture_reference)
            {
                invocation
            } else {
                capture.storage
            };
        }
        for capture in &program_captures {
            let binding = &mut context.bindings[capture.inner];
            binding.capture_reference = true;
            binding.mode = capture.access;
        }
        // Provisional recursive summaries can overstate borrowed lifetimes.
        // Cap the count just above the supported limit; inference never emits
        // this layout, and final checking still enforces the actual guard.
        let lifetime_count = captures.iter().fold(0usize, |total, capture| {
            let nested = crate::environment_lifetimes::count_with_storage(
                context.bindings[capture.inner].ty, &self.program.records, &self.callables.storage,
                |id| self.unions.members(id),
                |ty| match ty {
                    Type::Closure(id) => self.closures[id].lifetime_count,
                    Type::BorrowedCallable(id) => self.callables.storage.origins(id).expect("checked borrowed origins").iter().map(|&origin| self.closures[origin].lifetime_count).sum(),
                    // A captured scoped family keeps one lifetime for each carried view.
                    Type::StoredCallable(id) => self.callables.storage.scoped_native_family(id).map_or(0, crate::native_view_callable_storage::Family::lifetimes),
                    _ => 0,
                },
            );
            (total + usize::from(matches!(capture.storage, Mode::Read | Mode::Mutate)) + nested)
                .min(129)
        });
        if !self.effects_only && lifetime_count > 128 {
            return Err(Diagnostic::new(
                "unsupported",
                "Closure environment lifetime complexity exceeds the prototype limit of 128.",
                span,
            ));
        }
        let borrowed = lifetime_count != 0;
        let key = ClosureKey {
            program_scoped,
            origin,
            template: outer.template,
            escapes,
            depth: context.closure_depth,
            result,
            invocation,
            captures: captures
                .iter()
                .map(|capture| {
                    let binding = &context.bindings[capture.inner];
                    CaptureKey {
                        name: binding.name.clone(),
                        ty: binding.ty,
                        storage: capture.storage,
                        access: capture.access,
                        callback_invocation: binding.callback_invocation,
                        callback_reference: binding.callback_reference,
                        callback_kind: binding.callback_kind,
                    }
                })
                .collect(),
            parameters: parameters
                .iter()
                .map(|&id| {
                    let binding = &context.bindings[id];
                    (binding.ty, binding.mode)
                })
                .collect(),
        };
        if let Some(&id) = self.closure_types.get(&key) {
            // Creation effects and loans above still apply to this invocation.
            // Emission reuses the first fully checked body and capture layout.
            if !self.discovering_results {
                self.closures[id].function.native_returns = context.native_returns.clone();
                self.closures[id].function.native_mutations = context.native_mutations.clone();
                self.closures[id].function.storage_exits = context.storage_exits.clone();
                self.closures[id].function.bindings = context.bindings;
                self.closures[id].function.body = if method_template {
                    body.clear();
                    Vec::new()
                } else {
                    std::mem::take(body)
                };
                self.closures[id].captures = captures;
                self.closures[id].program_captures = program_captures;
                self.closures[id].parameters = parameters;
                self.validated_closures.insert(id);
            } else {
                body.clear();
            }
            if let (Some(context), Some(body)) = (method_context, source_body) {
                self.closure_method_sources.insert(
                    id,
                    ClosureMethodSource {
                        context,
                        parameters: self.closures[id].parameters.clone(),
                        result: declared_result,
                        body,
                    },
                );
            }
            self.remember_native_environment(outer,id);
            return Ok(Type::Closure(id));
        }
        let id = self.closures.len();
        self.closure_types.insert(key, id);
        if let (Some(context), Some(body)) = (method_context, source_body) {
            self.closure_method_sources.insert(
                id,
                ClosureMethodSource {
                    context,
                    parameters: parameters.clone(),
                    result: declared_result,
                    body,
                },
            );
        }
        if !self.discovering_results {
            self.validated_closures.insert(id);
        }
        self.closures.push(CheckedClosure {
                program_scoped, program_captures,
            polymorphic: method_template,
            methods: Vec::new(),
            forward: None,
            borrowed,
            lifetime_count,
            depth: context.closure_depth,
            parameters,
            function: CheckedFunction {
                storage_exits: context.storage_exits.clone(), native_returns: context.native_returns.clone(), native_mutations: context.native_mutations.clone(),
                template: outer.template,
                specialization: false,
                name: outer.function_name.clone(),
                bindings: context.bindings,
                parameter_count: 0,
                result,
                body: if method_template {
                    body.clear();
                    Vec::new()
                } else {
                    std::mem::take(body)
                },
            },
            captures,
            span,
            escapes,
            invocation,
        });
        self.remember_native_environment(outer,id);
        Ok(Type::Closure(id))
    }

}
