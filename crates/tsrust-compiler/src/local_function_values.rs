//! A nested function that uses variables of its enclosing function has no
//! standalone callable form: every call passes the captured variables as
//! hidden arguments. A use of such a function as a value therefore becomes the
//! arrow function `(parameters) => name(parameters)` before analysis. The arrow
//! follows the ordinary closure rules: it borrows the captured variables for as
//! long as the value lives, and the escape pass sees real closure syntax.
use crate::syntax::*;
use crate::{Diagnostic, Mode, Span};
use std::collections::HashMap;

/// Suffix the lexical resolver leaves on a local function name in value position.
pub(crate) const VALUE_MARK: &str = "#value";

struct Header {
    params: Vec<(Token, ParameterAnnotation)>,
    access: Vec<Option<(Mode, Span)>>,
}

fn header(function: &Function) -> Option<Header> {
    // These shapes keep the diagnostic of the direct function value.
    if function.rest_parameter
        || !function.type_parameters.is_empty()
        || function
            .parameter_initializers
            .iter()
            .any(|initializer| !matches!(initializer, ParameterInitializer::Required))
    {
        return None;
    }
    Some(Header {
        params: function
            .params
            .iter()
            .map(|(name, annotation, callback)| {
                let annotation = match callback {
                    Some(callback) => ParameterAnnotation::Callback(Box::new(callback.clone())),
                    None => ParameterAnnotation::Named(Box::new(annotation.clone())),
                };
                (name.clone(), annotation)
            })
            .collect(),
        access: function.parameter_access.clone(),
    })
}

fn wrapper(emitted: &str, header: &Header, id: usize, span: Span) -> Expr {
    // Parameter names cannot be written in source, so they never hide a
    // variable that the called function captures.
    let names: Vec<String> = header
        .params
        .iter()
        .enumerate()
        .map(|(index, (name, _))| format!("{}#{index}", name.text))
        .collect();
    let params = header
        .params
        .iter()
        .zip(&names)
        .enumerate()
        .map(|(index, ((name, annotation), text))| CallableParameter {
            rest: false,
            name: Token { text: text.clone(), span, kind: name.kind },
            ty: annotation.clone(),
            access: header.access.get(index).copied().flatten().map(|(mode, _)| Token {
                text: mode.as_str().to_owned(),
                span,
                kind: TokenKind::Name,
            }),
            initializer: ParameterInitializer::Required,
        })
        .collect();
    let arguments = names
        .into_iter()
        .map(|name| Expr::new(ExprKind::Name(name), span))
        .collect();
    let call = Expr::new(
        ExprKind::Call(Box::new(Expr::new(ExprKind::Name(emitted.to_owned()), span)), arguments),
        span,
    );
    let body = vec![Stmt {
        labels: Vec::new(),
        control_target: None,
        kind: StmtKind::Return(Some(call)),
        span,
    }];
    Expr::new(
        ExprKind::Closure { id, params, result: ResultAnnotation::Inferred(span), body },
        span,
    )
}

/// Replaces marked value uses. `capturing` names the emitted functions that
/// need captured variables, directly or through a function they call.
pub(crate) fn lower(
    functions: &mut [Function],
    lifted: &mut [Function],
    capturing: &dyn Fn(&str) -> bool,
) -> Result<(), Diagnostic> {
    let headers: HashMap<String, Header> = lifted
        .iter()
        .filter(|function| capturing(&function.name.text))
        .filter_map(|function| Some((function.name.text.clone(), header(function)?)))
        .collect();
    let mut next = 0;
    for function in functions.iter_mut().chain(lifted.iter_mut()) {
        crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |expression| {
            if let ExprKind::Closure { id, .. } = &expression.kind {
                next = next.max(*id + 1);
            }
            Ok(())
        })?;
    }
    for function in functions.iter_mut().chain(lifted.iter_mut()) {
        crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |expression| {
            let ExprKind::Name(name) = &mut expression.kind else {
                return Ok(());
            };
            let Some(emitted) = name.strip_suffix(VALUE_MARK) else {
                return Ok(());
            };
            match headers.get(emitted) {
                Some(header) => {
                    *expression = wrapper(emitted, header, next, expression.span);
                    next += 1;
                }
                None => name.truncate(emitted.len()),
            }
            Ok(())
        })?;
    }
    Ok(())
}
