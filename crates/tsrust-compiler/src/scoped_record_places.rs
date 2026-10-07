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
#[derive(Clone, Copy)]
enum Rendering {
    Pattern(bool),
    Value(crate::Mode),
}
impl Layout {
    pub(crate) fn view_value(&self, binding: usize, mutable: bool, records: &[Record], symbol: impl Fn(usize) -> String) -> String {
        let mode = if mutable { crate::Mode::Mutate } else { crate::Mode::Read };
        self.render_node(Type::Record(self.record), &mut Vec::new(), binding, Rendering::Value(mode), records, &symbol)
    }

    pub(crate) fn declaration(&self, binding: usize, mutable: bool, initializer: &str, records: &[Record], symbol: impl Fn(usize) -> String) -> String {
        let pattern = self.render_node(Type::Record(self.record), &mut Vec::new(), binding, Rendering::Pattern(mutable), records, &symbol);
        format!("let {pattern} = {initializer};")
    }
    pub(crate) fn owning_value(&self, binding: usize, records: &[Record], symbol: impl Fn(usize) -> String) -> String {
        self.render_node(Type::Record(self.record), &mut Vec::new(), binding, Rendering::Value(crate::Mode::Consume), records, &symbol)
    }
    fn render_node(&self, ty: Type, path: &mut Vec<usize>, binding: usize, rendering: Rendering, records: &[Record], symbol: &impl Fn(usize) -> String) -> String {
        if let Type::Record(record) = ty {
            let fields = &records[record].fields;
            if !fields.is_empty() {
                let fields = fields.iter().enumerate().map(|(index, field)| {
                    path.push(index);
                    let value = self.render_node(field.ty.expect("checked field layout"), path, binding, rendering, records, symbol);
                    path.pop();
                    format!("field{index}: {value}")
                }).collect::<Vec<_>>().join(", ");
                return format!("{} {{ {fields} }}", symbol(record));
            }
        }
        let name = Leaf { path: path.clone(), ty }.native_name(binding);
        match rendering {
            Rendering::Pattern(true) => format!("mut {name}"),
            Rendering::Value(crate::Mode::Read) => format!("&{name}"),
            Rendering::Value(crate::Mode::Mutate) => format!("&mut {name}"),
            _ => name,
        }
    }
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
    fn native(source: &str) {
        use std::{io::Write, process::{Command, Stdio}};
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let executable = std::env::temp_dir().join(format!("tsrust-field-storage-{}-{}", std::process::id(), NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst)));
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup { fn drop(&mut self) { let _ = std::fs::remove_file(&self.0); } }
        let _cleanup = Cleanup(executable.clone());
        let mut compiler = Command::new("rustc").args(["-", "--edition=2024"]).arg("-o").arg(&executable).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
        compiler.stdin.take().unwrap().write_all(source.as_bytes()).unwrap();
        let compiled = compiler.wait_with_output().unwrap();
        assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
        let executed = Command::new(&executable).output().unwrap();
        assert!(executed.status.success(), "{}", String::from_utf8_lossy(&executed.stderr));
        assert!(executed.stdout.is_empty());
    }
    #[test]
    fn native_shared_and_mutable_views_borrow_non_copy_data_in_place() {
        let checked = checked();
        let binding = checked.program.functions[0].bindings.iter().find(|binding| binding.name == "box").unwrap();
        let layout = Layout::for_binding(binding, &checked.program.records, true).unwrap();
        let Type::Record(inner) = checked.program.records[layout.record].fields[0].ty.unwrap() else { panic!("inner record"); };
        let declaration = layout.declaration(8, true, "initial()", &checked.program.records, |id| format!("Record{id}"));
        let shared = layout.view_value(8, false, &checked.program.records, |id| format!("ReadRecord{id}"));
        let exclusive = layout.view_value(8, true, &checked.program.records, |id| format!("MutRecord{id}"));
        let owned = layout.owning_value(8, &checked.program.records, |id| format!("Record{id}"));
        let label = layout.leaves[1].native_name(8);
        let source = format!(r#"
            struct Record{inner} {{ field0: i64, field1: String }}
            struct Record{outer} {{ field0: Record{inner}, field1: i64 }}
            struct ReadRecord{inner}<'a> {{ field0: &'a i64, field1: &'a String }}
            struct ReadRecord{outer}<'a> {{ field0: ReadRecord{inner}<'a>, field1: &'a i64 }}
            struct MutRecord{inner}<'a> {{ field0: &'a mut i64, field1: &'a mut String }}
            struct MutRecord{outer}<'a> {{ field0: MutRecord{inner}<'a>, field1: &'a mut i64 }}
            trait Readable {{ fn observe(&self) -> usize; }}
            impl Readable for Record{outer} {{ fn observe(&self)->usize {{ self.field0.field1.len() + self.field0.field0 as usize + self.field1 as usize }} }}
            impl Readable for ReadRecord{outer}<'_> {{ fn observe(&self)->usize {{ self.field0.field1.len() + *self.field0.field0 as usize + *self.field1 as usize }} }}
            fn inspect(value: &impl Readable)->usize {{ value.observe() }}
            fn initial()->Record{outer} {{ Record{outer} {{ field0: Record{inner} {{ field0:4,field1:"hello".into() }},field1:9 }} }}
            fn main() {{
                {declaration}
                {{ let view = {shared}; assert_eq!(inspect(&view),18); assert!(std::ptr::eq(view.field0.field1, &{label})); }}
                {{ let view = {exclusive}; view.field0.field1.push_str("!"); *view.field0.field0 += 3; *view.field1 += 2; }}
                assert_eq!({label}, "hello!");
                {{ let view = {shared}; assert_eq!(inspect(&view),24); }}
                let rebuilt = {owned};
                assert_eq!(inspect(&rebuilt),24);
            }}
        "#, outer=layout.record);
        native(&source);
    }
    #[test]
    fn native_split_storage_evaluates_once_restores_fields_and_moves_without_clone() {
        let checked = checked();
        let binding = checked.program.functions[0].bindings.iter().find(|binding| binding.name == "box").unwrap();
        let layout = Layout::for_binding(binding, &checked.program.records, true).unwrap();
        let Type::Record(inner) = checked.program.records[layout.record].fields[0].ty.unwrap() else { panic!("inner record"); };
        let declaration = layout.declaration(7, true, "{ calls += 1; initial() }", &checked.program.records, |id| format!("Record{id}"));
        let reconstructed = layout.owning_value(7, &checked.program.records, |id| format!("Record{id}"));
        let label = layout.leaves[1].native_name(7);
        let source = format!(r#"
            static DROPS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            struct Payload(String);
            impl Drop for Payload {{ fn drop(&mut self) {{ DROPS.fetch_add(1, std::sync::atomic::Ordering::SeqCst); }} }}
            struct Record{inner} {{ field0: i64, field1: Payload }}
            struct Record{outer} {{ field0: Record{inner}, field1: i64 }}
            fn initial() -> Record{outer} {{ Record{outer} {{ field0: Record{inner} {{ field0: 4, field1: Payload("first".into()) }}, field1: 9 }} }}
            fn main() {{
                let mut calls = 0;
                {declaration}
                assert_eq!(calls, 1);
                {{ let moved = {label}; assert_eq!(moved.0, "first"); }}
                assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst), 1);
                {label} = Payload("restored".into());
                let rebuilt = {reconstructed};
                assert_eq!(rebuilt.field0.field0, 4);
                assert_eq!(rebuilt.field0.field1.0, "restored");
                assert_eq!(rebuilt.field1, 9);
                drop(rebuilt);
                assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst), 2);
            }}
        "#, outer=layout.record);
        native(&source);
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
