//! Source field contracts do not certify concrete environment storage.
use crate::Diagnostic;
use crate::callable_resolution::Resolver;
use crate::syntax::Type;
use crate::type_syntax::ValueAnnotation;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FieldContract {
    Owned { ty: Type, declaration: usize },
    Scoped { ty: Type, declaration: usize },
}

pub(crate) fn resolve(
    resolver: &mut Resolver<'_>,
    annotation: &ValueAnnotation,
) -> Result<Option<FieldContract>, Diagnostic> {
    let kind = crate::callable_annotations::ownership(annotation)
        .or_else(|| crate::callable_annotations::nullable_kind(annotation));
    if kind.is_some_and(|kind| !kind.is_owned()) {
        let ty = resolver.resolve_value_annotation(annotation, false)?;
        let Type::Callback(declaration) = resolver.unions.payload(ty).unwrap_or(ty) else {
            return Err(Diagnostic::new(
                "callable-layout",
                "A scoped field requires one callable payload contract.",
                annotation.span,
            ));
        };
        return Ok(Some(FieldContract::Scoped { ty, declaration }));
    }
    resolver
        .storage_declaration(annotation)
        .map(|field| field.map(|(ty, declaration)| FieldContract::Owned { ty, declaration }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::callable_resolution::Tables;
    use crate::syntax::CallbackKind;
    use std::collections::HashMap;

    #[test]
    fn scoped_field_contract_keeps_access_labels_and_never_certifies_owned_storage() {
        let names = HashMap::new();
        let unions = crate::unions::UnionArena::default();
        let mut tables = Tables::default();
        for (source, kind) in [
            ("ReadOnly<Function<int, [value: int]>>", CallbackKind::Read),
            (
                "Mutable<Function<int, [amount: int]>>",
                CallbackKind::Reusable,
            ),
            (
                "(ReadOnly<Function<int, [other: int]>> | null)",
                CallbackKind::Read,
            ),
        ] {
            let annotation = crate::type_syntax::parse(source).unwrap();
            let contract = resolve(&mut tables.resolver(&names, &unions), &annotation).unwrap();
            let Some(FieldContract::Scoped { ty, declaration }) = contract else {
                panic!("expected scoped field contract: {contract:?}")
            };
            assert_eq!(tables.callbacks[declaration].kind, kind);
            assert!(matches!(
                unions.payload(ty).unwrap_or(ty),
                Type::Callback(_)
            ));
            assert!(
                crate::callable_storage::StorageContract::checked(
                    &tables.shapes,
                    tables.callbacks[declaration].structural
                )
                .is_none()
            );
            let error = tables
                .resolver(&names, &unions)
                .storage_declaration(&annotation)
                .unwrap_err();
            assert_eq!(error.code, "callback-escape");
        }
        assert_eq!(tables.callbacks[0].parameter_names, ["value"]);
        assert_eq!(tables.callbacks[2].parameter_names, ["other"]);
        assert_ne!(
            tables.callbacks[0].parameter_names,
            tables.callbacks[2].parameter_names
        );
    }

    #[test]
    fn owning_fields_keep_existing_storage_identity_and_data_is_not_callable() {
        let names = HashMap::new();
        let unions = crate::unions::UnionArena::default();
        let mut tables = Tables::default();
        for source in ["Function<int, [value: int]>", "Once<Function<int>>"] {
            let annotation = crate::type_syntax::parse(source).unwrap();
            let expected = tables
                .resolver(&names, &unions)
                .storage_declaration(&annotation)
                .unwrap()
                .unwrap();
            assert_eq!(
                resolve(&mut tables.resolver(&names, &unions), &annotation).unwrap(),
                Some(FieldContract::Owned {
                    ty: expected.0,
                    declaration: expected.1
                })
            );
        }
        assert_eq!(
            resolve(
                &mut tables.resolver(&names, &unions),
                &crate::type_syntax::parse("int").unwrap()
            )
            .unwrap(),
            None
        );
    }
}
