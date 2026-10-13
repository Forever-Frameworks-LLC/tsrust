//! Source callable annotations clone and release through explicit worklists.
use super::{CallableParameter, CallbackAnnotation, ParameterAnnotation, ResultAnnotation};
use crate::Span;

enum Work<'a> {
    Result(&'a ResultAnnotation),
    Callback(&'a CallbackAnnotation),
    Parameter(&'a ParameterAnnotation),
    FinishResult(&'a ResultAnnotation, usize),
    FinishCallback(&'a CallbackAnnotation, usize),
    AsParameter,
}
enum Built { Result(ResultAnnotation), Callback(CallbackAnnotation), Parameter(ParameterAnnotation) }

fn parameters(source: &[CallableParameter], values: impl Iterator<Item = Built>) -> Vec<CallableParameter> {
    source.iter().zip(values).map(|(parameter, value)| {
        let Built::Parameter(ty) = value else { unreachable!() };
        CallableParameter { rest: parameter.rest, name: parameter.name.clone(), ty,
            access: parameter.access.clone(), initializer: parameter.initializer.clone() }
    }).collect()
}
fn clone_annotation(root: Work<'_>) -> Built {
    let mut pending = vec![root];
    let mut built = Vec::new();
    while let Some(work) = pending.pop() {
        match work {
            Work::Result(value) => match value {
                ResultAnnotation::Inferred(span) => built.push(Built::Result(ResultAnnotation::Inferred(*span))),
                ResultAnnotation::Named(value) => built.push(Built::Result(ResultAnnotation::Named(value.clone()))),
                ResultAnnotation::Callable { params, result, .. } => {
                    pending.push(Work::FinishResult(value, built.len()));
                    pending.push(Work::Result(result));
                    pending.extend(params.iter().rev().map(|parameter| Work::Parameter(&parameter.ty)));
                }
            },
            Work::Callback(value) => {
                pending.push(Work::FinishCallback(value, built.len()));
                pending.push(Work::Result(&value.result));
                pending.extend(value.params.iter().rev().map(|parameter| Work::Parameter(&parameter.ty)));
            }
            Work::Parameter(value) => match value {
                ParameterAnnotation::Named(value) => built.push(Built::Parameter(ParameterAnnotation::Named(value.clone()))),
                ParameterAnnotation::Inferred(value) => built.push(Built::Parameter(ParameterAnnotation::Inferred(value.clone()))),
                ParameterAnnotation::Callback(value) => {
                    pending.push(Work::AsParameter);
                    pending.push(Work::Callback(value));
                }
            },
            Work::FinishResult(value, start) => {
                let ResultAnnotation::Callable { kind, params, span, .. } = value else { unreachable!() };
                let Built::Result(result) = built.pop().unwrap() else { unreachable!() };
                let params = parameters(params, built.drain(start..));
                built.push(Built::Result(ResultAnnotation::Callable { kind: *kind, params, result: Box::new(result), span: *span }));
            }
            Work::FinishCallback(value, start) => {
                let Built::Result(result) = built.pop().unwrap() else { unreachable!() };
                let params = parameters(&value.params, built.drain(start..));
                built.push(Built::Callback(CallbackAnnotation { kind: value.kind, params, result: Box::new(result), span: value.span }));
            }
            Work::AsParameter => {
                let Built::Callback(value) = built.pop().unwrap() else { unreachable!() };
                built.push(Built::Parameter(ParameterAnnotation::Callback(Box::new(value))));
            }
        }
    }
    built.pop().unwrap()
}
impl Clone for CallbackAnnotation {
    fn clone(&self) -> Self {
        let Built::Callback(value) = clone_annotation(Work::Callback(self)) else { unreachable!() };
        value
    }
}
impl Clone for ResultAnnotation {
    fn clone(&self) -> Self {
        let Built::Result(value) = clone_annotation(Work::Result(self)) else { unreachable!() };
        value
    }
}

// Default initializer expressions remain owned by their parameters. Their AST
// storage is a separate conversion; these worklists flatten annotation edges.
enum Owned { Result(ResultAnnotation), Callback(CallbackAnnotation) }
fn detach(params: &mut Vec<CallableParameter>, result: &mut ResultAnnotation, pending: &mut Vec<Owned>) {
    for parameter in std::mem::take(params) {
        if let ParameterAnnotation::Callback(value) = parameter.ty { pending.push(Owned::Callback(*value)); }
    }
    if matches!(result, ResultAnnotation::Callable { .. }) {
        pending.push(Owned::Result(std::mem::replace(result, ResultAnnotation::Inferred(Span::default()))));
    }
}
fn release(mut pending: Vec<Owned>) {
    while let Some(mut value) = pending.pop() {
        match &mut value {
            Owned::Result(ResultAnnotation::Callable { params, result, .. }) => detach(params, result, &mut pending),
            Owned::Callback(value) => detach(&mut value.params, &mut value.result, &mut pending),
            _ => {},
        }
    }
}
impl Drop for CallbackAnnotation {
    fn drop(&mut self) {
        let mut pending = Vec::new();
        detach(&mut self.params, &mut self.result, &mut pending);
        release(pending);
    }
}
impl Drop for ResultAnnotation {
    fn drop(&mut self) {
        if let Self::Callable { params, result, .. } = self {
            let mut pending = Vec::new();
            detach(params, result, &mut pending);
            release(pending);
        }
    }
}
impl CallbackAnnotation {
    pub(crate) fn into_parts(mut self) -> (super::CallbackKind, Vec<CallableParameter>, Box<ResultAnnotation>, Span) {
        (self.kind, std::mem::take(&mut self.params), std::mem::replace(&mut self.result, Box::new(ResultAnnotation::Inferred(self.span))), self.span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::{CallbackKind, Expr, ExprKind, ParameterInitializer, Token, TokenKind};

    #[test]
    fn wrapper_clone_retains_defaults_access_tokens_and_inferred_locations() {
        let span = Span::new(12, 19);
        let token = |text: &str| Token { text: text.into(), span, kind: TokenKind::Name };
        let source = CallbackAnnotation {
            kind: CallbackKind::OwnedOnce,
            params: vec![CallableParameter {
                rest: true, name: token("input"), access: Some(token("consume")),
                ty: ParameterAnnotation::Inferred(token("unknown")),
                initializer: ParameterInitializer::Default { value: Box::new(Expr::new(ExprKind::Integer("7".into()), span)), span },
            }],
            result: Box::new(ResultAnnotation::Inferred(span)), span,
        };
        let copied = source.clone();
        assert_eq!(format!("{source:?}"), format!("{copied:?}"));
        let (kind, params, result, location) = copied.into_parts();
        assert_eq!(kind, CallbackKind::OwnedOnce);
        assert_eq!(location, span);
        assert!(params[0].rest);
        assert!(matches!(&params[0].initializer, ParameterInitializer::Default { value, span: location } if *location == span && matches!(&value.kind, ExprKind::Integer(text) if text == "7")));
        assert!(matches!(*result, ResultAnnotation::Inferred(location) if location == span));
    }

    #[test]
    fn parameter_only_callback_chain_clones_and_drops_on_normal_stack() {
        let span = Span::new(3, 9);
        let mut source = ParameterAnnotation::Inferred(Token { text: "input".into(), span, kind: TokenKind::Name });
        for _ in 0..16_384 {
            source = ParameterAnnotation::Callback(Box::new(CallbackAnnotation {
                kind: CallbackKind::Read, span, result: Box::new(ResultAnnotation::Inferred(span)),
                params: vec![CallableParameter { rest: false, name: Token { text: "nested".into(), span, kind: TokenKind::Name }, ty: source, access: None, initializer: ParameterInitializer::Optional(span) }],
            }));
        }
        let copied = source.clone();
        let mut current = &copied;
        let mut levels = 0;
        while let ParameterAnnotation::Callback(value) = current {
            assert_eq!(value.kind, CallbackKind::Read);
            assert_eq!(value.span, span);
            assert!(matches!(*value.result, ResultAnnotation::Inferred(location) if location == span));
            assert!(matches!(value.params[0].initializer, ParameterInitializer::Optional(location) if location == span));
            current = &value.params[0].ty;
            levels += 1;
        }
        assert_eq!(levels, 16_384);
        assert!(matches!(current, ParameterAnnotation::Inferred(token) if token.text == "input" && token.span == span));
        drop(copied);
        drop(source);
    }
}
