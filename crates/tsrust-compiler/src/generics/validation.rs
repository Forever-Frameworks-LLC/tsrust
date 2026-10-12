//! Lexical annotation validation independent of concrete function instantiation.
use crate::syntax::Function;
use crate::type_syntax::{ValueAnnotation, ValueAnnotationKind as Kind};
use crate::Diagnostic;
use std::collections::{BTreeMap, BTreeSet, HashMap};

fn annotation(
    value: &ValueAnnotation,
    variables: &BTreeSet<String>,
    records: &BTreeMap<String, usize>,
) -> Result<(), Diagnostic> {
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        match &value.kind {
            Kind::Polymorphic { .. } => {},
            Kind::Named(name) => {
                if variables.contains(&name.text)
                    || matches!(
                        name.text.as_str(),
                        "int" | "number" | "boolean" | "string" | "null" | "undefined" | "void"
                    )
                {
                    continue;
                }
                match records.get(&name.text) {
                    Some(0) => {}
                    Some(_) => {
                        return Err(Diagnostic::new(
                            "arity",
                            "Generic record types require explicit type arguments.",
                            name.span,
                        ))
                    }
                    None => {
                        return Err(Diagnostic::new(
                            "type",
                            "Unbound or unknown type in generic function annotation.",
                            name.span,
                        ))
                    }
                }
            }
            Kind::Applied { name, arguments } => {
                if name.text == "Result" {
                    if arguments.len() != 2 { return Err(Diagnostic::new("arity", "Result requires success and error types.", value.span)); }
                    pending.extend(arguments); continue;
                }
                if name.text == "Map" {
                    if arguments.len() != 2 { return Err(Diagnostic::new("arity", "Map requires key and value types.", value.span)); }
                    pending.extend(arguments); continue;
                }
                if name.text == "Set" {
                    if arguments.len() != 1 { return Err(Diagnostic::new("arity", "Set requires one key type.", value.span)); }
                    pending.extend(arguments);
                    continue;
                }
                if name.text == "@testing:element" {
                    if arguments.len() != 1 { return Err(Diagnostic::new("arity", "Internal containment element projection requires one type argument.", value.span)); }
                    pending.extend(arguments);
                    continue;
                }
                if variables.contains(&name.text) {
                    return Err(Diagnostic::new(
                        "type",
                        "A type parameter cannot be used as a generic type constructor.",
                        name.span,
                    ));
                }
                let Some(arity) = records.get(&name.text) else {
                    return Err(Diagnostic::new(
                        "type",
                        "Unknown generic record type in function annotation.",
                        name.span,
                    ));
                };
                if *arity == 0 || *arity != arguments.len() {
                    return Err(Diagnostic::new(
                        "arity",
                        "Wrong number of generic record type arguments.",
                        value.span,
                    ));
                }
                pending.extend(arguments);
            }
            Kind::Group(inner) | Kind::Array(inner) => pending.push(inner),
            Kind::Union(members) | Kind::Tuple(members) => pending.extend(members),
            Kind::Callable {
                parameters, result, ..
            } => {
                let mut names = HashMap::new();
                for parameter in parameters {
                    if let Some(prior) =
                        names.insert(parameter.name.text.as_str(), parameter.name.span)
                    {
                        return Err(Diagnostic::new(
                            "duplicate",
                            "Callable parameter is already defined.",
                            parameter.name.span,
                        )
                        .related(prior, "Previous parameter declared here."));
                    }
                    pending.push(&parameter.annotation);
                }
                pending.push(result);
            }
        }
    }
    Ok(())
}

pub(crate) fn function(
    function: &mut Function,
    records: &BTreeMap<String, usize>,
    calls: &[(crate::Span, Vec<ValueAnnotation>)],
) -> Result<(), Diagnostic> {
    let variables = function
        .type_parameters
        .iter()
        .map(|parameter| parameter.text.clone())
        .collect();
    let mut names = HashMap::new();
    for (name, _, _) in &function.params {
        if let Some(prior) = names.insert(name.text.as_str(), name.span) {
            return Err(Diagnostic::new(
                "duplicate",
                "Function parameter is already defined.",
                name.span,
            )
            .related(prior, "Previous parameter declared here."));
        }
    }
    let mut validate = |value: &mut ValueAnnotation| annotation(value, &variables, records);
    crate::source_walk::function_annotations(function, &mut validate)?;
    crate::source_walk::function_body(function, &mut validate, &mut |_| Ok(()))?;
    let end = function.scope_span.end;
    for (span, arguments) in calls {
        if function.name.span.start <= span.start && span.end <= end {
            for value in arguments {
                annotation(value, &variables, records)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[test]
fn private_containment_projection_validates_its_operand_and_arity() {
    let variables=BTreeSet::from(["T".to_owned()]);let records=BTreeMap::new();
    let mut valid=crate::type_syntax::parse("ElementOf<T>").unwrap();
    let Kind::Applied{name,..}=&mut valid.kind else {panic!("projection")};name.text="@testing:element".into();
    annotation(&valid,&variables,&records).unwrap();
    let mut unknown=crate::type_syntax::parse("ElementOf<Missing>").unwrap();
    let Kind::Applied{name,..}=&mut unknown.kind else {panic!("projection")};name.text="@testing:element".into();
    assert_eq!(annotation(&unknown,&variables,&records).unwrap_err().code,"type");
    let mut invalid=crate::type_syntax::parse("ElementOf<T,T>").unwrap();
    let Kind::Applied{name,..}=&mut invalid.kind else {panic!("projection")};name.text="@testing:element".into();
    assert_eq!(annotation(&invalid,&variables,&records).unwrap_err().code,"arity");
    let ordinary=crate::type_syntax::parse("ElementOf<T>").unwrap();
    assert_eq!(annotation(&ordinary,&variables,&records).unwrap_err().code,"type");
}
