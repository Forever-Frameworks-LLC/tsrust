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
