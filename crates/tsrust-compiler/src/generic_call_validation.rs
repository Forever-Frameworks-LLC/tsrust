//! Source cardinality checks do not instantiate unused generic bodies.
use crate::syntax::*;
use crate::{Diagnostic, Span};
use std::collections::{HashMap, HashSet};
#[path = "generic_call_targets.rs"]
mod targets;
use targets::Target;

#[derive(Clone)]
struct Contract {
    generic: usize,
    required: usize,
    maximum: Option<usize>,
    declaration: Span,
    alternatives: Vec<Span>,
}
#[derive(Clone)]
struct Binding {
    name: String,
    scope: Span,
    targets: Vec<Target>,
    mutable: bool,
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
        alternatives: Vec::new(),
    }
}
fn parameters(params: &[CallableParameter], span: Span) -> Contract {
    let rest = params.last().is_some_and(|parameter| parameter.rest);
    let fixed = params.len().saturating_sub(usize::from(rest));
    Contract {
        generic: 0,
        required: params[..fixed]
            .iter()
            .filter(|parameter| !parameter.initializer.omission().is_omittable())
            .count(),
        maximum: (!rest).then_some(fixed),
        declaration: span,
        alternatives: Vec::new(),
    }
}

/// Known positions remain a lower bound even around unknown spread lengths.
fn cardinality(args: &[Expr]) -> (usize, bool) {
    let mut pending = args.iter().map(|value| (value, false)).collect::<Vec<_>>();
    let mut minimum = 0;
    let mut exact = true;
    while let Some((value, spread)) = pending.pop() {
        if spread {
            if let ExprKind::Array(values) = &value.kind {
                pending.extend(values.iter().map(|value| (value, false)));
            } else {
                exact = false;
            }
        } else if let ExprKind::Unary {
            op: UnaryOp::Spread,
            value,
            ..
        } = &value.kind
        {
            pending.push((value, true));
        } else {
            minimum += 1;
        }
    }
    (minimum, exact)
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
        targets: vec![Target::Unknown],
        mutable: false,
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
                } else {
                    binding.mutable = *rebindable;
                    binding.targets = targets::value(value);
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
    initial: Vec<Target>,
    bindings: &[Binding],
    declarations: &HashMap<String, Contract>,
) -> Option<Contract> {
    let mut pending = initial;
    let mut visited = HashSet::new();
    let mut joined: Option<Contract> = None;
    while let Some(target) = pending.pop() {
        let candidate = match target {
            Target::Unknown => return None,
            Target::Contract(contract) => contract,
            Target::Name(name, span) => {
                let binding = bindings
                    .iter()
                    .enumerate()
                    .filter(|(_, binding)| {
                        binding.name == name
                            && binding.scope.start <= span.start
                            && span.end <= binding.scope.end
                    })
                    .min_by_key(|(_, binding)| binding.scope.end - binding.scope.start);
                if let Some((index, binding)) = binding {
                    if let Some(contract) = &binding.contract {
                        contract.clone()
                    } else {
                        if visited.insert(index) {
                            pending.extend(binding.targets.clone());
                        }
                        continue;
                    }
                } else {
                    declarations.get(&name)?.clone()
                }
            }
        };
        if let Some(contract) = &mut joined {
            if (contract.generic, contract.required, contract.maximum)
                != (candidate.generic, candidate.required, candidate.maximum)
            {
                return None;
            }
            for span in std::iter::once(candidate.declaration).chain(candidate.alternatives) {
                if span != contract.declaration && !contract.alternatives.contains(&span) {
                    contract.alternatives.push(span);
                }
            }
        } else {
            joined = Some(candidate);
        }
    }
    joined
}

fn related(mut error: Diagnostic, contract: &Contract) -> Diagnostic {
    error = error.related(contract.declaration, "Callable declared here.");
    for &span in &contract.alternatives {
        error = error.related(span, "Alternative callable declared here.");
    }
    error
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
    let writes = targets::writes(program)?;
    let mut writes_by_name: HashMap<&str, Vec<&targets::Write>> = HashMap::new();
    for write in &writes {
        writes_by_name.entry(&write.name).or_default().push(write);
    }
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
                    targets: vec![Target::Unknown],
                    mutable: false,
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
                    targets: vec![Target::Unknown],
                    mutable: false,
                    contract:
                        match &parameter.ty {
                            ParameterAnnotation::Callback(callback) => {
                                Some(parameters(&callback.params, parameter.name.span))
                            }
                            ParameterAnnotation::Named(annotation) => {
                                crate::callable_annotations::callback(annotation).map(|callback| {
                                    parameters(&callback.params, parameter.name.span)
                                })
                            }
                            ParameterAnnotation::Inferred(_) => None,
                        },
                }));
                collect(body, &mut bindings);
            }
            Ok(())
        })?;
        let function_scope = function.scope_span;
        let direct_writes = function
            .body
            .iter()
            .filter_map(|statement| {
                if let StmtKind::Assign { target, .. } = &statement.kind {
                    Some((target.span.start, target.span.end))
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>();
        let mut linear = function.body.iter().all(|statement| {
            matches!(
                statement.kind,
                StmtKind::Let { .. }
                    | StmtKind::Assign { .. }
                    | StmtKind::Return(_)
                    | StmtKind::Expr(_)
                    | StmtKind::Throw(_)
            )
        });
        // Nested and expression writes may execute out of source order. Keep
        // their whole-function over-approximation until structured flow exists.
        linear &= writes
            .iter()
            .filter(|write| {
                function_scope.start <= write.span.start && write.span.end <= function_scope.end
            })
            .all(|write| direct_writes.contains(&(write.span.start, write.span.end)));
        crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |expression| {
            if matches!(expression.kind, ExprKind::Closure { .. }) {
                linear = false;
            }
            Ok(())
        })?;
        crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |expression| {
            let ExprKind::Call(callee, args) = &expression.kind else {
                return Ok(());
            };
            let span = callee.span;
            let mut bindings = bindings.clone();
            for binding in &mut bindings {
                if !binding.mutable || binding.contract.is_some() {
                    continue;
                }
                for write in writes_by_name
                    .get(binding.name.as_str())
                    .into_iter()
                    .flatten()
                {
                    if function_scope.start <= write.span.start
                        && write.span.end <= function_scope.end
                        && (!linear || write.span.start <= expression.span.start)
                    {
                        binding.targets.extend(write.targets.clone());
                    }
                }
            }
            let Some(contract) = resolve(targets::value(callee), &bindings, &declarations) else {
                return Ok(());
            };
            if let Some(types) = explicit.get(&(expression.span.start, expression.span.end))
                && (contract.generic == 0 || types.len() != contract.generic)
            {
                return Err(related(
                    Diagnostic::new(
                        if contract.generic == 0 {
                            "type"
                        } else {
                            "arity"
                        },
                        if contract.generic == 0 {
                            "Explicit type arguments require a declared generic function."
                                .to_owned()
                        } else {
                            format!(
                                "Generic function expects {} type argument{}, but this call supplies {}.",
                                contract.generic,
                                if contract.generic == 1 { "" } else { "s" },
                                types.len()
                            )
                        },
                        types.first().map_or(span, |value| value.span),
                    ),
                    &contract,
                ));
            }
            let (supplied, exact) = cardinality(args);
            if (exact && supplied < contract.required)
                || contract.maximum.is_some_and(|maximum| supplied > maximum)
            {
                return Err(related(
                    Diagnostic::new(
                        "arity",
                        format!(
                            "Callable expects {}, but this call supplies {}{}.",
                            match contract.maximum {
                                Some(maximum) if maximum == contract.required => format!(
                                    "{} argument{}",
                                    maximum,
                                    if maximum == 1 { "" } else { "s" }
                                ),
                                Some(maximum) => format!(
                                    "between {} and {} arguments",
                                    contract.required, maximum
                                ),
                                None => format!(
                                    "at least {} argument{}",
                                    contract.required,
                                    if contract.required == 1 { "" } else { "s" }
                                ),
                            },
                            if exact { "" } else { "at least " },
                            supplied
                        ),
                        expression.span,
                    ),
                    &contract,
                ));
            }
            Ok(())
        })?;
    }
    Ok(())
}
