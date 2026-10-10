//! `type Name = Type;` declarations.
//!
//! An alias is another spelling of its right side. Aliases are expanded where
//! a written type is parsed, so later phases see only the expanded type.
use crate::Diagnostic;
use crate::syntax::Token;
use crate::type_syntax::{ValueAnnotation, ValueAnnotationKind as Kind};
use std::collections::HashMap;

pub(crate) type Aliases = HashMap<String, ValueAnnotation>;

/// Applies `visit` to the annotation and every annotation inside it.
fn each(annotation: &mut ValueAnnotation, visit: &mut impl FnMut(&mut ValueAnnotation)) {
    let mut pending = vec![annotation];
    while let Some(current) = pending.pop() {
        visit(current);
        match &mut current.kind {
            Kind::Named(_) | Kind::Polymorphic { .. } => {}
            Kind::Applied { arguments, .. } | Kind::Union(arguments) | Kind::Tuple(arguments) => {
                pending.extend(arguments.iter_mut())
            }
            Kind::Array(inner) | Kind::Group(inner) => pending.push(inner),
            Kind::Callable { parameters, result, .. } => {
                pending.extend(parameters.iter_mut().map(|parameter| &mut parameter.annotation));
                pending.push(result);
            }
        }
    }
}

/// Replaces every alias name with its expansion. The replacement takes the
/// span of the use, so diagnostics point at the place that names the alias.
pub(crate) fn expand(annotation: &mut ValueAnnotation, aliases: &Aliases) {
    each(annotation, &mut |current| {
        let Kind::Named(name) = &current.kind else { return };
        let Some(target) = aliases.get(&name.text) else { return };
        let span = current.span;
        let mut replacement = ValueAnnotation { kind: Kind::Group(Box::new(target.clone())), span };
        each(&mut replacement, &mut |inner| {
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
        *current = replacement;
    });
}

/// Checks names and expands aliases that mention other aliases. `imported`
/// holds aliases of other modules under their local import names. Returns
/// every usable alias and the names declared here.
pub(crate) fn resolve(raw: Vec<(Token, ValueAnnotation)>, imported: Aliases) -> Result<(Aliases, Vec<String>), Diagnostic> {
    let mut declared: HashMap<String, (Token, ValueAnnotation)> = HashMap::new();
    let mut order = Vec::new();
    for (name, annotation) in raw {
        if declared.contains_key(&name.text) || imported.contains_key(&name.text) {
            return Err(Diagnostic::new(
                "duplicate",
                format!("Type name {:?} is already declared.", name.text),
                name.span,
            ));
        }
        order.push(name.text.clone());
        declared.insert(name.text.clone(), (name, annotation));
    }
    // Each round expands the aliases whose right sides mention only resolved
    // aliases. A round without progress means the rest refer to themselves.
    let mut resolved = imported;
    let local = order.clone();
    let mut remaining = order;
    while !remaining.is_empty() {
        let mut next = Vec::new();
        for name in &remaining {
            let (_, annotation) = &declared[name];
            let mut probe = annotation.clone();
            let mut waits = false;
            each(&mut probe, &mut |current| {
                if let Kind::Named(used) = &current.kind {
                    waits |= declared.contains_key(&used.text) && !resolved.contains_key(&used.text);
                }
            });
            if waits {
                next.push(name.clone());
            } else {
                expand(&mut probe, &resolved);
                resolved.insert(name.clone(), probe);
            }
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

/// Prefix of a module-qualified name produced by project linking.
pub(crate) const LINKED_PREFIX: &str = "__tsrust_module_";

/// Rewrites the declared type names of an exported alias to their linked
/// names, so an importing module can use the expansion unchanged.
pub(crate) fn link_names(annotation: &mut ValueAnnotation, linked: &dyn Fn(&str) -> String) {
    each(annotation, &mut |current| {
        if let Kind::Named(name) | Kind::Applied { name, .. } = &mut current.kind {
            let builtin = matches!(
                name.text.as_str(),
                "int" | "number" | "boolean" | "string" | "void" | "null" | "undefined" | "Set" | "Map" | "Result"
                    | "Function" | "Mutable" | "ReadOnly" | "Once" | "Owned"
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
pub(crate) type ImportedRecords = HashMap<String, Vec<crate::syntax::RecordField>>;
