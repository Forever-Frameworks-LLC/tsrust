//! Structured lexical substitution shared by generic declarations.
use crate::type_syntax::{ValueAnnotation, ValueAnnotationKind as Kind};
use crate::Diagnostic;
use std::collections::HashMap;

/// Arguments are already concrete. A replacement is not substituted again.
/// Replacement coordinates remain those of the argument that supplied it.
pub(crate) fn apply(value: &mut ValueAnnotation, bindings: &HashMap<String, ValueAnnotation>) -> Result<(), Diagnostic> {
    // Resolution still has recursive consumers. Preserve its current admission
    // boundary until those consumers are converted, independently of this stack.
    apply_with_limit(value, bindings, Some(128))
}

#[derive(Clone, Copy)]
enum Child { Inner, Member(usize), Parameter(usize), Result }
struct Frame { parent: ValueAnnotation, child: Child, adjust_access: bool }

fn child(value: &mut ValueAnnotation, path: Child) -> &mut ValueAnnotation {
    match (&mut value.kind, path) {
        (Kind::Array(inner) | Kind::Group(inner), Child::Inner) => inner,
        (Kind::Applied { arguments, .. } | Kind::Union(arguments) | Kind::Tuple(arguments), Child::Member(index)) => &mut arguments[index],
        (Kind::Callable { parameters, .. }, Child::Parameter(index)) => &mut parameters[index].annotation,
        (Kind::Callable { result, .. }, Child::Result) => result,
        _ => unreachable!("substitution child address"),
    }
}

fn next_child(value: &ValueAnnotation, previous: Option<Child>) -> Option<Child> {
    match (&value.kind, previous) {
        (Kind::Array(_) | Kind::Group(_), None) => Some(Child::Inner),
        (Kind::Applied { arguments, .. } | Kind::Union(arguments) | Kind::Tuple(arguments), previous) => {
            let index = match previous { None => 0, Some(Child::Member(index)) => index + 1, _ => unreachable!() };
            (index < arguments.len()).then_some(Child::Member(index))
        }
        (Kind::Callable { parameters, .. }, previous) => {
            let index = match previous { None => 0, Some(Child::Parameter(index)) => index + 1, Some(Child::Result) => return None, _ => unreachable!() };
            Some(if index < parameters.len() { Child::Parameter(index) } else { Child::Result })
        }
        _ => None,
    }
}

fn take(value: &mut ValueAnnotation) -> ValueAnnotation {
    let span = value.span;
    std::mem::replace(value, ValueAnnotation { kind: Kind::Tuple(Vec::new()), span })
}

fn descend(mut parent: ValueAnnotation, path: Child, bindings: &HashMap<String, ValueAnnotation>, frames: &mut Vec<Frame>) -> ValueAnnotation {
    let value = take(child(&mut parent, path));
    let adjust_access = matches!(path, Child::Parameter(_)) &&
        (matches!(&value.kind, Kind::Named(name) if bindings.contains_key(&name.text)) ||
         matches!(&value.kind, Kind::Applied { name, .. } if name.text == "@testing:element"));
    frames.push(Frame { parent, child: path, adjust_access });
    value
}

fn apply_with_limit(value: &mut ValueAnnotation, bindings: &HashMap<String, ValueAnnotation>, limit: Option<usize>) -> Result<(), Diagnostic> {
    if bindings.is_empty() { return Ok(()); }
    let mut current = take(value);
    let mut frames: Vec<Frame> = Vec::new();
    let mut entering = true;
    let mut project = false;
    let outcome = loop {
        if entering {
            if let Some(limit) = limit && frames.len() > limit {
                break Err(depth_error(current.span));
            }
            if let Kind::Named(name) = &current.kind {
                if let Some(argument) = bindings.get(&name.text) {
                    if let Some(limit) = limit && let Err(error) = validate_depth(argument, frames.len(), limit) { break Err(error); }
                    current = argument.clone();
                }
                // Simultaneous replacements never re-enter a supplied argument.
                project = false;
            } else {
                if let Kind::Applied { name, .. } = &current.kind && bindings.contains_key(&name.text) {
                    break Err(Diagnostic::new("type", "A type parameter cannot be used as a generic type constructor.", name.span));
                }
                if let Some(path) = next_child(&current, None) {
                    current = descend(current, path, bindings, &mut frames);
                    continue;
                }
                project = true;
            }
        }
        if project && let Kind::Applied { name, arguments } = &current.kind && name.text == "@testing:element" {
            if arguments.len() != 1 {
                break Err(Diagnostic::new("arity", "Internal containment element projection requires one type argument.", current.span));
            }
            current = containment_element(&arguments[0]);
        }
        let Some(mut frame) = frames.pop() else { break Ok(()); };
        if frame.adjust_access && copy_annotation(&current) {
            let (Kind::Callable { parameters, .. }, Child::Parameter(index)) = (&mut frame.parent.kind, frame.child) else { unreachable!() };
            parameters[index].access = None;
        }
        *child(&mut frame.parent, frame.child) = current;
        if let Some(path) = next_child(&frame.parent, Some(frame.child)) {
            current = descend(frame.parent, path, bindings, &mut frames);
            entering = true;
        } else {
            current = frame.parent;
            entering = false;
            project = true;
        }
    };
    // Restore the exact partially visited tree on error. Earlier substitutions
    // remain, later siblings are untouched, and unfinished access adjustments do
    // not run, matching the former recursive visitor's first-error behavior.
    while let Some(mut frame) = frames.pop() {
        *child(&mut frame.parent, frame.child) = current;
        current = frame.parent;
    }
    *value = current;
    outcome
}

fn depth_error(span: crate::Span) -> Diagnostic {
    Diagnostic::new("generic-limit", "Substituted type nesting exceeds 128.", span)
}
fn containment_element(root: &ValueAnnotation) -> ValueAnnotation {
    enum Work<'a> { Visit(&'a ValueAnnotation), Union(crate::Span, usize) }
    let mut pending = vec![Work::Visit(root)];
    let mut results = Vec::new();
    while let Some(work) = pending.pop() {
        match work {
            Work::Union(span, start) => {
                let members = results.drain(start..).collect();
                results.push(ValueAnnotation { kind: Kind::Union(members), span });
            }
            Work::Visit(value) => match &value.kind {
                Kind::Group(inner) => pending.push(Work::Visit(inner)),
                Kind::Array(inner) => results.push((**inner).clone()),
                Kind::Tuple(members) if members.len() == 1 => results.push(members[0].clone()),
                Kind::Tuple(members) if !members.is_empty() => results.push(ValueAnnotation { kind: Kind::Union(members.clone()), span: value.span }),
                Kind::Union(members) => {
                    pending.push(Work::Union(value.span, results.len()));
                    pending.extend(members.iter().rev().map(Work::Visit));
                }
                _ => results.push(value.clone()),
            }
        }
    }
    results.pop().expect("projected containment element")
}

fn copy_annotation(value: &ValueAnnotation) -> bool {
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        match &value.kind {
            Kind::Literal(value) if value.copy() => {}
            Kind::Named(name) if matches!(name.text.as_str(), "int" | "number" | "boolean" | "null" | "undefined") => {}
            Kind::Group(inner) => pending.push(inner),
            Kind::Union(members) => pending.extend(members),
            _ => return false,
        }
    }
    true
}

fn validate_depth(value: &ValueAnnotation, depth: usize, limit: usize) -> Result<(), Diagnostic> {
    let mut pending = vec![(value, depth)];
    while let Some((value, depth)) = pending.pop() {
        if depth > limit {
            return Err(Diagnostic::new(
                "generic-limit",
                "Substituted type nesting exceeds 128.",
                value.span,
            ));
        }
        match &value.kind {
            Kind::Literal(_) | Kind::Named(_) | Kind::Polymorphic { .. } => {}
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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_syntax::{parse, CallableOwnership};
    #[test]
    fn deep_substitution_engine_preserves_spans_and_production_disposal() {
        let mut value = parse("T").unwrap();
        for _ in 0..16_384 { let span = value.span; value = ValueAnnotation { kind: Kind::Group(Box::new(value)), span }; }
        let argument = parse("/* actual */ int").unwrap();
        let expected = argument.clone();
        let bindings = HashMap::from([("T".into(), argument)]);
        // Public admission remains bounded until recursive type resolution is
        // converted. The same worklist engine has no call-stack depth dependency.
        apply_with_limit(&mut value, &bindings, None).unwrap();
        let mut leaf = &value;
        let mut depth = 0;
        while let Kind::Group(inner) = &leaf.kind { leaf = inner; depth += 1; }
        assert_eq!(depth, 16_384);
        assert_eq!(leaf, &expected);
        drop(value);
    }

    #[test]
    fn first_error_restores_partial_tree_without_adjusting_unfinished_parameters() {
        let mut value = parse("Function<T,[first:ReadOnly<T>,second:ReadOnly<Box<T,U<int>>>]>").unwrap();
        let original = value.clone();
        let bindings = HashMap::from([("T".into(), parse("int").unwrap()), ("U".into(), parse("string").unwrap())]);
        let error = apply(&mut value, &bindings).unwrap_err();
        let Kind::Callable { parameters, result, .. } = &value.kind else { panic!("callable") };
        let Kind::Callable { parameters: before, result: before_result, .. } = &original.kind else { unreachable!() };
        assert!(parameters[0].access.is_none());
        assert_eq!(parameters[0].annotation, bindings["T"]);
        assert_eq!(parameters[1].access, before[1].access);
        let Kind::Applied { arguments, .. } = &parameters[1].annotation.kind else { unreachable!() };
        assert_eq!(arguments[0], bindings["T"]);
        let Kind::Applied { name, .. } = &arguments[1].kind else { unreachable!() };
        assert_eq!(error.code, "type"); assert_eq!(error.span, name.span);
        assert_eq!(result, before_result, "later result must remain unsubstituted");
    }

    #[test]
    fn deep_containment_projection_and_copy_modes_use_worklists() {
        let mut argument = parse("int[]").unwrap();
        for _ in 0..16_384 { let span = argument.span; argument = ValueAnnotation { kind: Kind::Union(vec![argument]), span }; }
        let mut value = parse("Function<void,[value:ReadOnly<ElementOf<T>>]>").unwrap();
        let Kind::Callable { parameters, .. } = &mut value.kind else { unreachable!() };
        let Kind::Applied { name, .. } = &mut parameters[0].annotation.kind else { unreachable!() };
        name.text = "@testing:element".into();
        apply_with_limit(&mut value, &HashMap::from([("T".into(), argument)]), None).unwrap();
        let Kind::Callable { parameters, .. } = &value.kind else { unreachable!() };
        assert!(parameters[0].access.is_none());
        let mut leaf = &parameters[0].annotation;
        let mut depth = 0;
        while let Kind::Union(members) = &leaf.kind { assert_eq!(members.len(), 1); leaf = &members[0]; depth += 1; }
        assert_eq!(depth, 16_384);
        assert!(matches!(&leaf.kind, Kind::Named(name) if name.text == "int"));
    }
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
        } = value.into_kind()
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
        let Kind::Array(inner) = result.into_kind() else {
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
        assert!(matches!(value.into_kind(),Kind::Named(name) if name.text=="U"));
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
            Kind::Literal(value)=>value.text(),
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
    assert!(matches!(ordinary.into_kind(),Kind::Applied{name,..} if name.text=="ElementOf"));
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
    let Kind::Named(name)=annotation.into_kind() else {panic!("element")};assert_eq!(name.span,expected_span);assert_eq!(name.text,"string");
}

#[cfg(test)]
#[test]
fn literal_arguments_remain_values_during_generic_substitution() {
    use crate::type_syntax::{parse,LiteralAnnotation};
    let mut value=parse("Function<T,['T',T]>").unwrap();
    apply(&mut value,&HashMap::from([("T".into(),parse("9007199254740993").unwrap())])).unwrap();
    let Kind::Callable{parameters,result,..}=value.into_kind() else{panic!("callable")};
    assert_eq!(result.kind,Kind::Literal(LiteralAnnotation::Int("9007199254740993".into())));
    assert_eq!(parameters[0].annotation.kind,Kind::Literal(LiteralAnnotation::String("T".into())));
    assert_eq!(parameters[1].annotation.kind,result.kind);
}
