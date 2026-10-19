//! `type Name = Type;` declarations.
//!
//! An alias is another spelling of its right side. Aliases are expanded where
//! a written type is parsed, so later phases see only the expanded type.
use crate::Diagnostic;
use crate::syntax::Token;
use crate::type_syntax::{ValueAnnotation, ValueAnnotationKind as Kind};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub(crate) struct Parameter {
    pub name: crate::type_syntax::TypeName,
    pub default: Option<ValueAnnotation>,
}
#[derive(Clone, Debug)]
pub(crate) struct Alias {
    pub span: crate::Span,
    pub parameters: Vec<Parameter>,
    pub annotation: ValueAnnotation,
}
pub(crate) type Aliases = HashMap<String, Alias>;
const EXPANSION_LIMIT: usize = 65536;

fn each(annotation: &mut ValueAnnotation, visit: &mut impl FnMut(&mut ValueAnnotation)) {
    let mut pending = vec![annotation];
    while let Some(current) = pending.pop() {
        visit(current);
        push_children(current, &mut pending);
    }
}
fn push_children<'a>(current: &'a mut ValueAnnotation, pending: &mut Vec<&'a mut ValueAnnotation>) {
    match &mut current.kind {
        Kind::Literal(_) | Kind::Named(_) | Kind::Polymorphic { .. } => {}
        Kind::Applied { arguments, .. } | Kind::Union(arguments) | Kind::Tuple(arguments) => {
            pending.extend(arguments.iter_mut())
        }
        Kind::Array(inner) | Kind::Group(inner) => pending.push(inner),
        Kind::Callable {
            parameters, result, ..
        } => {
            pending.extend(parameters.iter_mut().map(|p| &mut p.annotation));
            pending.push(result);
        }
    }
}
fn size(value: &ValueAnnotation) -> usize {
    let mut pending = vec![value];
    let mut count = 0;
    while let Some(value) = pending.pop() {
        count += 1;
        match &value.kind {
            Kind::Literal(_) | Kind::Named(_) | Kind::Polymorphic { .. } => {}
            Kind::Applied { arguments, .. } | Kind::Union(arguments) | Kind::Tuple(arguments) => {
                pending.extend(arguments)
            }
            Kind::Array(value) | Kind::Group(value) => pending.push(value),
            Kind::Callable {
                parameters, result, ..
            } => {
                pending.extend(parameters.iter().map(|p| &p.annotation));
                pending.push(result);
            }
        }
    }
    count
}
fn limit(span: crate::Span) -> Diagnostic {
    Diagnostic::new(
        "unsupported",
        "Type alias expansion exceeds 65536 annotation nodes.",
        span,
    )
}
fn substitute(
    value: &mut ValueAnnotation,
    bindings: &HashMap<String, ValueAnnotation>,
    budget: &mut usize,
) -> Result<(), Diagnostic> {
    let mut required = 0usize;
    each(value, &mut |node| {
        if let Kind::Named(name) = &node.kind
            && let Some(argument) = bindings.get(&name.text)
        {
            required = required.saturating_add(size(argument));
        }
    });
    *budget = budget
        .checked_sub(required)
        .ok_or_else(|| limit(value.span))?;
    crate::type_substitution::apply(value, bindings)
}
/// Expand aliases with the same parameter substitution used by all source contexts.
pub(crate) fn expand(
    annotation: &mut ValueAnnotation,
    aliases: &Aliases,
) -> Result<(), Diagnostic> {
    expand_scoped(annotation, aliases, &std::collections::HashSet::new())
}
fn expand_scoped(
    annotation: &mut ValueAnnotation,
    aliases: &Aliases,
    protected: &std::collections::HashSet<&str>,
) -> Result<(), Diagnostic> {
    let mut budget = EXPANSION_LIMIT;
    let mut pending = vec![annotation];
    while let Some(current) = pending.pop() {
        budget = budget.checked_sub(1).ok_or_else(|| limit(current.span))?;
        loop {
            let (name, written) = match &current.kind {
                Kind::Named(name) => (name, &[][..]),
                Kind::Applied { name, arguments } => (name, arguments.as_slice()),
                _ => break,
            };
            if protected.contains(name.text.as_str()) {
                break;
            }
            let Some(alias) = aliases.get(&name.text) else {
                break;
            };
            let span = current.span;
            if written.len() > alias.parameters.len() {
                return Err(Diagnostic::new(
                    "arity",
                    format!(
                        "Type alias {:?} expects at most {} type arguments, received {}.",
                        name.text,
                        alias.parameters.len(),
                        written.len()
                    ),
                    span,
                ));
            }
            let mut bindings = HashMap::new();
            for (index, parameter) in alias.parameters.iter().enumerate() {
                let value = if let Some(value) = written.get(index) {
                    value.clone()
                } else if let Some(default) = &parameter.default {
                    let mut value = default.clone();
                    substitute(&mut value, &bindings, &mut budget)?;
                    value
                } else {
                    return Err(Diagnostic::new(
                        "arity",
                        format!(
                            "Type alias {:?} is missing type argument {:?}.",
                            name.text, parameter.name.text
                        ),
                        span,
                    ));
                };
                bindings.insert(parameter.name.text.clone(), value);
            }
            budget = budget
                .checked_sub(size(&alias.annotation))
                .ok_or_else(|| limit(span))?;
            let mut replacement = alias.annotation.clone();
            substitute(&mut replacement, &bindings, &mut budget)?;
            respan(&mut replacement, span);
            *current = ValueAnnotation {
                kind: Kind::Group(Box::new(replacement)),
                span,
            };
        }
        push_children(current, &mut pending);
    }
    Ok(())
}
/// Resolve dependencies once, preserving parameters as lexical names rather than globals.
pub(crate) fn resolve(
    raw: Vec<(Token, Alias)>,
    imported: Aliases,
) -> Result<(Aliases, Vec<String>), Diagnostic> {
    let mut declared = HashMap::new();
    let mut order = Vec::new();
    for (name, alias) in raw {
        if declared.contains_key(&name.text) || imported.contains_key(&name.text) {
            return Err(Diagnostic::new(
                "duplicate",
                format!("Type name {:?} is already declared.", name.text),
                name.span,
            ));
        }
        order.push(name.text.clone());
        declared.insert(name.text.clone(), (name, alias));
    }
    let local = order.clone();
    let mut remaining = order;
    let mut resolved = imported;
    while !remaining.is_empty() {
        let mut next = Vec::new();
        for name in &remaining {
            let (_, alias) = &declared[name];
            let protected = alias
                .parameters
                .iter()
                .map(|p| p.name.text.as_str())
                .collect::<std::collections::HashSet<_>>();
            let mut waits = false;
            let mut probe = alias.clone();
            for annotation in std::iter::once(&mut probe.annotation).chain(
                probe
                    .parameters
                    .iter_mut()
                    .filter_map(|p| p.default.as_mut()),
            ) {
                each(annotation, &mut |value| {
                    if let Kind::Named(used) | Kind::Applied { name: used, .. } = &value.kind {
                        waits |= !protected.contains(used.text.as_str())
                            && declared.contains_key(&used.text)
                            && !resolved.contains_key(&used.text);
                    }
                });
            }
            if waits {
                next.push(name.clone());
                continue;
            }
            expand_scoped(&mut probe.annotation, &resolved, &protected)?;
            for parameter in &mut probe.parameters {
                if let Some(value) = &mut parameter.default {
                    expand_scoped(value, &resolved, &protected)?;
                }
            }
            resolved.insert(name.clone(), probe);
        }
        if next.len() == remaining.len() {
            let (name, _) = &declared[&next[0]];
            return Err(Diagnostic::new(
                "recursive-alias",
                format!(
                    "Type alias {:?} refers to itself. Declare a record type for a recursive shape.",
                    name.text
                ),
                name.span,
            ));
        }
        remaining = next;
    }
    Ok((resolved, local))
}

pub(crate) fn link_alias(alias: &mut Alias, linked: &dyn Fn(&str) -> String) {
    let protected = alias
        .parameters
        .iter()
        .map(|p| p.name.text.clone())
        .collect::<std::collections::HashSet<_>>();
    for annotation in std::iter::once(&mut alias.annotation).chain(
        alias
            .parameters
            .iter_mut()
            .filter_map(|p| p.default.as_mut()),
    ) {
        link_names(annotation, &|name| {
            if protected.contains(name) {
                name.to_owned()
            } else {
                linked(name)
            }
        });
    }
}

/// Prefix of a module-qualified name produced by project linking.
pub(crate) const LINKED_PREFIX: &str = "__tsrust_module_";

/// Rewrites the declared type names of an exported alias to their linked
/// names, so an importing module can use the expansion unchanged.
pub(crate) fn link_names(annotation: &mut ValueAnnotation, linked: &dyn Fn(&str) -> String) {
    each(annotation, &mut |current| {
        if let Kind::Named(name) | Kind::Applied { name, .. } = &mut current.kind {
            let builtin = matches!(
                name.text.as_str(),
                "int"
                    | "number"
                    | "boolean"
                    | "string"
                    | "void"
                    | "null"
                    | "undefined"
                    | "Set"
                    | "Map"
                    | "Result"
                    | "Function"
                    | "Mutable"
                    | "ReadOnly"
                    | "Once"
                    | "Owned"
            );
            if !builtin && !name.text.starts_with(LINKED_PREFIX) {
                name.text = linked(&name.text);
            }
        }
    });
}

/// Gives the annotation and everything inside it one span, the place in the
/// current file that brought it in.
pub(crate) fn respan(annotation: &mut ValueAnnotation, span: crate::Span) {
    each(annotation, &mut |inner| {
        inner.span = span;
        match &mut inner.kind {
            Kind::Named(name) | Kind::Applied { name, .. } => name.span = span,
            Kind::Callable { parameters, .. } => {
                for parameter in parameters {
                    parameter.span = span;
                }
            }
            _ => {}
        }
    });
}

/// Fields of record types that a module imports, under their local names,
/// with every type name already in linked form.
pub(crate) type ImportedRecords = HashMap<
    String,
    (
        Vec<crate::type_syntax::TypeName>,
        Vec<crate::syntax::RecordField>,
    ),
>;
