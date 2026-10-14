//! One worklist owns data, callable, parameter and storage resolution continuations.
use super::*;
use crate::type_syntax::{CallableOwnership, ValueAnnotation, ValueAnnotationKind as Kind};
use crate::Span;

#[derive(Clone, Copy)]
enum Parameters<'a> { Source(&'a [CallableParameter]), Structured(&'a [crate::type_syntax::CallableParameter]) }
impl Parameters<'_> {
    fn len(self) -> usize { match self { Self::Source(values) => values.len(), Self::Structured(values) => values.len() } }
    fn names(self) -> Vec<String> { match self {
        Self::Source(values) => values.iter().map(|value| value.name.text.clone()).collect(),
        Self::Structured(values) => values.iter().map(|value| value.name.text.clone()).collect(),
    } }
}
struct ParameterState<'a> { source: Parameters<'a>, index: usize, explicit: bool, names: HashSet<&'a str>, arguments: Vec<ArgumentContract> }
struct ParameterMeta { name: Span, annotation: Span, access: Option<crate::type_syntax::ParameterAccess>, omission: crate::ParameterOmission, rest: bool }
struct UnionState<'a> { members: &'a [ValueAnnotation], next: usize, types: Vec<Type>, span: Span, declaration: Option<usize> }
enum Work<'a> {
    CacheValue(usize, bool), CacheStorage(usize), CacheArray(usize),
    Value(&'a ValueAnnotation, bool), AfterValueArray(&'a ValueAnnotation, bool),
    ArrayStorage(&'a ValueAnnotation), AfterArrayStorage(usize), AfterStorageArray,
    Storage(&'a ValueAnnotation), StoredCallback(Span), MapStorageError(Span, Span),
    Callback(&'a CallbackAnnotation), StructuredCallback(&'a ValueAnnotation),
    Result(&'a ResultAnnotation), StructuredResult(&'a ValueAnnotation),
    FinishCallable(Parameters<'a>, Span, Option<CallbackKind>, bool),
    Parameters(ParameterState<'a>), AfterParameter(ParameterState<'a>, ParameterMeta),
    ValueUnion(UnionState<'a>), AfterValueUnion(UnionState<'a>, Option<CallbackKind>),
    StorageUnion(UnionState<'a>), AfterStorageUnion(UnionState<'a>), AfterStorageFallback(UnionState<'a>),
}
enum Output { Type(Type), Parameters(Vec<ArgumentContract>), Storage(Option<(Type, usize)>), ArrayStorage(Option<(Type, usize, usize)>) }
fn pop_type(values: &mut Vec<Output>) -> Type { let Some(Output::Type(ty)) = values.pop() else { unreachable!("resolved type") }; ty }
fn parameter_state(source: Parameters<'_>, explicit: bool) -> ParameterState<'_> { ParameterState { source, index: 0, explicit, names: HashSet::new(), arguments: Vec::new() } }
fn union_state(members: &[ValueAnnotation], span: Span) -> UnionState<'_> { UnionState { members, next: 0, types: Vec::with_capacity(members.len()), span, declaration: None } }
fn callback_kind(value: &ValueAnnotation) -> Option<CallbackKind> {
    let (leaf, dimensions) = crate::callable_annotations::storage_leaf(value);
    (dimensions == 0).then(|| crate::callable_annotations::ownership(leaf)).flatten()
}
fn ungroup(mut value: &ValueAnnotation) -> &ValueAnnotation { while let Kind::Group(inner) = &value.kind { value = inner; } value }
fn result_borrow_check(value: &ValueAnnotation) -> Result<(), Diagnostic> {
    let kind = crate::callable_annotations::ownership(value).or_else(|| crate::callable_annotations::nullable_kind(value));
    if kind.is_some_and(|kind| !kind.is_owned()) {
        return Err(Diagnostic::new("callback-escape", "A factory result must own its callable environment; borrowed callable results cannot escape.", value.span));
    }
    Ok(())
}

#[derive(Default)]
struct Cache {
    values: HashMap<(usize, bool), Type>,
    storage: HashMap<usize, Option<(Type, usize)>>,
    arrays: HashMap<usize, Option<(Type, usize, usize)>>,
}
// References stay live and immutable for this one resolution run. Address keys
// distinguish equal spans in different declarations; no pointer is dereferenced.
fn identity(value: &ValueAnnotation) -> usize { value as *const ValueAnnotation as usize }

fn run<'a>(resolver: &mut Resolver<'_>, root: Work<'a>) -> Result<Output, Diagnostic> {
    let mut pending = vec![root];
    let mut values = Vec::new();
    let mut cache = Cache::default();
    while let Some(work) = pending.pop() {
        if let Err(mut error) = step(resolver, work, &mut pending, &mut values, &mut cache) {
            // Only active grouped storage frames widen errors. Unvisited union
            // siblings have no frames yet and cannot affect the first location.
            for frame in pending.iter().rev() {
                if let Work::MapStorageError(inner, outer) = frame && error.span == *inner { error.span = *outer; }
            }
            return Err(error);
        }
    }
    Ok(values.pop().expect("root resolution result"))
}

fn step<'a>(r: &mut Resolver<'_>, work: Work<'a>, pending: &mut Vec<Work<'a>>, values: &mut Vec<Output>, cache: &mut Cache) -> Result<(), Diagnostic> {
    match work {
        Work::CacheValue(id, allow_void) => { let Some(Output::Type(ty)) = values.last() else { unreachable!() }; cache.values.insert((id, allow_void), *ty); }
        Work::CacheStorage(id) => { let Some(Output::Storage(found)) = values.last() else { unreachable!() }; cache.storage.insert(id, *found); }
        Work::CacheArray(id) => { let Some(Output::ArrayStorage(found)) = values.last() else { unreachable!() }; cache.arrays.insert(id, *found); }
        Work::Value(value, allow_void) => {
            if let Some(&ty) = cache.values.get(&(identity(value), allow_void)) { values.push(Output::Type(ty)); }
            else { pending.push(Work::CacheValue(identity(value), allow_void)); pending.push(Work::AfterValueArray(value, allow_void)); pending.push(Work::ArrayStorage(value)); }
        }
        Work::AfterValueArray(value, allow_void) => {
            let Some(Output::ArrayStorage(array)) = values.pop() else { unreachable!() };
            if let Some((ty, _, _)) = array { values.push(Output::Type(ty)); }
            else { match &value.kind {
                Kind::Group(_) => pending.push(Work::Value(ungroup(value), allow_void)),
                Kind::Union(members) => pending.push(Work::ValueUnion(union_state(members, value.span))),
                Kind::Callable { .. } => pending.push(Work::StructuredCallback(value)),
                _ => values.push(Output::Type(resolve_value_type(value, allow_void, r.record_names, r.unions)?)),
            } }
        }
        Work::ArrayStorage(value) => {
            if let Some(&found) = cache.arrays.get(&identity(value)) { values.push(Output::ArrayStorage(found)); return Ok(()); }
            pending.push(Work::CacheArray(identity(value)));
            let (leaf, dimensions) = crate::callable_annotations::storage_leaf(value);
            if dimensions == 0 { values.push(Output::ArrayStorage(None)); }
            else {
                if dimensions > 128 { return Err(Diagnostic::new("unsupported", "Array type nesting exceeds the prototype limit of 128.", value.span)); }
                pending.push(Work::AfterArrayStorage(dimensions)); pending.push(Work::Storage(leaf));
            }
        }
        Work::AfterArrayStorage(dimensions) => {
            let Some(Output::Storage(storage)) = values.pop() else { unreachable!() };
            values.push(Output::ArrayStorage(storage.map(|(ty, declaration)| {
                let Type::StoredCallable(family) = ty else { unreachable!("stored callable leaf") };
                (Type::Array(crate::syntax::ArrayElement::StoredCallable(family, (dimensions - 1) as u8)), declaration, dimensions)
            })));
        }
        Work::AfterStorageArray => {
            let Some(Output::ArrayStorage(storage)) = values.pop() else { unreachable!() };
            values.push(Output::Storage(storage.map(|(ty, declaration, _)| (ty, declaration))));
        }
        Work::Storage(value) => {
            if let Some(&found) = cache.storage.get(&identity(value)) { values.push(Output::Storage(found)); return Ok(()); }
            pending.push(Work::CacheStorage(identity(value)));
            if crate::callable_annotations::storage_leaf(value).1 != 0 {
                pending.push(Work::AfterStorageArray); pending.push(Work::ArrayStorage(value));
            } else { match &value.kind {
                Kind::Group(_) => {
                    let mut inner = value;
                    while let Kind::Group(child) = &inner.kind { pending.push(Work::MapStorageError(child.span, inner.span)); inner = child; }
                    pending.push(Work::Storage(inner));
                }
                Kind::Union(members) => pending.push(Work::StorageUnion(union_state(members, value.span))),
                _ => if let Some(kind) = callback_kind(value) {
                    if !kind.is_owned() { return Err(Diagnostic::new("callback-escape", "A field stores a Function<...> or Once<Function<...>>; ReadOnly and Mutable functions are borrowed and belong in local bindings or parameters.", value.span)); }
                    pending.push(Work::StoredCallback(value.span)); pending.push(Work::StructuredCallback(value));
                } else { values.push(Output::Storage(None)); },
            } }
        }
        Work::MapStorageError(_, _) => {}
        Work::StoredCallback(span) => {
            let Type::Callback(id) = pop_type(values) else { unreachable!("resolved callback signature") };
            crate::callable_dispatch::Family::checked(&r.callback_contracts[id], r.callable_shapes, [], &[], span)?;
            values.push(Output::Storage(r.storage.intern(r.callable_shapes, &r.callback_contracts[id], id).map(|ty| (ty, id))));
        }
        Work::Callback(value) => {
            let params = Parameters::Source(&value.params);
            pending.extend([Work::FinishCallable(params, value.span, Some(value.kind), true), Work::Result(&value.result), Work::Parameters(parameter_state(params, true))]);
        }
        Work::StructuredCallback(value) => {
            let Kind::Callable { parameters, result, .. } = &ungroup(value).kind else { unreachable!() };
            let params = Parameters::Structured(parameters);
            pending.extend([Work::FinishCallable(params, value.span, callback_kind(value), true), Work::StructuredResult(result), Work::Parameters(parameter_state(params, true))]);
        }
        Work::Result(value) => match value {
            ResultAnnotation::Inferred(_) => values.push(Output::Type(Type::Void)),
            ResultAnnotation::Named(value) => { result_borrow_check(value)?; pending.push(Work::Value(value, true)); }
            ResultAnnotation::Callable { kind, params, result, span } => {
                let params = Parameters::Source(params);
                pending.extend([Work::FinishCallable(params, *span, *kind, false), Work::Result(result), Work::Parameters(parameter_state(params, true))]);
            }
        }
        Work::StructuredResult(value) => {
            if let Kind::Callable { ownership, parameters, result } = &ungroup(value).kind
                && matches!(ownership, CallableOwnership::Default | CallableOwnership::Owned | CallableOwnership::OwnedOnce | CallableOwnership::SharedRead) {
                let kind = if *ownership == CallableOwnership::Default { None } else { callback_kind(value) };
                let params = Parameters::Structured(parameters);
                pending.extend([Work::FinishCallable(params, value.span, kind, false), Work::StructuredResult(result), Work::Parameters(parameter_state(params, true))]);
            } else { result_borrow_check(value)?; pending.push(Work::Value(value, true)); }
        }
        Work::FinishCallable(source, span, kind, callback) => {
            let result = pop_type(values);
            let Some(Output::Parameters(params)) = values.pop() else { unreachable!() };
            let key = CallableSourceKey { span: (span.start, span.end), contract: r.intern_callable_shape(&params, result, kind) };
            let ty = if callback {
                if let Some(&id) = r.callback_source_types.get(&key) { Type::Callback(id) }
                else {
                    let id = r.callback_contracts.len();
                    r.callback_contracts.push(CallbackContract { structural: key.contract, parameter_names: source.names(), params, result, kind: kind.expect("callback mode") });
                    r.storage.intern(r.callable_shapes, &r.callback_contracts[id], id);
                    r.callback_source_types.insert(key, id); Type::Callback(id)
                }
            } else if let Some(&id) = r.result_source_types.get(&key) { Type::Callable(id) }
            else {
                let id = r.result_contracts.len();
                r.result_contracts.push(ResultContract { structural: key.contract, params, result });
                r.result_source_types.insert(key, id); Type::Callable(id)
            };
            values.push(Output::Type(ty));
        }
        Work::Parameters(state) => {
            if state.index == state.source.len() { values.push(Output::Parameters(state.arguments)); }
            else {
                let mut state = state;
                let (meta, next) = match state.source {
                    Parameters::Source(params) => {
                        let p = &params[state.index];
                        crate::callable_contracts::parameter_name(&p.name.text, p.name.span, &mut state.names)?;
                        let next = match &p.ty {
                            ParameterAnnotation::Named(value) => Work::Value(value, true),
                            ParameterAnnotation::Callback(value) => Work::Callback(value),
                            ParameterAnnotation::Inferred(token) => return Err(Diagnostic::new("parameter-inference", "Parameter type must be inferred from its initializer or callback context.", token.span)),
                        };
                        let access = p.access.as_ref().map(|access| crate::type_syntax::ParameterAccess { mode: match access.text.as_str() { "read" => Mode::Read, "mutate" => Mode::Mutate, "consume" => Mode::Consume, _ => unreachable!("parsed access modifier") }, span: access.span });
                        (ParameterMeta { name: p.name.span, annotation: p.ty.span(), access, omission: p.initializer.omission(), rest: p.rest }, next)
                    }
                    Parameters::Structured(params) => {
                        let p = &params[state.index];
                        crate::callable_contracts::parameter_name(&p.name.text, p.name.span, &mut state.names)?;
                        let next = if callback_kind(&p.annotation).is_some() { Work::StructuredCallback(&p.annotation) } else { Work::Value(&p.annotation, true) };
                        // The existing structured adapter represents every omitted
                        // declaration as Optional; executable defaults live in source.
                        let omission = if p.omission.is_omittable() { crate::ParameterOmission::Optional } else { crate::ParameterOmission::Required };
                        (ParameterMeta { name: p.name.span, annotation: p.annotation.span, access: p.access.clone(), omission, rest: p.rest }, next)
                    }
                };
                pending.push(Work::AfterParameter(state, meta)); pending.push(next);
            }
        }
        Work::AfterParameter(mut state, meta) => {
            let ty = pop_type(values);
            let callback = match r.unions.payload(ty).unwrap_or(ty) { Type::Callback(id) => Some(r.callback_contracts[id].kind), _ => None };
            let mode = if meta.rest {
                if !matches!(ty, Type::Array(_)) { return Err(Diagnostic::new("rest-type", "A rest parameter requires a supported homogeneous array type.", meta.annotation)); }
                if meta.access.as_ref().is_some_and(|access| access.mode != Mode::Consume) { return Err(Diagnostic::new("rest-access", "A rest parameter owns its pack; only explicit consume access is supported.", meta.name)); }
                Mode::Consume
            } else { crate::callable_contracts::parameter_mode(ty, callback, state.explicit, meta.access.as_ref(), meta.annotation)? };
            let ty = if meta.omission.is_omittable() { r.unions.intern([ty, Type::Undefined], meta.name)? } else { ty };
            state.arguments.push(ArgumentContract { ty, mode, omission: meta.omission, rest: meta.rest });
            state.index += 1; pending.push(Work::Parameters(state));
        }
        Work::ValueUnion(state) => {
            if state.next == state.members.len() { values.push(Output::Type(r.unions.intern(state.types, state.span)?)); }
            else {
                let member = &state.members[state.next]; let kind = callback_kind(member);
                pending.push(Work::AfterValueUnion(state, kind));
                pending.push(if kind.is_some() { Work::StructuredCallback(member) } else { Work::Value(member, true) });
            }
        }
        Work::AfterValueUnion(mut state, kind) => {
            let mut ty = pop_type(values);
            if let Some(kind) = kind {
                let Type::Callback(id) = ty else { unreachable!() };
                let contextual = matches!(r.callback_contracts[id].result, Type::Callable(_)) || r.callback_contracts[id].params.iter().any(|parameter| matches!(r.unions.payload(parameter.ty).unwrap_or(parameter.ty), Type::Callback(_)));
                ty = if kind.is_owned() && !contextual { r.storage.intern(r.callable_shapes, &r.callback_contracts[id], id).expect("owned callback storage") }
                else { state.types.iter().find_map(|previous| match previous { Type::Callback(previous) if r.callback_contracts[*previous].structural == r.callback_contracts[id].structural => Some(Type::Callback(*previous)), _ => None }).unwrap_or(Type::Callback(id)) };
            }
            state.types.push(ty); state.next += 1; pending.push(Work::ValueUnion(state));
        }
        Work::StorageUnion(state) => {
            if state.next == state.members.len() {
                values.push(Output::Storage(if let Some(id) = state.declaration { Some((r.unions.intern(state.types, state.span)?, id)) } else { None }));
            } else {
                let member = &state.members[state.next]; pending.push(Work::AfterStorageUnion(state)); pending.push(Work::Storage(member));
            }
        }
        Work::AfterStorageUnion(mut state) => {
            let Some(Output::Storage(storage)) = values.pop() else { unreachable!() };
            if let Some((ty, id)) = storage {
                state.types.push(ty); state.declaration = Some(id); state.next += 1; pending.push(Work::StorageUnion(state));
            } else {
                let member = &state.members[state.next]; pending.push(Work::AfterStorageFallback(state)); pending.push(Work::Value(member, false));
            }
        }
        Work::AfterStorageFallback(mut state) => { state.types.push(pop_type(values)); state.next += 1; pending.push(Work::StorageUnion(state)); }
    }
    Ok(())
}

pub(super) fn value(r: &mut Resolver<'_>, value: &ValueAnnotation, allow_void: bool) -> Result<Type, Diagnostic> { let Output::Type(ty) = run(r, Work::Value(value, allow_void))? else { unreachable!() }; Ok(ty) }
pub(super) fn callback(r: &mut Resolver<'_>, value: &CallbackAnnotation) -> Result<Type, Diagnostic> { let Output::Type(ty) = run(r, Work::Callback(value))? else { unreachable!() }; Ok(ty) }
pub(super) fn result(r: &mut Resolver<'_>, value: &ResultAnnotation) -> Result<Type, Diagnostic> { let Output::Type(ty) = run(r, Work::Result(value))? else { unreachable!() }; Ok(ty) }
pub(super) fn storage(r: &mut Resolver<'_>, value: &ValueAnnotation) -> Result<Option<(Type, usize)>, Diagnostic> { let Output::Storage(found) = run(r, Work::Storage(value))? else { unreachable!() }; Ok(found) }
#[cfg(test)]
pub(super) fn array_storage(r: &mut Resolver<'_>, value: &ValueAnnotation) -> Result<Option<(Type, usize, usize)>, Diagnostic> { let Output::ArrayStorage(found) = run(r, Work::ArrayStorage(value))? else { unreachable!() }; Ok(found) }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_syntax::{parse, TypeName};
    #[test]
    fn deep_callable_resolution_keeps_flat_contract_identity_and_production_drop() {
        let span = Span::new(11, 19);
        let leaf = || ValueAnnotation { kind: Kind::Named(TypeName { text: "int".into(), span }), span };
        let mut annotation = leaf();
        for depth in 0..16_384 {
            let (parameters, result) = if depth % 2 == 0 { (vec![], Box::new(annotation)) } else {
                (vec![crate::type_syntax::CallableParameter { rest: false, name: TypeName { text: "input".into(), span }, access: None, omission: crate::ParameterOmission::Required, annotation, span }], Box::new(leaf()))
            };
            annotation = ValueAnnotation { kind: Kind::Callable { ownership: CallableOwnership::OwnedOnce, parameters, result }, span };
        }
        let names = HashMap::new(); let unions = crate::unions::UnionArena::default(); let mut tables = Tables::default();
        let first = tables.resolver(&names, &unions).resolve_value_annotation(&annotation, false).unwrap();
        assert!(matches!(first, Type::Callback(_)));
        assert_eq!(tables.callbacks.len() + tables.results.len(), 16_384);
        assert_eq!(tables.resolver(&names, &unions).resolve_value_annotation(&annotation, false).unwrap(), first);
        assert_eq!(tables.callbacks.len() + tables.results.len(), 16_384);
        drop(annotation); drop(tables);
    }
    #[test]
    fn deep_data_unions_resolve_once_through_array_storage_fallback() {
        let mut annotation = parse("int").unwrap();
        for _ in 0..16_384 { let span = annotation.span; annotation = ValueAnnotation { kind: Kind::Union(vec![annotation]), span }; }
        let span = annotation.span;
        let annotation = ValueAnnotation { kind: Kind::Array(Box::new(annotation)), span };
        let names = HashMap::new(); let unions = crate::unions::UnionArena::default(); let mut tables = Tables::default();
        assert_eq!(tables.resolver(&names, &unions).resolve_value_annotation(&annotation, false).unwrap(), Type::Array(crate::syntax::ArrayElement::Int(0)));
        assert_eq!(resolve_value_type(&annotation, false, &names, &unions).unwrap(), Type::Array(crate::syntax::ArrayElement::Int(0)));
        drop(annotation);
    }
    #[test]
    fn deep_source_result_wrappers_resolve_through_the_same_worklist() {
        let span = Span::new(7, 11);
        let mut source = ResultAnnotation::Named(Box::new(parse("int").unwrap()));
        for _ in 0..16_384 {
            source = ResultAnnotation::Callable { kind: Some(CallbackKind::OwnedOnce), params: vec![], result: Box::new(source), span };
        }
        let names = HashMap::new(); let unions = crate::unions::UnionArena::default(); let mut tables = Tables::default();
        let resolved = tables.resolver(&names, &unions).resolve_result(&source);
        // Use production wrapper disposal for the deep resolver witness.
        drop(source);
        assert!(matches!(resolved.unwrap(), Type::Callable(_)));
        assert_eq!(tables.results.len(), 16_384);
    }
    #[test]
    fn grouped_storage_errors_and_first_parameter_error_keep_original_ranges() {
        let names = HashMap::new(); let unions = crate::unions::UnionArena::default(); let mut tables = Tables::default();
        let mut annotation = parse("ReadOnly<Function<int>>").unwrap();
        for _ in 0..16_384 { let span = Span::new(0, annotation.span.end + 1); annotation = ValueAnnotation { kind: Kind::Group(Box::new(annotation)), span }; }
        let error = tables.resolver(&names, &unions).storage_declaration(&annotation).unwrap_err();
        assert_eq!(error.code, "callback-escape"); assert_eq!(error.span, annotation.span);
        let source = "Function<int,[first:Missing,first:int]>";
        let error = tables.resolver(&names, &unions).resolve_value_annotation(&parse(source).unwrap(), false).unwrap_err();
        assert_eq!(&source[error.span.start..error.span.end], "Missing");
        assert_eq!(error.code, "type");
    }
}
