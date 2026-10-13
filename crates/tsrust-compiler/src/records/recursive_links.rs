//! Nullable record edges that require uniquely owned indirection.
use crate::syntax::{Record, Type, UnionId};
use std::collections::HashSet;

/// A leaf reached through inline union tags. Arrays terminate this traversal.
pub(crate) struct InlinePayload {
    pub ty: Type,
    pub union: Option<UnionId>,
    pub nullable: bool,
    pub tags: usize,
}

pub(crate) fn inline_payloads(ty:Type,members:impl Fn(UnionId)->Vec<Type>)->Vec<InlinePayload> {
    let mut pending=vec![(ty,None,false,Vec::<UnionId>::new())];
    let mut result=Vec::new();
    while let Some((ty,parent,nullable,path))=pending.pop() {
        if let Type::Union(id)=ty {
            if path.contains(&id) { continue; }
            let alternatives=members(id);
            let nullable=nullable||alternatives.iter().any(|ty|matches!(ty,Type::Null|Type::Undefined));
            let mut path=path;path.push(id);
            pending.extend(alternatives.into_iter().map(|ty|(ty,Some(id),nullable,path.clone())));
        }else{
            result.push(InlinePayload{ty,union:parent,nullable,tags:path.len()});
        }
    }
    result
}

#[derive(Default)]
pub(crate) struct Index {
    boxed: HashSet<(usize, usize)>,
    targets: HashSet<usize>,
}
impl Index {
    pub(crate) fn new(records: &[Record], members: impl Fn(UnionId) -> Vec<Type>) -> Self {
        let mut edges = vec![Vec::new(); records.len()];
        let mut candidates = Vec::new();
        for (parent, record) in records.iter().enumerate() {
            for field in &record.fields {
                let Some(ty) = field.ty else { continue; };
                for payload in inline_payloads(ty,&members) {
                    // Arrays already terminate inline layout. A nullable ancestor
                    // permits indirection at the union that stores this record.
                    if let Type::Record(child)=payload.ty && child<records.len() {
                        edges[parent].push(child);
                        if payload.nullable && let Some(union)=payload.union {
                            candidates.push((parent,union,child));
                        }
                    }
                }
            }
        }
        let mut reverse = vec![Vec::new(); records.len()];
        for (parent, children) in edges.iter().enumerate() {
            for &child in children { reverse[child].push(parent); }
        }
        let mut seen = vec![false; records.len()];
        let mut order = Vec::new();
        for root in 0..records.len() {
            if std::mem::replace(&mut seen[root], true) { continue; }
            let mut stack = vec![(root, 0)];
            while let Some((node, next)) = stack.last_mut() {
                if let Some(&child) = edges[*node].get(*next) {
                    *next += 1;
                    if !std::mem::replace(&mut seen[child], true) { stack.push((child, 0)); }
                } else { order.push(*node); stack.pop(); }
            }
        }
        let mut component = vec![usize::MAX; records.len()];
        for root in order.into_iter().rev() {
            if component[root] != usize::MAX { continue; }
            component[root] = root;
            let mut pending = vec![root];
            while let Some(child) = pending.pop() {
                for &parent in &reverse[child] {
                    if component[parent] == usize::MAX { component[parent] = root; pending.push(parent); }
                }
            }
        }
        let mut result = Self::default();
        for (parent, union, child) in candidates {
            if component[parent] == component[child] {
                result.boxed.insert((union.index(), child));
                result.targets.insert(child);
            }
        }
        result
    }
    pub(crate) fn target(&self, record: usize) -> bool { self.targets.contains(&record) }
    pub(crate) fn boxed(&self, union: UnionId, record: usize) -> bool {
        self.boxed.contains(&(union.index(), record))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cloneability::tests::record, syntax::ArrayElement, unions::UnionArena, Span};
    fn nullable(arena: &UnionArena, child: usize) -> Type {
        arena.intern([Type::Record(child), Type::Undefined], Span::new(0, 0)).unwrap()
    }
    #[test]
    fn self_link_boxes_only_the_recursive_payload() {
        let arena = UnionArena::default();
        let next = nullable(&arena, 0);
        let other = nullable(&arena, 1);
        let records = [record(&[next, other]), record(&[Type::Int])];
        let links = Index::new(&records, |id| arena.members(id));
        assert!(links.boxed(next.union().unwrap(), 0));
        assert!(!links.boxed(other.union().unwrap(), 1));
        assert!(!links.boxed(next.union().unwrap(), 1));
    }
    #[test]
    fn mutual_inline_cycle_boxes_nullable_edges_only() {
        let arena = UnionArena::default();
        let back = nullable(&arena, 0);
        let forward = nullable(&arena, 1);
        for first in [Type::Record(1), forward] {
            let records = [record(&[first]), record(&[back]), record(&[forward])];
            let links = Index::new(&records, |id| arena.members(id));
            assert!(links.boxed(back.union().unwrap(), 0));
            assert_eq!(links.boxed(forward.union().unwrap(), 1), first == forward);
        }
    }
    #[test]
    fn array_indirection_does_not_box_an_inline_nullable_field() {
        let arena = UnionArena::default();
        let forward = nullable(&arena, 1);
        let records = [record(&[forward]), record(&[Type::Array(ArrayElement::Record(0, 0))])];
        let links = Index::new(&records, |id| arena.members(id));
        assert!(!links.boxed(forward.union().unwrap(), 1));
    }
    #[test]
    fn nullable_union_boxes_only_members_that_close_a_cycle() {
        let arena = UnionArena::default();
        let next = arena.intern([Type::Record(0), Type::Record(1), Type::Null], Span::new(0, 0)).unwrap();
        let records = [record(&[next]), record(&[Type::Int])];
        let links = Index::new(&records, |id| arena.members(id));
        assert!(links.boxed(next.union().unwrap(), 0));
        assert!(!links.boxed(next.union().unwrap(), 1));
    }
    #[test]
    fn deep_mutual_schema_uses_an_iterative_graph_walk() {
        let arena = UnionArena::default();
        let count = 4096;
        let mut records = (0..count-1).map(|id| record(&[Type::Record(id+1)])).collect::<Vec<_>>();
        let back = nullable(&arena, 0);
        records.push(record(&[back]));
        assert!(Index::new(&records, |id| arena.members(id)).boxed(back.union().unwrap(), 0));
    }
}
