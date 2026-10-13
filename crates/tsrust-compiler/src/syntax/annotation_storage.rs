//! Source callable annotations release through explicit worklists.
use super::{CallableParameter, CallbackAnnotation, ParameterAnnotation, ResultAnnotation};
use crate::Span;

// AST disposal owns expression edges; this worklist releases annotation edges.
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
