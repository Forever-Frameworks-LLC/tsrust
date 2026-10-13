//! Compile-time recursion graph. Node identities never become runtime handles.
//!
//! Keep validation progress separate from storage termination: traversing an
//! object field consumes input structure, but a required self field still has no
//! finite value. Arrays can terminate empty; choices need one finite branch.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NodeId(pub usize);

#[derive(Clone, Debug)]
pub(crate) enum Shape {
    Leaf,
    Reference,
    Array(NodeId),
    Object(Vec<NodeId>),
    Choice(Vec<NodeId>),
    Adapter(NodeId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Invalid {
    MissingNode(NodeId),
    UnguardedReference(NodeId),
    NoFiniteValue(NodeId),
}

#[derive(Default)]
pub(crate) struct Definition {
    nodes: Vec<Shape>,
}

impl Definition {
    pub(crate) fn push(&mut self, shape: Shape) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(shape);
        id
    }

    /// Verify the reachable definition before generating recursive native types.
    /// Every path back to the root must descend into an input container. Then a
    /// least fixed point proves that the root can produce a finite value.
    pub(crate) fn validate(&self, root: NodeId) -> Result<(), Invalid> {
        let mut pending = vec![(root, false)];
        let mut visited = vec![[false; 2]; self.nodes.len()];
        while let Some((id, descended)) = pending.pop() {
            let Some(shape) = self.nodes.get(id.0) else {
                return Err(Invalid::MissingNode(id));
            };
            if visited[id.0][usize::from(descended)] { continue; }
            visited[id.0][usize::from(descended)] = true;
            match shape {
                Shape::Leaf => {},
                Shape::Reference if !descended => return Err(Invalid::UnguardedReference(id)),
                Shape::Reference => {},
                Shape::Array(child) => pending.push((*child, true)),
                Shape::Object(children) => pending.extend(children.iter().map(|child| (*child, true))),
                Shape::Choice(children) => pending.extend(children.iter().map(|child| (*child, descended))),
                Shape::Adapter(child) => pending.push((*child, descended)),
            }
        }
        let mut finite = vec![false; self.nodes.len()];
        loop {
            let mut changed = false;
            for (index, shape) in self.nodes.iter().enumerate() {
                if finite[index] || !visited[index].iter().any(|seen| *seen) { continue; }
                let terminates = match shape {
                    Shape::Leaf | Shape::Array(_) => true,
                    Shape::Reference => finite[root.0],
                    Shape::Object(children) => children.iter().all(|child| finite[child.0]),
                    Shape::Choice(children) => children.iter().any(|child| finite[child.0]),
                    Shape::Adapter(child) => finite[child.0],
                };
                if terminates { finite[index] = true; changed = true; }
            }
            if !changed { break; }
        }
        if finite[root.0] { Ok(()) } else { Err(Invalid::NoFiniteValue(root)) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrays_and_nullable_object_links_have_finite_values() {
        let mut definition = Definition::default();
        let reference = definition.push(Shape::Reference);
        let array = definition.push(Shape::Array(reference));
        assert_eq!(definition.validate(array), Ok(()));
        let null = definition.push(Shape::Leaf);
        let nullable = definition.push(Shape::Choice(vec![reference, null]));
        let object = definition.push(Shape::Object(vec![nullable, nullable]));
        assert_eq!(definition.validate(object), Ok(()));
        let adapted = definition.push(Shape::Adapter(object));
        assert_eq!(definition.validate(adapted), Ok(()));
    }

    #[test]
    fn a_terminal_union_branch_does_not_make_direct_self_progress() {
        let mut definition = Definition::default();
        let reference = definition.push(Shape::Reference);
        let leaf = definition.push(Shape::Leaf);
        let union = definition.push(Shape::Choice(vec![reference, leaf]));
        assert_eq!(definition.validate(union), Err(Invalid::UnguardedReference(reference)));
        let array = definition.push(Shape::Array(reference));
        let guarded = definition.push(Shape::Choice(vec![array, leaf]));
        assert_eq!(definition.validate(guarded), Ok(()));
    }

    #[test]
    fn required_self_fields_have_no_finite_value() {
        let mut definition = Definition::default();
        let reference = definition.push(Shape::Reference);
        let object = definition.push(Shape::Object(vec![reference]));
        assert_eq!(definition.validate(object), Err(Invalid::NoFiniteValue(object)));
    }

    #[test]
    fn shared_nodes_are_checked_with_each_progress_state() {
        let mut definition = Definition::default();
        let reference = definition.push(Shape::Reference);
        let array = definition.push(Shape::Array(reference));
        let union = definition.push(Shape::Choice(vec![reference, array]));
        assert_eq!(definition.validate(union), Err(Invalid::UnguardedReference(reference)));
    }

    #[test]
    fn unreachable_nodes_do_not_change_a_definition() {
        let mut definition = Definition::default();
        definition.push(Shape::Reference);
        definition.push(Shape::Array(NodeId(100)));
        let root = definition.push(Shape::Leaf);
        assert_eq!(definition.validate(root), Ok(()));
        assert_eq!(definition.validate(NodeId(100)), Err(Invalid::MissingNode(NodeId(100))));
    }
}


/// Early structural diagnostics use only linked namespace identity. Aliased local
/// schema values and arbitrary callback control flow remain the checker's job.
pub(super) fn check_source_call(expression: &crate::syntax::Expr) -> Result<(), crate::Diagnostic> {
    use crate::syntax::{ExprKind, StmtKind};
    let ExprKind::Call(callee, arguments) = &expression.kind else { return Ok(()); };
    if builder(callee) != Some("recursive") { return Ok(()); }
    let [callback] = arguments.as_slice() else {
        return Err(crate::Diagnostic::new("arity", "s.recursive expects one definition builder.", expression.span));
    };
    let ExprKind::Closure { params, body, .. } = &callback.kind else {
        // Callable identities require checked analysis, not spelling inference.
        return Ok(());
    };
    if params.len() != 1 || params[0].rest {
        return Err(crate::Diagnostic::new("schema-recursion", "A recursive definition builder expects one self parameter.", callback.span));
    }
    let Some(statement) = body.last() else { return Ok(()); };
    let StmtKind::Return(Some(output)) = &statement.kind else { return Ok(()); };
    let mut definition = Definition::default();
    let Some(root) = source_shape(&mut definition, output, &params[0].name.text) else { return Ok(()); };
    definition.validate(root).map_err(|error| crate::Diagnostic::new("schema-recursion", match error {
        Invalid::UnguardedReference(_) => "Recursive self references must descend through an input container before referring to the definition again.",
        Invalid::NoFiniteValue(_) => "This recursive schema has no finite value. Add a terminating alternative or use an array that can be empty.",
        Invalid::MissingNode(_) => "The recursive schema definition contains an unresolved reference.",
    }, output.span))
}

fn builder(expression: &crate::syntax::Expr) -> Option<&str> {
    use crate::syntax::ExprKind;
    let ExprKind::Field(base, method) = &expression.kind else { return None; };
    matches!(&base.kind, ExprKind::Name(name) if name == crate::native_exports::Namespace::Schema.symbol()).then_some(method)
}

fn source_shape(definition: &mut Definition, expression: &crate::syntax::Expr, self_name: &str) -> Option<NodeId> {
    use crate::syntax::ExprKind;
    if matches!(&expression.kind, ExprKind::Name(name) if name == self_name) {
        return Some(definition.push(Shape::Reference));
    }
    let ExprKind::Call(callee, arguments) = &expression.kind else { return None; };
    let shape = match builder(callee) {
        Some("string" | "int" | "number" | "boolean" | "null" | "literal" | "enum") => Shape::Leaf,
        Some("array") => {
            let [child] = arguments.as_slice() else { return None; };
            Shape::Array(source_shape(definition, child, self_name)?)
        },
        Some("object") => {
            let Some(fields) = arguments.first() else { return None; };
            let ExprKind::Record(fields) = &fields.kind else { return None; };
            Shape::Object(fields.iter().map(|(_, child)| source_shape(definition, child, self_name)).collect::<Option<Vec<_>>>()?)
        },
        Some("union") => {
            let [branches] = arguments.as_slice() else { return None; };
            let ExprKind::Array(branches) = &branches.kind else { return None; };
            Shape::Choice(branches.iter().map(|child| source_shape(definition, child, self_name)).collect::<Option<Vec<_>>>()?)
        },
        None => {
            let ExprKind::Field(base, method) = &callee.kind else { return None; };
            let child = source_shape(definition, base, self_name)?;
            match method.as_str() {
                "nullable" | "optional" if arguments.is_empty() => {
                    let absent = definition.push(Shape::Leaf);
                    Shape::Choice(vec![child, absent])
                },
                "min" | "max" | "gt" | "lt" | "refine" | "superRefine" | "strip" | "retain" => Shape::Adapter(child),
                _ => return None,
            }
        },
        _ => return None,
    };
    Some(definition.push(shape))
}

#[cfg(test)]
mod source_tests {
    use super::*;
    use crate::syntax::ExprKind;

    fn check(source: &str) -> Result<(), crate::Diagnostic> {
        let mut program = crate::syntax::parse(source)?;
        for function in &mut program.functions {
            crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |expression| {
                if matches!(&expression.kind, ExprKind::Name(name) if name == "s") {
                    expression.kind = ExprKind::Name(crate::native_exports::Namespace::Schema.symbol().into());
                }
                Ok(())
            })?;
            crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |expression| check_source_call(expression))?;
        }
        Ok(())
    }

    #[test]
    fn linked_definitions_reject_direct_and_required_self_cycles() {
        for definition in ["self", "s.union([self,s.int()])", "s.object({child:self})", "self.nullable()"] {
            let error = check(&format!("function main(){{s.recursive(self=>{definition});}}")).unwrap_err();
            assert_eq!(error.code, "schema-recursion");
        }
    }

    #[test]
    fn linked_definitions_accept_structurally_terminating_roots() {
        for definition in ["s.array(self)", "s.union([s.int(),s.array(self)])", "s.object({left:self.nullable(),right:self.nullable()})"] {
            check(&format!("function main(){{s.recursive(self=>{definition});}}")).unwrap();
        }
    }

    #[test]
    fn unrelated_recursive_methods_are_not_schema_calls() {
        check("function main(){other.recursive(self=>self);}").unwrap();
    }
}
