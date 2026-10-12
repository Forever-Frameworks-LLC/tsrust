//! Editor metadata derived only from successfully checked bodies.
use crate::Analysis;
use crate::analysis::CheckedProgram;
use crate::diagnostic::Span;
use crate::syntax::{
    BinaryOp, CallTarget, CallbackKind, ControlTarget, Expr, ExprKind, Stmt, StmtKind, Type, UnaryOp,
};
use std::collections::{BTreeMap, HashMap};

/// Checked lexical declaration identity in original UTF-8 source coordinates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalFunctionInfo {
    pub id: usize,
    pub scope: usize,
    pub name: String,
    pub declaration_span: Span,
    pub function_scope: Span,
    pub result: Option<String>,
}

/// One checked use of a nullable place with a more precise value type.
///
/// Spans use source UTF-8 byte offsets. The declared binding and its physical
/// tagged storage do not change. The same span may have different views in
/// different callable specializations; clients must not select one arbitrarily.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectionInfo {
    pub span: Span,
    /// Canonical Tsrust type spelling, including any remaining absent variant.
    pub ty: String,
}

/// A checked built-in, callable invocation or nullish expression result type.
/// Spans use source UTF-8 byte offsets.
/// Different callable specializations may have different types at one span.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MethodResultInfo {
    pub span: Span,
    pub ty: String,
}

/// One checked generic call; distinct specializations at a shared span remain explicit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenericCallInfo {
    pub(crate) parameter_types: Vec<Type>,
    pub(crate) body_parameter_types: Vec<Type>,
    pub(crate) result_type: Type,
    pub span: Span,
    pub callee_span: Span,
    pub declaration_span: Span,
    pub parameters: Vec<GenericParameterInfo>,
    pub result: String,
}

/// Checked parameters of a generic call whose result display remains unresolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenericParameterCallInfo {
    pub(crate) parameter_types: Vec<Type>,
    pub(crate) body_parameter_types: Vec<Type>,
    pub span: Span,
    pub callee_span: Span,
    pub declaration_span: Span,
    pub parameters: Vec<GenericParameterInfo>,
}

/// A concrete parameter contract. Unsupported display types remain unknown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenericParameterInfo {
    pub name: String,
    pub ty: Option<String>,
    pub body_ty: Option<String>,
    pub omission: crate::ParameterOmission,
    pub rest: bool,
    pub access: crate::Mode,
}

/// Checked editor facts collected in one iterative walk, without reanalysis.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CheckedTypeFacts {
    pub json_string_receivers: Vec<Span>,
    /// Generic value bindings retain their original declaration identity.
    pub generic_values: Vec<(Span, Span)>,
    pub(crate) identities: CheckedTypeIdentities,
    /// Initialized body types at exact checked parameter declaration spans.
    pub parameter_types: Vec<ProjectionInfo>,
    pub projections: Vec<ProjectionInfo>,
    /// Payload types derived from checked successful hops and readonly typeof probes.
    /// These do not authorize ordinary access or establish flow narrowing.
    pub optional_receivers: Vec<ProjectionInfo>,
    pub method_results: Vec<MethodResultInfo>,
    /// Checked fields of synthesized rest records: owner, name, source span, type.
    pub rest_record_fields: Vec<(String, String, Span, String)>,
    pub generic_calls: Vec<GenericCallInfo>,
    pub generic_parameter_calls: Vec<GenericParameterCallInfo>,
}

// Preserve checked identities independently of display spelling. Project
// consumers remap nominal declarations through the module graph; unknown
// alternatives stay explicit so a sibling specialization cannot win silently.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct CheckedTypeIdentities {
    pub parameter_types: BTreeMap<(usize, usize), Vec<Option<Type>>>,
    pub projections: BTreeMap<(usize, usize), Vec<Option<Type>>>,
    pub optional_receivers: BTreeMap<(usize, usize), Vec<Option<Type>>>,
    pub method_results: BTreeMap<(usize, usize), Vec<Option<Type>>>,
}

#[derive(Clone, Copy)]
enum Node<'a> {
    Statement(&'a Stmt),
    Expression(&'a Expr),
    Successful(&'a Expr, Span),
}

impl Analysis {
    /// Collect nullable projection metadata on demand from checked bodies.
    ///
    /// This does not run analysis again or retain editor strings in the syntax
    /// tree. Results are ordered by source span and identical views are deduped.
    /// Multiple distinct types at a shared source span remain explicit.
    pub fn checked_local_functions(&self) -> Vec<LocalFunctionInfo> {
        if self.program.local_declarations.is_empty() { return Vec::new(); }
        let mut names = HashMap::new();
        let functions = self.program.functions.iter().filter(|function| !function.template)
            .map(|function| (function.name.as_str(), function)).collect::<HashMap<_, _>>();
        self.program.local_declarations.iter().map(|declaration| {
            let result = functions.get(declaration.emitted_name.as_str())
                .and_then(|function| type_name(&self.program, function.result, &mut names));
            LocalFunctionInfo { id: declaration.id, scope: declaration.scope,
                name: declaration.source_name.text.clone(), declaration_span: declaration.source_name.span,
                function_scope: declaration.function_scope, result }
        }).collect()
    }

    /// Checked result types of top-level functions, by source name. Clients
    /// use them where the declaration leaves its result type to inference.
    pub fn checked_function_results(&self) -> Vec<(String, String)> {
        let mut names = HashMap::new();
        self.program.functions.iter()
            .filter(|function| !function.template && !function.specialization && !function.name.contains(['#', '<']))
            .filter_map(|function| Some((function.name.clone(), type_name(&self.program, function.result, &mut names)?)))
            .collect()
    }

    pub fn projections(&self) -> Vec<ProjectionInfo> {
        self.checked_type_facts().projections
    }

    /// Checked label declarations and references, paired with their declaration.
    /// Label names occupy a separate namespace from ordinary values.
    pub fn checked_control_labels(&self) -> Vec<(Span,Span)> {
        let mut definitions=BTreeMap::new();let mut references=Vec::new();let mut pending=Vec::new();
        for function in self.program.functions.iter().chain(self.program.closures.iter().flat_map(|closure|std::iter::once(&closure.function).chain(closure.methods.iter().map(|method|&method.function)))).filter(|function|!function.template) {
            pending.extend(function.body.iter());
        }
        while let Some(statement)=pending.pop(){
            if matches!(statement.kind,StmtKind::Break|StmtKind::Continue){
                if let (Some(label),Some(target))=(statement.labels.first(),statement.control_target){
                    let start=match target {ControlTarget::Loop(start)|ControlTarget::Tail(start)|ControlTarget::Block(start)=>start};
                    references.push((start,label.text.clone(),label.span));
                }
            }else{for label in &statement.labels {definitions.insert((statement.span.start,label.text.clone()),label.span);}}
            match &statement.kind {
                StmtKind::Try(region)=>pending.extend(region.statements()),
                StmtKind::Block(body)|StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>pending.extend(body.iter()),
                StmtKind::Loop{body,update,..}=>{pending.extend(body.iter());pending.extend(update.iter());}
                StmtKind::If{then_body,else_body,..}=>{pending.extend(then_body.iter());pending.extend(else_body.iter());}
                _=>{},
            }
        }
        let mut result=std::collections::BTreeSet::new();
        for span in definitions.values(){result.insert((span.start,span.end,span.start,span.end));}
        for (start,name,span) in references {if let Some(definition)=definitions.get(&(start,name)){result.insert((span.start,span.end,definition.start,definition.end));}}
        result.into_iter().map(|(start,end,definition_start,definition_end)|(Span::new(start,end),Span::new(definition_start,definition_end))).collect()
    }

    /// Collect checked nullable views and expression result types together.
    /// Conflicting specializations remain explicit; callers must not pick one.
    pub fn checked_type_facts(&self) -> CheckedTypeFacts {
        let program = &self.program;
        let mut json_string_receivers=Vec::new();
        let wrapper_spans = program.closures.iter().filter_map(|closure| {
            let [statement] = closure.function.body.as_slice() else { return None; };
            let StmtKind::Return(Some(call)) = &statement.kind else { return None; };
            let call = if let ExprKind::SpreadCall(plan) = &call.kind { plan.call.as_ref() } else { call };
            let ExprKind::Call(callee, _) = &call.kind else { return None; };
            (call.span == closure.span && callee.span == closure.span)
                .then_some((closure.span.start, closure.span.end))
        }).collect::<std::collections::BTreeSet<_>>();
        let mut pending = Vec::new();
        let mut parameter_types = BTreeMap::<(usize, usize), Vec<Option<Type>>>::new();
        let functions = program.functions.iter().chain(program.closures.iter().flat_map(|closure| {
            std::iter::once(&closure.function).chain(closure.methods.iter().map(|method| &method.function))
        }));
        for function in functions.filter(|function| !function.template) {
            statements(&mut pending, &function.body);
            for binding in function.bindings.iter().filter(|binding| binding.parameter) {
                if let Some(span) = binding.declaration_span {
                    let types = parameter_types.entry((span.start, span.end)).or_default();
                    if !types.contains(&Some(binding.ty)) { types.push(Some(binding.ty)); }
                }
                if let Some(initializer) = &binding.initializer {
                    pending.push(Node::Expression(initializer));
                }
            }
        }
        let mut generic_values = Vec::new();
        for function in program.functions.iter().chain(program.closures.iter().flat_map(|closure| std::iter::once(&closure.function).chain(closure.methods.iter().map(|method| &method.function)))) {
            for binding in &function.bindings {
                if let Type::Closure(origin) = binding.ty
                    && let Some(declaration) = program.deferred_generic_sources.get(&origin)
                    && let Some(span) = binding.declaration_span {
                    let fact = (span, declaration.span);
                    if !generic_values.contains(&fact) { generic_values.push(fact); }
                }
            }
        }
        let mut views = BTreeMap::<(usize, usize), Vec<Type>>::new();
        let mut methods = BTreeMap::<(usize, usize), Vec<Type>>::new();
        // Types of destructured names. The synthesized position expression of a
        // name shares its span; its index and length results must not be
        // reported as a second type of the name.
        let mut pattern_names = BTreeMap::<(usize, usize), Vec<Type>>::new();
        let mut exact_views = std::collections::BTreeSet::new();
        let mut optional = BTreeMap::<(usize, usize), Vec<Option<Type>>>::new();
        let mut generic_parameter_calls = Vec::new();
        let mut generic_calls = BTreeMap::<(usize, usize), Vec<GenericCallInfo>>::new();
        let mut generic_names = HashMap::new();
        while let Some(node) = pending.pop() {
            match node {
                Node::Statement(statement) => match &statement.kind {
                    StmtKind::LocalFunction(_) => unreachable!("local declarations must be resolved before checking"),
                    StmtKind::Try(region) => {
                        if let Some(handler)=&region.catch {
                            if handler.binding.is_some() {
                                if let (Some(name),Some(ty))=(&handler.name,handler.error_type) {
                                    let types=methods.entry((name.span.start,name.span.end)).or_default();
                                    if !types.contains(&ty) {types.push(ty);}
                                }
                            }
                        }
                        pending.extend(region.statements().map(Node::Statement));
                    }
                    StmtKind::Throw(value) => pending.push(Node::Expression(value)),
                    StmtKind::Block(body) => statements(&mut pending, body),
                    StmtKind::Loop {
                        condition,
                        body,
                        update,
                        ..
                    } => {
                        statements(&mut pending, update);
                        statements(&mut pending, body);
                        if let Some(condition) = condition {
                            pending.push(Node::Expression(condition));
                        }
                    }
                    StmtKind::ForOf { iterable, body, .. } => {
                        statements(&mut pending, body);
                        pending.push(Node::Expression(iterable));
                    }
                    StmtKind::While { condition, body } => {
                        statements(&mut pending, body);
                        pending.push(Node::Expression(condition));
                    }
                    StmtKind::If {
                        condition,
                        then_body,
                        else_body,
                    } => {
                        statements(&mut pending, else_body);
                        statements(&mut pending, then_body);
                        pending.push(Node::Expression(condition));
                    }
                    StmtKind::Let { name, value, .. } if value.span == name.span => {
                        // A destructured name: report the type of its position.
                        if let Some(ty) = value.ty {
                            let types = pattern_names.entry((name.span.start, name.span.end)).or_default();
                            if !types.contains(&ty) { types.push(ty); }
                        }
                        pending.push(Node::Expression(value));
                    }
                    StmtKind::Let { name, value, .. } => {
                        if let Some(Type::Closure(origin)) = value.ty
                            && let Some(declaration) = program.deferred_generic_sources.get(&origin) {
                            let fact = (name.span, declaration.span);
                            if !generic_values.contains(&fact) { generic_values.push(fact); }
                        }
                        pending.push(Node::Expression(value));
                    }
                    StmtKind::Expr(value)
                    | StmtKind::Return(Some(value)) => pending.push(Node::Expression(value)),
                    StmtKind::Assign { target, value, .. } => {
                        pending.push(Node::Expression(value));
                        pending.push(Node::Expression(target));
                    }
                    StmtKind::Return(None) | StmtKind::Break | StmtKind::Continue => {}
                },
                Node::Expression(expression) | Node::Successful(expression, _) => {
                    if expression.json_string_view {json_string_receivers.push(expression.span);}
                    let successful_span = match node {
                        Node::Successful(_, span) => Some(span),
                        _ => None,
                    };
                    let successful = successful_span.is_some();
                    if successful {
                        let receiver = match &expression.kind {
                            ExprKind::Field(receiver, _) | ExprKind::Index(receiver, _) => {
                                Some(receiver)
                            }
                            _ => None,
                        };
                        if let Some(receiver) = receiver
                            && let Some(ty) = receiver.ty
                        {
                            exact_views.insert((receiver.span.start, receiver.span.end));
                            let types = views
                                .entry((receiver.span.start, receiver.span.end))
                                .or_default();
                            if !types.contains(&ty) {
                                types.push(ty);
                            }
                            let payload = optional_payload(program, ty);
                            let types = optional
                                .entry((receiver.span.start, receiver.span.end))
                                .or_default();
                            if !types.contains(&payload) {
                                types.push(payload);
                            }
                        }
                    }
                    if let ExprKind::Unary {
                        op: UnaryOp::Typeof,
                        value,
                        ..
                    } = &expression.kind
                        && let Some(ty) = value.ty
                    {
                        let payload = optional_payload(program, ty);
                        let types = optional
                            .entry((value.span.start, value.span.end))
                            .or_default();
                        if !types.contains(&payload) {
                            types.push(payload);
                        }
                    }
                    if (matches!(expression.kind, ExprKind::SetConstructor(_, _) | ExprKind::MapConstructor(_, _) | ExprKind::Integer(_) | ExprKind::Number(_))
                        || matches!(&expression.kind, ExprKind::Field(base, _) if matches!(base.ty, Some(Type::Set(_) | Type::Map(_))))
                        || matches!(&expression.kind, ExprKind::Field(base, name) if name=="length" && matches!(base.ty,Some(Type::String | Type::Array(_))))
                        || matches!(&expression.kind, ExprKind::Field(base, _) if matches!(base.ty,Some(Type::Record(id)) if program.records[id].is_tuple()))
                        || (matches!(&expression.kind,ExprKind::Field(base,_) if matches!(base.ty,Some(Type::Record(id)) if self.program.records[id].name.text=="#RuntimeError")))
                        || (matches!(expression.kind, ExprKind::Name(_) | ExprKind::Field(_, _) | ExprKind::Closure { .. } | ExprKind::Conditional { .. })
                        && expression.ty.is_some_and(|ty| matches!(ty, Type::Closure(_) | Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_) | Type::Callable(_)))
                        || matches!(
                        expression.target,
                        Some(
                            CallTarget::SetMethod(_) | CallTarget::MapMethod(_)
                                | CallTarget::ArrayEntries(_)
                                | CallTarget::Number
                                | CallTarget::JsonStringView
                                | CallTarget::RecordSpread(_)
                                | CallTarget::ObjectHasOwn
                                | CallTarget::ObjectKeys
                                | CallTarget::ObjectValues
                                | CallTarget::ObjectEntries(_)
                                | CallTarget::ArrayFlat(_)
                                | CallTarget::ArrayMap
                                | CallTarget::MapGroupBy
                                | CallTarget::ArrayFlatMap
                                | CallTarget::ArrayReduce
                                | CallTarget::ArrayReduceRight
                                | CallTarget::ArrayOf
                                | CallTarget::Closure(_)
                                | CallTarget::ClosureMethod(_)
                                | CallTarget::Callback(_)
                                | CallTarget::ConditionalClosure(_)
                                | CallTarget::ConditionalCallback(_)
                                | CallTarget::StoredCallable(_)
                                | CallTarget::StoredCallableField(_)
                                | CallTarget::Function(_)
                                | CallTarget::ForwardFunction(_)
                        )
                    ) || matches!(
                        expression.kind,
                        ExprKind::Binary {
                            op: BinaryOp::Coalesce | BinaryOp::PatternDefault | BinaryOp::And | BinaryOp::Or | BinaryOp::Add,
                            ..
                        } | ExprKind::Unary {
                            op: UnaryOp::CoalesceSelected | UnaryOp::NonNull,
                            ..
                        } | ExprKind::Sequence(_) | ExprKind::TypeAssertion { .. } | ExprKind::CoalesceAssign { .. }
                            | ExprKind::Optional { .. }
                            | ExprKind::Unary {
                                op: UnaryOp::OptionalSelected,
                                ..
                            }
                    ))) && successful_span != Some(expression.span)
                        // A generated named-value wrapper shares the source
                        // identifier span with its hidden call. That call result
                        // is not the type of the source function value.
                        && !(wrapper_spans.contains(&(expression.span.start, expression.span.end))
                            && (matches!(&expression.kind, ExprKind::Call(callee, _) if callee.span == expression.span)
                                || matches!(expression.kind, ExprKind::SpreadCall(_))))
                        && let Some(ty) = expression.ty
                    {
                        let types = methods
                            .entry((expression.span.start, expression.span.end))
                            .or_default();
                        if !types.contains(&ty) {
                            types.push(ty);
                        }
                    }
                    if let Some(fact) = deferred_call_fact(program, expression, &mut generic_names) {
                        let variants = generic_calls.entry((expression.span.start, expression.span.end)).or_default();
                        if !variants.contains(&fact) { variants.push(fact); }
                    }
                    if let Some(CallTarget::Function(id) | CallTarget::ForwardFunction(id)) =
                        expression.target
                        && let Some(declaration) = program.generic_declarations.get(&id)
                        && let ExprKind::Call(callee, _) = &expression.kind
                    {
                        let function = &program.functions[id];
                        let parameter_types = program
                            .generic_parameter_types
                            .get(&id)
                            .cloned()
                            .unwrap_or_else(|| {
                                function
                                    .bindings
                                    .iter()
                                    .take(function.parameter_count)
                                    .map(|binding| binding.input_ty.unwrap_or(binding.ty))
                                    .collect()
                            });
                        let partial = GenericParameterCallInfo {
                            parameter_types: parameter_types.clone(),
                            body_parameter_types: function.bindings.iter().take(function.parameter_count).map(|binding| binding.ty).collect(),
                            span: expression.span,
                            callee_span: callee.span,
                            declaration_span: declaration.span,
                            parameters: function
                                .bindings
                                .iter()
                                .take(function.parameter_count)
                                .zip(&parameter_types)
                                .map(|(binding, ty)| GenericParameterInfo {
                                    name: binding.name.clone(),
                                    ty: type_name(program, *ty, &mut generic_names),
                                    body_ty: type_name(program, binding.ty, &mut generic_names),
                                    omission: binding.omission,
                                    rest: binding.rest,
                                    access: binding.mode,
                                })
                                .collect(),
                        };
                        if let Some(result) = type_name(program, function.result, &mut generic_names) {
                            let fact = GenericCallInfo {
                                parameter_types: partial.parameter_types,
                                body_parameter_types: partial.body_parameter_types,
                                result_type: function.result,
                                span: partial.span, callee_span: partial.callee_span,
                                declaration_span: partial.declaration_span,
                                parameters: partial.parameters, result,
                            };
                            let variants = generic_calls
                                .entry((expression.span.start, expression.span.end))
                                .or_default();
                            if !variants.contains(&fact) {
                                variants.push(fact);
                            }
                        } else if !generic_parameter_calls.contains(&partial) {
                            generic_parameter_calls.push(partial);
                        }
                    }
                    match &expression.kind {
                        ExprKind::SpreadCall(plan) => pending.extend(plan.expressions().map(Node::Expression)),
                        ExprKind::SpreadValue(_) => (),
                        ExprKind::Selected { .. } => {
                            if let Some(ty) = expression.ty {
                                exact_views.insert((expression.span.start, expression.span.end));
                                let types = views
                                    .entry((expression.span.start, expression.span.end))
                                    .or_default();
                                if !types.contains(&ty) {
                                    types.push(ty);
                                }
                                let payload = optional_payload(program, ty);
                                let types = optional
                                    .entry((expression.span.start, expression.span.end))
                                    .or_default();
                                if !types.contains(&payload) {
                                    types.push(payload);
                                }
                            }
                        }
                        ExprKind::Optional { base, body, .. } => {
                            pending.push(Node::Successful(body, expression.span));
                            pending.push(child(base, successful_span));
                        }
                        ExprKind::TypeAssertion {value,..} | ExprKind::OptionalChain(value) => {
                            pending.push(child(value, successful_span))
                        }
                        ExprKind::Unary {
                            op: UnaryOp::OptionalSelected,
                            value,
                            ..
                        } => {
                            pending.push(Node::Successful(value, expression.span));
                        }
                        ExprKind::Project {
                            value, narrowed, ..
                        } => {
                            let types = views
                                .entry((expression.span.start, expression.span.end))
                                .or_default();
                            if !types.contains(narrowed) {
                                types.push(*narrowed);
                            }
                            pending.push(child(value, successful_span));
                        }
                        ExprKind::Conditional {
                            predicate,
                            consequent,
                            alternate,
                        } => {
                            pending.push(child(alternate, successful_span));
                            pending.push(child(consequent, successful_span));
                            pending.push(child(predicate, successful_span));
                        }
                        ExprKind::CompoundAssign { target, value, .. }
                        | ExprKind::Assign { target, value, .. }
                        | ExprKind::LogicalAssign { target, value, .. }
                        | ExprKind::CoalesceAssign { target, value, .. } => {
                            pending.push(child(value, successful_span));
                            pending.push(child(target, successful_span));
                        }
                        ExprKind::Update { target, .. } => pending.push(Node::Expression(target)),
                        ExprKind::Unary { value, .. }
                        | ExprKind::Inject { value, .. }
                        | ExprKind::Field(value, _) => pending.push(child(value, successful_span)),
                        ExprKind::Binary { left, right, .. } | ExprKind::Index(left, right) => {
                            pending.push(child(right, successful_span));
                            pending.push(child(left, successful_span));
                        }
                        ExprKind::Sequence(values) | ExprKind::Template(values)
                        | ExprKind::Array(values)
                        | ExprKind::ArrayOf(values, _) | ExprKind::SetConstructor(values, _) | ExprKind::MapConstructor(values, _) | ExprKind::NativeConstructor(values, _) => {
                            pending.extend(
                                values
                                    .iter()
                                    .rev()
                                    .map(|value| child(value, successful_span)),
                            );
                        }
                        ExprKind::Record(fields) => {
                            pending.extend(
                                fields
                                    .iter()
                                    .rev()
                                    .map(|(_, value)| child(value, successful_span)),
                            );
                        }
                        ExprKind::Call(callee, arguments) => {
                            pending.extend(
                                arguments
                                    .iter()
                                    .rev()
                                    .map(|value| child(value, successful_span)),
                            );
                            pending.push(child(callee, successful_span));
                        }
                        // Source closure bodies are not authoritative checked
                        // bodies. Their checked functions/methods are above.
                        ExprKind::Closure { .. }
                        | ExprKind::Name(_)
                        | ExprKind::String(_)
                        | ExprKind::Integer(_)
                        | ExprKind::EnumMember { .. } | ExprKind::Number(_)
                        | ExprKind::Boolean(_)
                        | ExprKind::Null
                        | ExprKind::Undefined => {}
                    }
                }
            }
        }
        methods.extend(pattern_names);
        let identities = CheckedTypeIdentities {
            parameter_types: parameter_types.clone(),
            projections: views
                .iter()
                .map(|(&span, types)| (span, types.iter().copied().map(Some).collect()))
                .collect(),
            optional_receivers: optional.clone(),
            method_results: methods
                .iter()
                .map(|(&span, types)| (span, types.iter().copied().map(Some).collect()))
                .collect(),
        };
        let mut names = HashMap::new();
        let mut output = Vec::new();
        for ((start, end), types) in views {
            let mut spellings = if exact_views.contains(&(start, end)) {
                let Some(spellings) = types
                    .into_iter()
                    .map(|ty| type_name(program, ty, &mut names))
                    .collect::<Option<Vec<_>>>()
                else {
                    continue;
                };
                spellings
            } else {
                types
                    .into_iter()
                    .filter_map(|ty| type_name(program, ty, &mut names))
                    .collect()
            };
            spellings.sort();
            spellings.dedup();
            output.extend(spellings.into_iter().map(|ty| ProjectionInfo {
                span: Span::new(start, end),
                ty,
            }));
        }
        let mut method_results = Vec::new();
        for ((start, end), types) in methods {
            // An unknown callable specialization must not expose another
            // specialization's concrete result at the same source span.
            let Some(mut spellings) = types
                .into_iter()
                .map(|ty| type_name(program, ty, &mut names))
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };
            spellings.sort();
            spellings.dedup();
            method_results.extend(spellings.into_iter().map(|ty| MethodResultInfo {
                span: Span::new(start, end),
                ty,
            }));
        }
        let mut optional_receivers = Vec::new();
        for ((start, end), types) in optional {
            let Some(mut spellings) = types
                .into_iter()
                .map(|ty| ty.and_then(|ty| type_name(program, ty, &mut names)))
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };
            spellings.sort();
            spellings.dedup();
            optional_receivers.extend(spellings.into_iter().map(|ty| ProjectionInfo {
                span: Span::new(start, end),
                ty,
            }));
        }
        let mut parameter_results = Vec::new();
        for ((start, end), types) in parameter_types {
            let Some(mut spellings) = types.into_iter().map(|ty| ty.and_then(|ty| type_name(program, ty, &mut names))).collect::<Option<Vec<_>>>() else { continue; };
            spellings.sort(); spellings.dedup();
            parameter_results.extend(spellings.into_iter().map(|ty| ProjectionInfo { span: Span::new(start, end), ty }));
        }
        let mut rest_record_fields=Vec::new();
        for (id,record) in program.records.iter().enumerate().filter(|(_,record)|record.name.text.starts_with("rest#record")) {
            let Some(owner)=type_name(program,Type::Record(id),&mut names) else{continue;};
            for field in &record.fields {
                if let Some(ty)=field.ty.and_then(|ty|type_name(program,ty,&mut names)) {
                    rest_record_fields.push((owner.clone(),field.name.text.clone(),field.name.span,ty));
                }
            }
        }
        CheckedTypeFacts {
            json_string_receivers,
            generic_values,
            rest_record_fields,
            identities,
            parameter_types: parameter_results,
            optional_receivers,
            projections: output,
            method_results,
            generic_calls: generic_calls.into_values().flatten().collect(),
            generic_parameter_calls,
        }
    }
}

// Read checked identities, never reinterpret displayed nullable type strings.
fn optional_payload(program: &CheckedProgram, ty: Type) -> Option<Type> {
    match ty {
        Type::Union(id) | Type::BorrowedUnion(id) => {
            let mut payloads = program.unions[id.index()].members.iter().copied().filter(|ty| !matches!(ty, Type::Null | Type::Undefined));
            let first = payloads.next();
            if payloads.next().is_none() { first } else { Some(ty) }
        },
        Type::Null | Type::Undefined | Type::Void => None,
        ty => Some(ty),
    }
}

fn child(expression: &Expr, successful_span: Option<Span>) -> Node<'_> {
    if let Some(span) = successful_span {
        Node::Successful(expression, span)
    } else {
        Node::Expression(expression)
    }
}

fn statements<'a>(pending: &mut Vec<Node<'a>>, body: &'a [Stmt]) {
    pending.extend(body.iter().rev().map(Node::Statement));
}

fn type_name(
    program: &CheckedProgram,
    ty: Type,
    names: &mut HashMap<Type, Option<String>>,
) -> Option<String> {
    if let Some(name) = names.get(&ty) {
        return name.clone();
    }
    if let Some(union) = ty.union() {
        let members = &program.unions[union.index()].members;
        if let Some([ok, error]) = crate::result_identity::variants(&program.records, members) {
            // Guard recursive payload presentation just like callable types.
            names.insert(ty, None);
            let payload = |id: usize, names: &mut HashMap<Type, Option<String>>| {
                let fields = &program.records[id].fields;
                type_name(program, fields.first().map(|field| field.ty).unwrap_or(Some(Type::Void))?, names)
            };
            let name = payload(ok, names).zip(payload(error, names))
                .map(|(ok, error)| format!("Result<{ok}, {error}>"));
            names.insert(ty, name.clone());
            return name;
        }
    }
    let name = match ty {
        Type::Int => Some("int".into()),
        Type::Number => Some("number".into()),
        Type::Boolean => Some("boolean".into()),
        Type::String => Some("string".into()),
        Type::Null => Some("null".into()),
        Type::Undefined => Some("undefined".into()),
        Type::Record(id) if program.records[id].application.as_ref().is_some_and(|application| matches!(application.declaration.text.as_str(), "Result#Ok" | "Result#Err")) => {
            names.insert(ty, None);
            let record = &program.records[id];
            let variant = if record.application.as_ref()?.declaration.text == "Result#Ok" { "Ok" } else { "Err" };
            type_name(program, record.fields.first().map(|field| field.ty).unwrap_or(Some(Type::Void))?, names)
                .map(|payload| format!("{variant}({payload})"))
        }
        Type::Record(id) if program.records[id].name.text.starts_with("rest#record") => {
            program.records[id].fields.iter().map(|field|Some(format!("{}: {};",field.name.text,type_name(program,field.ty?,names)?))).collect::<Option<Vec<_>>>().map(|fields|format!("{{ {} }}",fields.join(" ")))
        }
        Type::Record(id) if program.records[id].name.text.starts_with("[rest") => {
            program.records[id].fields.iter().map(|field|type_name(program,field.ty?,names)).collect::<Option<Vec<_>>>().map(|fields|format!("[{}]",fields.join(", ")))
        }
        Type::Record(id) | Type::Enum(id, _) | Type::Set(id) | Type::Map(id) => Some(program.records[id].name.text.clone()),
        // Function<...> needs no grouping before an array suffix or in a union.
        // A union element needs grouping so the suffix applies to the whole union.
        Type::Array(element) => type_name(program, element.ty(), names).map(|name| {
            if matches!(element.ty(), Type::Union(_)) { format!("({name})[]") } else { format!("{name}[]") }
        }),
        Type::Union(id) | Type::BorrowedUnion(id) => program.unions[id.index()]
            .members
            .iter()
            .map(|member| type_name(program, *member, names))
            .collect::<Option<Vec<_>>>()
            .map(|members| members.join(" | ")),
        Type::Void => Some("void".into()),
        Type::Closure(id) => {
            let closure = &program.closures[id];
            if closure.function.template || program.deferred_generic_values.contains(&id) {
                None
            } else {
                // Reject recursive/abstract callable shapes conservatively.
                names.insert(ty, None);
                let parameters = closure
                    .parameters
                    .iter()
                    .enumerate()
                    .map(|(index, id)| {
                        let binding = &closure.function.bindings[*id];
                        callable_parameter(program, &binding.name, binding.input_ty.unwrap_or(binding.ty), binding.mode, binding.omission.is_omittable() && !closure.parameters[index + 1..].iter().any(|id| !closure.function.bindings[*id].rest && !closure.function.bindings[*id].omission.is_omittable()), binding.rest, names)
                    })
                    .collect::<Option<Vec<_>>>();
                parameters
                    .zip(type_name(program, closure.function.result, names))
                    .map(|(parameters, result)| {
                        let wrapper = (closure.invocation == crate::Mode::Consume).then_some("Once");
                        crate::type_syntax::function_type_text(wrapper, &parameters, &result)
                    })
            }
        }
        Type::Callback(id) => {
            names.insert(ty, None);
            let contract = &program.callback_contracts[id];
            let prefix = match program
                .callable_shapes
                .get(contract.structural)
                .kind
                .expect("callback structural contract")
            {
                CallbackKind::Read => Some("ReadOnly"),
                CallbackKind::Reusable => Some("Mutable"),
                CallbackKind::Once => Some("Once"),
                CallbackKind::Owned | CallbackKind::SharedRead => None,
                CallbackKind::OwnedOnce => Some("Once"),
            };
            let parameters = contract
                .params
                .iter()
                .enumerate()
                .map(|(index, parameter)| {
                    callable_parameter(
                        program,
                        &contract.parameter_names[index],
                        parameter.ty,
                        parameter.mode,
                        parameter.omission.is_omittable() && !contract.params[index + 1..].iter().any(|parameter| !parameter.rest && !parameter.omission.is_omittable()),
                        parameter.rest,
                        names,
                    )
                })
                .collect::<Option<Vec<_>>>();
            parameters
                .zip(type_name(program, contract.result, names))
                .map(|(parameters, result)| {
                    crate::type_syntax::function_type_text(prefix, &parameters, &result)
                })
        }
        Type::StoredCallable(id) | Type::BorrowedCallable(id) => program
            .stored_contracts
            .declaration(id)
            .and_then(|declaration| type_name(program, Type::Callback(declaration), names)),
        Type::Callable(_) => None,
    };
    names.insert(ty, name.clone());
    name
}

fn callable_parameter(
    program: &CheckedProgram,
    name: &str,
    ty: Type,
    access: crate::Mode,
    optional: bool,
    rest: bool,
    names: &mut HashMap<Type, Option<String>>,
) -> Option<String> {
    // Callback ownership already carries the environment access contract.
    // Do not render invalid doubled prefixes such as `read read (...)`.
    let callback = matches!(ty, Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_));
    let ty = type_name(program, ty, names)?;
    let ty = crate::type_syntax::access_type_text(if callback || rest { crate::Mode::Copy } else { access }, &ty);
    let marker = if optional && !rest { "?" } else { "" };
    let prefix=if rest {"..."} else {""};
    Some(format!("{prefix}{name}{marker}: {ty}"))
}

#[cfg(test)]
#[test]
fn stored_family_metadata_uses_checked_owned_signature_and_source_names() {
    let checked = crate::check("function first(action: Function<int, [value: ReadOnly<string>]>): int { return action(\"a\"); } function second(other: Function<int, [item: ReadOnly<string>]>): int { return other(\"b\"); } function main() {}").unwrap_or_else(|error| panic!("{error:?}"));
    let program = &checked.program;
    assert_eq!(program.stored_contracts.declaration(0), Some(0));
    assert_eq!(program.stored_contracts.declaration(1), None);
    assert_ne!(
        program.functions[0].bindings[0].ty,
        program.functions[1].bindings[0].ty
    );
    assert_eq!(
        type_name(program, Type::StoredCallable(0), &mut HashMap::new()).as_deref(),
        Some("Function<int, [value: ReadOnly<string>]>")
    );
    assert_eq!(
        type_name(program, Type::StoredCallable(99), &mut HashMap::new()),
        None
    );
}

// Deferred environments have no prototype parameter contract. Only a checked
// concrete method may supply invocation facts.
fn deferred_call_fact(program: &CheckedProgram, expression: &Expr, names: &mut HashMap<Type, Option<String>>) -> Option<GenericCallInfo> {
    let CallTarget::ClosureMethod(method) = expression.target? else { return None; };
    let ExprKind::Call(callee, _) = &expression.kind else { return None; };
    let Type::Closure(origin) = callee.ty? else { return None; };
    let selected=callee.generic_invocation.map(|id|program.polymorphic_invocations.checked(id,&program.closures,callee.span)).transpose().ok()?;
    let (origin,method)=if let Some(plan)=&selected {
        let case=plan.cases.iter().find(|case|case.pair.origin==origin && case.pair.method==method)
            .or_else(||if plan.cases.len()==1 {plan.cases.first()} else {None})?;
        (case.pair.origin,case.pair.method)
    } else {(origin,method)};
    let declaration = program.deferred_generic_sources.get(&origin)?;
    let method = program.closures[origin].methods.get(method)?;
    let bindings = method.parameters.iter().map(|id| &method.function.bindings[*id]).collect::<Vec<_>>();
    Some(GenericCallInfo {
        span: expression.span, callee_span: callee.span, declaration_span: declaration.span,
        parameter_types: bindings.iter().map(|binding| binding.input_ty.unwrap_or(binding.ty)).collect(),
        body_parameter_types: bindings.iter().map(|binding| binding.ty).collect(),
        result_type: method.function.result,
        parameters: bindings.iter().enumerate().map(|(index,binding)| GenericParameterInfo {
            name: selected.as_ref().map_or_else(||binding.name.clone(),|plan| {
                let mut labels=Vec::new();
                for case in &plan.cases {
                    let label=&case.body.function.bindings[case.body.parameters[index]].name;
                    if !labels.contains(label) {labels.push(label.clone());}
                }
                labels.join(" | ")
            }), ty: type_name(program, binding.input_ty.unwrap_or(binding.ty), names),
            body_ty: type_name(program, binding.ty, names), omission: binding.omission,
            rest: binding.rest, access: selected.as_ref().map_or(binding.mode,|plan|plan.argument_modes[index]),
        }).collect(),
        result: type_name(program, method.function.result, names)?,
    })
}

#[test]
fn deferred_projection_uses_selected_declaration_contract() {
    let source=crate::format("function first<T>(left:T):T{return left;}function second<T>(right:T):T{return right;}function main(){const one=first;const two=second;one<int>(1);two<int>(2);}").unwrap();
    assert_eq!(crate::format(&source).unwrap(),source);
    let mut program=crate::check(&source).unwrap().program;
    let main=program.functions.iter().position(|function|function.name=="main").unwrap();
    let calls=program.functions[main].body.iter().filter_map(|statement|match &statement.kind {StmtKind::Expr(call)=>Some(call),_=>None}).collect::<Vec<_>>();
    let ExprKind::Call(callee,_) = &calls[1].kind else {panic!("second call");};
    let Some(Type::Closure(origin))=callee.ty else {panic!("second environment");};
    let Some(CallTarget::ClosureMethod(method))=calls[1].target else {panic!("second method");};
    let expected_span=program.deferred_generic_sources[&origin].span;
    let mut expression=calls[0].clone();
    let id=program.polymorphic_invocations.publish([crate::polymorphic_invocation_plans::Pair{origin,method}],&program.closures,expression.span).unwrap();
    let ExprKind::Call(callee,_) = &mut expression.kind else {panic!("first call");};
    callee.generic_invocation=Some(id);
    let fact=deferred_call_fact(&program,&expression,&mut HashMap::new()).unwrap();
    assert_eq!(fact.parameters[0].name,"right");
    assert_eq!(fact.declaration_span,expected_span);
    assert_eq!(fact.result,"int");
}
