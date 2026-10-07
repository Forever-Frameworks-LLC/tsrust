//! Constraints over checked nominal types. This never checks/evaluates expressions.
use crate::syntax::{Record, Type};
use crate::type_syntax::{ValueAnnotation, ValueAnnotationKind as Kind};
use crate::{Diagnostic, Span};
use std::collections::{BTreeSet, HashMap};

pub(crate) type CallableShapes = HashMap<Type, (Vec<Type>, Type)>;
pub(crate) type InlineShapes = HashMap<(usize, usize), (Vec<Type>, Type)>;

pub(crate) struct Types<'a> {
    pub callable_shapes: Option<&'a CallableShapes>,
    pub inline_shapes: Option<&'a InlineShapes>,
    pub records: &'a [Record],
    pub names: &'a HashMap<String, usize>,
    pub unions: &'a crate::unions::UnionArena,
}
fn may_defer(pattern: &ValueAnnotation) -> bool {
    match &pattern.kind {
        Kind::Union(_) => true,
        Kind::Group(inner) | Kind::Array(inner) => may_defer(inner),
        Kind::Applied { arguments, .. } | Kind::Tuple(arguments) => arguments.iter().any(may_defer),
        Kind::Named(_) | Kind::Callable { .. } => false,
    }
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
        if let (Some(pattern), Some(expected)) = (request.result, request.expected) {
            self.unify(pattern, expected, &variables, bindings)?;
        }
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
        let mismatch = || {
            Diagnostic::new(
                "type",
                "Generic type constraints do not match the checked argument type.",
                pattern.span,
            )
        };
        match &pattern.kind {
            Kind::Named(name) if variables.contains(&name.text) => {
                if let Some(prior) = bindings.get(&name.text) {
                    if *prior != actual {
                        return Err(mismatch());
                    }
                } else {
                    bindings.insert(name.text.clone(), actual);
                }
                Ok(())
            }
            Kind::Named(name) if name.text == "void" && actual == Type::Void => Ok(()),
            Kind::Named(_) => {
                let expected =
                    crate::analysis::resolve_value_type(pattern, false, self.names, self.unions)?;
                if expected == actual {
                    Ok(())
                } else {
                    Err(mismatch())
                }
            }
            Kind::Group(inner) => self.unify(inner, actual, variables, bindings),
            Kind::Array(inner) => {
                let Type::Array(element) = actual else {
                    return Err(mismatch());
                };
                self.unify(inner, element.ty(), variables, bindings)
            }
            Kind::Tuple(positions) => {
                let Type::Record(id) = actual else { return Err(mismatch()); };
                let record = &self.records[id];
                if !record.is_tuple() || record.fields.len() != positions.len() { return Err(mismatch()); }
                for (position, field) in positions.iter().zip(&record.fields) {
                    let ty = field.ty.unwrap_or(crate::analysis::resolve_value_type(&field.annotation, false, self.names, self.unions)?);
                    self.unify(position, ty, variables, bindings)?;
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
                for (pattern, argument) in arguments.iter().zip(&application.arguments) {
                    let actual = crate::analysis::resolve_value_type(
                        argument,
                        false,
                        self.names,
                        self.unions,
                    )?;
                    self.unify(pattern, actual, variables, bindings)?;
                }
                Ok(())
            }
            Kind::Union(_) => {
                let mut pending = vec![pattern];
                let mut unbound = false;
                while let Some(value) = pending.pop() {
                    match &value.kind {
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
                    let mut pending = vec![pattern];
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
                    return self.unify(unresolved[0], payload, variables, bindings);
                }
                // Fully known constraints admit checked absent/payload variants.
                let mut value = pattern.clone();
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
                    for member in members {
                        if matches!(member, Type::Null | Type::Undefined) {
                            if !absent.contains(&member) {
                                return Err(mismatch());
                            }
                        } else {
                            self.unify(callable, member, variables, bindings)?;
                        }
                    }
                    return Ok(());
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
                self.unify_callable(
                    pattern,
                    actual_parameters,
                    *actual_result,
                    variables,
                    bindings,
                )
            }
        }
    }
    fn unify_callable(
        &self,
        pattern: &ValueAnnotation,
        actual_parameters: &[Type],
        actual_result: Type,
        variables: &BTreeSet<String>,
        bindings: &mut HashMap<String, Type>,
    ) -> Result<(), Diagnostic> {
        match &pattern.kind {
            Kind::Group(inner) => {
                self.unify_callable(inner, actual_parameters, actual_result, variables, bindings)
            }
            Kind::Callable {
                parameters, result, ..
            } => {
                if parameters.len() != actual_parameters.len() {
                    return Err(Diagnostic::new(
                        "type",
                        "Generic callable parameter count does not match the argument signature.",
                        pattern.span,
                    ));
                }
                for (parameter, actual) in parameters.iter().zip(actual_parameters) {
                    // Optional callback inputs add undefined at the call boundary;
                    // that omission does not constrain the declared payload variable.
                    let actual = if parameter.omission.is_omittable() {
                        match actual {
                            Type::Union(id) => {
                                let members = self.unions.members(*id).into_iter().filter(|ty| *ty != Type::Undefined).collect::<Vec<_>>();
                                if members.len() == 1 { members[0] } else { *actual }
                            }
                            _ => *actual,
                        }
                    } else { *actual };
                    self.unify(&parameter.annotation, actual, variables, bindings)?;
                }
                self.unify(result, actual_result, variables, bindings)
            }
            _ => Ok(()),
        }
    }

    fn materialize_names(&self, value: &mut ValueAnnotation) -> Result<(), Diagnostic> {
        match &mut value.kind {
            Kind::Applied { name, arguments } => {
                for argument in arguments.iter_mut() {
                    self.materialize_names(argument)?;
                }
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
            Kind::Array(inner) | Kind::Group(inner) => self.materialize_names(inner)?,
            Kind::Union(members) | Kind::Tuple(members) => {
                for member in members {
                    self.materialize_names(member)?;
                }
            }
            Kind::Callable { .. } | Kind::Named(_) => {}
        }
        Ok(())
    }
    pub(crate) fn annotation(&self, ty: Type, span: Span) -> Result<ValueAnnotation, Diagnostic> {
        let kind = match ty {
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
            Type::Array(element) => Kind::Array(Box::new(self.annotation(element.ty(), span)?)),
            Type::Union(id) => Kind::Union(
                self.unions
                    .members(id)
                    .iter()
                    .map(|ty| self.annotation(*ty, span))
                    .collect::<Result<_, _>>()?,
            ),
            _ => {
                return Err(Diagnostic::new(
                    "unsupported",
                    "Generic argument is not a supported value type.",
                    span,
                ));
            }
        };
        Ok(ValueAnnotation { kind, span })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_syntax::parse;
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
        let types = Types {
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
        let types = Types {
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
        let types = Types {
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
        let types = Types {
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
