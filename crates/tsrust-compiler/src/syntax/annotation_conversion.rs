//! Source callback/result wrappers lower without nesting Rust calls.
use super::*;
use crate::type_syntax::{CallableOwnership, ValueAnnotation, ValueAnnotationKind as Kind};

enum Work<'a> {
    Result(&'a ResultAnnotation),
    Parameter(&'a ParameterAnnotation),
    Callable(&'a [CallableParameter], &'a ResultAnnotation, CallableOwnership, Span),
    Finish(&'a [CallableParameter], CallableOwnership, Span, usize),
}

fn callback_ownership(kind: CallbackKind) -> CallableOwnership {
    match kind {
        CallbackKind::ProgramReusable => CallableOwnership::ProgramReusable,
        CallbackKind::Program => CallableOwnership::Program,
        CallbackKind::Read => CallableOwnership::Read,
        CallbackKind::Reusable => CallableOwnership::Reusable,
        CallbackKind::Once => CallableOwnership::Once,
        CallbackKind::Owned => CallableOwnership::Owned,
        CallbackKind::OwnedOnce => CallableOwnership::OwnedOnce,
        CallbackKind::Task => CallableOwnership::Task,
        CallbackKind::SharedRead => CallableOwnership::SharedRead,
    }
}

fn convert(root: Work<'_>) -> ValueAnnotation {
    let mut pending = vec![root];
    let mut results: Vec<ValueAnnotation> = Vec::new();
    let inferred = |text: &str, span| ValueAnnotation {
        kind: Kind::Named(crate::type_syntax::TypeName { text: text.into(), span }), span,
    };
    while let Some(work) = pending.pop() {
        match work {
            Work::Result(value) => match value {
                ResultAnnotation::Inferred(span) => results.push(inferred("__inferred_arrow_result", *span)),
                ResultAnnotation::Named(value) => results.push((**value).clone()),
                ResultAnnotation::Callable { kind, params, result, span } => {
                    let ownership = match kind {
                        Some(CallbackKind::ProgramReusable) => CallableOwnership::ProgramReusable,
                        Some(CallbackKind::Program) => CallableOwnership::Program,
                        Some(CallbackKind::OwnedOnce) => CallableOwnership::OwnedOnce,
        Some(CallbackKind::Task) => CallableOwnership::Task,
                        Some(CallbackKind::Owned) => CallableOwnership::Owned,
                        Some(CallbackKind::SharedRead) => CallableOwnership::SharedRead,
                        _ => CallableOwnership::Default,
                    };
                    pending.push(Work::Callable(params, result, ownership, *span));
                }
            },
            Work::Parameter(value) => match value {
                ParameterAnnotation::Named(value) => results.push((**value).clone()),
                ParameterAnnotation::Inferred(token) => results.push(inferred("__inferred_parameter", token.span)),
                ParameterAnnotation::Callback(value) => pending.push(Work::Callable(&value.params, &value.result, callback_ownership(value.kind), value.span)),
            },
            Work::Callable(params, result, ownership, span) => {
                pending.push(Work::Finish(params, ownership, span, results.len()));
                pending.push(Work::Result(result));
                pending.extend(params.iter().rev().map(|parameter| Work::Parameter(&parameter.ty)));
            }
            Work::Finish(params, ownership, span, start) => {
                let result = Box::new(results.pop().expect("converted callable result"));
                let parameters = params.iter().zip(results.drain(start..)).map(|(parameter, annotation)| crate::type_syntax::CallableParameter {
                    name: crate::type_syntax::TypeName { text: parameter.name.text.clone(), span: parameter.name.span },
                    omission: parameter.initializer.omission(), rest: parameter.rest,
                    access: parameter.access.as_ref().map(|access| crate::type_syntax::ParameterAccess {
                        mode: match access.text.as_str() {
                            "read" => crate::Mode::Read, "mutate" => crate::Mode::Mutate,
                            "consume" => crate::Mode::Consume, _ => unreachable!("parsed parameter access"),
                        }, span: access.span,
                    }),
                    span: Span::new(parameter.name.span.start, annotation.span.end), annotation,
                }).collect();
                results.push(ValueAnnotation { kind: Kind::Callable { ownership, parameters, result }, span });
            }
        }
    }
    results.pop().expect("converted root annotation")
}

impl ResultAnnotation {
    pub(crate) fn value_annotation(&self) -> ValueAnnotation { convert(Work::Result(self)) }
}

impl CallbackAnnotation {
    pub(crate) fn value_annotation(&self) -> ValueAnnotation {
        convert(Work::Callable(&self.params, &self.result, callback_ownership(self.kind), self.span))
    }
}
