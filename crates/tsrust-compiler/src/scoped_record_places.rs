//! Independent native places for owned scoped record fields.
use crate::{analysis::Binding, syntax::{Record, Type}};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Leaf {
    pub(crate) path: Vec<usize>,
    pub(crate) ty: Type,
}
impl Leaf {
    pub(crate) fn native_name(&self, binding: usize) -> String {
        let mut name = format!("value{binding}");
        for field in &self.path {
            use std::fmt::Write;
            write!(name, "_field{field}").expect("string write");
        }
        name
    }
}
#[derive(Debug)]
pub(crate) struct Layout {
    pub(crate) record: usize,
    pub(crate) leaves: Vec<Leaf>,
}
impl Layout {
    pub(crate) fn for_binding(binding: &Binding, records: &[Record], scoped: bool) -> Option<Self> {
        if !scoped || binding.capture_reference || binding.borrowed_place.is_some()
            || binding.borrowed_element.is_some()
            || (binding.parameter && binding.mode != crate::Mode::Consume) {
            return None;
        }
        let Type::Record(record) = binding.ty else { return None; };
        let mut pending = vec![(binding.ty, Vec::new(), Vec::new())];
        let mut leaves = Vec::new();
        while let Some((ty, path, ancestors)) = pending.pop() {
            if path.len() > 128 { return None; }
            if let Type::Record(id) = ty {
                let declaration = records.get(id)?;
                if ancestors.contains(&id) { return None; }
                if !declaration.fields.is_empty() {
                    let mut ancestors = ancestors;
                    ancestors.push(id);
                    for (field, item) in declaration.fields.iter().enumerate().rev() {
                        let mut path = path.clone();
                        path.push(field);
                        pending.push((item.ty?, path, ancestors.clone()));
                    }
                    continue;
                }
            }
            leaves.push(Leaf { path, ty });
        }
        Some(Self { record, leaves })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn checked() -> crate::Analysis {
        let source = r#"type Inner = { value: int; label: string; }; type Outer = { nested: Inner; score: int; }; function main(){const box:Outer={nested:{value:4,label:"x"},score:9};}"#;
        let source = crate::format(source).unwrap();
        assert_eq!(crate::format(&source).unwrap(), source);
        crate::check(&source).unwrap()
    }
    #[test]
    fn nested_owned_storage_preserves_declaration_order_and_numeric_paths() {
        let checked = checked();
        let binding = checked.program.functions[0].bindings.iter().find(|binding| binding.name == "box").unwrap();
        let layout = Layout::for_binding(binding, &checked.program.records, true).unwrap();
        let Type::Record(record) = binding.ty else { panic!("record"); };
        assert_eq!(layout.record, record);
        assert_eq!(layout.leaves[0].native_name(7), "value7_field0_field0");
        assert_eq!(layout.leaves[1].native_name(7), "value7_field0_field1");
        assert_eq!(layout.leaves, vec![
            Leaf { path: vec![0,0], ty: Type::Int },
            Leaf { path: vec![0,1], ty: Type::String },
            Leaf { path: vec![1], ty: Type::Int },
        ]);
    }
    #[test]
    fn borrowed_storage_and_non_scoped_records_keep_their_original_place() {
        let checked = checked();
        let mut binding = checked.program.functions[0].bindings.iter().find(|binding| binding.name == "box").unwrap().clone();
        assert!(Layout::for_binding(&binding, &checked.program.records, false).is_none());
        binding.parameter = true;
        binding.mode = crate::Mode::Read;
        assert!(Layout::for_binding(&binding, &checked.program.records, true).is_none());
        binding.parameter = false;
        binding.capture_reference = true;
        assert!(Layout::for_binding(&binding, &checked.program.records, true).is_none());
        binding.capture_reference = false;
        binding.borrowed_place = Some(crate::Span::new(0, 1));
        assert!(Layout::for_binding(&binding, &checked.program.records, true).is_none());
        binding.borrowed_place = None;
        binding.borrowed_element = Some(crate::Span::new(0, 1));
        assert!(Layout::for_binding(&binding, &checked.program.records, true).is_none());
    }
    #[test]
    fn opaque_union_stays_a_leaf_and_invalid_or_cyclic_metadata_is_not_flattened() {
        let checked = checked();
        let binding = checked.program.functions[0].bindings.iter().find(|binding| binding.name == "box").unwrap();
        let Type::Record(record) = binding.ty else { panic!("record"); };
        let mut records = checked.program.records.clone();
        records[record].fields[0].ty = Some(binding.ty);
        assert!(Layout::for_binding(binding, &records, true).is_none());
        records[record].fields[0].ty = None;
        assert!(Layout::for_binding(binding, &records, true).is_none());
        let source = crate::format("type Box = { value: string | undefined; }; function main(){const box:Box={value:undefined};}").unwrap();
        assert_eq!(crate::format(&source).unwrap(), source);
        let checked = crate::check(&source).unwrap();
        let binding = checked.program.functions[0].bindings.iter().find(|binding| binding.name == "box").unwrap();
        let layout = Layout::for_binding(binding, &checked.program.records, true).unwrap();
        assert_eq!(layout.leaves.len(), 1);
        assert_eq!(layout.leaves[0].path, vec![0]);
        assert!(matches!(layout.leaves[0].ty, Type::Union(_)));
    }
}
