//! Source cardinality checks do not instantiate unused generic bodies.
use crate::syntax::*;
use crate::{Diagnostic, Span};
use std::collections::{HashMap, HashSet};
#[path = "generic_call_branch_flow.rs"]
mod branch_flow;
#[path = "generic_call_effect_bodies.rs"]
mod effect_bodies;
#[path = "generic_call_capture_modes.rs"]
mod capture_modes;
#[path = "generic_call_body_origins.rs"]
mod body_origins;
#[path = "generic_call_linear_flow.rs"]
mod linear_flow;
#[path = "generic_call_targets.rs"]
mod targets;
use targets::Target;
#[path = "generic_call_contract_sets.rs"]
mod contract_sets;
use contract_sets::{resolve_all, resolve_indexed};

#[derive(Clone, PartialEq, Eq, Hash)]
struct Contract {
    generic: usize,
    required: usize,
    maximum: Option<usize>,
    declaration: Span,
    alternatives: Vec<Span>,
}
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
struct Selection {
    truth: Option<bool>,
    nullish: Option<bool>,
    undefined: Option<bool>,
}
#[derive(Clone, PartialEq, Eq, Hash)]
struct Element {
    alternatives: Option<Vec<branch_flow::environment_store::Alternative>>,
    origins: Vec<body_origins::Origin>,
    contracts: Option<Vec<Contract>>,
    selection: Selection,
}
#[derive(Clone, PartialEq, Eq, Hash)]
struct Binding {
    alternatives: Option<Vec<branch_flow::environment_store::Alternative>>,
    identity: Option<crate::local_capture_plan::BindingId>,
    name: String,
    scope: Span,
    targets: Vec<Target>,
    origins: Vec<body_origins::Origin>,
    selection: Selection,
    elements: Option<Vec<Element>>,
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

fn collect(body: &[Stmt], bindings: &mut Vec<Binding>, effects: &effect_bodies::Index) {
    let mut scopes = Vec::new();
    crate::generic_functions::locals(body, &mut scopes);
    bindings.extend(scopes.into_iter().map(|(name, scope)| Binding { alternatives: None,
        identity: effects.in_scope(&name, scope),
        name,
        scope,
        targets: vec![Target::Unknown],
        origins: vec![body_origins::Origin::Unknown],
        selection: Selection::default(), elements: None,
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
                binding.origins = body_origins::value(value);
                binding.mutable = *rebindable;
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
/// Legacy conservative paths retain their common-shape summary.
fn resolve(
    initial: Vec<Target>,
    bindings: &[Binding],
    declarations: &HashMap<String, Contract>,
    effects: &effect_bodies::Index,
) -> Option<Contract> {
    let candidates = resolve_indexed(initial, bindings, declarations, effects)?;
    let mut joined: Option<Contract> = None;
    for candidate in candidates {
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
    let effects = effect_bodies::Index::new(program)?;
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
                Binding { alternatives: None,
                    identity: effects.declared(name.span),
                    name: name.text.clone(),
                    scope: function.scope_span,
                    targets: vec![Target::Unknown],
                    origins: vec![body_origins::Origin::Unknown],
        selection: Selection::default(), elements: None,
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
        collect(&function.body, &mut bindings, &effects);
        crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |expression| {
            if let ExprKind::Closure { params, body, .. } = &expression.kind {
                bindings.extend(params.iter().map(|parameter| Binding { alternatives: None,
                    identity: effects.declared(parameter.name.span),
                    name: parameter.name.text.clone(),
                    scope: expression.span,
                    targets: vec![Target::Unknown],
                    origins: vec![body_origins::Origin::Unknown],
        selection: Selection::default(), elements: None,
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
                collect(body, &mut bindings, &effects);
            }
            Ok(())
        })?;
        let branch_facts = branch_flow::analyze(function, &bindings, &writes, &declarations, &effects)?;
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
            let resolved = if let Some(contract) = branch_facts
                .as_ref()
                .and_then(|facts| facts.get(&(expression.span.start, expression.span.end)))
            {
                contract.clone()
            } else {
                let mut bindings = if linear {
                    linear_flow::at_call(
                        &bindings,
                        &writes,
                        function_scope,
                        expression.span.start,
                        &declarations,
                        &effects,
                    )
                } else {
                    bindings.clone()
                };
                for binding in &mut bindings {
                    if linear {
                        continue;
                    }
                    if !binding.mutable {
                        continue;
                    }
                    for write in writes_by_name
                        .get(binding.name.as_str())
                        .into_iter()
                        .flatten()
                    {
                        if function_scope.start <= write.span.start
                            && write.span.end <= function_scope.end
                            && effects.binding(&write.name, write.span).is_none_or(|identity| binding.identity == Some(identity))
                            && (!linear || write.span.start <= expression.span.start)
                        {
                            binding.origins.extend(write.origins.clone());
                            binding.origins = body_origins::canonical(std::mem::take(&mut binding.origins));
                            if binding.contract.is_none() { binding.targets.extend(write.targets.clone()); }
                        }
                    }
                }
                resolve(targets::value(callee), &bindings, &declarations, &effects)
                    .map(|contract| vec![contract])
            };
            let Some(contracts) = resolved else {
                return Ok(());
            };
            for contract in contracts {
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
            }
            Ok(())
        })?;
    }
    Ok(())
}
