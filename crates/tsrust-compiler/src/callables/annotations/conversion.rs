//! Structured callable signatures adapt through explicit construction frames.
use super::*;

enum Work<'a> {
    Result(&'a ValueAnnotation),
    Parameter(&'a ValueAnnotation),
    Callback(&'a ValueAnnotation),
    Finish(&'a ValueAnnotation, &'a [crate::type_syntax::CallableParameter], usize),
    AsResult(Option<CallbackKind>, crate::Span),
    AsParameter,
}
enum Converted { Result(ResultAnnotation), Parameter(ParameterAnnotation), Callback(CallbackAnnotation) }

fn convert(root: Work<'_>) -> Converted {
    let mut pending = vec![root];
    let mut results = Vec::new();
    while let Some(work) = pending.pop() {
        match work {
            Work::Result(annotation) => {
                // Borrowed result wrappers remain named and retain the existing
                // resolver rejection; bare results still infer invocation mode.
                if matches!(&ungroup(annotation).kind, Kind::Callable { ownership: CallableOwnership::Default | CallableOwnership::Owned | CallableOwnership::OwnedOnce | CallableOwnership::SharedRead, .. }) {
                    let kind = match &ungroup(annotation).kind {
                        Kind::Callable { ownership: CallableOwnership::Default, .. } => None,
                        _ => ownership(annotation),
                    };
                    pending.push(Work::AsResult(kind, annotation.span));
                    pending.push(Work::Callback(annotation));
                } else { results.push(Converted::Result(ResultAnnotation::Named(Box::new(annotation.clone())))); }
            }
            Work::Parameter(annotation) => {
                let (inner, dimensions) = storage_leaf(annotation);
                if dimensions == 0 && matches!(inner.kind, Kind::Callable { .. }) {
                    pending.push(Work::AsParameter);
                    pending.push(Work::Callback(annotation));
                } else { results.push(Converted::Parameter(ParameterAnnotation::Named(Box::new(annotation.clone())))); }
            }
            Work::Callback(annotation) => {
                let Kind::Callable { parameters, result, .. } = &ungroup(annotation).kind else { unreachable!("callable conversion entry") };
                pending.push(Work::Finish(annotation, parameters, results.len()));
                pending.push(Work::Result(result));
                pending.extend(parameters.iter().rev().map(|parameter| Work::Parameter(&parameter.annotation)));
            }
            Work::Finish(annotation, parameters, start) => {
                let Converted::Result(result) = results.pop().expect("converted callable result") else { unreachable!() };
                let params = parameters.iter().zip(results.drain(start..)).map(|(parameter, converted)| {
                    let Converted::Parameter(ty) = converted else { unreachable!("converted parameter") };
                    CallableParameter {
                        rest: parameter.rest,
                        name: Token { text: parameter.name.text.clone(), span: parameter.name.span, kind: TokenKind::Name },
                        ty,
                        initializer: if parameter.omission.is_omittable() { crate::syntax::ParameterInitializer::Optional(parameter.span) } else { crate::syntax::ParameterInitializer::Required },
                        access: parameter.access.as_ref().map(|access| Token { text: access.mode.as_str().into(), span: access.span, kind: TokenKind::Name }),
                    }
                }).collect();
                results.push(Converted::Callback(CallbackAnnotation {
                    kind: ownership(annotation).expect("callable ownership"), params, result: Box::new(result), span: annotation.span,
                }));
            }
            Work::AsResult(kind, span) => {
                let Converted::Callback(value) = results.pop().expect("converted callback") else { unreachable!() };
                results.push(Converted::Result(ResultAnnotation::Callable { kind, params: value.params, result: value.result, span }));
            }
            Work::AsParameter => {
                let Converted::Callback(value) = results.pop().expect("converted callback") else { unreachable!() };
                results.push(Converted::Parameter(ParameterAnnotation::Callback(Box::new(value))));
            }
        }
    }
    results.pop().expect("converted root annotation")
}

pub(super) fn callback(annotation: &ValueAnnotation) -> CallbackAnnotation {
    let Converted::Callback(value) = convert(Work::Callback(annotation)) else { unreachable!() };
    value
}

pub(super) fn result(annotation: &ValueAnnotation) -> ResultAnnotation {
    let Converted::Result(value) = convert(Work::Result(annotation)) else { unreachable!() };
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Span, type_syntax::TypeName};

    // Source wrapper disposal is still a separate conversion. Do not let this
    // isolated adapter witness claim that source AST storage is already safe.
    fn dispose(root: ResultAnnotation) {
        let mut pending = vec![root];
        while let Some(value) = pending.pop() {
            if let ResultAnnotation::Callable { params, result, .. } = value {
                pending.push(*result);
                for parameter in params {
                    assert!(matches!(parameter.initializer, crate::syntax::ParameterInitializer::Required));
                    if let ParameterAnnotation::Callback(callback) = parameter.ty {
                        pending.push(ResultAnnotation::Callable { kind: Some(callback.kind), params: callback.params, result: callback.result, span: callback.span });
                    }
                }
            }
        }
    }

    #[test]
    fn deep_bidirectional_annotation_conversion_preserves_owned_signatures() {
        let span = Span::new(11, 19);
        let leaf = || ValueAnnotation { kind: Kind::Named(TypeName { text: "int".into(), span }), span };
        let mut annotation = leaf();
        for depth in 0..16_384 {
            let (parameters, result) = if depth % 2 == 0 { (vec![], Box::new(annotation)) } else {
                (vec![crate::type_syntax::CallableParameter {
                    rest: false, name: TypeName { text: "input".into(), span }, access: None,
                    omission: crate::ParameterOmission::Required, annotation, span,
                }], Box::new(leaf()))
            };
            annotation = ValueAnnotation { kind: Kind::Callable { ownership: CallableOwnership::OwnedOnce, parameters, result }, span };
        }
        let source = result(&annotation);
        let round_trip = source.value_annotation();
        assert!(round_trip == annotation);
        dispose(source);
        let source = callback(&annotation);
        assert!(source.value_annotation() == annotation);
        dispose(ResultAnnotation::Callable { kind: Some(source.kind), params: source.params, result: source.result, span: source.span });
        // Structured annotations use the production stack-safe destructor.
        drop(annotation); drop(round_trip);
    }

    #[test]
    fn deeply_grouped_nullable_callable_detection_keeps_container_boundaries() {
        let mut annotation = crate::type_syntax::parse("Function<int>|null").unwrap();
        for _ in 0..16_384 {
            let span = annotation.span;
            annotation = ValueAnnotation { kind: Kind::Union(vec![ValueAnnotation { kind: Kind::Group(Box::new(annotation)), span }]), span };
        }
        assert!(super::super::nullable_callable(&annotation));
        let span = annotation.span;
        let array = ValueAnnotation { kind: Kind::Array(Box::new(annotation)), span };
        assert!(!super::super::nullable_callable(&array));
        assert!(!super::super::nullable_callable(&crate::type_syntax::parse("Function<int>").unwrap()));
    }

    #[test]
    fn result_modes_and_parameter_metadata_retain_existing_conversion_rules() {
        for (text, kind) in [("Function<int>", None), ("Owned<Function<int>>", Some(CallbackKind::Owned)), ("Once<Function<int>>", Some(CallbackKind::OwnedOnce))] {
            let annotation = crate::type_syntax::parse(text).unwrap();
            let source = result(&annotation);
            let ResultAnnotation::Callable { kind: actual, .. } = &source else { panic!("callable result") };
            assert_eq!(*actual, kind);
            assert_eq!(source.value_annotation(), annotation);
        }
        for text in ["ReadOnly<Function<int>>", "Mutable<Function<int>>"] {
            let annotation = crate::type_syntax::parse(text).unwrap();
            assert!(matches!(result(&annotation), ResultAnnotation::Named(_)));
        }
        let annotation = crate::type_syntax::parse("Once<Function<int,[head:Owned<string>,tail?:ReadOnly<int[]>]>>").unwrap();
        let source = callback(&annotation);
        let mut expected = annotation.clone();
        let Kind::Callable { parameters, .. } = &mut expected.kind else { unreachable!() };
        // The existing source wrapper range ends at the payload, excluding
        // the closing delimiter of a parameter access wrapper.
        for parameter in parameters { parameter.span.end = parameter.annotation.span.end; }
        assert_eq!(source.value_annotation(), expected);
        assert_eq!(source.params[0].access.as_ref().unwrap().text, "consume");
        assert_eq!(source.params[1].access.as_ref().unwrap().text, "read");
        assert!(matches!(source.params[1].initializer, crate::syntax::ParameterInitializer::Optional(_)));
    }
}
