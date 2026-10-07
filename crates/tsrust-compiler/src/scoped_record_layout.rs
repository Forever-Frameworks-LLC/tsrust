//! Concrete field environments retain a separate nominal declaration identity.
use crate::syntax::{Record, Type};
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct Registry {
    bases: HashMap<usize, usize>,
    instances: HashMap<(usize, Vec<Type>), usize>,
}

impl Registry {
    pub(crate) fn declaration(&self, id: usize) -> usize {
        self.bases.get(&id).copied().unwrap_or(id)
    }

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
