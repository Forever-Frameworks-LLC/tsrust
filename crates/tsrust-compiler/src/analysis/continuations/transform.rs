//! Collection transforms preserve receiver, callback and method completion order.
use super::super::*;
pub(in crate::analysis) type Options = (bool, bool, Mode, bool, bool, bool, bool);

pub(in crate::analysis) struct Start {
    pub(in crate::analysis) options: Options,
    callback_hint: Option<Type>, value_hint: Option<Type>,
    set_for_each: bool, map_for_each: bool,
    pub(in crate::analysis) receiver_hint: Option<Type>,
}
pub(in crate::analysis) struct Callback {
    options: Options, receiver: Type, value: Type, position_type: Type, lends_callable: bool,
    pub(in crate::analysis) active: Loans,
    pub(in crate::analysis) supplied_types: Vec<Type>,
    pub(in crate::analysis) checked_hint: Option<Type>,
    boundary: crate::capture_loan_identity::Boundary,
}
pub(in crate::analysis) struct MethodRequest { pub(in crate::analysis) origin: usize, pub(in crate::analysis) types: Vec<Type> }
pub(in crate::analysis) struct CompletionFrame {
    options: Options, receiver: Type, lends_callable: bool,
    boundary: crate::capture_loan_identity::Boundary, result: Type,
    pub(in crate::analysis) method: Option<MethodRequest>,
}

impl Analyzer {
    pub(in crate::analysis) fn prepare_transform(&mut self, context: &Context, base: &Expr, args: &mut [Expr], span: Span, options: Options) -> Result<Start, Diagnostic> {
        let (_, discard, _, _, _, _, from_array) = options;
        let set_for_each=discard && self.set_receiver_hint(context,base);
        let map_for_each=discard && self.map_receiver_hint(context,base);
        let collection_for_each=set_for_each||map_for_each;
        let collection_from=from_array && (self.set_receiver_hint(context,base)||self.map_receiver_hint(context,base));
        Self::arity(args.len(), 1, span)?;
        args[0].traversal_method = None;
        let callback_hint = self.callback_value_hint(context, &args[0])?;
        let value_hint = match &args[0].kind {
            ExprKind::Closure { params, .. } => params
                .first()
                .map(|p| match &p.ty {
                    ParameterAnnotation::Named(token) => {
                        self.resolve_value_annotation(token, false)
                    }
                    ParameterAnnotation::Callback(annotation) => self.resolve_callback(annotation),
                    ParameterAnnotation::Inferred(token) => match &p.initializer {
                        ParameterInitializer::Default { value, .. } => self.expression_type_hint(context, value).ok_or_else(|| Diagnostic::new("parameter-inference", "This parameter needs its checked initializer or contextual type.", token.span)),
                        _ => Ok(Type::Void),
                    },
                })
                .transpose()?,
            _ => match callback_hint {
                Some(Type::Closure(id)) => self.closures[id]
                    .parameters
                    .first()
                    .map(|&p| self.closures[id].function.bindings[p].ty),
                Some(Type::Callback(id)) => {
                    self.callables.callbacks[id].params.first().map(|p| p.ty)
                }
                Some(Type::Callable(id)) => self.callables.results[id]
                    .params
                    .first()
                    .map(|parameter| parameter.ty),
                _ => None,
            },
        };
        let value_hint = value_hint.map(|ty| Self::array_callback_hint_type(ty, self.array_callback_rest_position(&args[0], callback_hint) == Some(0)));
        let value_hint = value_hint.map(|ty| self.unions.payload(ty).unwrap_or(ty));
        let absent_elements = matches!(self.expression_type_hint(context, base), Some(Type::Array(element)) if matches!(element.ty(), Type::Null | Type::Undefined)) || matches!(&base.kind, ExprKind::Array(values) if !values.is_empty() && values.iter().all(|value| matches!(value.kind, ExprKind::Null | ExprKind::Undefined)));
        let receiver_hint = value_hint
            .filter(|_| !collection_for_each && !collection_from)
            .filter(|ty| *ty != Type::Void)
            .filter(|_| !absent_elements)
            .filter(|ty| !matches!(ty, Type::Callback(_)))
            .map(|ty| Self::transform_array_type(ty, base.span))
            .transpose()?;
        Ok(Start { options, callback_hint, value_hint, set_for_each, map_for_each, receiver_hint })
    }
    pub(in crate::analysis) fn receive_transform(&mut self, context: &mut Context, base: &Expr, args: &mut [Expr], loans: &Loans, receiver: Type, start: Start) -> Result<Callback, Diagnostic> {
        let Start { options, callback_hint, value_hint, set_for_each, map_for_each, .. } = start;
        let (_, _, _, _, _, callable_elements, from_array) = options;
        let (value,position_type)=match receiver {
            Type::Array(element)=>(element.ty(),Type::Int),
            Type::Set(id) if from_array => (self.program.records[id].set_definition.as_ref().unwrap().key,Type::Int),
            Type::Map(id) if from_array => (Type::Record(self.map_iteration_entry(id,base.span)?),Type::Int),
            Type::Set(id) if set_for_each=>{let key=self.program.records[id].set_definition.as_ref().unwrap().key;(key,key)},
            Type::Map(id) if map_for_each=>{let map=self.program.records[id].map_definition.as_ref().unwrap();(map.value,map.key)},
            _=>return Err(Diagnostic::new("type","Array callback methods require an array.",base.span)),
        };
        let supplied_types=if from_array{vec![value,position_type]}else{vec![value,position_type,receiver]};
        self.contextual_arrow_parameters(&mut args[0], &supplied_types)?;
        let lends_callable = callable_elements
            && matches!(value, Type::StoredCallable(_))
            && matches!(value_hint, Some(Type::Callback(id)) if self.callables.callbacks[id].kind == CallbackKind::Reusable);
        let mut active = loans.clone();
        for root in Self::loan_roots(base, Mode::Read) {
            active.entry(root).or_default().push((
                if lends_callable {
                    Mode::Mutate
                } else {
                    Mode::Read
                },
                base.span,
            ));
        }
        let capture_boundary = crate::capture_loan_identity::Boundary::capture(&context.capture_loans);
        let checked_hint = callback_hint.filter(|ty| {
            !(matches!(args[0].kind, ExprKind::Call(_, _))
                && matches!(ty, Type::Closure(id) if self.closures[*id].polymorphic))
        });
        Ok(Callback { options, receiver, value, position_type, lends_callable, active, supplied_types, checked_hint, boundary: capture_boundary })
    }
    pub(in crate::analysis) fn check_transform_callback(&mut self, context: &Context, args: &mut [Expr], _span: Span, frame: Callback, ty: Type) -> Result<CompletionFrame, Diagnostic> {
        let Callback { options, receiver, value, position_type, lends_callable, boundary, .. } = frame;
        let (_, _, _, _, _, _, from_array) = options;
        let mut method = None;
        let (parameters, result, once) = match ty {
            Type::Closure(id) => {
                let closure = &self.closures[id];
                let once = closure.invocation == Mode::Consume
                    || args[0].binding.is_some_and(|binding| {
                        context.bindings[binding].callback_invocation == Some(Mode::Consume)
                    });
                (
                    closure
                        .parameters
                        .iter()
                        .map(|&p| ArgumentContract::from(&closure.function.bindings[p]))
                        .collect::<Vec<_>>(),
                    closure.function.result,
                    once,
                )
            }
            Type::Callback(id) | Type::StoredCallable(id) | Type::BorrowedCallable(id) => {
                let id = if matches!(ty, Type::StoredCallable(_) | Type::BorrowedCallable(_)) {
                    self.callables.storage.declaration(id).expect("checked traversal storage")
                } else { id };
                let contract = &self.callables.callbacks[id];
                (
                    contract.params.clone(),
                    contract.result,
                    contract.kind.is_once(),
                )
            }
            _ => {
                return Err(Diagnostic::new(
                    "type",
                    "Array callback methods require a typed reusable callback.",
                    args[0].span,
                ));
            }
        };
        if once {
            return Err(Diagnostic::new(
                "callback-access",
                "Array callbacks must be reusable; a consuming callback cannot run for each element.",
                args[0].span,
            ));
        }

        let expected = [
            ArgumentContract { rest: false,
                ty: value,
                mode: if lends_callable {
                    Mode::Mutate
                } else if value.is_copy() {
                    Mode::Copy
                } else {
                    Mode::Read
                },
             omission: crate::ParameterOmission::Required, },
            ArgumentContract { rest: false,
                ty: position_type,
                mode: if position_type.is_copy(){Mode::Copy}else{Mode::Read},
                omission: crate::ParameterOmission::Required,
            },
            ArgumentContract { rest: false,
                ty: receiver,
                mode: Mode::Read,
                omission: crate::ParameterOmission::Required,
            },
        ];
        if lends_callable {
            if parameters.iter().skip(2).any(|parameter| !parameter.rest && !parameter.omission.is_omittable()) {
                return Err(Diagnostic::new(
                    "conflicting-access",
                    "A mutable callable element cannot be borrowed together with its source array.",
                    args[0].span,
                ));
            }
            self.require(value, Some(parameters[0].ty), args[0].span)?;
            if parameters[0].mode != Mode::Mutate {
                return Err(Diagnostic::new(
                    "callback-access",
                    "Callable element invocation requires a mutate callback parameter.",
                    args[0].span,
                ));
            }
            self.require_array_callback_parameters(&parameters[1..], &expected[1..2], args[0].span)?;
            let Type::Closure(origin) = ty else {
                return Err(Diagnostic::new(
                    "unsupported",
                    "Callable element traversal currently requires a concrete callback environment.",
                    args[0].span,
                ));
            };
            method = Some(MethodRequest { origin, types: Self::array_callback_parameter_types(&parameters, &expected[..2].iter().map(|parameter| parameter.ty).collect::<Vec<_>>()) });
        } else {
            let supplied=if from_array{&expected[..2]}else{&expected[..]};
            self.require_array_callback_parameters(&parameters, supplied, args[0].span)?;
            if let Type::Closure(origin) = ty
                && self.closures[origin].polymorphic {
                let types = Self::array_callback_parameter_types(&parameters, &supplied.iter().map(|parameter| parameter.ty).collect::<Vec<_>>());
                method = Some(MethodRequest { origin, types });
            }
        }
        Ok(CompletionFrame { options, receiver, lends_callable, boundary, result, method })
    }
    pub(in crate::analysis) fn finish_transform(&mut self, context: &mut Context, base: &Expr, args: &mut [Expr], loans: &Loans, span: Span, frame: CompletionFrame, specialized: Option<(usize, Type)>) -> Result<Type, Diagnostic> {
        let CompletionFrame { options, receiver, lends_callable, boundary: capture_boundary, mut result, .. } = frame;
        let (filter, discard, _, _, flatten, _, _) = options;
        if let Some((method, specialized_result)) = specialized {
            args[0].traversal_method = Some(TraversalMethod { id: method, access: if lends_callable { Mode::Mutate } else { Mode::Read } });
            result = specialized_result;
            if lends_callable { Self::storage_access(context, base, Mode::Mutate, span, loans)?; }
        }
        if flatten && matches!(result,Type::Record(id) if self.program.records[id].is_tuple()) {
            return Err(Diagnostic::new("unsupported", "flatMap() cannot yet flatten a tuple result into a homogeneous array.", args[0].span));
        }
        let result = if discard {
            // Any otherwise valid callback result (including callable owners) is
            // evaluated and dropped at the end of this invocation.
            Type::Void
        } else if filter {
            self.require(result, Some(Type::Boolean), args[0].span)?;
            receiver
        } else if flatten && matches!(result, Type::Array(_)) {
            result
        } else {
            Self::transform_array_type(result, args[0].span)?
        };
        // Temporary callback captures end after traversal. Named closures retain their existing loans.
        capture_boundary.unwind(&mut context.capture_loans);
        Ok(result)
    }
}

pub(in crate::analysis) enum ResultKind { Direct, Find(bool), Predicate(bool), ArrayFrom, GroupBy }
pub(in crate::analysis) struct Request {
    pub(in crate::analysis) options: Options,
    pub(in crate::analysis) target: CallTarget,
    pub(in crate::analysis) kind: ResultKind,
    pub(in crate::analysis) expected: Option<Type>,
}

impl Analyzer {
    pub(in crate::analysis) fn transform_request(&self, context: &Context, expr: &Expr, expected: Option<Type>, collection: bool) -> Option<Request> {
        let ExprKind::Call(callee, _) = &expr.kind else { return None; };
        let ExprKind::Field(base, name) = &callee.kind else { return None; };
        let (options, target, kind) = match name.as_str() {
            "forEach" => ((false,true,Mode::Read,false,false,!collection,false), CallTarget::ArrayForEach, ResultKind::Direct),
            "map" if !collection => ((false,false,Mode::Read,false,false,true,false), CallTarget::ArrayMap, ResultKind::Direct),
            "filter" if !collection => ((true,false,Mode::Read,false,false,false,false), CallTarget::ArrayFilter, ResultKind::Direct),
            "flatMap" if !collection => ((false,false,Mode::Read,true,true,true,false), CallTarget::ArrayFlatMap, ResultKind::Direct),
            "find" | "findLast" if !collection => {
                let stable = Self::stable_at_receiver(base);
                ((true,false,if stable { Mode::Read } else { Mode::Consume },true,false,false,false),
                    if name == "find" { CallTarget::ArrayFind } else { CallTarget::ArrayFindLast }, ResultKind::Find(stable))
            }
            "some" | "every" | "findIndex" | "findLastIndex" if !collection => {
                let target = match name.as_str() { "some" => CallTarget::ArraySome, "every" => CallTarget::ArrayEvery, "findIndex" => CallTarget::ArrayFindIndex, _ => CallTarget::ArrayFindLastIndex };
                ((true,false,Mode::Read,false,false,true,false), target, ResultKind::Predicate(matches!(name.as_str(), "findIndex" | "findLastIndex")))
            }
            _ => return None,
        };
        let _ = context;
        Some(Request { options, target, kind, expected: if collection { expected } else { None } })
    }

    pub(in crate::analysis) fn finish_transform_request(&mut self, context: &mut Context, expr: &mut Expr, request: Request, result: Type) -> Result<Type, Diagnostic> {
        let result = match request.kind {
            ResultKind::Direct | ResultKind::ArrayFrom => result,
            ResultKind::Predicate(index) => if index { Type::Int } else { Type::Boolean },
            ResultKind::Find(stable) => {
                let Type::Array(element) = result else { unreachable!("predicate array") };
                let result = self.unions.intern([element.ty(), Type::Undefined], expr.span)?;
                if stable && !result.is_copy() { Type::BorrowedUnion(result.union().expect("owned optional")) } else { result }
            }
            ResultKind::GroupBy => self.finish_group_by_transform(expr, result)?,
        };
        self.require(result, request.expected, expr.span)?;
        expr.target = Some(request.target);
        let result = self.finish_builtin_result(context, expr, result)?;
        self.resolve_completed_capture_invocation(context, expr, result);
        Ok(result)
    }
}
