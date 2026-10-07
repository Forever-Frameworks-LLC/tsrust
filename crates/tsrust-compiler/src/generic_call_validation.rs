//! Source cardinality checks do not instantiate unused generic bodies.
use crate::syntax::*;
use crate::{Diagnostic, Span};
use std::collections::HashMap;

#[derive(Clone)]
struct Contract {
    generic: usize,
    required: usize,
    maximum: Option<usize>,
    declaration: Span,
}
struct Binding {
    name: String,
    scope: Span,
    alias: Option<(String, Span)>,
    contract: Option<Contract>,
}

fn contract(
    generic: usize,
    initializers: &[ParameterInitializer],
    rest: bool,
    span: Span,
) -> Contract {
    let fixed = initializers.len().saturating_sub(usize::from(rest));
    Contract {
        generic,
        required: initializers[..fixed]
            .iter()
            .filter(|value| !value.omission().is_omittable())
            .count(),
        maximum: (!rest).then_some(fixed),
        declaration: span,
    }
}
fn name(mut value: &Expr) -> Option<(&str, Span)> {
    loop {
        match &value.kind {
            ExprKind::Name(name) => {
                return Some((
                    name.strip_suffix(crate::local_function_values::VALUE_MARK)
                        .unwrap_or(name),
                    value.span,
                ));
            }
            ExprKind::Unary {
                op: UnaryOp::OptionalBoundary | UnaryOp::OptionalSelected,
                value: inner,
                ..
            }
            | ExprKind::OptionalChain(inner) => value = inner,
            _ => return None,
        }
    }
}

fn collect(body: &[Stmt], bindings: &mut Vec<Binding>) {
    let mut scopes = Vec::new();
    crate::generic_functions::locals(body, &mut scopes);
    bindings.extend(scopes.into_iter().map(|(name, scope)| Binding {
        name,
        scope,
        alias: None,
        contract: None,
    }));
    let mut pending = body.iter().collect::<Vec<_>>();
    while let Some(statement) = pending.pop() {
        match &statement.kind {
            StmtKind::Let {
                name: local,
                value,
                annotation,
                rebindable,
                ..
            } => {
                let Some(binding) = bindings.iter_mut().find(|binding| {
                    binding.name == local.text && binding.scope.start == value.span.end
                }) else {
                    continue;
                };
                if let Some(callback) = annotation
                    .as_ref()
                    .and_then(crate::callable_annotations::callback)
                {
                    binding.contract = Some(contract(
                        0,
                        &callback
                            .params
                            .iter()
                            .map(|parameter| parameter.initializer.clone())
                            .collect::<Vec<_>>(),
                        callback
                            .params
                            .last()
                            .is_some_and(|parameter| parameter.rest),
                        local.span,
                    ));
                } else if !rebindable {
                    if let ExprKind::Closure { params, body, .. } = &value.kind {
                        let forwarded = if let [statement] = body.as_slice()
                            && let StmtKind::Return(Some(call)) = &statement.kind
                            && let ExprKind::Call(callee, _) = &call.kind
                            && call.span == value.span
                            && callee.span == value.span
                        {
                            name(callee).map(|(name, span)| (name.to_owned(), span))
                        } else {
                            None
                        };
                        if let Some(alias) = forwarded {
                            binding.alias = Some(alias);
                        } else {
                            binding.contract = Some(contract(
                                0,
                                &params
                                    .iter()
                                    .map(|parameter| parameter.initializer.clone())
                                    .collect::<Vec<_>>(),
                                params.last().is_some_and(|parameter| parameter.rest),
                                local.span,
                            ));
                        }
                    } else {
                        binding.alias = name(value).map(|(name, span)| (name.to_owned(), span));
                    }
                }
            }
            StmtKind::Block(body) | StmtKind::While { body, .. } | StmtKind::ForOf { body, .. } => {
                pending.extend(body)
            }
            StmtKind::If {
                then_body,
                else_body,
                ..
            } => pending.extend(then_body.iter().chain(else_body)),
            StmtKind::Loop { body, update, .. } => pending.extend(body.iter().chain(update)),
            StmtKind::Try(region) => pending.extend(region.statements()),
            _ => {}
        }
    }
}
fn resolve(
    name: &str,
    span: Span,
    bindings: &[Binding],
    declarations: &HashMap<String, Contract>,
) -> Option<Contract> {
    let mut name = name;
    let mut span = span;
    for _ in 0..=bindings.len() {
        let binding = bindings
            .iter()
            .filter(|binding| {
                binding.name == name
                    && binding.scope.start <= span.start
                    && span.end <= binding.scope.end
            })
            .min_by_key(|binding| binding.scope.end - binding.scope.start);
        let Some(binding) = binding else {
            return declarations.get(name).cloned();
        };
        if let Some(contract) = &binding.contract {
            return Some(contract.clone());
        }
        let Some((target, location)) = &binding.alias else {
            return None;
        };
        name = target;
        span = *location;
    }
    None
}
pub(crate) fn validate(program: &mut Program) -> Result<(), Diagnostic> {
    let declarations = program
        .functions
        .iter()
        .map(|function| {
            (
                function.name.text.clone(),
                contract(
                    function.type_parameters.len(),
                    &function.parameter_initializers,
                    function.rest_parameter,
                    function.name.span,
                ),
            )
        })
        .collect::<HashMap<_, _>>();
    let explicit = program
        .call_type_arguments
        .iter()
        .map(|(span, args)| ((span.start, span.end), args.clone()))
        .collect::<HashMap<_, _>>();
    for function in &mut program.functions {
        if function.type_parameters.is_empty() {
            continue;
        }
        let mut bindings = function
            .params
            .iter()
            .map(|(name, annotation, callback)| {
                let callback = callback
                    .clone()
                    .or_else(|| crate::callable_annotations::callback(annotation));
                Binding {
                    name: name.text.clone(),
                    scope: function.scope_span,
                    alias: None,
                    contract: callback.map(|callback| {
                        contract(
                            0,
                            &callback
                                .params
                                .iter()
                                .map(|parameter| parameter.initializer.clone())
                                .collect::<Vec<_>>(),
                            callback
                                .params
                                .last()
                                .is_some_and(|parameter| parameter.rest),
                            name.span,
                        )
                    }),
                }
            })
            .collect::<Vec<_>>();
        collect(&function.body, &mut bindings);
        crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |expression| {
            if let ExprKind::Closure { params, body, .. } = &expression.kind {
                bindings.extend(params.iter().map(|parameter| Binding {
                    name: parameter.name.text.clone(),
                    scope: expression.span,
                    alias: None,
                    contract: None,
                }));
                collect(body, &mut bindings);
            }
            Ok(())
        })?;
        crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |expression| {
            let ExprKind::Call(callee, args) = &expression.kind else {
                return Ok(());
            };
            let Some((name, span)) = name(callee) else {
                return Ok(());
            };
            let Some(contract) = resolve(name, span, &bindings, &declarations) else {
                return Ok(());
            };
            if let Some(types) = explicit.get(&(expression.span.start, expression.span.end))
                && (contract.generic == 0 || types.len() != contract.generic)
            {
                return Err(Diagnostic::new(
                    if contract.generic == 0 {
                        "type"
                    } else {
                        "arity"
                    },
                    if contract.generic == 0 {
                        "Explicit type arguments require a declared generic function."
                    } else {
                        "Wrong number of generic function type arguments."
                    },
                    types.first().map_or(span, |value| value.span),
                )
                .related(contract.declaration, "Callable declared here."));
            }
            // A dynamic spread has an unknown cardinality at declaration time.
            let mut supplied = 0;
            for argument in args {
                if let ExprKind::Unary {
                    op: UnaryOp::Spread,
                    value,
                    ..
                } = &argument.kind
                {
                    let ExprKind::Array(values) = &value.kind else {
                        return Ok(());
                    };
                    if values.iter().any(|value| {
                        matches!(
                            value.kind,
                            ExprKind::Unary {
                                op: UnaryOp::Spread,
                                ..
                            }
                        )
                    }) {
                        return Ok(());
                    }
                    supplied += values.len();
                } else {
                    supplied += 1
                }
            }
            if supplied < contract.required
                || contract.maximum.is_some_and(|maximum| supplied > maximum)
            {
                return Err(Diagnostic::new(
                    "arity",
                    "Argument count does not match the declared callable contract.",
                    expression.span,
                )
                .related(contract.declaration, "Callable declared here."));
            }
            Ok(())
        })?;
    }
    Ok(())
}
