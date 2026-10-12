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
#[cfg(test)]
#[derive(Clone, Copy)]
enum Rendering {
    Pattern(bool),
    Value(crate::Mode),
}
impl Layout {
    #[cfg(test)]
    pub(crate) fn view_declarations(&self, records: &[Record], symbol: impl Fn(usize) -> String, native: impl Fn(usize, usize, Type) -> (String, Vec<String>), mutable: bool) -> String {
        self.view_declarations_cached(records,symbol,native,mutable,&mut std::collections::BTreeMap::new())
    }
    #[cfg(test)]
    pub(crate) fn view_declarations_cached(&self, records: &[Record], symbol: impl Fn(usize) -> String, native: impl Fn(usize, usize, Type) -> (String, Vec<String>), mutable: bool, seen: &mut std::collections::BTreeMap<usize,Vec<String>>) -> String {
        self.view_declarations_with_metadata(records,symbol,native,mutable,seen,|_|String::new())
    }
    pub(crate) fn view_declarations_with_metadata(&self, records: &[Record], symbol: impl Fn(usize) -> String, native: impl Fn(usize, usize, Type) -> (String, Vec<String>), mutable: bool, seen: &mut std::collections::BTreeMap<usize,Vec<String>>, metadata: impl Fn(usize)->String) -> String {
        fn emit(id: usize, records: &[Record], symbol: &impl Fn(usize) -> String, native: &impl Fn(usize, usize, Type) -> (String, Vec<String>), mutable: bool, seen: &mut std::collections::BTreeMap<usize, Vec<String>>, output: &mut String, metadata: &impl Fn(usize)->String, leaves:&std::collections::HashSet<usize>) -> Vec<String> {
            use std::fmt::Write;
            if let Some(parameters) = seen.get(&id) { return parameters.clone(); }
            let mut parameters = Vec::new();
            let fields = records[id].fields.iter().enumerate().map(|(index, field)| {
                let ty = field.ty.expect("checked field layout");
                let (field_type, field_parameters) = if let Type::Record(child) = ty
                    && !records[child].fields.is_empty() && !leaves.contains(&child) {
                    let child_parameters = emit(child, records, symbol, native, mutable, seen, output,metadata,leaves);
                    // Each child occurrence has its own environment arguments,
                    // even when both occurrences use the same nominal view.
                    let child_parameters=child_parameters.into_iter().map(|parameter| {
                        if let Some(lifetime)=parameter.strip_prefix('\'') {format!("'nested{index}_{lifetime}")}else{format!("N{index}{parameter}")}
                    }).collect::<Vec<_>>();
                    let arguments = std::iter::once("'a".to_owned()).chain(child_parameters.iter().cloned()).collect::<Vec<_>>().join(", ");
                    (format!("{}<{arguments}>", symbol(child)), child_parameters)
                } else {
                    let (native_type, parameters) = native(id, index, ty);
                    (format!("&'a {}{native_type}", if mutable { "mut " } else { "" }), parameters)
                };
                for parameter in field_parameters {
                    if parameter != "'a" && !parameters.contains(&parameter) { parameters.push(parameter); }
                }
                format!("field{index}: {field_type}")
            }).collect::<Vec<_>>().join(", ");
            let arguments = std::iter::once("'a".to_owned()).chain(parameters.iter().cloned()).collect::<Vec<_>>().join(", ");
            writeln!(output, "struct {}<{arguments}> {{ {fields}{} }}", symbol(id),metadata(id)).expect("string write");
            seen.insert(id, parameters.clone());
            parameters
        }
        let mut output = String::new();
        let leaves=self.leaves.iter().filter_map(|leaf|if let Type::Record(id)=leaf.ty {Some(id)}else{None}).collect();
        emit(self.record, records, &symbol, &native, mutable, seen, &mut output,&metadata,&leaves);
        output
    }

    #[cfg(test)]
    pub(crate) fn selected_value(&self, binding: usize, path: &[usize], mode: crate::Mode, records: &[Record], symbol: impl Fn(usize) -> String) -> Option<String> {
        let mut ty = Type::Record(self.record);
        for &field in path {
            let Type::Record(id) = ty else { return None; };
            ty = records.get(id)?.fields.get(field)?.ty?;
        }
        Some(self.render_node(ty, &mut path.to_vec(), binding, Rendering::Value(mode), records, &symbol))
    }

    #[cfg(test)]
    pub(crate) fn view_value(&self, binding: usize, mutable: bool, records: &[Record], symbol: impl Fn(usize) -> String) -> String {
        let mode = if mutable { crate::Mode::Mutate } else { crate::Mode::Read };
        self.render_node(Type::Record(self.record), &mut Vec::new(), binding, Rendering::Value(mode), records, &symbol)
    }

    #[cfg(test)]
    pub(crate) fn declaration(&self, binding: usize, mutable: bool, initializer: &str, records: &[Record], symbol: impl Fn(usize) -> String) -> String {
        let pattern = self.render_node(Type::Record(self.record), &mut Vec::new(), binding, Rendering::Pattern(mutable), records, &symbol);
        format!("let {pattern} = {initializer};")
    }
    #[cfg(test)]
    pub(crate) fn owning_value(&self, binding: usize, records: &[Record], symbol: impl Fn(usize) -> String) -> String {
        self.render_node(Type::Record(self.record), &mut Vec::new(), binding, Rendering::Value(crate::Mode::Consume), records, &symbol)
    }
    #[cfg(test)]
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
        Self::for_record(record,records)
    }
    pub(crate) fn for_record(record: usize, records: &[Record]) -> Option<Self> {
        Self::for_record_with_leaf(record,records,|_|false)
    }
    pub(crate) fn for_record_with_leaf(record:usize,records:&[Record],leaf:impl Fn(usize)->bool)->Option<Self> {
        let mut pending = vec![(Type::Record(record), Vec::new(), Vec::new())];
        let mut leaves = Vec::new();
        while let Some((ty, path, ancestors)) = pending.pop() {
            if path.len() > 128 { return None; }
            if let Type::Record(id) = ty {
                let declaration = records.get(id)?;
                if ancestors.contains(&id) { return None; }
                if !declaration.fields.is_empty() && (path.is_empty() || !leaf(id)) {
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
pub(crate) mod tests {
    use super::*;
    fn checked() -> crate::Analysis {
        let source = r#"type Inner = { value: int; label: string; }; type Outer = { nested: Inner; score: int; }; function main(){const box:Outer={nested:{value:4,label:"x"},score:9};}"#;
        let source = crate::format(source).unwrap();
        assert_eq!(crate::format(&source).unwrap(), source);
        crate::check(&source).unwrap()
    }
    pub(crate) fn native(source: &str) { native_expectation(source, None, b""); }
    pub(crate) fn native_rejection(source: &str, diagnostic: &str) { native_expectation(source, Some(diagnostic), b""); }
    pub(crate) fn native_output(source:&str,expected:&str){native_expectation(source,None,expected.as_bytes());}
    fn native_expectation(source: &str, rejection: Option<&str>, expected: &[u8]) {
        use std::{io::Write, process::{Command, Stdio}};
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let executable = std::env::temp_dir().join(format!("tsrust-field-storage-{}-{}", std::process::id(), NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst)));
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup { fn drop(&mut self) { let _ = std::fs::remove_file(&self.0); } }
        let _cleanup = Cleanup(executable.clone());
        let mut compiler = Command::new("rustc").args(["-", "--edition=2024"]).arg("-o").arg(&executable).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
        compiler.stdin.take().unwrap().write_all(source.as_bytes()).unwrap();
        let compiled = compiler.wait_with_output().unwrap();
        if let Some(diagnostic) = rejection {
            assert!(!compiled.status.success(), "invalid native use compiled");
            assert!(String::from_utf8_lossy(&compiled.stderr).contains(diagnostic), "{}", String::from_utf8_lossy(&compiled.stderr));
            return;
        }
        assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
        let executed = Command::new(&executable).output().unwrap();
        assert!(executed.status.success(), "{}", String::from_utf8_lossy(&executed.stderr));
        assert_eq!(executed.stdout,expected);
    }
    #[test]
    fn native_repeated_child_views_keep_distinct_environment_arguments() {
        let source=crate::format(r#"type Holder={run:ReadOnly<Function<int>>;};type Pair={left:Holder;right:Holder;};function main(){const a=[3];const b=[11];const pair:Pair={left:{run:():int=>a[0]},right:{run:():int=>b[0]}};console.log(pair.left.run(),pair.right.run());}"#).unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        let mut binding=checked.program.functions[0].bindings.iter().find(|binding|binding.name=="pair").unwrap().clone();
        let Type::Record(instance)=binding.ty else{panic!("record instance")};
        binding.ty=Type::Record(checked.program.scoped_records.declaration(instance));
        let Type::Record(outer)=binding.ty else{unreachable!()};
        let mut records=checked.program.records.clone();
        records[outer].fields[1].ty=records[outer].fields[0].ty;
        let Type::Record(inner)=records[outer].fields[0].ty.unwrap() else{panic!("child record")};
        let layout=Layout::for_binding(&binding,&records,true).unwrap();
        let definitions=layout.view_declarations(&records,|id|format!("ReadRecord{id}"),|_,_,_|("F".into(),vec!["F".into()]),false);
        let declaration=layout.declaration(13,false,"initial",&records,|id|format!("Record{id}"));
        let view=layout.view_value(13,false,&records,|id|format!("ReadRecord{id}"));
        let source=format!(r#"
            struct Record{inner}<F>{{field0:F}}
            struct Record{outer}<L,R>{{field0:Record{inner}<L>,field1:Record{inner}<R>}}
            {definitions}
            fn inspect<L:Fn()->i64,R:Fn()->i64>(value:&ReadRecord{outer}<'_,L,R>)->i64{{(value.field0.field0)()+(value.field1.field0)()}}
            fn main(){{
                let text=String::from("abc");let values=vec![11i64];
                let initial=Record{outer}{{field0:Record{inner}{{field0:move ||text.len() as i64}},field1:Record{inner}{{field0:move ||values[0]}}}};
                {declaration}
                {{let borrowed={view};assert_eq!(inspect(&borrowed),14);}}
            }}
        "#);
        native(&source);
    }
    #[test]
    fn native_nested_generic_child_uses_nominal_access_bounds() {
        use crate::scoped_record_bridge::{Types, OwnedParameters, OwnedAccess};
        let checked = checked();
        let binding = checked.program.functions[0].bindings.iter().find(|binding| binding.name == "box").unwrap();
        let layout = Layout::for_binding(binding, &checked.program.records, true).unwrap();
        let Type::Record(inner) = checked.program.records[layout.record].fields[0].ty.unwrap() else { panic!("nested record"); };
        let child = Layout { record:inner, leaves:layout.leaves.iter().filter(|leaf| leaf.path[0]==0).map(|leaf| Leaf {path:leaf.path[1..].to_vec(),ty:leaf.ty}).collect() };
        let native_type = |_, _, ty| if ty==Type::String {("S".into(),vec!["S".into()])}else{("i64".into(),vec![])};
        let read_definitions=layout.view_declarations(&checked.program.records,|id|format!("ReadRecord{id}"),native_type,false);
        let mut_definitions=layout.view_declarations(&checked.program.records,|id|format!("MutRecord{id}"),native_type,true);
        let child_bridge=crate::scoped_record_bridge::declarations(&child,Types {
            read_trait:"ReadInner",mutate_trait:"MutateInner",owned:&format!("Record{inner}<S>"),shared:&format!("ReadRecord{inner}<'a,S>"),exclusive:&format!("MutRecord{inner}<'a,S>"),parameters:&["S".into()],
        },|leaf|if leaf.ty==Type::String{"S".into()}else{"i64".into()});
        let outer_bridge=crate::scoped_record_bridge::declarations_with_owned(&layout,Types {
            read_trait:"ReadOuter",mutate_trait:"MutateOuter",owned:&format!("Record{}<C>",layout.record),shared:&format!("ReadRecord{}<'a,S>",layout.record),exclusive:&format!("MutRecord{}<'a,S>",layout.record),parameters:&["S".into()],
        },|leaf|if leaf.ty==Type::String{"S".into()}else{"i64".into()},OwnedParameters {
            parameters:&["C".into()],read_bounds:&["C:ReadInner<Field0=i64>".into()],mutate_bounds:&["C:MutateInner<Field0=i64>".into()],
        },|leaf|match leaf.path.as_slice(){
            [0,0]=>OwnedAccess{ty:"i64".into(),read:"self.field0.tsr_field0()".into(),mutate:"self.field0.tsr_field0_mut()".into()},
            [0,1]=>OwnedAccess{ty:"<C as ReadInner>::Field1".into(),read:"self.field0.tsr_field1()".into(),mutate:"self.field0.tsr_field1_mut()".into()},
            [1]=>OwnedAccess{ty:"i64".into(),read:"&self.field1".into(),mutate:"&mut self.field1".into()},
            _=>panic!("unexpected field path"),
        });
        let declaration=layout.declaration(12,true,"initial",&checked.program.records,|id|format!("Record{id}"));
        let shared=layout.view_value(12,false,&checked.program.records,|id|format!("ReadRecord{id}"));
        let exclusive=layout.view_value(12,true,&checked.program.records,|id|format!("MutRecord{id}"));
        let owned=layout.owning_value(12,&checked.program.records,|id|format!("Record{id}"));
        let source=format!(r#"
            struct Record{inner}<S>{{field0:i64,field1:S}}
            struct Record{outer}<C>{{field0:C,field1:i64}}
            {read_definitions}{mut_definitions}{child_bridge}{outer_bridge}
            fn inspect<V:ReadOuter<Field0Field0=i64,Field0Field1=String,Field1=i64>>(value:&V)->usize{{*value.tsr_field0_field0() as usize+value.tsr_field0_field1().len()+*value.tsr_field1() as usize}}
            fn update<V:MutateOuter<Field0Field0=i64,Field0Field1=String,Field1=i64>>(value:&mut V){{value.tsr_field0_field1_mut().push('!');*value.tsr_field0_field0_mut()+=3;*value.tsr_field1_mut()+=2;}}
            fn main(){{
                let initial=Record{outer}{{field0:Record{inner}{{field0:4,field1:String::from("hello")}},field1:9}};
                assert_eq!(inspect(&initial),18);
                {declaration}
                {{let child=ReadRecord{inner}{{field0:&value12_field0_field0,field1:&value12_field0_field1}};let mut wrapped=Record{outer}{{field0:child,field1:9}};assert_eq!(inspect(&wrapped),18);}}
                {{let read={shared};assert_eq!(inspect(&read),18);}}
                {{let mut mutable={exclusive};update(&mut mutable);}}
                let mut rebuilt={owned};assert_eq!(inspect(&rebuilt),24);update(&mut rebuilt);assert_eq!(inspect(&rebuilt),30);
            }}
        "#,outer=layout.record);
        native(&source);
        native_rejection(&source.replace("assert_eq!(inspect(&wrapped),18);","update(&mut wrapped);"),"MutateInner");
    }
    #[test]
    fn native_same_signature_fields_keep_distinct_view_environment_slots() {
        let source = crate::format(r#"type Pair={left:ReadOnly<Function<int>>;right:ReadOnly<Function<int>>;};function main(){const a=[3];const b=[11];const pair:Pair={left:():int=>a[0],right:():int=>b[0]};console.log(pair.left(),pair.right());}"#).unwrap();
        assert_eq!(crate::format(&source).unwrap(), source);
        let checked = crate::check(&source).unwrap();
        let mut binding = checked.program.functions[0].bindings.iter().find(|binding| binding.name == "pair").unwrap().clone();
        let Type::Record(instance) = binding.ty else { panic!("record instance"); };
        binding.ty = Type::Record(checked.program.scoped_records.declaration(instance));
        let mut records = checked.program.records.clone();
        let Type::Record(nominal) = binding.ty else { unreachable!() };
        // Equivalent contracts may share a type identity; field slots must not.
        records[nominal].fields[1].ty = records[nominal].fields[0].ty;
        let layout = Layout::for_binding(&binding, &records, true).unwrap();
        let native_type = |_, index, _| { let name=format!("F{index}"); (name.clone(),vec![name]) };
        let shared_definitions = layout.view_declarations(&records, |id| format!("ReadRecord{id}"), native_type, false);
        let mutable_definitions = layout.view_declarations(&records, |id| format!("MutRecord{id}"), native_type, true);
        let bridges = crate::scoped_record_bridge::declarations(&layout, crate::scoped_record_bridge::Types {
            read_trait:"ReadPair",mutate_trait:"MutatePair",owned:&format!("Record{}<F0,F1>",layout.record),shared:&format!("ReadRecord{}<'a,F0,F1>",layout.record),exclusive:&format!("MutRecord{}<'a,F0,F1>",layout.record),parameters:&["F0".into(),"F1".into()],
        }, |leaf| format!("F{}",leaf.path[0]));
        let declaration = layout.declaration(11, false, "initial", &records, |id| format!("Record{id}"));
        let view = layout.view_value(11, false, &records, |id| format!("ReadRecord{id}"));
        native(&format!(r#"
            struct Record{id}<L,R> {{field0:L,field1:R}}
            {shared_definitions}
            {mutable_definitions}
            {bridges}
            fn inspect<V:ReadPair>(value:&V)->i64 where V::Field0:Fn()->i64,V::Field1:Fn()->i64 {{(value.tsr_field0())()+(value.tsr_field1())()}}
            fn main() {{
                let text=String::from("abc");let values=vec![11i64];
                let initial=Record{id} {{field0:move ||text.len() as i64,field1:move ||values[0]}};
                {declaration}
                {{let borrowed={view};assert_eq!(inspect(&borrowed),14);}}
            }}
        "#,id=layout.record));
    }
    #[test]
    fn native_selected_callback_move_releases_only_its_capture() {
        let source = crate::format(r#"type Pair={left:ReadOnly<Function<int>>;right:ReadOnly<Function<int>>;};function main(){const leftValues=[3];const rightValues=[11];const pair:Pair={left:():int=>leftValues[0],right:():int=>rightValues[0]};{const left=pair.left;console.log(left());}leftValues[0]=5;console.log(pair.right());}"#).unwrap();
        assert_eq!(crate::format(&source).unwrap(), source);
        let checked = crate::check(&source).unwrap();
        let binding = checked.program.functions[0].bindings.iter().find(|binding| binding.name == "pair").unwrap();
        let layout = Layout::for_binding(binding, &checked.program.records, true).unwrap();
        let declaration = layout.declaration(10, false, "initial", &checked.program.records, |id| format!("Record{id}"));
        let left = layout.selected_value(10, &[0], crate::Mode::Consume, &checked.program.records, |id| format!("Record{id}")).unwrap();
        let right = layout.selected_value(10, &[1], crate::Mode::Read, &checked.program.records, |id| format!("Record{id}")).unwrap();
        assert!(layout.selected_value(10, &[2], crate::Mode::Read, &checked.program.records, |id| format!("Record{id}")).is_none());
        assert!(layout.selected_value(10, &[0, 0], crate::Mode::Read, &checked.program.records, |id| format!("Record{id}")).is_none());
        native(&format!(r#"
            struct Record{id}<L,R> {{field0:L,field1:R}}
            fn main() {{
                let mut left_values=vec![3i64];let right_values=vec![11i64];
                let initial=Record{id} {{field0:||left_values[0],field1:||right_values[0]}};
                {declaration}
                {{let moved={left};assert_eq!(moved(),3);}}
                left_values[0]=5;assert_eq!(left_values[0],5);
                assert_eq!(({right})(),11);
            }}
        "#,id=layout.record));
    }
    #[test]
    fn native_generic_views_borrow_mutable_callback_environment_without_moving_it() {
        let source = crate::format(r#"type Holder={run:Mutable<Function<int>>;label:string;};function main(){let calls=0;const holder:Holder={run:():int=>{calls+=1;return calls;},label:"counter"};}"#).unwrap();
        assert_eq!(crate::format(&source).unwrap(), source);
        let checked = crate::check(&source).unwrap();
        let binding = checked.program.functions[0].bindings.iter().find(|binding| binding.name == "holder").unwrap();
        let layout = Layout::for_binding(binding, &checked.program.records, true).unwrap();
        let native_type = |_, _, ty| if ty == Type::String { ("String".into(), vec![]) } else { ("F".into(), vec!["F".into()]) };
        let definitions = layout.view_declarations(&checked.program.records, |id| format!("MutRecord{id}"), native_type, true);
        let read_definitions = layout.view_declarations(&checked.program.records, |id| format!("ReadRecord{id}"), native_type, false);
        let bridges = crate::scoped_record_bridge::declarations(&layout, crate::scoped_record_bridge::Types {
            read_trait: "ReadHolder", mutate_trait: "MutateHolder",
            owned: &format!("Record{}<F>",layout.record), shared: &format!("ReadRecord{}<'a, F>",layout.record), exclusive: &format!("MutRecord{}<'a, F>",layout.record), parameters: &["F".into()],
        }, |leaf| if leaf.ty == Type::String { "String".into() } else { "F".into() });
        let declaration = layout.declaration(9, true, "initial", &checked.program.records, |id| format!("Record{id}"));
        let view = layout.view_value(9, true, &checked.program.records, |id| format!("MutRecord{id}"));
        let owned = layout.owning_value(9, &checked.program.records, |id| format!("Record{id}"));
        native(&format!(r#"
            struct Record{id}<F> {{ field0:F,field1:String }}
            {definitions}
            {read_definitions}
            {bridges}
            fn invoke<V:MutateHolder<Field1=String>>(view: &mut V)->i64 where V::Field0:FnMut()->i64 {{ (view.tsr_field0_mut())() }}
            fn main() {{
                let resource=String::from("abc");
                let mut calls=0;
                let initial=Record{id} {{ field0:move || {{calls+=1; calls+resource.len() as i64}},field1:String::from("counter") }};
                {declaration}
                {{ let mut borrowed={view}; assert_eq!(invoke(&mut borrowed),4); assert_eq!(invoke(&mut borrowed),5); borrowed.field1.push('!'); }}
                let mut rebuilt={owned}; assert_eq!(invoke(&mut rebuilt),6); assert_eq!(rebuilt.field1,"counter!");
            }}
        "#,id=layout.record));
    }
    #[test]
    fn native_shared_and_mutable_views_borrow_non_copy_data_in_place() {
        let checked = checked();
        let binding = checked.program.functions[0].bindings.iter().find(|binding| binding.name == "box").unwrap();
        let Type::Record(record)=binding.ty else{panic!("record binding")};
        let layout=Layout::for_record(record,&checked.program.records).expect("nominal record layout");
        assert_eq!(layout.leaves,Layout::for_binding(binding,&checked.program.records,true).unwrap().leaves);
        let Type::Record(inner) = checked.program.records[layout.record].fields[0].ty.unwrap() else { panic!("inner record"); };
        let declaration = layout.declaration(8, true, "initial()", &checked.program.records, |id| format!("Record{id}"));
        let shared = layout.selected_value(8, &[], crate::Mode::Read, &checked.program.records, |id| format!("ReadRecord{id}")).expect("whole shared selection");
        let exclusive = layout.selected_value(8, &[], crate::Mode::Mutate, &checked.program.records, |id| format!("MutRecord{id}")).expect("whole exclusive selection");
        let owned = layout.owning_value(8, &checked.program.records, |id| format!("Record{id}"));
        let label = layout.leaves[1].native_name(8);
        let native_type = |_, _, ty| match ty { Type::Int => ("i64".into(), vec![]), Type::String => ("S".into(), vec!["S".into()]), _ => panic!("unexpected leaf type") };
        let read_declarations = layout.view_declarations(&checked.program.records, |id| format!("ReadRecord{id}"), native_type, false);
        let mut_declarations = layout.view_declarations(&checked.program.records, |id| format!("MutRecord{id}"), native_type, true);
        assert!(read_declarations.contains(&format!("struct ReadRecord{outer}<'a, N0S>" , outer=layout.record)));
        assert!(read_declarations.contains(&format!("field0: ReadRecord{inner}<'a, N0S>")));
        let bridges = crate::scoped_record_bridge::declarations(&layout, crate::scoped_record_bridge::Types {
            read_trait: "ReadOuter", mutate_trait: "MutateOuter",
            owned: &format!("Record{}<S>",layout.record), shared: &format!("ReadRecord{}<'a, S>",layout.record), exclusive: &format!("MutRecord{}<'a, S>",layout.record), parameters: &["S".into()],
        }, |leaf| if leaf.ty == Type::String { "S".into() } else { "i64".into() });
        let source = format!(r#"
            struct Record{inner}<S> {{ field0: i64, field1: S }}
            struct Record{outer}<S> {{ field0: Record{inner}<S>, field1: i64 }}
            {read_declarations}
            {mut_declarations}
            {bridges}
            fn inspect(value: &impl ReadOuter<Field0Field0=i64,Field0Field1=String,Field1=i64>)->usize {{ value.tsr_field0_field1().len() + *value.tsr_field0_field0() as usize + *value.tsr_field1() as usize }}
            fn update(value: &mut impl MutateOuter<Field0Field0=i64,Field0Field1=String,Field1=i64>) {{ value.tsr_field0_field1_mut().push_str("!"); *value.tsr_field0_field0_mut() += 3; *value.tsr_field1_mut() += 2; }}
            fn initial()->Record{outer}<String> {{ Record{outer} {{ field0: Record{inner} {{ field0:4,field1:"hello".into() }},field1:9 }} }}
            fn main() {{
                {declaration}
                {{ let view = {shared}; assert_eq!(inspect(&view),18); assert!(std::ptr::eq(view.field0.field1, &{label})); }}
                {{ let mut view = {exclusive}; update(&mut view); }}
                assert_eq!({label}, "hello!");
                {{ let view = {shared}; assert_eq!(inspect(&view),24); }}
                let rebuilt = {owned};
                assert_eq!(inspect(&rebuilt),24);
            }}
        "#, outer=layout.record);
        native(&source);
        let invalid = source.replace(&declaration, &format!("{declaration} {{ let mut readonly = {shared}; update(&mut readonly); }}"));
        native_rejection(&invalid, "MutateOuter");
    }
    #[test]
    fn native_split_storage_evaluates_once_restores_fields_and_moves_without_clone() {
        let checked = checked();
        let binding = checked.program.functions[0].bindings.iter().find(|binding| binding.name == "box").unwrap();
        let layout = Layout::for_binding(binding, &checked.program.records, true).unwrap();
        let Type::Record(inner) = checked.program.records[layout.record].fields[0].ty.unwrap() else { panic!("inner record"); };
        let declaration = layout.declaration(7, true, "{ calls += 1; initial() }", &checked.program.records, |id| format!("Record{id}"));
        let reconstructed = layout.owning_value(7, &checked.program.records, |id| format!("Record{id}"));
        let label = layout.selected_value(7, &[0, 1], crate::Mode::Consume, &checked.program.records, |id| format!("Record{id}")).expect("selected owned leaf");
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
