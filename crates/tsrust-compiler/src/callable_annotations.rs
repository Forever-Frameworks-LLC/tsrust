//! Adapt structured callable syntax to existing source-aware checking contracts.
use crate::syntax::{
    CallableParameter, CallbackAnnotation, CallbackKind, ParameterAnnotation, ResultAnnotation,
    Token, TokenKind,
};
use crate::type_syntax::{CallableOwnership, ValueAnnotation, ValueAnnotationKind as Kind};

fn ungroup(mut annotation: &ValueAnnotation) -> &ValueAnnotation {
    while let Kind::Group(inner) = &annotation.kind {
        annotation = inner;
    }
    annotation
}

pub(crate) fn ownership(annotation: &ValueAnnotation) -> Option<CallbackKind> {
    let Kind::Callable { ownership, .. } = &ungroup(annotation).kind else {
        return None;
    };
    Some(match ownership {
        CallableOwnership::Default => CallbackKind::Owned,
        CallableOwnership::Read => CallbackKind::Read,
        CallableOwnership::Reusable => CallbackKind::Reusable,
        CallableOwnership::Once => CallbackKind::Once,
        CallableOwnership::Owned => CallbackKind::Owned,
        CallableOwnership::OwnedOnce => CallbackKind::OwnedOnce,
    })
}

/// Separate array/group wrappers without traversing callable arguments or results.
/// Dimensions count containers around the leaf, not arrays inside its signature.
pub(crate) fn storage_leaf(mut annotation: &ValueAnnotation) -> (&ValueAnnotation, usize) {
    let mut dimensions = 0;
    loop {
        match &annotation.kind {
            Kind::Group(inner) => annotation = inner,
            Kind::Array(inner) => {
                dimensions += 1;
                annotation = inner;
            }
            _ => return (annotation, dimensions),
        }
    }
}

/// A union or group whose members include a callable signature.
pub(crate) fn nullable_callable(annotation: &ValueAnnotation) -> bool {
    match &ungroup(annotation).kind {
        Kind::Union(members) => members.iter().any(|member| {
            matches!(&ungroup(member).kind, Kind::Callable { .. }) || nullable_callable(member)
        }),
        _ => false,
    }
}

pub(crate) fn callback(annotation: &ValueAnnotation) -> Option<CallbackAnnotation> {
    let (inner, dimensions) = storage_leaf(annotation);
    if dimensions != 0 {
        return None;
    }
    let Kind::Callable {
        ownership,
        parameters,
        result,
    } = &inner.kind
    else {
        return None;
    };
    Some(CallbackAnnotation {
        kind: match ownership {
            CallableOwnership::Default => CallbackKind::Owned,
            CallableOwnership::Read => CallbackKind::Read,
            CallableOwnership::Reusable => CallbackKind::Reusable,
            CallableOwnership::Once => CallbackKind::Once,
            CallableOwnership::Owned => CallbackKind::Owned,
            CallableOwnership::OwnedOnce => CallbackKind::OwnedOnce,
        },
        params: parameters
            .iter()
            .map(|parameter| CallableParameter {
                rest: parameter.rest,
                name: Token {
                    text: parameter.name.text.clone(),
                    span: parameter.name.span,
                    kind: TokenKind::Name,
                },
                ty: match callback(&parameter.annotation) {
                    Some(value) => ParameterAnnotation::Callback(Box::new(value)),
                    None => ParameterAnnotation::Named(Box::new(parameter.annotation.clone())),
                },
                initializer: if parameter.omission.is_omittable() { crate::syntax::ParameterInitializer::Optional(parameter.span) } else { crate::syntax::ParameterInitializer::Required },
                access: parameter.access.as_ref().map(|access| Token {
                    text: access.mode.as_str().to_owned(),
                    span: access.span,
                    kind: TokenKind::Name,
                }),
            })
            .collect(),
        result: Box::new(result_annotation(result)),
        span: annotation.span,
    })
}

pub(crate) fn result_annotation(annotation: &ValueAnnotation) -> ResultAnnotation {
    // Bare results infer invocation. Explicit once results retain a consuming
    // contract; borrowed result forms are rejected by resolution.
    if matches!(
        &ungroup(annotation).kind,
        Kind::Callable {
            ownership: CallableOwnership::Default
                | CallableOwnership::Owned
                | CallableOwnership::OwnedOnce,
            ..
        }
    ) {
        let value = callback(annotation).expect("callable annotation");
        ResultAnnotation::Callable {
            kind: if matches!(
                &ungroup(annotation).kind,
                Kind::Callable {
                    ownership: CallableOwnership::Default,
                    ..
                }
            ) {
                None
            } else {
                Some(value.kind)
            },
            params: value.params,
            result: value.result,
            span: annotation.span,
        }
    } else {
        ResultAnnotation::Named(Box::new(annotation.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grouping_preserves_checked_access_and_generated_native_code() {
        let plain = "function apply(action: Function<int, [value: int]>): int { return action(4); } function main() { console.log(apply((value: int): int => { return value + 1; })); }";
        let grouped = plain.replace(
            "action: Function<int, [value: int]>",
            "action: (((Function<int, [value: int]>)))",
        );
        // Equal-length spelling keeps runtime diagnostic byte offsets equal too.
        let plain = plain.replace(
            "action: Function<int, [value: int]>",
            "action: Function<int, [value: int]>      ",
        );
        let plain = crate::check(&plain).unwrap_or_else(|error| panic!("{error:?}"));
        let grouped = crate::check(&grouped).unwrap_or_else(|error| panic!("{error:?}"));
        assert_eq!(plain.contracts(), grouped.contracts());
        assert_eq!(plain.emit_rust(), grouped.emit_rust());
    }

    #[test]
    fn structured_adapter_preserves_names_access_ranges_and_ownership() {
        let source = "((Once<Function<string, [value: Owned<string>, inner: ((Function<int>))]>>))";
        let annotation = crate::type_syntax::parse(source).unwrap();
        let value = callback(&annotation).unwrap();
        assert_eq!(value.kind, CallbackKind::OwnedOnce);
        assert_eq!(value.span, annotation.span);
        let parameter = &value.params[0];
        assert_eq!(parameter.name.text, "value");
        assert_eq!(
            &source[parameter.name.span.start..parameter.name.span.end],
            "value"
        );
        let access = parameter.access.as_ref().unwrap();
        assert_eq!(access.text, "consume");
        assert_eq!(&source[access.span.start..access.span.end], "Owned");
        let ParameterAnnotation::Callback(child) = &value.params[1].ty else {
            panic!("nested callback");
        };
        assert_eq!(child.kind, CallbackKind::Owned);
        assert!(matches!(*child.result, ResultAnnotation::Named(_)));
    }

    #[test]
    fn data_unions_inside_callable_contracts_are_not_callable_unions() {
        let value =
            crate::type_syntax::parse("(((value: int | null) => string | undefined))").unwrap();
        assert!(callback(&value).is_some());
        let nullable = crate::type_syntax::parse("((() => int) | null)").unwrap();
        assert!(
            callback(&nullable).is_none(),
            "nullable transport is distinct from a callable returning nullable data"
        );
    }
}

/// Ownership of the single callable payload in a checked nullable annotation.
pub(crate) fn nullable_kind(annotation: &ValueAnnotation) -> Option<CallbackKind> {
    match &ungroup(annotation).kind {
        Kind::Union(members) => members.iter().find_map(ownership),
        _ => None,
    }
}

#[cfg(test)]
mod storage_leaf_tests {
    use super::*;
    #[test]
    fn callable_storage_wrappers_preserve_leaf_contract_and_container_rank() {
        for (source, dimensions, kind) in [
            ("((() => int))", 0, CallbackKind::Owned),
            ("((() => int)[])[]", 2, CallbackKind::Owned),
            (
                "(Once<Function<string, [values: ReadOnly<int[]>]>>)[]",
                1,
                CallbackKind::OwnedOnce,
            ),
            ("(Mutable<Function<int>>)[]", 1, CallbackKind::Reusable),
        ] {
            let annotation = crate::type_syntax::parse(source).unwrap();
            let (leaf, rank) = storage_leaf(&annotation);
            assert_eq!(rank, dimensions);
            assert_eq!(callback(leaf).unwrap().kind, kind);
            assert_eq!(callback(&annotation).is_some(), dimensions == 0);
            assert!(leaf.span.start >= annotation.span.start);
            assert!(leaf.span.end <= annotation.span.end);
        }
        let annotation = crate::type_syntax::parse("((() => int) | null)[]").unwrap();
        let (leaf, rank) = storage_leaf(&annotation);
        assert_eq!(rank, 1);
        assert!(callback(leaf).is_none());
    }
}
