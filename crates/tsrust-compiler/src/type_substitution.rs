//! Structured lexical substitution shared by generic declarations.
use crate::type_syntax::{ValueAnnotation, ValueAnnotationKind as Kind};
use crate::Diagnostic;
use std::collections::HashMap;

/// Arguments are already concrete. A replacement is not substituted again.
/// Replacement coordinates remain those of the argument that supplied it.
pub(crate) fn apply(
    value: &mut ValueAnnotation,
    bindings: &HashMap<String, ValueAnnotation>,
) -> Result<(), Diagnostic> {
    if bindings.is_empty() {
        return Ok(());
    }
    fn visit(
        value: &mut ValueAnnotation,
        bindings: &HashMap<String, ValueAnnotation>,
        depth: usize,
    ) -> Result<(), Diagnostic> {
        if depth > 128 {
            return Err(Diagnostic::new(
                "generic-limit",
                "Substituted type nesting exceeds 128.",
                value.span,
            ));
        }
        match &mut value.kind {
            Kind::Named(name) => {
                if let Some(argument) = bindings.get(&name.text) {
                    validate_depth(argument, depth)?;
                    *value = argument.clone();
                }
            }
            Kind::Group(inner) | Kind::Array(inner) => visit(inner, bindings, depth + 1)?,
            Kind::Union(members) | Kind::Tuple(members) => {
                for member in members {
                    visit(member, bindings, depth + 1)?;
                }
            }
            Kind::Applied { name, arguments } => {
                if bindings.contains_key(&name.text) {
                    return Err(Diagnostic::new(
                        "type",
                        "A type parameter cannot be used as a generic type constructor.",
                        name.span,
                    ));
                }
                for argument in arguments.iter_mut() {
                    visit(argument, bindings, depth + 1)?;
                }
                if name.text == "@testing:element" {
                    if arguments.len() != 1 {
                        return Err(Diagnostic::new("arity", "Internal containment element projection requires one type argument.", value.span));
                    }
                    *value = containment_element(&arguments[0]);
                }
            }
            Kind::Callable {
                parameters, result, ..
            } => {
                for parameter in parameters {
                    let generic_access = matches!(&parameter.annotation.kind, Kind::Named(name) if bindings.contains_key(&name.text))
                        || matches!(&parameter.annotation.kind, Kind::Applied { name, .. } if name.text == "@testing:element");
                    visit(&mut parameter.annotation, bindings, depth + 1)?;
                    if generic_access
                        && copy_annotation(&parameter.annotation)
                    {
                        // Ownership applies to the generic data case. A concrete
                        // scalar specialization copies its argument instead.
                        parameter.access = None;
                    }
                }
                visit(result, bindings, depth + 1)?;
            }
        }
        Ok(())
    }
    fn containment_element(value: &ValueAnnotation) -> ValueAnnotation {
        match &value.kind {
            Kind::Group(inner) => containment_element(inner),
            Kind::Array(inner) => (**inner).clone(),
            Kind::Tuple(members) if members.len() == 1 => members[0].clone(),
            Kind::Tuple(members) if !members.is_empty() => ValueAnnotation { kind: Kind::Union(members.clone()), span: value.span },
            Kind::Union(members) => ValueAnnotation { kind: Kind::Union(members.iter().map(containment_element).collect()), span: value.span },
            _ => value.clone(),
        }
    }
    fn copy_annotation(value: &ValueAnnotation) -> bool {
        match &value.kind {
            Kind::Named(name) => matches!(name.text.as_str(), "int" | "number" | "boolean" | "null" | "undefined"),
            Kind::Group(inner) => copy_annotation(inner),
            Kind::Union(members) => members.iter().all(copy_annotation),
            _ => false,
        }
    }
    fn validate_depth(value: &ValueAnnotation, depth: usize) -> Result<(), Diagnostic> {
        let mut pending = vec![(value, depth)];
        while let Some((value, depth)) = pending.pop() {
            if depth > 128 {
                return Err(Diagnostic::new(
                    "generic-limit",
                    "Substituted type nesting exceeds 128.",
                    value.span,
                ));
            }
            match &value.kind {
                Kind::Named(_) => {}
                Kind::Group(inner) | Kind::Array(inner) => pending.push((inner, depth + 1)),
                Kind::Applied { arguments, .. } | Kind::Union(arguments) | Kind::Tuple(arguments) => {
                    pending.extend(arguments.iter().map(|value| (value, depth + 1)))
                }
                Kind::Callable {
                    parameters, result, ..
                } => {
                    pending.extend(
                        parameters
                            .iter()
                            .map(|parameter| (&parameter.annotation, depth + 1)),
                    );
                    pending.push((result, depth + 1));
                }
            }
        }
        Ok(())
    }
    visit(value, bindings, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_syntax::{parse, CallableOwnership};
    #[test]
    fn substitutions_preserve_callable_modes_nominal_names_and_argument_spans() {
        let mut value = parse("Once<Function<T[], [value:ReadOnly<T>, other:Mutable<Box<T>>]>>").unwrap();
        let argument = parse("/* 🦀 */ string|null").unwrap();
        let bindings = HashMap::from([("T".into(), argument.clone())]);
        apply(&mut value, &bindings).unwrap();
        let Kind::Callable {
            ownership,
            parameters,
            result,
        } = value.kind
        else {
            panic!("callable");
        };
        assert_eq!(ownership, CallableOwnership::OwnedOnce);
        assert_eq!(
            parameters[0].access.as_ref().unwrap().mode,
            crate::Mode::Read
        );
        assert_eq!(
            parameters[1].access.as_ref().unwrap().mode,
            crate::Mode::Mutate
        );
        assert_eq!(parameters[0].annotation, argument);
        let Kind::Applied { name, arguments } = &parameters[1].annotation.kind else {
            panic!("application");
        };
        assert_eq!(name.text, "Box");
        assert_eq!(arguments[0], argument);
        let Kind::Array(inner) = result.kind else {
            panic!("array");
        };
        assert_eq!(*inner, argument);
    }
    #[test]
    fn replacements_are_simultaneous_and_constructors_cannot_be_parameters() {
        let mut value = parse("T").unwrap();
        let bindings = HashMap::from([
            ("T".into(), parse("U").unwrap()),
            ("U".into(), parse("int").unwrap()),
        ]);
        apply(&mut value, &bindings).unwrap();
        assert!(matches!(value.kind,Kind::Named(name) if name.text=="U"));
        let mut value = parse("T<int>").unwrap();
        let error = apply(&mut value, &bindings).unwrap_err();
        assert_eq!(error.code, "type");
        assert_eq!(error.span, crate::Span::new(0, 1));
    }
    #[test]
    fn composed_annotation_depth_is_bounded() {
        let mut value = parse(&format!("{}T{}", "(".repeat(70), ")".repeat(70))).unwrap();
        let argument = parse(&format!("{}int{}", "(".repeat(70), ")".repeat(70))).unwrap();
        let error = apply(&mut value, &HashMap::from([("T".into(), argument)])).unwrap_err();
        assert_eq!(error.code, "generic-limit");
    }
}

#[cfg(test)]
#[test]
fn private_containment_parameter_substitution_preserves_element_ownership() {
    use crate::type_syntax::parse;
    fn shape(value: &ValueAnnotation) -> String {
        match &value.kind {
            Kind::Named(name) => name.text.clone(),
            Kind::Array(inner) => format!("{}[]",shape(inner)),
            Kind::Union(members) => members.iter().map(shape).collect::<Vec<_>>().join("|"),
            Kind::Applied{name,arguments} => format!("{}<{}>",name.text,arguments.iter().map(shape).collect::<Vec<_>>().join(",")),
            _ => panic!("unexpected parameter shape: {value:?}"),
        }
    }
    for (actual, expected, borrowed) in [("int[]","int",false),("int[][]","int[]",true),("string","string",true),("[int,string]","int|string",true),("number[]|null","number|null",false)] {
        let mut value=parse("Function<void,[expected:ReadOnly<ElementOf<T>>]>").unwrap();
        let Kind::Callable {parameters,..}=&mut value.kind else {panic!("callable")};
        let Kind::Applied{name,..}=&mut parameters[0].annotation.kind else {panic!("element constructor")};
        name.text="@testing:element".into();
        apply(&mut value,&HashMap::from([("T".into(),parse(actual).unwrap())])).unwrap();
        let Kind::Callable {parameters,..}=&value.kind else {panic!("callable")};
        assert_eq!(shape(&parameters[0].annotation),expected);
        assert_eq!(parameters[0].access.is_some(),borrowed);
    }
}

#[cfg(test)]
#[test]
fn containment_projection_remains_private_and_checks_arity() {
    use crate::type_syntax::parse;
    let bindings=HashMap::from([("T".into(),parse("int[]").unwrap())]);
    let mut ordinary=parse("ElementOf<T>").unwrap();apply(&mut ordinary,&bindings).unwrap();
    assert!(matches!(ordinary.kind,Kind::Applied{name,..} if name.text=="ElementOf"));
    let mut invalid=parse("ElementOf<T,T>").unwrap();
    let Kind::Applied{name,..}=&mut invalid.kind else {panic!("constructor")};name.text="@testing:element".into();
    assert_eq!(apply(&mut invalid,&bindings).unwrap_err().code,"arity");
}

#[cfg(test)]
#[test]
fn containment_element_retains_actual_element_coordinates() {
    let argument=crate::type_syntax::parse("/* actual */ string[]").unwrap();
    let Kind::Array(element)=&argument.kind else {panic!("array")};let expected_span=element.span;
    let mut annotation=crate::type_syntax::parse("ElementOf<T>").unwrap();
    let Kind::Applied{name,..}=&mut annotation.kind else {panic!("projection")};name.text="@testing:element".into();
    apply(&mut annotation,&HashMap::from([("T".into(),argument)])).unwrap();
    assert_eq!(annotation.span,expected_span);
    let Kind::Named(name)=annotation.kind else {panic!("element")};assert_eq!(name.span,expected_span);assert_eq!(name.text,"string");
}
