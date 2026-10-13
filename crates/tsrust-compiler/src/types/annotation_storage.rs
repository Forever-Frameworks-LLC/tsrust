//! Owned annotation trees clone and release through heap continuations.
use super::syntax::{CallableParameter, ValueAnnotation, ValueAnnotationKind as Kind};

impl ValueAnnotation {
    /// Extract an owned variant without recursively dropping its children.
    pub fn into_kind(mut self) -> Kind {
        std::mem::replace(&mut self.kind, Kind::Tuple(Vec::new()))
    }

    pub(super) fn nesting(&self) -> usize {
        let mut pending = vec![(self, 0)];
        let mut deepest = 0;
        while let Some((value, depth)) = pending.pop() {
            let children = depth + 1;
            let level = match &value.kind {
                Kind::Literal(_) | Kind::Named(_) | Kind::Polymorphic { .. } => depth,
                Kind::Array(inner) | Kind::Group(inner) => { pending.push((inner, children)); children }
                Kind::Applied { arguments, .. } | Kind::Union(arguments) | Kind::Tuple(arguments) => {
                    pending.extend(arguments.iter().map(|value| (value, children))); children
                }
                Kind::Callable { parameters, result, .. } => {
                    pending.extend(parameters.iter().map(|parameter| (&parameter.annotation, children)));
                    pending.push((result, children)); children
                }
            };
            deepest = deepest.max(level);
        }
        deepest
    }
}

impl Clone for ValueAnnotation {
    fn clone(&self) -> Self {
        let leaf = match &self.kind {
            Kind::Literal(value) => Some(Kind::Literal(value.clone())),
            Kind::Named(name) => Some(Kind::Named(name.clone())),
            Kind::Polymorphic { origin, declaration } => Some(Kind::Polymorphic { origin: *origin, declaration: declaration.clone() }),
            _ => None,
        };
        if let Some(kind) = leaf { return Self { kind, span: self.span }; }
        enum Work<'a> { Visit(&'a ValueAnnotation), Finish(&'a ValueAnnotation, usize) }
        let mut pending = vec![Work::Visit(self)];
        let mut results: Vec<ValueAnnotation> = Vec::new();
        while let Some(work) = pending.pop() {
            match work {
                Work::Visit(value) => {
                    pending.push(Work::Finish(value, results.len()));
                    let start = pending.len();
                    match &value.kind {
                        Kind::Literal(_) | Kind::Named(_) | Kind::Polymorphic { .. } => {}
                        Kind::Array(inner) | Kind::Group(inner) => pending.push(Work::Visit(inner)),
                        Kind::Applied { arguments, .. } | Kind::Union(arguments) | Kind::Tuple(arguments) => pending.extend(arguments.iter().map(Work::Visit)),
                        Kind::Callable { parameters, result, .. } => {
                            pending.extend(parameters.iter().map(|parameter| Work::Visit(&parameter.annotation)));
                            pending.push(Work::Visit(result));
                        }
                    }
                    pending[start..].reverse();
                }
                Work::Finish(value, start) => {
                    let kind = match &value.kind {
                        Kind::Literal(literal) => Kind::Literal(literal.clone()),
                        Kind::Named(name) => Kind::Named(name.clone()),
                        Kind::Polymorphic { origin, declaration } => Kind::Polymorphic { origin: *origin, declaration: declaration.clone() },
                        Kind::Array(_) => Kind::Array(Box::new(results.pop().expect("array annotation"))),
                        Kind::Group(_) => Kind::Group(Box::new(results.pop().expect("group annotation"))),
                        Kind::Applied { name, .. } => Kind::Applied { name: name.clone(), arguments: results.drain(start..).collect() },
                        Kind::Union(_) => Kind::Union(results.drain(start..).collect()),
                        Kind::Tuple(_) => Kind::Tuple(results.drain(start..).collect()),
                        Kind::Callable { ownership, parameters, .. } => {
                            let result = Box::new(results.pop().expect("callable result annotation"));
                            let parameters = parameters.iter().zip(results.drain(start..)).map(|(parameter, annotation)| CallableParameter {
                                rest: parameter.rest, name: parameter.name.clone(), access: parameter.access.clone(),
                                omission: parameter.omission, annotation, span: parameter.span,
                            }).collect();
                            Kind::Callable { ownership: *ownership, parameters, result }
                        }
                    };
                    results.push(ValueAnnotation { kind, span: value.span });
                }
            }
        }
        results.pop().expect("root annotation clone")
    }
}

impl Drop for ValueAnnotation {
    fn drop(&mut self) {
        fn children(kind: Kind, pending: &mut Vec<ValueAnnotation>) {
            match kind {
                Kind::Literal(_) | Kind::Named(_) | Kind::Polymorphic { .. } => {}
                Kind::Array(inner) | Kind::Group(inner) => pending.push(*inner),
                Kind::Applied { arguments, .. } | Kind::Union(arguments) | Kind::Tuple(arguments) => pending.extend(arguments),
                Kind::Callable { parameters, result, .. } => {
                    pending.extend(parameters.into_iter().map(|parameter| parameter.annotation));
                    pending.push(*result);
                }
            }
        }
        // Empty/leaf drops do not allocate, including the detached placeholders
        // left by into_kind. Every queued value has its children removed first.
        let mut pending = Vec::new();
        children(std::mem::replace(&mut self.kind, Kind::Tuple(Vec::new())), &mut pending);
        while let Some(value) = pending.pop() { children(value.into_kind(), &mut pending); }
    }
}

impl PartialEq for ValueAnnotation {
    fn eq(&self, other: &Self) -> bool {
        if self.span != other.span { return false; }
        match &self.kind {
            Kind::Named(left) => return matches!(&other.kind, Kind::Named(right) if left == right),
            Kind::Literal(left) => return matches!(&other.kind, Kind::Literal(right) if left == right),
            Kind::Polymorphic { origin: a, declaration: left } => return matches!(&other.kind, Kind::Polymorphic { origin: b, declaration: right } if a == b && left == right),
            _ => {}
        }
        let mut pending = vec![(self, other)];
        while let Some((left, right)) = pending.pop() {
            if left.span != right.span { return false; }
            match (&left.kind, &right.kind) {
                (Kind::Named(left), Kind::Named(right)) if left == right => {}
                (Kind::Literal(left), Kind::Literal(right)) if left == right => {}
                (Kind::Polymorphic { origin: a, declaration: left }, Kind::Polymorphic { origin: b, declaration: right }) if a == b && left == right => {}
                (Kind::Array(left), Kind::Array(right)) | (Kind::Group(left), Kind::Group(right)) => pending.push((left, right)),
                (Kind::Applied { name: a, arguments: left }, Kind::Applied { name: b, arguments: right }) if a == b && left.len() == right.len() => pending.extend(left.iter().zip(right)),
                (Kind::Union(left), Kind::Union(right)) | (Kind::Tuple(left), Kind::Tuple(right)) if left.len() == right.len() => pending.extend(left.iter().zip(right)),
                (Kind::Callable { ownership: a, parameters: left, result: x }, Kind::Callable { ownership: b, parameters: right, result: y }) if a == b && left.len() == right.len() => {
                    for (left, right) in left.iter().zip(right) {
                        if left.rest != right.rest || left.name != right.name || left.access != right.access || left.omission != right.omission || left.span != right.span { return false; }
                        pending.push((&left.annotation, &right.annotation));
                    }
                    pending.push((x, y));
                }
                _ => return false,
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_syntax::{parse, CallableOwnership, TypeName};
    use crate::Span;

    #[test]
    fn deep_annotations_clone_compare_measure_and_drop_on_normal_stack() {
        let span = Span::new(11, 19);
        let mut value = ValueAnnotation { kind: Kind::Named(TypeName { text: "int".into(), span }), span };
        for depth in 0..16_384 {
            let kind = match depth % 6 {
                0 => Kind::Array(Box::new(value)),
                1 => Kind::Group(Box::new(value)),
                2 => Kind::Union(vec![value]),
                3 => Kind::Tuple(vec![value]),
                4 => Kind::Applied { name: TypeName { text: "Box".into(), span }, arguments: vec![value] },
                _ if depth % 12 == 5 => Kind::Callable { ownership: CallableOwnership::OwnedOnce, parameters: vec![], result: Box::new(value) },
                _ => Kind::Callable {
                    ownership: CallableOwnership::Read,
                    parameters: vec![CallableParameter {
                        rest: false, name: TypeName { text: "input".into(), span }, access: None,
                        omission: crate::ParameterOmission::Required, annotation: value, span,
                    }],
                    result: Box::new(ValueAnnotation { kind: Kind::Named(TypeName { text: "void".into(), span }), span }),
                },
            };
            value = ValueAnnotation { kind, span };
        }
        let copy = value.clone();
        assert!(copy == value);
        assert_eq!(copy.nesting(), 16_384);
        let mut different = copy.clone();
        different.span = Span::new(12, 19);
        assert!(different != value);
        // Ordinary production Drop, including both success and differing trees.
        drop(value); drop(copy); drop(different);
    }

    #[test]
    fn cloning_retains_every_callable_and_literal_coordinate() {
        for source in ["Once<Function<string,[head:ReadOnly<int[]>,...tail:string[]]>>", "Function<void,[value?:Mutable<Box<string>>]>", "['text',9007199254740993,-1.5,true]", "(int|null)[]"] {
            let value = parse(source).unwrap();
            let copy = value.clone();
            assert_eq!(copy, value);
            assert_eq!(format!("{copy:?}"), format!("{value:?}"));
        }
        let span = Span::new(11, 19);
        let value = ValueAnnotation { kind: Kind::Polymorphic { origin: 19, declaration: TypeName { text: "select".into(), span } }, span };
        assert_eq!(value.clone(), value);
    }

    #[test]
    fn equality_distinguishes_callable_metadata_and_child_order() {
        let value = parse("Function<int,[value:ReadOnly<int[]>]>").unwrap();
        for field in 0..8 {
            let mut changed = value.clone();
            let Kind::Callable { ownership, parameters, result } = &mut changed.kind else { panic!("callable") };
            let parameter = &mut parameters[0];
            match field {
                0 => *ownership = CallableOwnership::OwnedOnce,
                1 => parameter.rest = true,
                2 => parameter.name.text = "other".into(),
                3 => parameter.access = None,
                4 => parameter.span = Span::new(0, 0),
                5 => parameter.annotation.span = Span::new(0, 0),
                6 => parameter.omission = crate::ParameterOmission::Optional,
                _ => result.span = Span::new(0, 0),
            }
            assert!(changed != value, "metadata field {field}");
        }
        let value = parse("int|string").unwrap();
        let mut changed = value.clone();
        let Kind::Union(members) = &mut changed.kind else { panic!("union") };
        members.reverse();
        assert!(changed != value);
    }
}
