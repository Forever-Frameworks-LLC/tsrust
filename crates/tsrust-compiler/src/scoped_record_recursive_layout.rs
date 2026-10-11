//! Finite specializations retain recursive edges within their concrete component.
use super::Registry;
use crate::{Diagnostic, Span, syntax::{ArrayElement, Record, Type}, unions::UnionArena};
use std::collections::{HashMap, HashSet};

impl Registry {
    pub(crate) fn materialize_recursive(
        &mut self,
        base: usize,
        types: Vec<Type>,
        records: &mut Vec<Record>,
        unions: &UnionArena,
    ) -> Result<usize, Diagnostic> {
        let base = self.declaration(base);
        let key = (base, types);
        if let Some(&id) = self.recursive_instances.get(&key) { return Ok(id); }
        let mut edges = vec![Vec::new(); records.len()];
        let mut reverse = vec![Vec::new(); records.len()];
        for (id, record) in records.iter().enumerate() {
            let mut pending = record.fields.iter().filter_map(|field| field.ty).collect::<Vec<_>>();
            let mut seen = HashSet::new();
            while let Some(ty) = pending.pop() {
                if !seen.insert(ty) { continue; }
                match ty {
                    Type::Record(child) => { edges[id].push(child); reverse[child].push(id); }
                    Type::Array(element) => pending.push(element.with_depth(0).ty()),
                    Type::Union(union) | Type::BorrowedUnion(union) => pending.extend(unions.members(union)),
                    _ => {}
                }
            }
        }
        let reach = |graph: &[Vec<usize>]| {
            let mut seen = HashSet::new();
            let mut pending = vec![base];
            while let Some(id) = pending.pop() {
                if seen.insert(id) { pending.extend(graph[id].iter().copied()); }
            }
            seen
        };
        let forward = reach(&edges);
        let back = reach(&reverse);
        let mut component = forward.intersection(&back).copied().collect::<Vec<_>>();
        component.sort_unstable();
        if component.len() == 1 && !edges[base].contains(&base) {
            return Ok(self.materialize(base, key.1, records));
        }
        // Reserve identities before rewriting any edge. No recursively expanded
        // record or callback environment is needed, even for mutually recursive types.
        let first = records.len();
        let remap = component.iter().enumerate().map(|(offset, &id)| (id, first + offset)).collect::<HashMap<_, _>>();
        let mut instances = Vec::new();
        for &id in &component {
            let mut record = records[id].clone();
            if id == base {
                assert_eq!(record.fields.len(), key.1.len(), "checked concrete field count");
                for (field, &ty) in record.fields.iter_mut().zip(&key.1) { field.ty = Some(ty); }
            }
            for field in &mut record.fields {
                if let Some(ty) = field.ty { field.ty = Some(remap_type(ty, &remap, unions)?); }
            }
            instances.push(record);
        }
        records.extend(instances);
        for (&original, &instance) in &remap { self.bases.insert(instance, self.declaration(original)); }
        let instance = remap[&base];
        self.recursive_instances.insert(key, instance);
        Ok(instance)
    }
}

fn remap_type(ty: Type, records: &HashMap<usize, usize>, unions: &UnionArena) -> Result<Type, Diagnostic> {
    Ok(match ty {
        Type::Record(id) => Type::Record(records.get(&id).copied().unwrap_or(id)),
        Type::Array(element) => {
            let leaf = remap_type(element.with_depth(0).ty(), records, unions)?;
            Type::Array(ArrayElement::from_value_type(leaf).expect("preserved array leaf").with_depth(element.depth()))
        }
        Type::Union(id) | Type::BorrowedUnion(id) => {
            let members = unions.members(id).into_iter().map(|member| remap_type(member, records, unions)).collect::<Result<Vec<_>, _>>()?;
            let mapped = unions.intern(members, Span::default())?;
            if matches!(ty, Type::BorrowedUnion(_)) { Type::BorrowedUnion(mapped.union().expect("preserved union alternatives")) } else { mapped }
        }
        _ => ty,
    })
}
