//! Flat canonical keys keep generic instance lookup independent of tree depth.
use super::*;
use crate::Mode;
use crate::type_syntax::LiteralAnnotation;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct Key(Vec<Token>);
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Token {
    Polymorphic(usize, String), Named(String), Literal(LiteralAnnotation),
    Array, Applied(String, usize), Union(usize), Tuple(usize),
    Callable(CallbackKind, usize), Parameter(bool, Mode, bool),
}

// Construction IDs are local to one key. Only the final prefix token stream is
// retained, so unrelated arena insertion order cannot affect cache identity.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Node {
    Polymorphic(usize, String), Named(String), Literal(LiteralAnnotation),
    Array(usize), Applied(String, Vec<usize>), Union(Vec<usize>), Tuple(Vec<usize>),
    Callable(CallbackKind, Vec<(bool, Mode, bool, usize)>, usize),
}
#[derive(Default)]
struct Arena { nodes: Vec<Node>, ids: HashMap<Node, usize> }
impl Arena {
    fn intern(&mut self, node: Node) -> usize {
        if let Some(id) = self.ids.get(&node) { return *id; }
        let id = self.nodes.len();
        self.ids.insert(node.clone(), id);
        self.nodes.push(node);
        id
    }
    fn union(&mut self, mut members: Vec<usize>) -> usize {
        members.sort_by_key(|id| match &self.nodes[*id] {
            Node::Named(name) if name == "null" => 1,
            Node::Named(name) if name == "undefined" => 2,
            _ => 0,
        });
        // Preserve the existing adjacent-only deduplication rule after stable
        // absent-member sorting; nonadjacent payload alternatives remain distinct.
        members.dedup();
        if members.len() == 1 { members[0] } else { self.intern(Node::Union(members)) }
    }
    fn serialize(&self, root: usize) -> Key {
        enum Work { Node(usize), Parameter(bool, Mode, bool) }
        let mut pending = vec![Work::Node(root)];
        let mut output = Vec::new();
        while let Some(work) = pending.pop() {
            let id = match work {
                Work::Parameter(rest, mode, optional) => { output.push(Token::Parameter(rest, mode, optional)); continue; }
                Work::Node(id) => id,
            };
            match &self.nodes[id] {
                Node::Polymorphic(origin, name) => output.push(Token::Polymorphic(*origin, name.clone())),
                Node::Named(name) => output.push(Token::Named(name.clone())),
                Node::Literal(value) => output.push(Token::Literal(value.clone())),
                Node::Array(child) => { output.push(Token::Array); pending.push(Work::Node(*child)); }
                Node::Applied(name, children) => {
                    output.push(Token::Applied(name.clone(), children.len()));
                    pending.extend(children.iter().rev().copied().map(Work::Node));
                }
                Node::Union(children) | Node::Tuple(children) => {
                    output.push(if matches!(&self.nodes[id], Node::Union(_)) { Token::Union(children.len()) } else { Token::Tuple(children.len()) });
                    pending.extend(children.iter().rev().copied().map(Work::Node));
                }
                Node::Callable(kind, parameters, result) => {
                    output.push(Token::Callable(*kind, parameters.len()));
                    pending.push(Work::Node(*result));
                    for &(rest, mode, optional, child) in parameters.iter().rev() {
                        pending.push(Work::Node(child));
                        pending.push(Work::Parameter(rest, mode, optional));
                    }
                }
            }
        }
        Key(output)
    }
}

pub(super) fn key(value: &ValueAnnotation) -> Result<Key, Diagnostic> {
    enum Work<'a> { Visit(&'a ValueAnnotation), Finish(&'a ValueAnnotation, usize) }
    let mut pending = vec![Work::Visit(value)];
    let mut values = Vec::new();
    let mut arena = Arena::default();
    while let Some(work) = pending.pop() {
        match work {
            Work::Visit(value) => {
                let children = match &value.kind {
                    Kind::Polymorphic { origin, declaration } => { values.push(arena.intern(Node::Polymorphic(*origin, declaration.text.clone()))); continue; }
                    Kind::Named(name) => { values.push(arena.intern(Node::Named(name.text.clone()))); continue; }
                    Kind::Literal(literal) => { values.push(arena.intern(Node::Literal(literal.clone()))); continue; }
                    Kind::Group(inner) => { pending.push(Work::Visit(inner)); continue; }
                    Kind::Array(inner) => vec![inner.as_ref()],
                    Kind::Applied { arguments, .. } | Kind::Tuple(arguments) => arguments.iter().collect(),
                    Kind::Union(_) => {
                        // Flatten direct union/group boundaries before constructing
                        // children, avoiding repeated prefix copies in skewed unions.
                        let mut leaves = Vec::new();
                        let mut work = vec![value];
                        while let Some(value) = work.pop() {
                            match &value.kind {
                                Kind::Group(inner) => work.push(inner),
                                Kind::Union(members) => work.extend(members.iter().rev()),
                                _ => leaves.push(value),
                            }
                        }
                        leaves
                    }
                    Kind::Callable { parameters, result, .. } => parameters.iter().map(|parameter| &parameter.annotation).chain(std::iter::once(result.as_ref())).collect(),
                };
                pending.push(Work::Finish(value, values.len()));
                pending.extend(children.into_iter().rev().map(Work::Visit));
            }
            Work::Finish(value, start) => {
                let mut children = values.drain(start..).collect::<Vec<_>>();
                let node = match &value.kind {
                    Kind::Array(_) => Node::Array(children.pop().expect("array element")),
                    Kind::Applied { name, .. } => Node::Applied(name.text.clone(), children),
                    Kind::Tuple(_) => Node::Tuple(children),
                    Kind::Union(_) => { values.push(arena.union(children)); continue; }
                    Kind::Callable { parameters, .. } => {
                        let result = children.pop().expect("callable result");
                        let parameters = parameters.iter().zip(children).map(|(parameter, child)| (parameter.rest,
                            parameter.access.as_ref().map_or(Mode::Copy, |access| access.mode),
                            parameter.omission.is_omittable(), child)).collect();
                        Node::Callable(crate::callable_annotations::ownership(value).expect("callable ownership"), parameters, result)
                    }
                    _ => unreachable!("leaf keys finish immediately"),
                };
                values.push(arena.intern(node));
            }
        }
    }
    Ok(arena.serialize(values.pop().expect("root contract key")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hash::{Hash, Hasher};

    // Bounded differential oracle from the pre-worklist implementation.
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    enum Reference {
        Polymorphic(usize, String),
        Named(String),
        Literal(crate::type_syntax::LiteralAnnotation),
        Array(Box<Reference>),
        Applied(String, Vec<Reference>),
        Union(Vec<Reference>),
        Tuple(Vec<Reference>),
        Callable(CallbackKind, Vec<(bool, crate::Mode, bool, Reference)>, Box<Reference>),
    }
    fn reference(value: &ValueAnnotation) -> Result<Reference, Diagnostic> {
        Ok(match &value.kind {
            Kind::Polymorphic { origin, declaration } => Reference::Polymorphic(*origin,declaration.text.clone()),
            Kind::Named(name) => Reference::Named(name.text.clone()),
            Kind::Literal(value) => Reference::Literal(value.clone()),
            Kind::Group(inner) => return reference(inner),
            Kind::Array(inner) => Reference::Array(Box::new(reference(inner)?)),
            Kind::Applied { name, arguments } => Reference::Applied(name.text.clone(),arguments.iter().map(reference).collect::<Result<_,_>>()?),
            Kind::Tuple(members) => Reference::Tuple(members.iter().map(reference).collect::<Result<_,_>>()?),
            Kind::Union(members) => {
                let mut keys=Vec::new();
                for member in members {
                    match reference(member)? { Reference::Union(inner)=>keys.extend(inner), other=>keys.push(other) }
                }
                keys.sort_by_key(|key|match key { Reference::Named(name) if name=="null"=>1,Reference::Named(name) if name=="undefined"=>2,_=>0 });
                keys.dedup();
                if keys.len()==1 {return Ok(keys.remove(0));}
                Reference::Union(keys)
            }
            Kind::Callable { parameters, result, .. } => Reference::Callable(crate::callable_annotations::ownership(value).expect("callable ownership"),
                parameters.iter().map(|parameter| Ok((parameter.rest,
                    parameter.access.as_ref().map_or(crate::Mode::Copy, |access| access.mode),
                    parameter.omission.is_omittable(), reference(&parameter.annotation)?))).collect::<Result<_, Diagnostic>>()?,
                Box::new(reference(result)?)),
        })
    }

    #[test]
    fn flat_keys_preserve_canonical_contract_equivalence() {
        let bases = ["int", "string", "null", "undefined", "'ready'", "\"ready\"", "'done'",
            "int|string|int", "int|int|string", "(int|undefined)|(null|int)", "int|null|undefined",
            "[int,string]", "Box<int,string>", "Function<int,[value?:int,...tail:int[]]>",
            "Owned<(value:string)=>int>", "Once<(value:string)=>int>"];
        let mut corpus = Vec::new();
        for base in bases {
            for source in [base.to_owned(), format!("({base})"), format!("({base})[]"), format!("({base})|null"), format!("(({base})|null)|(undefined|({base}))")] {
                let annotation = crate::type_syntax::parse(&source).unwrap();
                corpus.push((source, key(&annotation).unwrap(), reference(&annotation).unwrap()));
            }
        }
        for (left_source, left, left_reference) in &corpus {
            for (right_source, right, right_reference) in &corpus {
                assert_eq!(left == right, left_reference == right_reference, "{left_source} versus {right_source}");
            }
        }
    }

    #[test]
    fn deep_keys_construct_clone_hash_compare_and_drop_on_normal_stack() {
        let span = Span::new(3, 8);
        let mut annotation = crate::type_syntax::parse("int").unwrap();
        for level in 0..16_384 {
            annotation = if level % 2 == 0 {
                ValueAnnotation { kind: Kind::Array(Box::new(annotation)), span }
            } else {
                ValueAnnotation { kind: Kind::Callable { ownership: crate::type_syntax::CallableOwnership::OwnedOnce,
                    parameters: Vec::new(), result: Box::new(annotation) }, span }
            };
        }
        let original = key(&annotation).unwrap();
        let copy = original.clone();
        assert_eq!(original, copy);
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        original.hash(&mut hasher);
        let mut other = std::collections::hash_map::DefaultHasher::new();
        copy.hash(&mut other);
        assert_eq!(hasher.finish(), other.finish());
        let mut cache = HashMap::new();
        cache.insert(original, 7);
        assert_eq!(cache.get(&copy), Some(&7));
        assert_ne!(copy, key(&crate::type_syntax::parse("int").unwrap()).unwrap());
        drop(cache);
        drop(copy);
        drop(annotation);
    }

    #[test]
    fn skewed_unions_flatten_before_constructing_intermediate_keys() {
        let span = Span::new(0, 1);
        let mut annotation = crate::type_syntax::parse("int").unwrap();
        for _ in 0..16_384 {
            annotation = ValueAnnotation { kind: Kind::Union(vec![annotation, crate::type_syntax::parse("null").unwrap()]), span };
        }
        assert_eq!(key(&annotation).unwrap(), key(&crate::type_syntax::parse("int|null").unwrap()).unwrap());
        drop(annotation);
    }
}
