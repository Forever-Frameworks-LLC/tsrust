//! Finite lifetime boundaries for recursive records containing native byte views.
use crate::{native_return_borrows::BorrowAccess, syntax::{Record, Type, UnionId}};
use std::collections::HashSet;

pub(crate) struct Index {
    shared: Vec<Option<BorrowAccess>>,
    cyclic: Vec<bool>,
}

impl Index {
    pub(crate) fn new(records: &[Record], members: &impl Fn(UnionId) -> Vec<Type>) -> Self {
        Self::with_leaves(records, members, &|_| None)
    }

    pub(crate) fn with_storage(records: &[Record], members: &impl Fn(UnionId) -> Vec<Type>, storage: &crate::callable_storage::Registry) -> Self {
        Self::with_leaves(records, members, &|ty| {
            let Type::StoredCallable(family) = ty else { return None; };
            let layout = storage.scoped_native_family(family)?;
            layout.alternatives.iter().flat_map(|alternative| &alternative.slots).map(|slot| slot.access)
                .reduce(merge)
        })
    }

    fn with_leaves(records: &[Record], members: &impl Fn(UnionId) -> Vec<Type>, leaf: &impl Fn(Type) -> Option<BorrowAccess>) -> Self {
        let mut edges = vec![Vec::new(); records.len()];
        let mut incomplete = vec![false; records.len()];
        let mut access = vec![None; records.len()];
        for (id, record) in records.iter().enumerate() {
            access[id] = record.native_type.and_then(|native| crate::native_types::declaration(native).borrow_access());
            let mut pending = record.native_element().into_iter().collect::<Vec<_>>();
            for field in &record.fields {
                if let Some(ty) = field.ty { pending.push(ty); } else { incomplete[id] = true; }
            }
            let mut seen = HashSet::new();
            while let Some(ty) = pending.pop() {
                if !seen.insert(ty) { continue; }
                match ty {
                    Type::Record(child) if child < records.len() => edges[id].push(child),
                    Type::Record(_) => incomplete[id] = true,
                    Type::Array(element) => pending.push(element.ty()),
                    Type::Map(map) => { if let Some(map)=records.get(map).and_then(|record|record.map_definition.as_ref()) { pending.push(map.value); } },
                    Type::Union(union) | Type::BorrowedUnion(union) => pending.extend(members(union)),
                    _ => if let Some(borrow) = leaf(ty) {
                        access[id] = Some(access[id].map_or(borrow, |existing| merge(existing, borrow)));
                    }
                }
            }
            edges[id].sort_unstable();
            edges[id].dedup();
        }
        let mut reverse = vec![Vec::new(); records.len()];
        for (parent, children) in edges.iter().enumerate() {
            for &child in children { reverse[child].push(parent); }
        }
        // Propagate both access strength and incomplete shapes to all containers.
        let mut pending = (0..records.len()).filter(|&id| access[id].is_some() || incomplete[id]).collect::<Vec<_>>();
        while let Some(child) = pending.pop() {
            for &parent in &reverse[child] {
                let merged = match (access[parent], access[child]) {
                    (Some(BorrowAccess::Exclusive), _) | (_, Some(BorrowAccess::Exclusive)) => Some(BorrowAccess::Exclusive),
                    (Some(BorrowAccess::ReadOnly), _) | (_, Some(BorrowAccess::ReadOnly)) => Some(BorrowAccess::ReadOnly),
                    _ => None,
                };
                let unknown = incomplete[parent] || incomplete[child];
                if merged != access[parent] || unknown != incomplete[parent] {
                    access[parent] = merged;
                    incomplete[parent] = unknown;
                    pending.push(parent);
                }
            }
        }
        // Iterative Kosaraju traversal avoids stack growth on user type graphs.
        let mut seen = vec![false; records.len()];
        let mut order = Vec::new();
        for root in 0..records.len() {
            if std::mem::replace(&mut seen[root], true) { continue; }
            let mut stack = vec![(root, 0)];
            while let Some((node, next)) = stack.last_mut() {
                if let Some(&child) = edges[*node].get(*next) {
                    *next += 1;
                    if !std::mem::replace(&mut seen[child], true) { stack.push((child, 0)); }
                } else {
                    order.push(*node);
                    stack.pop();
                }
            }
        }
        seen.fill(false);
        let mut shared = vec![None; records.len()];
        let mut cyclic = vec![false; records.len()];
        for root in order.into_iter().rev() {
            if std::mem::replace(&mut seen[root], true) { continue; }
            let mut component = Vec::new();
            let mut pending = vec![root];
            while let Some(node) = pending.pop() {
                component.push(node);
                for &parent in &reverse[node] {
                    if !std::mem::replace(&mut seen[parent], true) { pending.push(parent); }
                }
            }
            if component.len() > 1 || edges[root].contains(&root) {
                for member in component {
                    cyclic[member] = true;
                    if !incomplete[member] { shared[member] = access[member]; }
                }
            }
        }
        Self { shared, cyclic }
    }

    pub(crate) fn access(&self, record: usize) -> Option<BorrowAccess> {
        self.shared.get(record).copied().flatten()
    }

    pub(crate) fn recursive(&self, record: usize) -> bool { self.cyclic.get(record).copied().unwrap_or(false) }
}

fn merge(first: BorrowAccess, second: BorrowAccess) -> BorrowAccess {
    if first == BorrowAccess::Exclusive || second == BorrowAccess::Exclusive { BorrowAccess::Exclusive } else { BorrowAccess::ReadOnly }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cloneability::tests::record, native_signatures::NativeTypeId, syntax::ArrayElement};

    fn view(native: NativeTypeId) -> Record {
        let mut result = record(&[]);
        result.native_type = Some(native);
        result
    }

    #[test]
    fn captured_slots_seed_recursive_access_without_a_direct_view_field() {
        let records = vec![
            record(&[Type::Array(ArrayElement::Record(1, 0)), Type::StoredCallable(0)]),
            record(&[Type::Array(ArrayElement::Record(0, 0)), Type::StoredCallable(1)]),
            record(&[Type::Record(0)]),
        ];
        let symbolic = Index::new(&records, &|_| vec![]);
        assert!(symbolic.recursive(0));
        assert_eq!(symbolic.access(0), None);
        let concrete = Index::with_leaves(&records, &|_| vec![], &|ty| match ty {
            Type::StoredCallable(0) => Some(BorrowAccess::ReadOnly),
            Type::StoredCallable(1) => Some(BorrowAccess::Exclusive),
            _ => None,
        });
        assert_eq!(concrete.access(0), Some(BorrowAccess::Exclusive));
        assert_eq!(concrete.access(1), Some(BorrowAccess::Exclusive));
        assert_eq!(concrete.access(2), None);
    }

    #[test]
    fn nullable_mutual_cycles_merge_access_but_not_outer_records() {
        let union = UnionId { index: 0, copy: false, scalar_members:false };
        let records = vec![
            view(NativeTypeId::ByteView),
            view(NativeTypeId::ByteViewMut),
            record(&[Type::Union(union), Type::Record(0)]),
            record(&[Type::Array(ArrayElement::Record(2, 2)), Type::Record(1)]),
            record(&[Type::Record(2), Type::Record(0)]),
            record(&[Type::Array(ArrayElement::Record(5, 0))]),
        ];
        let index = Index::new(&records, &|_| vec![Type::Record(3), Type::Undefined]);
        assert_eq!(index.access(2), Some(BorrowAccess::Exclusive));
        assert_eq!(index.access(3), Some(BorrowAccess::Exclusive));
        for id in [0, 1, 4, 5] { assert_eq!(index.access(id), None); }
    }

    #[test]
    fn incomplete_cycle_does_not_claim_a_complete_lifetime_layout() {
        let mut records = vec![
            view(NativeTypeId::ByteView),
            record(&[Type::Array(ArrayElement::Record(1, 0)), Type::Record(0), Type::Int]),
        ];
        records[1].fields[2].ty = None;
        assert_eq!(Index::new(&records, &|_| vec![]).access(1), None);
    }

    #[test]
    fn long_record_cycle_uses_bounded_call_stack() {
        let count = 4096;
        let mut records = vec![view(NativeTypeId::ByteView)];
        for id in 1..=count {
            records.push(record(&[
                Type::Array(ArrayElement::Record(if id == count { 1 } else { id + 1 }, 0)),
                if id == count { Type::Record(0) } else { Type::Int },
            ]));
        }
        let index = Index::new(&records, &|_| vec![]);
        assert_eq!(index.access(0), None);
        for id in 1..=count { assert_eq!(index.access(id), Some(BorrowAccess::ReadOnly)); }
    }
}
