//! Bounded source annotation substitution for concrete record member discovery.
use super::SourceIndex;
use std::collections::{BTreeMap, BTreeSet};
use tsrust_compiler::type_syntax::{parse, ValueAnnotation, ValueAnnotationKind as Kind};

pub(super) fn canonical(source: &str) -> Option<String> {
    fn render(value: &ValueAnnotation) -> Option<String> {
        Some(match &value.kind {
            Kind::Literal(value) => value.text(),
            Kind::Named(name) => name.text.clone(),
            Kind::Group(inner) => render(inner)?,
            Kind::Applied { name, arguments } => format!(
                "{}<{}>",
                name.text,
                arguments
                    .iter()
                    .map(render)
                    .collect::<Option<Vec<_>>>()?
                    .join(",")
            ),
            Kind::Array(inner) => {
                let element = render(inner)?;
                if element.contains('|') && !matches!(inner.kind, Kind::Applied { .. }) {
                    format!("({element})[]")
                } else {
                    format!("{element}[]")
                }
            }
            Kind::Union(members) => {
                let mut parts = Vec::new();
                fn collect(value: &ValueAnnotation, parts: &mut Vec<String>) -> Option<()> {
                    match &value.kind {
                        Kind::Group(inner) => collect(inner, parts)?,
                        Kind::Union(members) => {
                            for member in members {
                                collect(member, parts)?;
                            }
                        }
                        _ => parts.push(render(value)?),
                    }
                    Some(())
                }
                for member in members {
                    collect(member, &mut parts)?;
                }
                parts.sort_by_key(|part| match part.as_str() {
                    "null" => 1,
                    "undefined" => 2,
                    _ => 0,
                });
                parts.dedup();
                parts.join("|")
            }
            Kind::Tuple(positions) => format!("[{}]",positions.iter().map(render).collect::<Option<Vec<_>>>()?.join(",")),
            Kind::Callable { .. } | Kind::Polymorphic { .. } => return None,
        })
    }
    render(&parse(source).ok()?)
}

fn applications(value: &ValueAnnotation, source: &str, output: &mut Vec<String>) {
    match &value.kind {
        Kind::Applied { arguments, .. } => {
            output.push(source[value.span.start..value.span.end].to_owned());
            for argument in arguments {
                applications(argument, source, output);
            }
        }
        Kind::Array(inner) | Kind::Group(inner) => applications(inner, source, output),
        Kind::Union(members) | Kind::Tuple(members) => {
            for member in members {
                applications(member, source, output);
            }
        }
        Kind::Callable {
            parameters, result, ..
        } => {
            for parameter in parameters {
                applications(&parameter.annotation, source, output);
            }
            applications(result, source, output);
        }
        Kind::Literal(_) | Kind::Named(_) | Kind::Polymorphic { .. } => {}
    }
}
fn substitute(
    value: &ValueAnnotation,
    bindings: &BTreeMap<String, String>,
    edits: &mut Vec<(usize, usize, String)>,
) {
    match &value.kind {
        Kind::Literal(_) | Kind::Polymorphic { .. } => {},
        Kind::Named(name) => {
            if let Some(argument) = bindings.get(&name.text) {
                let replacement =
                    if parse(argument).is_ok_and(|value| matches!(value.kind, Kind::Union(_))) {
                        format!("({argument})")
                    } else {
                        argument.clone()
                    };
                edits.push((name.span.start, name.span.end, replacement));
            }
        }
        Kind::Applied { arguments, .. } | Kind::Union(arguments) | Kind::Tuple(arguments) => {
            for argument in arguments {
                substitute(argument, bindings, edits);
            }
        }
        Kind::Array(inner) | Kind::Group(inner) => substitute(inner, bindings, edits),
        Kind::Callable {
            parameters, result, ..
        } => {
            for parameter in parameters {
                if let Some(access) = &parameter.access
                    && let Kind::Named(name) = &parameter.annotation.kind
                    && let Some(concrete) = bindings.get(&name.text)
                    && parse(concrete).is_ok_and(|value| copy_annotation(&value))
                {
                    edits.push((access.span.start, parameter.span.end, concrete.clone()));
                    continue;
                }
                substitute(&parameter.annotation, bindings, edits);
            }
            substitute(result, bindings, edits);
        }
    }
}
fn copy_annotation(value: &ValueAnnotation) -> bool {
    match &value.kind {
        Kind::Literal(value) => !matches!(value,tsrust_compiler::type_syntax::LiteralAnnotation::String(_)),
        Kind::Named(name) => matches!(name.text.as_str(), "int" | "number" | "boolean" | "null" | "undefined"),
        Kind::Group(inner) => copy_annotation(inner),
        Kind::Union(members) => members.iter().all(copy_annotation),
        _ => false,
    }
}
impl SourceIndex {
    pub(super) fn materialize_generic_fields(&mut self) {
        if self.generic_parameters.is_empty() {
            return;
        }
        let mut pending: Vec<String> = self
            .symbols
            .iter()
            .map(|symbol| symbol.ty.clone())
            .collect();
        // Direct factory calls have no local variable carrying their concrete
        // result. Their checked expression types still need concrete members.
        pending.extend(self.checked_method_results.iter().map(|(_, ty)| ty.clone()));
        let mut visited = self
            .symbols
            .iter()
            .filter_map(|symbol| symbol.owner.as_ref())
            .filter(|owner| owner.contains('<'))
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut count = 0;
        let mut counts = BTreeMap::<String, usize>::new();
        while let Some(source) = pending.pop() {
            if source.len() > 65536 {
                continue;
            }
            let Ok(annotation) = parse(&source) else {
                continue;
            };
            let mut instances = Vec::new();
            applications(&annotation, &source, &mut instances);
            for instance in instances {
                let owner = super::canonical_data_type(&instance);
                if !visited.insert(owner.clone()) {
                    continue;
                }
                let Ok(annotation) = parse(&instance) else {
                    continue;
                };
                let Kind::Applied { name, arguments } = annotation.kind else {
                    continue;
                };
                let Some(parameters) = self.generic_parameters.get(&name.text) else {
                    continue;
                };
                if parameters.len() != arguments.len() {
                    continue;
                }
                // Match the compiler program budget without advertising unchecked expansions.
                if count >= 1024 {
                    return;
                }
                let declaration_count = counts.entry(name.text.clone()).or_default();
                if *declaration_count >= 128 {
                    continue;
                }
                *declaration_count += 1;
                count += 1;
                let bindings = parameters
                    .iter()
                    .cloned()
                    .zip(arguments.iter().map(|argument| {
                        instance[argument.span.start..argument.span.end].to_owned()
                    }))
                    .collect();
                let fields: Vec<_> = self
                    .symbols
                    .iter()
                    .enumerate()
                    .filter(|(_, symbol)| symbol.owner.as_deref() == Some(name.text.as_str()))
                    .map(|(id, symbol)| (id, symbol.clone()))
                    .collect();
                for (original, mut field) in fields {
                    let Ok(annotation) = parse(&field.ty) else {
                        continue;
                    };
                    let mut edits = Vec::new();
                    substitute(&annotation, &bindings, &mut edits);
                    edits.sort_by_key(|edit| std::cmp::Reverse(edit.0));
                    if field.ty.len() + edits.iter().map(|edit| edit.2.len()).sum::<usize>() > 65536
                    {
                        continue;
                    }
                    for (start, end, replacement) in edits {
                        field.ty.replace_range(start..end, &replacement);
                    }
                    field.ty = super::canonical_data_type(&field.ty);
                    pending.push(field.ty.clone());
                    let mut display = field.ty.clone();
                    let mut names = self.generic_display_names.iter().collect::<Vec<_>>();
                    names.sort_by_key(|(identity, _)| std::cmp::Reverse(identity.len()));
                    for (identity, label) in names {
                        display = display.replace(identity, label);
                    }
                    field.signature = format!("{}: {}", field.name, display);
                    field.owner = Some(owner.clone());
                    if let Some(definition) = self.foreign_definitions.get(&original).cloned() {
                        self.foreign_definitions
                            .insert(self.symbols.len(), definition);
                    }
                    self.symbols.push(field);
                }
            }
        }
    }
}

#[cfg(test)]
mod literal_tests {
    use super::*;
    #[test]
    fn literal_annotation_spelling_and_substitution_preserve_values() {
        assert_eq!(canonical("Result<'ready', false>"),Some("Result<\"ready\",false>".into()));
        assert_eq!(canonical("9007199254740993 | -9223372036854775808"),Some("9007199254740993|-9223372036854775808".into()));
        let source="Box<'T' | T>";let annotation=parse(source).unwrap();let mut edits=Vec::new();
        substitute(&annotation,&BTreeMap::from([("T".into(),"7".into())]),&mut edits);
        assert_eq!(edits.len(),1);assert_eq!(&source[edits[0].0..edits[0].1],"T");
        assert_eq!(edits[0].2,"7");
        assert!(!copy_annotation(&parse("'owned text'").unwrap()));
        assert!(copy_annotation(&parse("7 | false").unwrap()));
    }
}
