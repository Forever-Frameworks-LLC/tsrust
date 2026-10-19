//! Constraints over checked nominal types. This never checks/evaluates expressions.
use crate::syntax::{Record, Type};
use crate::type_syntax::{ValueAnnotation, ValueAnnotationKind as Kind};
use crate::{Diagnostic, Span};
use crate::types::reconstruction::{reconstruct, Plan};
use std::collections::{BTreeSet, HashMap};
use std::borrow::Cow;

pub(crate) type CallableShapes = HashMap<Type, (Vec<Type>, Type)>;
pub(crate) type InlineShapes = HashMap<(usize, usize), (Vec<Type>, Type)>;

pub(crate) struct Types<'a> {
    pub callable_shapes: Option<&'a CallableShapes>,
    pub callable_annotations: Option<&'a HashMap<Type, ValueAnnotation>>,
    pub inline_shapes: Option<&'a InlineShapes>,
    pub records: &'a [Record],
    pub names: &'a HashMap<String, usize>,
    pub unions: &'a crate::unions::UnionArena,
}
fn may_defer(pattern: &ValueAnnotation) -> bool {
    let mut pending = vec![pattern];
    while let Some(pattern) = pending.pop() {
        match &pattern.kind {
            Kind::Union(_) => return true,
            Kind::Group(inner) | Kind::Array(inner) => pending.push(inner),
            Kind::Applied { arguments, .. } | Kind::Tuple(arguments) => pending.extend(arguments.iter().rev()),
            Kind::Literal(_) | Kind::Named(_) | Kind::Callable { .. } | Kind::Polymorphic { .. } => {},
        }
    }
    false
}

// Borrow source constraints; own only substituted union payloads. Moving owned
// children out keeps traversal linear instead of cloning each remaining subtree.
type Pattern<'a> = Cow<'a, ValueAnnotation>;
enum ConstraintWork<'a> {
    Unify(Pattern<'a>, Type),
    Resolve(Pattern<'a>, &'a ValueAnnotation, Option<Type>),
    Callable(Pattern<'a>, &'a [Type], Type),
    UnionMembers(ValueAnnotation, std::vec::IntoIter<Type>, Vec<Type>, Span),
}
fn constraint_children(pattern: Pattern<'_>) -> Vec<Pattern<'_>> {
    match pattern {
        Cow::Borrowed(value) => match &value.kind {
            Kind::Group(inner) | Kind::Array(inner) => vec![Cow::Borrowed(inner)],
            Kind::Tuple(values) | Kind::Union(values) | Kind::Applied { arguments: values, .. } => values.iter().map(Cow::Borrowed).collect(),
            Kind::Callable { parameters, result, .. } => parameters.iter().map(|parameter| Cow::Borrowed(&parameter.annotation)).chain(std::iter::once(Cow::Borrowed(result.as_ref()))).collect(),
            _ => Vec::new(),
        },
        Cow::Owned(value) => match value.into_kind() {
            Kind::Group(inner) | Kind::Array(inner) => vec![Cow::Owned(*inner)],
            Kind::Tuple(values) | Kind::Union(values) | Kind::Applied { arguments: values, .. } => values.into_iter().map(Cow::Owned).collect(),
            Kind::Callable { parameters, result, .. } => parameters.into_iter().map(|parameter| Cow::Owned(parameter.annotation)).chain(std::iter::once(Cow::Owned(*result))).collect(),
            _ => Vec::new(),
        },
    }
}
fn constraint_mismatch(span: Span) -> Diagnostic {
    Diagnostic::new("type", "Generic type constraints do not match the checked argument type.", span)
}

pub(crate) struct Constraints<'a> {
    pub variables: &'a [crate::type_syntax::TypeName],
    pub parameters: &'a [ValueAnnotation],
    pub arguments: &'a [Option<Type>],
    pub result: Option<&'a ValueAnnotation>,
    pub expected: Option<Type>,
    pub span: Span,
}
impl Types<'_> {
    pub(crate) fn contextual_type(
        &self,
        pattern: &ValueAnnotation,
        bindings: &HashMap<String, Type>,
    ) -> Option<Type> {
        let replacements = bindings
            .iter()
            .map(|(name, ty)| {
                self.annotation(*ty, pattern.span)
                    .map(|annotation| (name.clone(), annotation))
            })
            .collect::<Result<HashMap<_, _>, _>>()
            .ok()?;
        let mut annotation = pattern.clone();
        crate::type_substitution::apply(&mut annotation, &replacements).ok()?;
        self.materialize_names(&mut annotation).ok()?;
        crate::analysis::resolve_value_type(&annotation, false, self.names, self.unions).ok()
    }

    pub(crate) fn solve(
        &self,
        request: Constraints<'_>,
        bindings: &mut HashMap<String, Type>,
    ) -> Result<Vec<Type>, Diagnostic> {
        if request.parameters.len() != request.arguments.len() {
            return Err(Diagnostic::new(
                "arity",
                "Generic function argument count does not match its declaration.",
                request.span,
            ));
        }
        let variables = request
            .variables
            .iter()
            .map(|variable| variable.text.clone())
            .collect::<BTreeSet<_>>();
        let mut deferred = Vec::new();
        for (pattern, actual) in request.parameters.iter().zip(request.arguments) {
            if let Some((parameters, result)) = self
                .inline_shapes
                .and_then(|shapes| shapes.get(&(pattern.span.start, pattern.span.end)))
            {
                self.unify_callable(pattern, parameters, *result, &variables, bindings)?;
                continue;
            }
            let Some(actual) = actual else {
                continue;
            };
            if matches!(pattern.kind, Kind::Callable { .. })
                && self
                    .callable_shapes
                    .and_then(|shapes| shapes.get(actual))
                    .is_none()
            {
                // Ordinary callback checking owns callable contracts and access modes.
                continue;
            }
            let prior = may_defer(pattern).then(|| bindings.clone());
            match self.unify(pattern, *actual, &variables, bindings) {
                Ok(()) => {}
                Err(error) if error.code == "generic-inference" && prior.is_some() => {
                    *bindings = prior.expect("nullable constraint checkpoint");
                    deferred.push((pattern, *actual));
                }
                Err(error) => return Err(error),
            }
        }
        if let (Some(pattern), Some(expected)) = (request.result, request.expected) {
            // Inputs establish the payload first. A contextual result can accept
            // that concrete result as one union member without changing it.
            let accepted = self.contextual_type(pattern, bindings)
                .is_some_and(|actual| self.unions.accepts(expected, actual));
            if !accepted {
                let prior=bindings.clone();
                if let Err(error)=self.unify(pattern, expected, &variables, bindings) {
                    *bindings=prior.clone();
                    let Type::Union(union)=expected else{return Err(error);};
                    // A concrete result shape can determine its payload from a
                    // unique compatible destination member, never the first one.
                    let mut candidates=Vec::new();
                    for member in self.unions.members(union) {
                        let mut candidate=prior.clone();
                        if self.unify(pattern,member,&variables,&mut candidate).is_ok() && !candidates.contains(&candidate) {candidates.push(candidate);}
                    }
                    match candidates.len() {
                        0=>return Err(error),
                        1=>*bindings=candidates.pop().unwrap(),
                        _=>return Err(Diagnostic::new("generic-inference","Result context has several compatible generic payloads; provide argument types or explicit type arguments.",pattern.span)),
                    }
                }
            }
        }
        for (pattern, actual) in deferred {
            self.unify(pattern, actual, &variables, bindings)?;
        }
        request.variables.iter().map(|variable| {
            bindings.get(&variable.text).copied().ok_or_else(|| Diagnostic::new(
                "generic-inference",
                format!("Cannot infer type parameter {:?}; provide explicit type arguments or a result annotation.", variable.text),
                request.span,
            ))
        }).collect()
    }

    pub(crate) fn unify(
        &self,
        pattern: &ValueAnnotation,
        actual: Type,
        variables: &BTreeSet<String>,
        bindings: &mut HashMap<String, Type>,
    ) -> Result<(), Diagnostic> {
        self.unify_work(vec![ConstraintWork::Unify(Cow::Borrowed(pattern), actual)], variables, bindings)
    }

    fn unify_work<'a>(
        &'a self,
        mut work: Vec<ConstraintWork<'a>>,
        variables: &BTreeSet<String>,
        bindings: &mut HashMap<String, Type>,
    ) -> Result<(), Diagnostic> {
        while let Some(next) = work.pop() {
            let (pattern, actual) = match next {
                ConstraintWork::Unify(pattern, actual) => (pattern, actual),
                ConstraintWork::Resolve(pattern, annotation, checked) => {
                    // Resolve immediately before this child, preserving first-error
                    // order and the old eager fallback even for a checked field.
                    let resolved = crate::analysis::resolve_value_type(annotation, false, self.names, self.unions)?;
                    work.push(ConstraintWork::Unify(pattern, checked.unwrap_or(resolved)));
                    continue;
                }
                ConstraintWork::UnionMembers(pattern, mut members, absent, span) => {
                    let Some(actual) = members.next() else { continue; };
                    if matches!(actual, Type::Null | Type::Undefined) {
                        if !absent.contains(&actual) { return Err(constraint_mismatch(span)); }
                        work.push(ConstraintWork::UnionMembers(pattern, members, absent, span));
                        continue;
                    }
                    // Retain one template, and clone only the member being checked.
                    // The final member consumes it; wide unions do not retain one
                    // deep annotation copy per pending member.
                    let current = if members.len() == 0 { pattern } else {
                        let current = pattern.clone();
                        work.push(ConstraintWork::UnionMembers(pattern, members, absent, span));
                        current
                    };
                    (Cow::Owned(current), actual)
                }
                ConstraintWork::Callable(pattern, actual_parameters, actual_result) => {
                    self.schedule_callable(pattern, actual_parameters, actual_result, &mut work)?;
                    continue;
                }
            };
            let mismatch_span = pattern.span;
            let mismatch = || constraint_mismatch(mismatch_span);
            match &pattern.kind {
                Kind::Polymorphic { origin, .. } => if actual==Type::Closure(*origin) {Ok(())} else {Err(mismatch())},
                Kind::Named(name) if variables.contains(&name.text) => {
                    if let Some(prior) = bindings.get(&name.text) {
                        let same_callable = *prior != actual
                            && self.callable_annotations.is_some_and(|annotations| annotations.contains_key(prior) && annotations.contains_key(&actual))
                            && self.annotation(*prior, pattern.span).ok().zip(self.annotation(actual, pattern.span).ok())
                                .is_some_and(|(left, right)| crate::generic_functions::same_annotation_contract(&left, &right));
                        if *prior != actual && !self.unions.accepts(*prior, actual) && !same_callable {
                            return Err(mismatch());
                        }
                    } else {
                        bindings.insert(name.text.clone(), actual);
                    }
                    Ok(())
                }
                Kind::Named(name) if name.text == "void" && actual == Type::Void => Ok(()),
                Kind::Literal(_) | Kind::Named(_) => {
                    let expected =
                        crate::analysis::resolve_value_type(&pattern, false, self.names, self.unions)?;
                    if expected == actual {
                        Ok(())
                    } else {
                        Err(mismatch())
                    }
                }
                Kind::Group(_) => {
                    work.push(ConstraintWork::Unify(constraint_children(pattern).pop().unwrap(), actual));
                    Ok(())
                },
                Kind::Array(_) => {
                    let Type::Array(element) = actual else {
                        return Err(mismatch());
                    };
                    work.push(ConstraintWork::Unify(constraint_children(pattern).pop().unwrap(), element.ty()));
                    Ok(())
                }
                Kind::Tuple(positions) => {
                    let Type::Record(id) = actual else { return Err(mismatch()); };
                    let record = &self.records[id];
                    if !record.is_tuple() || record.fields.len() != positions.len() { return Err(mismatch()); }
                    for (position, field) in constraint_children(pattern).into_iter().zip(&record.fields).rev() {
                        work.push(ConstraintWork::Resolve(position, &field.annotation, field.ty));
                    }
                    Ok(())
                }
                Kind::Applied { name, arguments } => {
                    let (Type::Record(id) | Type::Set(id) | Type::Map(id)) = actual else {
                        return Err(mismatch());
                    };
                    let Some(application) = &self.records[id].application else {
                        return Err(mismatch());
                    };
                    if application.declaration.text != name.text
                        || application.arguments.len() != arguments.len()
                    {
                        return Err(mismatch());
                    }
                    for (pattern, argument) in constraint_children(pattern).into_iter().zip(&application.arguments).rev() {
                        work.push(ConstraintWork::Resolve(pattern, argument, None));
                    }
                    Ok(())
                }
                Kind::Union(_) => {
                    let mut pending = vec![pattern.as_ref()];
                    let mut unbound = false;
                    while let Some(value) = pending.pop() {
                        match &value.kind {
                            Kind::Literal(_) | Kind::Polymorphic { .. } => {},
                            Kind::Named(name) => {
                                unbound |=
                                    variables.contains(&name.text) && !bindings.contains_key(&name.text)
                            }
                            Kind::Group(inner) | Kind::Array(inner) => pending.push(inner),
                            Kind::Union(members) | Kind::Tuple(members)
                            | Kind::Applied {
                                arguments: members, ..
                            } => pending.extend(members),
                            Kind::Callable {
                                parameters, result, ..
                            } => {
                                pending
                                    .extend(parameters.iter().map(|parameter| &parameter.annotation));
                                pending.push(result);
                            }
                        }
                    }
                    if unbound {
                        let mut absent = Vec::new();
                        let mut payloads = Vec::new();
                        let mut pending = vec![pattern.as_ref()];
                        while let Some(value) = pending.pop() {
                            match &value.kind {
                                Kind::Group(inner) => pending.push(inner),
                                Kind::Union(members) => pending.extend(members),
                                Kind::Named(name) if name.text == "null" => absent.push(Type::Null),
                                Kind::Named(name) if name.text == "undefined" => {
                                    absent.push(Type::Undefined)
                                }
                                _ => payloads.push(value),
                            }
                        }
                        let mut unresolved = Vec::new();
                        for value in payloads {
                            if let Some(known) = self.contextual_type(value, bindings) {
                                let known = known.union().map(|id| self.unions.members(id)).unwrap_or_else(|| vec![known]);
                                absent.extend(known);
                            } else { unresolved.push(value); }
                        }
                        if unresolved.len() != 1 {
                            return Err(Diagnostic::new("generic-inference", "Union inference requires one unresolved payload pattern; provide explicit type arguments for ambiguous patterns.", pattern.span));
                        }
                        let members = match actual {
                            Type::Union(id) => self.unions.members(id),
                            value => vec![value],
                        };
                        let remaining = members
                            .into_iter()
                            .filter(|member| !absent.contains(member))
                            .collect::<Vec<_>>();
                        if remaining.is_empty() {
                            return Err(Diagnostic::new(
                                "generic-inference",
                                "An absent value does not determine the generic payload type; provide type arguments or a result context.",
                                pattern.span,
                            ));
                        }
                        let payload = self.unions.intern(remaining, pattern.span)?;
                        work.push(ConstraintWork::Unify(Cow::Owned(unresolved[0].clone()), payload));
                        continue;
                    }
                    // Fully known constraints admit checked absent/payload variants.
                    let mut value = pattern.as_ref().clone();
                    let arguments = bindings
                        .iter()
                        .map(|(name, ty)| Ok((name.clone(), self.annotation(*ty, pattern.span)?)))
                        .collect::<Result<HashMap<_, _>, Diagnostic>>()?;
                    crate::type_substitution::apply(&mut value, &arguments)?;
                    self.materialize_names(&mut value)?;
                    // Callable payload constraints use checked shapes; ordinary argument
                    // checking still validates ownership, access and origin contracts.
                    let mut pending = vec![&value];
                    let mut callable = None;
                    let mut absent = Vec::new();
                    while let Some(member) = pending.pop() {
                        match &member.kind {
                            Kind::Group(inner) => pending.push(inner),
                            Kind::Union(members) => pending.extend(members),
                            Kind::Callable { .. } => callable = Some(member),
                            Kind::Named(name) if name.text == "null" => absent.push(Type::Null),
                            Kind::Named(name) if name.text == "undefined" => {
                                absent.push(Type::Undefined)
                            }
                            _ => (),
                        }
                    }
                    if let Some(callable) = callable {
                        let members = actual
                            .union()
                            .map(|id| self.unions.members(id))
                            .unwrap_or_else(|| vec![actual]);
                        work.push(ConstraintWork::UnionMembers(callable.clone(), members.into_iter(), absent, mismatch_span));
                        continue;
                    }
                    let expected =
                        crate::analysis::resolve_value_type(&value, false, self.names, self.unions)?;
                    let expected_members = match expected {
                        Type::Union(id) => self.unions.members(id),
                        value => vec![value],
                    };
                    let actual_members = match actual {
                        Type::Union(id) => self.unions.members(id),
                        value => vec![value],
                    };
                    if actual_members
                        .iter()
                        .all(|member| expected_members.contains(member))
                    {
                        Ok(())
                    } else {
                        Err(mismatch())
                    }
                }
                Kind::Callable { .. } => {
                    let Some((actual_parameters, actual_result)) =
                        self.callable_shapes.and_then(|shapes| shapes.get(&actual))
                    else {
                        return Err(Diagnostic::new(
                            "generic-inference",
                            "Callable signature is not yet known.",
                            pattern.span,
                        ));
                    };
                    work.push(ConstraintWork::Callable(pattern, actual_parameters, *actual_result));
                    Ok(())
                }
            }?;
        }
        Ok(())
    }

    fn unify_callable(
        &self,
        pattern: &ValueAnnotation,
        actual_parameters: &[Type],
        actual_result: Type,
        variables: &BTreeSet<String>,
        bindings: &mut HashMap<String, Type>,
    ) -> Result<(), Diagnostic> {
        self.unify_work(vec![ConstraintWork::Callable(Cow::Borrowed(pattern), actual_parameters, actual_result)], variables, bindings)
    }

    fn schedule_callable<'a>(
        &self,
        pattern: Pattern<'a>,
        actual_parameters: &'a [Type],
        actual_result: Type,
        work: &mut Vec<ConstraintWork<'a>>,
    ) -> Result<(), Diagnostic> {
        match &pattern.kind {
            Kind::Group(_) => work.push(ConstraintWork::Callable(constraint_children(pattern).pop().unwrap(), actual_parameters, actual_result)),
            Kind::Callable { parameters, .. } => {
                if parameters.len() != actual_parameters.len() {
                    return Err(Diagnostic::new("type", "Generic callable parameter count does not match the argument signature.", pattern.span));
                }
                let actuals = parameters.iter().zip(actual_parameters).map(|(parameter, actual)| {
                    // Omission adds undefined at the boundary, not to the payload constraint.
                    if parameter.omission.is_omittable() && let Type::Union(id) = actual {
                        let members = self.unions.members(*id).into_iter().filter(|ty| *ty != Type::Undefined).collect::<Vec<_>>();
                        if members.len() == 1 { return members[0]; }
                    }
                    *actual
                }).chain(std::iter::once(actual_result)).collect::<Vec<_>>();
                for (pattern, actual) in constraint_children(pattern).into_iter().zip(actuals).rev() {
                    work.push(ConstraintWork::Unify(pattern, actual));
                }
            }
            _ => {},
        }
        Ok(())
    }

    fn materialize_names(&self, value: &mut ValueAnnotation) -> Result<(), Diagnostic> {
        // Owned parent frames allow postorder replacement without recursive borrows.
        // On failure, restore every suspended parent around the partially visited child.
        enum Child { Inner, Member(usize) }
        fn child<'a>(value: &'a mut ValueAnnotation, path: &Child) -> &'a mut ValueAnnotation {
            match (&mut value.kind, path) {
                (Kind::Array(inner) | Kind::Group(inner), Child::Inner) => inner,
                (Kind::Applied { arguments, .. } | Kind::Union(arguments) | Kind::Tuple(arguments), Child::Member(index)) => &mut arguments[*index],
                _ => unreachable!("constraint child address"),
            }
        }
        fn next(value: &ValueAnnotation, previous: Option<&Child>) -> Option<Child> {
            match (&value.kind, previous) {
                (Kind::Array(_) | Kind::Group(_), None) => Some(Child::Inner),
                (Kind::Applied { arguments, .. } | Kind::Union(arguments) | Kind::Tuple(arguments), previous) => {
                    let index = match previous { None => 0, Some(Child::Member(index)) => index + 1, _ => unreachable!() };
                    (index < arguments.len()).then_some(Child::Member(index))
                }
                // Callable contracts remain opaque here, as in the original visitor.
                _ => None,
            }
        }
        fn take(value: &mut ValueAnnotation) -> ValueAnnotation {
            let span = value.span;
            std::mem::replace(value, ValueAnnotation { kind: Kind::Tuple(Vec::new()), span })
        }
        let mut current = take(value);
        let mut frames = Vec::new();
        let mut entering = true;
        let outcome = loop {
            if entering && let Some(path) = next(&current, None) {
                let nested = take(child(&mut current, &path));
                frames.push((current, path));
                current = nested;
                continue;
            }
            if let Err(error) = self.materialize_application_name(&mut current) { break Err(error); }
            let Some((mut parent, path)) = frames.pop() else { break Ok(()); };
            *child(&mut parent, &path) = current;
            if let Some(path) = next(&parent, Some(&path)) {
                current = take(child(&mut parent, &path));
                frames.push((parent, path));
                entering = true;
            } else {
                current = parent;
                entering = false;
            }
        };
        while let Some((mut parent, path)) = frames.pop() {
            *child(&mut parent, &path) = current;
            current = parent;
        }
        *value = current;
        outcome
    }

    fn materialize_application_name(&self, value: &mut ValueAnnotation) -> Result<(), Diagnostic> {
        if let Kind::Applied { name, arguments } = &value.kind {
            let types = arguments
                .iter()
                .map(|argument| {
                    crate::analysis::resolve_value_type(
                        argument,
                        false,
                        self.names,
                        self.unions,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut found = None;
            for record in self.records {
                let Some(application) = &record.application else {
                    continue;
                };
                if application.declaration.text != name.text
                    || application.arguments.len() != types.len()
                {
                    continue;
                }
                let actual = application
                    .arguments
                    .iter()
                    .map(|argument| {
                        crate::analysis::resolve_value_type(
                            argument,
                            false,
                            self.names,
                            self.unions,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if actual == types {
                    found = Some(record.name.text.clone());
                    break;
                }
            }
            let Some(text) = found else {
                return Err(Diagnostic::new(
                    "type",
                    "Generic constraint has no matching concrete record identity.",
                    value.span,
                ));
            };
            value.kind = Kind::Named(crate::type_syntax::TypeName {
                text,
                span: value.span,
            });
        }
        Ok(())
    }
    pub(crate) fn annotation(&self, ty: Type, span: Span) -> Result<ValueAnnotation, Diagnostic> {
        reconstruct(ty, span, |ty| self.annotation_plan(ty, span))
    }

    fn annotation_plan(&self, ty: Type, span: Span) -> Result<Plan, Diagnostic> {
        if let Some(annotation) = self.callable_annotations.and_then(|annotations| annotations.get(&ty)) {
            let mut annotation = annotation.clone();
            annotation.span = span;
            return Ok(Plan::Leaf(annotation));
        }
        let kind = match ty {
            Type::Literal(id,_)=>Kind::Literal(self.records[id].literal_definition.as_ref().unwrap().annotation()),
            Type::Int => Kind::Named(crate::type_syntax::TypeName {
                text: "int".into(),
                span,
            }),
            Type::Number => Kind::Named(crate::type_syntax::TypeName {
                text: "number".into(),
                span,
            }),
            Type::Boolean => Kind::Named(crate::type_syntax::TypeName {
                text: "boolean".into(),
                span,
            }),
            Type::String => Kind::Named(crate::type_syntax::TypeName {
                text: "string".into(),
                span,
            }),
            Type::Null => Kind::Named(crate::type_syntax::TypeName {
                text: "null".into(),
                span,
            }),
            Type::Undefined => Kind::Named(crate::type_syntax::TypeName {
                text: "undefined".into(),
                span,
            }),
            Type::Record(id) | Type::Enum(id, _) | Type::Set(id) | Type::Map(id) => Kind::Named(crate::type_syntax::TypeName {
                text: self.records[id].name.text.clone(),
                span,
            }),
            Type::Array(element) => return Ok(Plan::Array(element.ty())),
            Type::Union(id) if self.unions.is_recursive(id) => {
                let declaration=self.unions.recursive_declaration(id).and_then(|id|self.records.get(id))
                    .ok_or_else(||Diagnostic::new("schema-recursion","Recursive type annotation has no resolved declaration.",span))?;
                Kind::Named(crate::type_syntax::TypeName{text:declaration.name.text.clone(),span})
            },
            Type::Union(id) => return Ok(Plan::Union(self.unions.members(id))),
            _ => {
                return Err(Diagnostic::new(
                    "unsupported",
                    "Generic argument is not a supported value type.",
                    span,
                ));
            }
        };
        Ok(Plan::Leaf(ValueAnnotation { kind, span }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_syntax::parse;
    #[test]
    fn deep_callable_constraints_use_one_worklist_for_borrowed_and_owned_patterns() {
        let mut shapes = HashMap::new();
        let mut pattern = parse("T").unwrap();
        let mut actual = Type::Int;
        for index in 0..16_384 {
            let mut outer = parse("(value: int) => int").unwrap();
            let Kind::Callable { parameters, result, .. } = &mut outer.kind else { unreachable!() };
            let signature = if index % 2 == 0 {
                parameters[0].annotation = pattern;
                (vec![actual], Type::Int)
            } else {
                *result = Box::new(pattern);
                (vec![Type::Int], actual)
            };
            pattern = outer;
            actual = Type::Closure(index);
            shapes.insert(actual, signature);
        }
        let names = HashMap::new();
        let unions = crate::unions::UnionArena::default();
        let types = Types { callable_annotations: None, inline_shapes: None, callable_shapes: Some(&shapes), records: &[], names: &names, unions: &unions };
        let mut bindings = HashMap::new();
        types.unify(&pattern, actual, &BTreeSet::from(["T".into()]), &mut bindings).unwrap();
        assert_eq!(bindings["T"], Type::Int);
        // Exercise owned task disposal and child moves through a substituted union
        // payload without invoking the still-retained substitution admission guard.
        let mut concrete = pattern.clone();
        let mut current = &mut concrete;
        for index in (0..16_384).rev() {
            let Kind::Callable { parameters, result, .. } = &mut current.kind else { unreachable!() };
            current = if index % 2 == 0 { &mut parameters[0].annotation } else { result };
        }
        *current = parse("int").unwrap();
        let span = concrete.span;
        let nullable = ValueAnnotation { kind: Kind::Union(vec![concrete, parse("null").unwrap()]), span };
        types.unify(&nullable, actual, &BTreeSet::new(), &mut HashMap::new()).unwrap();
        drop(nullable);
        drop(pattern);
    }

    #[test]
    fn tuple_constraints_resolve_each_field_only_after_prior_constraints() {
        let mut program = crate::syntax::parse("function observe(value:[int,int]){}function main(){}").unwrap();
        crate::generic_records::specialize(&mut program).unwrap();
        let names = program.records.iter().enumerate().map(|(id, record)| (record.name.text.clone(), id)).collect();
        let unions = crate::unions::UnionArena::default();
        let actual = crate::analysis::resolve_value_type(&program.functions[0].params[0].1, false, &names, &unions).unwrap();
        let Type::Record(id) = actual else { unreachable!() };
        program.records[id].fields[1].annotation = parse("Missing").unwrap();
        program.records[id].fields[1].ty = Some(Type::Int);
        let types = Types { callable_annotations: None, inline_shapes: None, callable_shapes: None, records: &program.records, names: &names, unions: &unions };
        let variables = BTreeSet::from(["T".into()]);
        let pattern = parse("[string,T]").unwrap();
        let Kind::Tuple(positions) = &pattern.kind else { unreachable!() };
        let mut bindings = HashMap::new();
        let error = types.unify(&pattern, actual, &variables, &mut bindings).unwrap_err();
        assert_eq!(error.message, "Generic type constraints do not match the checked argument type.");
        assert_eq!(error.span, positions[0].span);
        assert!(bindings.is_empty());
        let expected = crate::analysis::resolve_value_type(&program.records[id].fields[1].annotation, false, &names, &unions).unwrap_err();
        let error = types.unify(&parse("[T,int]").unwrap(), actual, &variables, &mut bindings).unwrap_err();
        assert_eq!(error.code, expected.code);
        assert_eq!(error.message, expected.message);
        assert_eq!(error.span, expected.span);
        assert_eq!(bindings, HashMap::from([("T".into(), Type::Int)]));
    }

    #[test]
    fn callable_constraints_preserve_first_error_and_partial_bindings() {
        let names = HashMap::new();
        let unions = crate::unions::UnionArena::default();
        let types = Types { callable_annotations: None, inline_shapes: None, callable_shapes: None, records: &[], names: &names, unions: &unions };
        let pattern = parse("(first:T, second:T, third:U) => V").unwrap();
        let Kind::Callable { parameters, .. } = &pattern.kind else { unreachable!() };
        let span = parameters[1].annotation.span;
        let mut bindings = HashMap::new();
        let error = types.unify_callable(&pattern, &[Type::Int, Type::String, Type::Boolean], Type::Int,
            &BTreeSet::from(["T".into(), "U".into(), "V".into()]), &mut bindings).unwrap_err();
        assert_eq!(error.code, "type");
        assert_eq!(error.span, span);
        assert_eq!(bindings, HashMap::from([("T".into(), Type::Int)]));
        let optional = parse("(value?:T) => T").unwrap();
        let actual = unions.intern([Type::Int, Type::Undefined], Span::default()).unwrap();
        let mut bindings = HashMap::new();
        types.unify_callable(&optional, &[actual], Type::Int, &BTreeSet::from(["T".into()]), &mut bindings).unwrap();
        assert_eq!(bindings["T"], Type::Int);
    }

    #[test]
    fn deep_constraint_names_materialize_and_drop_on_normal_stack() {
        let mut program = crate::syntax::parse("type Box<T>={value:T;};function observe(value:Box<int>){}function main(){}").unwrap();
        crate::generic_records::specialize(&mut program).unwrap();
        let names = program.records.iter().enumerate().map(|(id, record)| (record.name.text.clone(), id)).collect();
        let unions = crate::unions::UnionArena::default();
        let types = Types { callable_annotations: None, inline_shapes: None, callable_shapes: None, records: &program.records, names: &names, unions: &unions };
        let mut value = parse("Box<int>").unwrap();
        let expected = crate::analysis::resolve_value_type(&program.functions[0].params[0].1, false, &names, &unions).unwrap();
        for _ in 0..16_384 {
            let span = value.span;
            value = ValueAnnotation { kind: Kind::Group(Box::new(value)), span };
        }
        assert!(!may_defer(&value));
        types.materialize_names(&mut value).unwrap();
        let mut current = &value;
        let mut levels = 0;
        while let Kind::Group(inner) = &current.kind { current = inner; levels += 1; }
        assert_eq!(levels, 16_384);
        assert!(matches!(current.kind, Kind::Named(_)));
        assert_eq!(crate::analysis::resolve_value_type(current, false, &names, &unions).unwrap(), expected);
        drop(value);
        let mut nullable = parse("T|null").unwrap();
        for _ in 0..16_384 {
            let span = nullable.span;
            nullable = ValueAnnotation { kind: Kind::Group(Box::new(nullable)), span };
        }
        assert!(may_defer(&nullable));
        drop(nullable);
        // The defer classifier deliberately leaves callable constraints to callback checking.
        assert!(!may_defer(&parse("(value: T|null) => int").unwrap()));
    }

    #[test]
    fn materialization_failure_restores_parents_and_keeps_first_error_order() {
        let mut program = crate::syntax::parse("type Box<T>={value:T;};function observe(value:Box<int>){}function main(){}").unwrap();
        crate::generic_records::specialize(&mut program).unwrap();
        let names = program.records.iter().enumerate().map(|(id, record)| (record.name.text.clone(), id)).collect();
        let unions = crate::unions::UnionArena::default();
        let types = Types { callable_annotations: None, inline_shapes: None, callable_shapes: None, records: &program.records, names: &names, unions: &unions };
        let mut value = parse("[Box<int>, Missing<Box<int>>, Box<int>]").unwrap();
        let mut expected = value.clone();
        let Kind::Tuple(members) = &mut expected.kind else { unreachable!() };
        types.materialize_names(&mut members[0]).unwrap();
        let error_span = members[1].span;
        let Kind::Applied { arguments, .. } = &mut members[1].kind else { unreachable!() };
        types.materialize_names(&mut arguments[0]).unwrap();
        for _ in 0..16_384 {
            let span = value.span;
            value = ValueAnnotation { kind: Kind::Group(Box::new(value)), span };
            expected = ValueAnnotation { kind: Kind::Group(Box::new(expected)), span };
        }
        let error = types.materialize_names(&mut value).unwrap_err();
        assert_eq!(error.code, "type");
        assert_eq!(error.span, error_span);
        assert_eq!(value, expected);
    }

    #[test]
    fn deep_union_array_reconstruction_uses_normal_stack_and_production_drop() {
        let unions = crate::unions::UnionArena::default();
        let records = Vec::new();
        let names = HashMap::new();
        let types = Types { callable_shapes: None, callable_annotations: None, inline_shapes: None, records: &records, names: &names, unions: &unions };
        let span = Span::new(7, 13);
        let mut ty = Type::Int;
        for _ in 0..16_384 {
            ty = unions.intern([Type::Array(crate::syntax::ArrayElement::from_value_type(ty).unwrap()), Type::Null], span).unwrap();
        }
        let annotation = types.annotation(ty, span).unwrap();
        let mut current = &annotation;
        let mut levels = 0;
        while let Kind::Union(members) = &current.kind {
            assert_eq!(current.span, span);
            let element = members.iter().find_map(|member| if let Kind::Array(element) = &member.kind { Some(element) } else { None }).unwrap();
            current = element;
            levels += 1;
        }
        assert_eq!(levels, 16_384);
        assert!(matches!(&current.kind, Kind::Named(name) if name.text == "int"));
        drop(annotation);
    }

    #[test]
    fn recursive_annotations_use_the_reserved_declaration_without_expansion() {
        let program=crate::syntax::parse("type RecursiveValue={};function main(){}").unwrap();
        let names=HashMap::from([("RecursiveValue".into(),0)]);
        let unions=crate::unions::UnionArena::default();
        let root=unions.reserve_recursive(Span::default()).unwrap();
        let Type::Union(id)=root else {unreachable!()};
        let array=Type::Array(crate::syntax::ArrayElement::from_value_type(root).unwrap());
        unions.bind_recursive(id,vec![array],Span::default()).unwrap();
        let types=Types{callable_annotations:None,inline_shapes:None,callable_shapes:None,records:&program.records,names:&names,unions:&unions};
        assert!(types.annotation(root,Span::default()).is_err());
        unions.name_recursive(id,0,Span::default()).unwrap();
        let annotation=types.annotation(array,Span::default()).unwrap();
        let Kind::Array(child)=annotation.into_kind() else {panic!("array annotation")};
        let Kind::Named(name)=child.into_kind() else {panic!("named recursive element")};
        assert_eq!(name.text,"RecursiveValue");
        assert_eq!(unions.named_type(0),root);
        let resumed=crate::unions::UnionArena::from_snapshot_with_records(unions.snapshot(),&program.records);
        assert_eq!(resumed.named_type(0),root);
        assert!(resumed.name_recursive(id,1,Span::default()).is_err());
    }

    #[test]
    fn checked_nominal_record_and_array_constraints_bind_without_display_parsing() {
        let mut program=crate::syntax::parse("type Box<T>={value:T;};type Other<T>={value:T;};function observe(value:Box<Box<int[]>>,other:Other<int>){}function main(){}").unwrap();
        crate::generic_records::specialize(&mut program).unwrap();
        let names = program
            .records
            .iter()
            .enumerate()
            .map(|(id, record)| (record.name.text.clone(), id))
            .collect();
        let unions = crate::unions::UnionArena::default();
        let types = Types { callable_annotations: None,
            inline_shapes: None,
            callable_shapes: None,
            records: &program.records,
            names: &names,
            unions: &unions,
        };
        let actual = crate::analysis::resolve_value_type(
            &program.functions[0].params[0].1,
            false,
            &names,
            &unions,
        )
        .unwrap();
        let variables = BTreeSet::from(["T".into()]);
        let mut bindings = HashMap::new();
        types
            .unify(
                &parse("Box<Box<T[]>>").unwrap(),
                actual,
                &variables,
                &mut bindings,
            )
            .unwrap();
        assert_eq!(bindings["T"], Type::Int);
        let other = crate::analysis::resolve_value_type(
            &program.functions[0].params[1].1,
            false,
            &names,
            &unions,
        )
        .unwrap();
        assert_eq!(
            types
                .unify(
                    &parse("Box<T>").unwrap(),
                    other,
                    &variables,
                    &mut HashMap::new()
                )
                .unwrap_err()
                .code,
            "type"
        );
    }
    #[test]
    fn repeated_variables_require_one_checked_identity_and_unions_keep_their_tag() {
        let names = HashMap::new();
        let unions = crate::unions::UnionArena::default();
        let types = Types { callable_annotations: None,
            inline_shapes: None,
            callable_shapes: None,
            records: &[],
            names: &names,
            unions: &unions,
        };
        let variables = BTreeSet::from(["T".into()]);
        let mut bindings = HashMap::new();
        let pattern = parse("T").unwrap();
        types
            .unify(&pattern, Type::Int, &variables, &mut bindings)
            .unwrap();
        assert_eq!(
            types
                .unify(&pattern, Type::String, &variables, &mut bindings)
                .unwrap_err()
                .code,
            "type"
        );
        let union = crate::analysis::resolve_value_type(
            &parse("int|null").unwrap(),
            false,
            &names,
            &unions,
        )
        .unwrap();
        let mut bindings = HashMap::from([("T".into(), Type::Int)]);
        types
            .unify(&parse("T|null").unwrap(), union, &variables, &mut bindings)
            .unwrap();
        let mut bindings = HashMap::new();
        types
            .unify(&pattern, union, &variables, &mut bindings)
            .unwrap();
        assert_eq!(bindings["T"], union);
    }
}

#[cfg(test)]
mod nullable_tests {
    use super::*;
    use crate::type_syntax::parse;
    #[test]
    fn nullable_patterns_infer_payloads_without_erasing_other_absent_variants() {
        let names = HashMap::new();
        let unions = crate::unions::UnionArena::default();
        let types = Types { callable_annotations: None,
            inline_shapes: None,
            callable_shapes: None,
            records: &[],
            names: &names,
            unions: &unions,
        };
        let variables = BTreeSet::from(["T".into()]);
        for (actual, expected) in [
            ("int|null", "int"),
            ("int|null|undefined", "int|undefined"),
            ("int", "int"),
        ] {
            let actual = crate::analysis::resolve_value_type(
                &parse(actual).unwrap(),
                false,
                &names,
                &unions,
            )
            .unwrap();
            let expected = crate::analysis::resolve_value_type(
                &parse(expected).unwrap(),
                false,
                &names,
                &unions,
            )
            .unwrap();
            let mut bindings = HashMap::new();
            types
                .unify(&parse("T|null").unwrap(), actual, &variables, &mut bindings)
                .unwrap();
            assert_eq!(bindings["T"], expected);
        }
        let mut bindings = HashMap::new();
        assert_eq!(
            types
                .unify(
                    &parse("T|null").unwrap(),
                    Type::Null,
                    &variables,
                    &mut bindings
                )
                .unwrap_err()
                .code,
            "generic-inference"
        );
        let mut bindings = HashMap::from([("T".into(), Type::Int)]);
        types
            .unify(
                &parse("T|null").unwrap(),
                Type::Null,
                &variables,
                &mut bindings,
            )
            .unwrap();
        assert_eq!(bindings["T"], Type::Int);
        assert_eq!(
            types
                .unify(
                    &parse("T|null").unwrap(),
                    Type::String,
                    &variables,
                    &mut bindings
                )
                .unwrap_err()
                .code,
            "type"
        );
    }
    #[test]
    fn contextual_constraints_defer_absent_arguments_and_leave_unknown_hints_unknown() {
        let names = HashMap::new();
        let unions = crate::unions::UnionArena::default();
        let types = Types { callable_annotations: None,
            inline_shapes: None,
            callable_shapes: None,
            records: &[],
            names: &names,
            unions: &unions,
        };
        let infer = |source: &str, hints: &[Option<Type>], expected| {
            let program = crate::syntax::parse(source).unwrap();
            let function = &program.functions[0];
            let parameters = function
                .params
                .iter()
                .map(|(_, annotation, _)| annotation.clone())
                .collect::<Vec<_>>();
            let result = function.result.as_ref().and_then(|result| match result {
                crate::syntax::ResultAnnotation::Named(value) => Some(value.as_ref()),
                _ => None,
            });
            types.solve(
                Constraints {
                    variables: &function.type_parameters,
                    parameters: &parameters,
                    arguments: hints,
                    result,
                    expected,
                    span: function.name.span,
                },
                &mut HashMap::new(),
            )
        };
        let choose = "function choose<T>(left:T|null,right:T):T{return right;}";
        assert_eq!(
            infer(choose, &[Some(Type::Null), Some(Type::Int)], None).unwrap(),
            vec![Type::Int]
        );
        assert_eq!(
            infer(choose, &[Some(Type::Null), None], Some(Type::String)).unwrap(),
            vec![Type::String]
        );
        assert_eq!(
            infer(choose, &[Some(Type::Null), None], None)
                .unwrap_err()
                .code,
            "generic-inference"
        );
        let identity = "function identity<T>(value:T):T{return value;}";
        assert_eq!(
            infer(identity, &[None], None).unwrap_err().code,
            "generic-inference"
        );
        assert_eq!(
            infer(identity, &[None], Some(Type::Int)).unwrap(),
            vec![Type::Int]
        );
        assert_eq!(
            infer(identity, &[Some(Type::String)], Some(Type::Int))
                .unwrap_err()
                .code,
            "type"
        );
        assert_eq!(infer(identity, &[], None).unwrap_err().code, "arity");
        let array = Type::Array(crate::syntax::ArrayElement::Int(0));
        assert_eq!(
            infer(
                "function items<T>(values:T[]):T[]{return values;}",
                &[None],
                Some(array)
            )
            .unwrap(),
            vec![Type::Int]
        );
        assert_eq!(
            infer("function phantom<T>():int{return 7;}", &[], Some(Type::Int))
                .unwrap_err()
                .code,
            "generic-inference"
        );
        let nullable = unions
            .intern([Type::Int, Type::Null], Span::new(0, 0))
            .unwrap();
        assert_eq!(
            infer(identity, &[Some(nullable)], None).unwrap(),
            vec![nullable]
        );
    }
}
