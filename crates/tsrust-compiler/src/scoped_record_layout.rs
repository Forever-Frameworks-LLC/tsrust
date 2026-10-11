//! Concrete field environments retain a separate nominal declaration identity.
use crate::syntax::{Record, Type};
use std::collections::HashMap;

#[path = "scoped_record_recursive_layout.rs"]
mod recursive;

#[derive(Default)]
pub(crate) struct Registry {
    bases: HashMap<usize, usize>,
    instances: HashMap<(usize, Vec<Type>), usize>,
    recursive_instances: HashMap<(usize, Vec<Type>), usize>,
    recursive_parameters: HashMap<usize, std::collections::HashSet<usize>>,
    parameter_contracts: std::collections::HashSet<usize>,
}

impl Registry {
    pub(crate) fn declaration(&self, id: usize) -> usize {
        self.bases.get(&id).copied().unwrap_or(id)
    }

    pub(crate) fn mark_parameter_contract(&mut self, id: usize) { self.parameter_contracts.insert(id); }
    pub(crate) fn parameter_contract(&self, id: usize) -> bool { self.parameter_contracts.contains(&id) }

    pub(crate) fn materialize(
        &mut self,
        base: usize,
        types: Vec<Type>,
        records: &mut Vec<Record>,
    ) -> usize {
        let base = self.declaration(base);
        let key = (base, types);
        if let Some(&id) = self.instances.get(&key) {
            return id;
        }
        let mut record = records[base].clone();
        assert_eq!(
            record.fields.len(),
            key.1.len(),
            "checked concrete field count"
        );
        for (field, ty) in record.fields.iter_mut().zip(&key.1) {
            field.ty = Some(*ty);
        }
        let id = records.len();
        records.push(record);
        self.instances.insert(key, id);
        self.bases.insert(id, base);
        id
    }
}

#[cfg(test)]
mod recursive_tests {
    use super::*;
    use crate::{cloneability::tests::record, syntax::ArrayElement, unions::UnionArena, Span};

    #[test]
    fn recursive_specialization_keeps_callbacks_and_retargets_owned_links() {
        let unions = UnionArena::default();
        let next = unions.intern([Type::Record(0), Type::Undefined], Span::default()).unwrap();
        let mut records = vec![record(&[Type::StoredCallable(0), next])];
        let mut registry = Registry::default();
        let fields = vec![Type::StoredCallable(1), next];
        let specialized = registry.materialize_recursive(0, fields.clone(), &mut records, &unions).unwrap();
        assert_ne!(specialized, 0);
        assert_eq!(registry.declaration(specialized), 0);
        assert_eq!(records[0].fields[0].ty, Some(Type::StoredCallable(0)));
        assert_eq!(records[specialized].fields[0].ty, Some(Type::StoredCallable(1)));
        let next = records[specialized].fields[1].ty.unwrap().union().unwrap();
        assert!(unions.members(next).contains(&Type::Record(specialized)));
        let links = crate::recursive_links::Index::new(&records, |id| unions.members(id));
        assert!(links.boxed(next, specialized));
        assert_eq!(registry.materialize_recursive(0, fields, &mut records, &unions).unwrap(), specialized);
        let concrete_fields = records[specialized].fields.iter().map(|field| field.ty.unwrap()).collect();
        assert_eq!(registry.materialize_recursive(specialized, concrete_fields, &mut records, &unions).unwrap(), specialized);
        assert_eq!(registry.recursive_parameter(Type::Record(0)), Type::Record(specialized));
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn mutual_specialization_preserves_array_rank_and_unrelated_records() {
        let unions = UnionArena::default();
        let back = unions.intern([Type::Record(0), Type::Null], Span::default()).unwrap();
        let mut records = vec![
            record(&[Type::StoredCallable(0), Type::Array(ArrayElement::Record(1, 2)), Type::Record(2)]),
            record(&[back]),
            record(&[Type::String]),
        ];
        let mut registry = Registry::default();
        let specialized = registry.materialize_recursive(0,
            vec![Type::StoredCallable(1), Type::Array(ArrayElement::Record(1, 2)), Type::Record(2)],
            &mut records, &unions).unwrap();
        let Type::Array(ArrayElement::Record(child, 2)) = records[specialized].fields[1].ty.unwrap() else { panic!("preserved array rank") };
        assert_eq!(registry.declaration(child), 1);
        assert_eq!(registry.recursive_parameter(Type::Record(1)), Type::Record(child));
        let back = records[child].fields[0].ty.unwrap().union().unwrap();
        assert!(unions.members(back).contains(&Type::Record(specialized)));
        assert_eq!(records[specialized].fields[2].ty, Some(Type::Record(2)));
        assert_eq!(records.len(), 5);
    }
}
