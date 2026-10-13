//! Source callable resolution independent of function bodies and environment origins.
mod traversal;
use crate::analysis::{ArgumentContract, CallbackContract, resolve_value_type};
use crate::syntax::{
    CallableParameter, CallbackAnnotation, CallbackKind, ParameterAnnotation, ResultAnnotation,
    Type,
};
use crate::{Diagnostic, Mode};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct CallableSourceKey {
    span: (usize, usize),
    contract: crate::callable_contracts::ContractId,
}

pub(crate) struct ResultContract {
    pub(crate) structural: crate::callable_contracts::ContractId,
    pub(crate) params: Vec<ArgumentContract>,
    pub(crate) result: Type,
}

/// One owner for structural identities and the distinct source ownership paths.
#[derive(Default)]
pub(crate) struct Tables {
    pub(crate) storage: crate::callable_storage::Registry,
    pub(crate) shapes: crate::callable_contracts::Contracts,
    pub(crate) callback_sources: HashMap<CallableSourceKey, usize>,
    pub(crate) result_sources: HashMap<CallableSourceKey, usize>,
    pub(crate) callbacks: Vec<CallbackContract>,
    pub(crate) results: Vec<ResultContract>,
}

impl Tables {
    pub(crate) fn resolver<'a>(
        &'a mut self,
        record_names: &'a HashMap<String, usize>,
        unions: &'a crate::unions::UnionArena,
    ) -> Resolver<'a> {
        Resolver {
            storage: &mut self.storage,
            record_names,
            unions,
            callable_shapes: &mut self.shapes,
            callback_source_types: &mut self.callback_sources,
            result_source_types: &mut self.result_sources,
            callback_contracts: &mut self.callbacks,
            result_contracts: &mut self.results,
        }
    }
}

pub(crate) struct Resolver<'a> {
    pub(crate) storage: &'a mut crate::callable_storage::Registry,
    pub(crate) record_names: &'a HashMap<String, usize>,
    pub(crate) unions: &'a crate::unions::UnionArena,
    pub(crate) callable_shapes: &'a mut crate::callable_contracts::Contracts,
    pub(crate) callback_source_types: &'a mut HashMap<CallableSourceKey, usize>,
    pub(crate) result_source_types: &'a mut HashMap<CallableSourceKey, usize>,
    pub(crate) callback_contracts: &'a mut Vec<CallbackContract>,
    pub(crate) result_contracts: &'a mut Vec<ResultContract>,
}

impl Resolver<'_> {
    pub(crate) fn resolve_value_annotation(
        &mut self,
        annotation: &crate::type_syntax::ValueAnnotation,
        allow_void: bool,
    ) -> Result<Type, Diagnostic> {
        traversal::value(self, annotation, allow_void)
    }
    fn callable_shape_type(&self, ty: Type) -> crate::callable_contracts::ContractType {
        use crate::callable_contracts::ContractType;
        match ty {
            Type::Callback(id) => ContractType::Callable(self.callback_contracts[id].structural),
            Type::Callable(id) => ContractType::Callable(self.result_contracts[id].structural),
            _ => ContractType::Data(ty),
        }
    }

    pub(crate) fn intern_callable_shape(
        &mut self,
        params: &[ArgumentContract],
        result: Type,
        kind: Option<CallbackKind>,
    ) -> crate::callable_contracts::ContractId {
        let parameters = params
            .iter()
            .map(|parameter| (self.callable_shape_type(parameter.ty), parameter.mode))
            .collect::<Vec<_>>();
        let result = self.callable_shape_type(result);
        let rest = params.last().is_some_and(|parameter|parameter.rest);
        let minimum = crate::parameter_arity::minimum_arguments(&params[..params.len()-usize::from(rest)], |parameter|parameter.omission.is_omittable());
        self.callable_shapes.intern_rest(parameters, result, kind, minimum, rest)
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Source callable field construction integration is pending"
        )
    )]
    pub(crate) fn storage_annotation(
        &mut self,
        annotation: &crate::type_syntax::ValueAnnotation,
    ) -> Result<Option<Type>, Diagnostic> {
        self.storage_declaration(annotation)
            .map(|resolved| resolved.map(|(ty, _)| ty))
    }

    /// Retain source labels independently of the canonical family's representative.
    pub(crate) fn array_storage_declaration(
        &mut self,
        annotation: &crate::type_syntax::ValueAnnotation,
    ) -> Result<Option<(Type, usize, usize)>, Diagnostic> {
        traversal::array_storage(self, annotation)
    }

    pub(crate) fn storage_declaration(
        &mut self,
        annotation: &crate::type_syntax::ValueAnnotation,
    ) -> Result<Option<(Type, usize)>, Diagnostic> {
        traversal::storage(self, annotation)
    }

    pub(crate) fn resolve_callback(
        &mut self,
        annotation: &CallbackAnnotation,
    ) -> Result<Type, Diagnostic> {
        traversal::callback(self, annotation)
    }

    pub(crate) fn resolve_result(
        &mut self,
        annotation: &ResultAnnotation,
    ) -> Result<Type, Diagnostic> {
        traversal::result(self, annotation)
    }

    pub(crate) fn parameter_contracts(
        &mut self,
        params: &[CallableParameter],
        explicit: bool,
    ) -> Result<Vec<ArgumentContract>, Diagnostic> {
        traversal::parameters(self, params, explicit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_registry_moves_between_phases_without_reallocating_contracts() {
        let names = HashMap::from([("Item".to_owned(), 0)]);
        let unions = crate::unions::UnionArena::default();
        let annotation = crate::type_syntax::parse("Function<Item, [item: Owned<Item>]>").unwrap();
        let annotation = crate::callable_annotations::callback(&annotation).unwrap();
        let mut tables = Tables::default();
        let Type::Callback(original) = tables
            .resolver(&names, &unions)
            .resolve_callback(&annotation)
            .unwrap()
        else {
            panic!("callback");
        };
        let structural = tables.callbacks[original].structural;
        let address = std::ptr::from_ref(tables.shapes.get(structural));
        // Declaration and body phases transfer ownership of this same registry.
        let mut body_tables = tables;
        assert_eq!(
            address,
            std::ptr::from_ref(body_tables.shapes.get(structural))
        );
        assert_eq!(
            body_tables
                .resolver(&names, &unions)
                .resolve_callback(&annotation)
                .unwrap(),
            Type::Callback(original)
        );
        assert_eq!(body_tables.callbacks.len(), 1);
        assert_eq!(body_tables.callbacks[original].parameter_names, ["item"]);
        assert_eq!(body_tables.callbacks[original].result, Type::Record(0));
    }

    #[derive(Default)]
    struct TestTables {
        names: HashMap<String, usize>,
        unions: crate::unions::UnionArena,
        callables: Tables,
    }
    impl TestTables {
        fn resolve(&mut self, source: &str) -> Result<Type, Diagnostic> {
            let annotation = crate::type_syntax::parse(source)?;
            let callback =
                crate::callable_annotations::callback(&annotation).expect("callback syntax");
            self.callables
                .resolver(&self.names, &self.unions)
                .resolve_callback(&callback)
        }
    }

    #[test]
    fn standalone_tables_resolve_nested_contracts_without_function_analysis() {
        let mut tables = TestTables::default();
        tables.names.insert("Item".into(), 0);
        let source = "Function<(value: int) => int, [action: Function<Item, [value: Owned<Item>]>]>";
        let Type::Callback(first) = tables.resolve(source).unwrap() else {
            panic!("callback");
        };
        assert_eq!(tables.resolve(source).unwrap(), Type::Callback(first));
        let Type::Callback(second) = tables.resolve(&format!("  {source}")).unwrap() else {
            panic!("callback");
        };
        assert_ne!(first, second);
        assert_eq!(
            tables.callables.callbacks[first].structural,
            tables.callables.callbacks[second].structural
        );
        assert_eq!(
            tables.callables.callbacks[first].parameter_names,
            ["action"]
        );
        let Type::Callback(child) = tables.callables.callbacks[first].params[0].ty else {
            panic!("nested callback");
        };
        assert_eq!(tables.callables.callbacks[child].parameter_names, ["value"]);
        assert_eq!(
            tables.callables.callbacks[child].params[0].ty,
            Type::Record(0)
        );
        assert_eq!(
            tables.callables.callbacks[child].params[0].mode,
            Mode::Consume
        );
        let Type::Callable(result) = tables.callables.callbacks[first].result else {
            panic!("callable result");
        };
        assert_eq!(tables.callables.results[result].params[0].ty, Type::Int);
        assert_eq!(tables.callables.results[result].params[0].mode, Mode::Copy);
        assert_eq!(
            tables
                .callables
                .shapes
                .get(tables.callables.results[result].structural)
                .kind,
            None
        );
    }

    #[test]
    fn same_source_span_with_different_nominals_keeps_distinct_contracts() {
        let mut tables = TestTables::default();
        let source = "Function<Item, [value: Owned<Item>]>";
        tables.names.insert("Item".into(), 0);
        let Type::Callback(first) = tables.resolve(source).unwrap() else {
            panic!("callback");
        };
        tables.names.insert("Item".into(), 1);
        let Type::Callback(second) = tables.resolve(source).unwrap() else {
            panic!("callback");
        };
        assert_ne!(first, second);
        assert_ne!(
            tables.callables.callbacks[first].structural,
            tables.callables.callbacks[second].structural
        );
        assert_eq!(tables.callables.callbacks[first].result, Type::Record(0));
        assert_eq!(tables.callables.callbacks[second].result, Type::Record(1));
    }

    #[test]
    fn standalone_resolution_preserves_access_and_unknown_type_ranges() {
        let mut tables = TestTables::default();
        let source = "Function<int, [value: ReadOnly<int>]>";
        let error = tables.resolve(source).unwrap_err();
        assert_eq!(error.code, "callback-access");
        assert_eq!(&source[error.span.start..error.span.end], "ReadOnly");
        let source = "Function<int, [value: Owned<Missing>]>";
        let error = tables.resolve(source).unwrap_err();
        assert_eq!(error.code, "type");
        assert_eq!(&source[error.span.start..error.span.end], "Missing");
    }
}

#[cfg(test)]
#[test]
fn storage_annotation_resolves_owned_groups_without_function_body_state() {
    let names = HashMap::from([("Item".to_owned(), 0)]);
    let unions = crate::unions::UnionArena::default();
    let mut tables = Tables::default();
    for source in [
        "Function<int, [item: ReadOnly<Item>]>",
        "((Function<int, [other: ReadOnly<Item>]>))",
    ] {
        let annotation = crate::type_syntax::parse(source).unwrap();
        assert_eq!(
            tables
                .resolver(&names, &unions)
                .storage_annotation(&annotation)
                .unwrap(),
            Some(Type::StoredCallable(0))
        );
    }
    let declaration = tables.storage.declaration(0).unwrap();
    assert_eq!(tables.callbacks[declaration].parameter_names, ["item"]);
    assert_eq!(tables.callbacks[declaration].params[0].ty, Type::Record(0));
    assert_eq!(tables.callbacks[declaration].params[0].mode, Mode::Read);
    let once = crate::type_syntax::parse("Once<Function<int, [item: ReadOnly<Item>]>>").unwrap();
    assert_eq!(
        tables
            .resolver(&names, &unions)
            .storage_annotation(&once)
            .unwrap(),
        Some(Type::StoredCallable(1))
    );
    let data = crate::type_syntax::parse("Item").unwrap();
    assert_eq!(
        tables
            .resolver(&names, &unions)
            .storage_annotation(&data)
            .unwrap(),
        None
    );
}

#[cfg(test)]
#[test]
fn borrowed_field_signatures_reject_before_registering_storage() {
    let names = HashMap::new();
    let unions = crate::unions::UnionArena::default();
    let mut tables = Tables::default();
    for source in ["Mutable<Function<int>>", "ReadOnly<Function<int>>", "((Mutable<Function<int>>))"] {
        let annotation = crate::type_syntax::parse(source).unwrap();
        let error = tables
            .resolver(&names, &unions)
            .storage_annotation(&annotation)
            .unwrap_err();
        assert_eq!(error.code, "callback-escape");
        assert_eq!(error.span, annotation.span);
        assert!(tables.callbacks.is_empty());
    }
    for (source, code) in [
        ("Function<int, [item: string]>", "callback-access"),
        ("Function<Missing>", "type"),
    ] {
        let annotation = crate::type_syntax::parse(source).unwrap();
        assert_eq!(
            tables
                .resolver(&names, &unions)
                .storage_annotation(&annotation)
                .unwrap_err()
                .code,
            code
        );
    }
}

#[cfg(test)]
#[test]
fn callable_array_resolution_retains_family_identity_rank_and_source_labels() {
    let names = HashMap::from([("Item".to_owned(), 0)]);
    let unions = crate::unions::UnionArena::default();
    let mut tables = Tables::default();
    for (source, rank) in [
        ("((item: ReadOnly<Item>) => int)[]", 1),
        ("(((other: ReadOnly<Item>) => int)[])[]", 2),
    ] {
        let annotation = crate::type_syntax::parse(source).unwrap();
        let (ty, declaration, dimensions) = tables
            .resolver(&names, &unions)
            .array_storage_declaration(&annotation)
            .unwrap()
            .unwrap();
        assert_eq!(
            ty,
            Type::Array(crate::syntax::ArrayElement::StoredCallable(
                0,
                (rank - 1) as u8
            ))
        );
        assert_eq!(dimensions, rank);
        assert_eq!(tables.callbacks[declaration].params[0].ty, Type::Record(0));
        assert_eq!(tables.callbacks[declaration].params[0].mode, Mode::Read);
        assert_eq!(
            tables.callbacks[declaration].parameter_names[0],
            if rank == 1 { "item" } else { "other" }
        );
    }
    let once = crate::type_syntax::parse("(Once<Function<int, [item: ReadOnly<Item>]>>)[]").unwrap();
    assert_eq!(
        tables
            .resolver(&names, &unions)
            .array_storage_declaration(&once)
            .unwrap()
            .unwrap()
            .0,
        Type::Array(crate::syntax::ArrayElement::StoredCallable(1, 0))
    );
    for source in ["Item[]", "() => int"] {
        let annotation = crate::type_syntax::parse(source).unwrap();
        assert!(
            tables
                .resolver(&names, &unions)
                .array_storage_declaration(&annotation)
                .unwrap()
                .is_none()
        );
    }
    let prior_callbacks = tables.callbacks.len();
    for source in ["(ReadOnly<Function<int>>)[]", "(Mutable<Function<int>>)[][]"] {
        let annotation = crate::type_syntax::parse(source).unwrap();
        let (leaf, _) = crate::callable_annotations::storage_leaf(&annotation);
        let error = tables
            .resolver(&names, &unions)
            .array_storage_declaration(&annotation)
            .unwrap_err();
        assert_eq!(error.code, "callback-escape");
        assert_eq!(error.span, leaf.span);
        assert_eq!(tables.callbacks.len(), prior_callbacks);
    }
    let invalid = crate::type_syntax::parse("((value: ReadOnly<Missing>) => int)[]").unwrap();
    let error = tables
        .resolver(&names, &unions)
        .array_storage_declaration(&invalid)
        .unwrap_err();
    assert_eq!(error.code, "type");
    assert_eq!(tables.callbacks.len(), prior_callbacks);
}

#[cfg(test)]
#[test]
fn callable_array_checked_rank_is_bounded_before_family_registration() {
    let names = HashMap::new();
    let unions = crate::unions::UnionArena::default();
    let mut tables = Tables::default();
    let mut annotation = crate::type_syntax::parse("() => int").unwrap();
    for _ in 0..128 {
        let span = annotation.span;
        annotation = crate::type_syntax::ValueAnnotation {
            kind: crate::type_syntax::ValueAnnotationKind::Array(Box::new(annotation)),
            span,
        };
    }
    let resolved = tables
        .resolver(&names, &unions)
        .array_storage_declaration(&annotation)
        .unwrap()
        .unwrap();
    assert_eq!(
        resolved.0,
        Type::Array(crate::syntax::ArrayElement::StoredCallable(0, 127))
    );
    let count = tables.callbacks.len();
    let span = annotation.span;
    annotation = crate::type_syntax::ValueAnnotation {
        kind: crate::type_syntax::ValueAnnotationKind::Array(Box::new(annotation)),
        span,
    };
    let error = tables
        .resolver(&names, &unions)
        .array_storage_declaration(&annotation)
        .unwrap_err();
    assert_eq!(error.code, "unsupported");
    assert_eq!(error.span, span);
    assert_eq!(tables.callbacks.len(), count);
}
