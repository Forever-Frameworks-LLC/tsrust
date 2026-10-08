//! Call-spread contracts select argument access before ordinary builtin checking.
//! The ordinary checker remains authoritative for receiver types, callback shapes,
//! range errors and contextual literal typing. Omitted slots are not undefined:
//! lowering must dispatch using the actual argument count.
use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) struct SpreadBuiltinArgument {
    pub ty: Option<Type>,
    pub mode: Mode,
}
impl SpreadBuiltinArgument {
    fn typed(ty: Type, mode: Mode) -> Self {
        Self { ty: Some(ty), mode }
    }
    fn actual(mode: Mode) -> Self {
        Self { ty: None, mode }
    }
    pub fn contract(self, actual: Type) -> ArgumentContract {
        ArgumentContract {
            ty: self.ty.unwrap_or(actual),
            mode: self.mode,
            omission: crate::ParameterOmission::Required,
            rest: false,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct SpreadBuiltinContract {
    pub arguments: Vec<SpreadBuiltinArgument>,
    pub tail: Option<SpreadBuiltinArgument>,
    pub minimum: usize,
    pub maximum: Option<usize>,
    pub receiver_access: Mode,
    pub kind: SpreadBuiltinKind,
}
impl SpreadBuiltinContract {
    pub fn argument(&self, position: usize) -> Option<SpreadBuiltinArgument> {
        self.arguments.get(position).copied().or(self.tail)
    }
    fn fixed(minimum: usize, arguments: Vec<SpreadBuiltinArgument>, receiver_access: Mode) -> Self {
        Self {
            minimum,
            maximum: Some(arguments.len()),
            arguments,
            tail: None,
            receiver_access,
            kind: SpreadBuiltinKind::Fixed,
        }
    }
    fn variadic(
        arguments: Vec<SpreadBuiltinArgument>,
        tail: SpreadBuiltinArgument,
        receiver_access: Mode,
        kind: SpreadBuiltinKind,
    ) -> Self {
        Self {
            minimum: 0,
            maximum: None,
            arguments,
            tail: Some(tail),
            receiver_access,
            kind,
        }
    }
}

impl Analyzer {
    pub(super) fn builtin_spread_contract(
        &mut self,
        context: &Context,
        callee: &Expr,
        segments: &[SpreadSegment],
        _span: Span,
    ) -> Result<Option<SpreadBuiltinContract>, Diagnostic> {
        use SpreadBuiltinArgument as Arg;
        use SpreadBuiltinContract as Contract;
        use SpreadBuiltinKind as Kind;
        let integer = Arg::typed(Type::Int, Mode::Copy);
        let number = Arg::typed(Type::Number, Mode::Copy);
        let string = Arg::typed(Type::String, Mode::Read);
        let observed = Arg::actual(Mode::Read);
        let owned = Arg::actual(Mode::Consume);
        let callback = Arg::actual(Mode::Mutate);
        if let ExprKind::Name(name) = &callee.kind {
            // A local shadow is a callable, never a builtin namespace.
            if let Some(&local)=context.names.get(name) {
                if let Type::Closure(origin)=context.bindings[local].ty
                    && let Some(ClosureForward::Native(id))=self.closures[origin].forward
                    && crate::native_bindings::BINDINGS[id].parameters.last().is_some_and(|parameter|parameter.rest) {
                    return Ok(Some(Contract::variadic(vec![], string, Mode::Read, Kind::NativeStringRest)));
                }
                return Ok(None);
            }
            let contract = match name.as_str() {
                "String" | "Boolean" => {
                    Contract::variadic(vec![], observed, Mode::Read, Kind::ObserveVariadic)
                }
                "Number" => Contract::fixed(0, vec![observed], Mode::Read),
                "clone" => Contract::fixed(1, vec![observed], Mode::Read),
                _ => {
                    if let Some((_, binding)) = crate::native_bindings::by_symbol(name) {
                        if binding.parameters.len()==1 && binding.parameters[0].rest && binding.parameters[0].ty==crate::native_bindings::NativeType::String {
                            return Ok(Some(Contract::variadic(vec![], string, Mode::Read, Kind::NativeStringRest)));
                        }
                        Contract::fixed(
                            binding
                                .parameters
                                .iter()
                                .take_while(|p| !p.optional)
                                .count(),
                            binding
                                .parameters
                                .iter()
                                .map(|p| Arg::typed(native_type(p.ty), p.access))
                                .collect(),
                            Mode::Read,
                        )
                    } else {
                        return Ok(None);
                    }
                }
            };
            return Ok(Some(contract));
        }
        let ExprKind::Field(base, name) = &callee.kind else {
            return Ok(None);
        };
        if let ExprKind::Name(namespace) = &base.kind {
            if !context.names.contains_key(namespace) {
                if namespace == "Math" && math_target(name).is_some() {
                    return Ok(Some(match name.as_str() {
                        "min" | "max" | "hypot" => {
                            Contract::variadic(vec![], number, Mode::Copy, Kind::MathVariadic)
                        }
                        "imul" | "atan2" | "pow" => Contract::fixed(2, vec![number; 2], Mode::Copy),
                        "random" => Contract::fixed(0, vec![], Mode::Copy),
                        _ => Contract::fixed(1, vec![number], Mode::Copy),
                    }));
                }
                if namespace == "Number" {
                    if name == "parseFloat" {
                        return Ok(Some(Contract::fixed(1, vec![string], Mode::Read)));
                    }
                    if number_predicate_target(name).is_some() {
                        return Ok(Some(Contract::fixed(1, vec![number], Mode::Copy)));
                    }
                }
                if namespace == "Array" && name == "isArray" {
                    return Ok(Some(Contract::variadic(
                        vec![],
                        observed,
                        Mode::Read,
                        Kind::ObserveVariadic,
                    )));
                }
                if namespace == "console" {
                    let contract = match name.as_str() {
                        "table" => Contract::fixed(0, vec![observed, observed], Mode::Read),
                        "log" | "info" | "debug" | "warn" | "error" | "assert" | "dir"
                        | "dirxml" | "group" | "groupCollapsed" | "groupEnd" | "count"
                        | "countReset" | "time" | "timeLog" | "timeEnd" | "clear" | "trace"
                        | "profile" | "profileEnd" | "timeStamp" => {
                            Contract::variadic(vec![], observed, Mode::Read, Kind::ConsoleVariadic)
                        }
                        _ => return Ok(None),
                    };
                    return Ok(Some(contract));
                }
            }
        }
        // Callable fields retain their own declared contracts.
        if self
            .expression_type_hint(context, callee)
            .is_some_and(|ty| {
                matches!(
                    ty,
                    Type::StoredCallable(_)
                        | Type::Closure(_)
                        | Type::Callback(_)
                        | Type::Callable(_)
                )
            })
        {
            return Ok(None);
        }
        let receiver = self.expression_type_hint(context, base);
        let element = receiver.and_then(|ty| {
            if let Type::Array(element) = ty {
                Some(element.ty())
            } else {
                None
            }
        });
        let value = element
            .map(|ty| Arg::typed(ty, Mode::Consume))
            .unwrap_or(owned);
        let search = element
            .map(|ty| Arg::typed(ty, Mode::Read))
            .unwrap_or(string);
        let mut contract = match name.as_str() {
            "trim" | "trimStart" | "trimEnd" | "trimLeft" | "trimRight" | "toLowerCase"
            | "toUpperCase" | "isWellFormed" | "toWellFormed" | "valueOf" | "toReversed" | "keys" => {
                Contract::fixed(0, vec![], Mode::Read)
            }
            "toString" if receiver == Some(Type::Int) => {
                Contract::fixed(0, vec![integer], Mode::Copy)
            }
            "toString" => Contract::fixed(0, vec![], Mode::Read),
            "toFixed" | "toExponential" | "toPrecision" => {
                Contract::fixed(0, vec![integer], Mode::Copy)
            }
            "slice" | "substr" => Contract::fixed(0, vec![integer; 2], Mode::Read),
            "substring" => Contract::fixed(1, vec![integer; 2], Mode::Read),
            "charAt" | "charCodeAt" => Contract::fixed(0, vec![integer], Mode::Read),
            name if name == crate::analysis::STRING_CODE_POINT_SLOT => Contract::fixed(0, vec![integer], Mode::Read),
            "repeat" => Contract::fixed(1, vec![integer], Mode::Read),
            "replace" | "replaceAll" => Contract::fixed(2, vec![string; 2], Mode::Read),
            "split" => Contract::fixed(0, vec![string, integer], Mode::Read),
            "padStart" | "padEnd" => Contract::fixed(1, vec![integer, string], Mode::Read),
            "startsWith" | "endsWith" | "includes" | "indexOf" | "lastIndexOf" => {
                Contract::fixed(1, vec![search, integer], Mode::Read)
            }
            "join" => Contract::fixed(0, vec![string], Mode::Read),
            "push" => Contract::fixed(1, vec![value], Mode::Mutate),
            "remove" => Contract::fixed(1, vec![integer], Mode::Mutate),
            "at" => Contract::fixed(1, vec![integer], Mode::Read),
            "pop" | "shift" | "reverse" => Contract::fixed(0, vec![], Mode::Mutate),
            "fill" => Contract::fixed(1, vec![value, integer, integer], Mode::Mutate),
            "copyWithin" => Contract::fixed(2, vec![integer; 3], Mode::Mutate),
            "with" => Contract::fixed(2, vec![integer, value], Mode::Read),
            "unshift" => Contract::variadic(vec![], value, Mode::Mutate, Kind::ArrayUnshift),
            "splice" => {
                Contract::variadic(vec![integer; 2], value, Mode::Mutate, Kind::ArraySplice)
            }
            "toSpliced" => {
                Contract::variadic(vec![integer; 2], value, Mode::Read, Kind::ArrayToSpliced)
            }
            "concat" if receiver == Some(Type::String) => {
                Contract::variadic(vec![], string, Mode::Read, Kind::StringConcat)
            }
            "concat" => Contract::variadic(vec![], observed, Mode::Read, Kind::ArrayConcat),
            "sort" => Contract::fixed(0, vec![callback], Mode::Mutate),
            "toSorted" => Contract::fixed(0, vec![callback], Mode::Read),
            "forEach" | "map" | "flatMap" | "filter" | "some" | "every" | "find" | "findLast"
            | "findIndex" | "findLastIndex" => Contract::fixed(1, vec![callback], Mode::Read),
            "reduce" | "reduceRight" => Contract::fixed(1, vec![callback, owned], Mode::Read),
            "flat" => Contract::fixed(0, vec![integer], Mode::Read),
            _ => {
                if let Some(CallTarget::StringHtml(wrapper)) = string_html_target(name) {
                    let count = usize::from(wrapper.markup().1.is_some());
                    Contract::fixed(count, vec![string; count], Mode::Read)
                } else {
                    return Ok(None);
                }
            }
        };
        if let Some(receiver) = receiver
            && self.spread_builtin_traversal_access(context, receiver, name, segments)?
                == Mode::Mutate
        {
            contract.receiver_access = Mode::Mutate;
        }
        if matches!(contract.kind, Kind::ArrayUnshift | Kind::ArraySplice)
            && segments
                .iter()
                .all(|segment| segment.spread && segment.length == Some(0))
        {
            contract.receiver_access = Mode::Read;
        }
        Ok(Some(contract))
    }

    /// Match ordinary protected traversal before lending the staged receiver.
    /// Read-only callable markers do not permit invoking element environments.
    pub(super) fn spread_builtin_traversal_access(
        &mut self,
        context: &Context,
        receiver: Type,
        method: &str,
        segments: &[SpreadSegment],
    ) -> Result<Mode, Diagnostic> {
        if !matches!(method, "forEach" | "map" | "flatMap")
            || !matches!(receiver, Type::Array(element) if matches!(element.ty(), Type::StoredCallable(_)))
        {
            return Ok(Mode::Read);
        }
        for segment in segments {
            if segment.length == Some(0) {
                continue;
            }
            if !segment.spread {
                // A spread source must first construct its owning array. Its
                // element syntax cannot be reinterpreted as a direct closure.
                let callback = segment.value.as_ref();
                let callback_hint = self.callback_value_hint(context, callback)?;
                let value_hint = match &callback.kind {
                    ExprKind::Closure { params, .. } => params.first().map(|parameter| {
                        match &parameter.ty {
                            ParameterAnnotation::Named(annotation) =>
                                self.resolve_value_annotation(annotation, false),
                            ParameterAnnotation::Callback(annotation) =>
                                self.resolve_callback(annotation),
                            ParameterAnnotation::Inferred(token) => match &parameter.initializer {
                                ParameterInitializer::Default { value, .. } =>
                                    self.expression_type_hint(context, value).ok_or_else(|| {
                                        Diagnostic::new("parameter-inference",
                                            "This parameter needs its checked initializer or contextual type.", token.span)
                                    }),
                                _ => Err(Diagnostic::new("parameter-inference",
                                    "This parameter needs its checked initializer or contextual type.", token.span)),
                            },
                        }
                    }).transpose()?,
                    _ => match callback_hint {
                        Some(Type::Closure(id)) => self.closures[id].parameters.first()
                            .map(|&parameter| self.closures[id].function.bindings[parameter].ty),
                        Some(Type::Callback(id)) => self.callables.callbacks[id].params.first()
                            .map(|parameter| parameter.ty),
                        Some(Type::Callable(id)) => self.callables.results[id].params.first()
                            .map(|parameter| parameter.ty),
                        _ => None,
                    },
                };
                let value_hint = value_hint.map(|ty| {
                    Self::array_callback_hint_type(
                        ty,
                        self.array_callback_rest_position(callback, callback_hint) == Some(0),
                    )
                });
                let value_hint = value_hint.map(|ty| self.unions.payload(ty).unwrap_or(ty));
                if matches!(value_hint, Some(Type::Callback(id))
                    if self.callables.callbacks[id].kind == CallbackKind::Reusable)
                {
                    return Ok(Mode::Mutate);
                }
            }
            if segment.length.is_some_and(|length| length > 0) {
                break;
            }
        }
        Ok(Mode::Read)
    }
}
