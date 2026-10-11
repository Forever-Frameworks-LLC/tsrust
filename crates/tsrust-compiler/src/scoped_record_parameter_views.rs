//! One whole-record transport at borrowed parameter entry.
use super::*;

impl Emitter<'_> {
    /// Closure binding tables retain outer bindings before their actual parameters.
    /// Their parameter count is an arity, not a binding-index boundary.
    fn record_parameter_binding(&self, fun: &CheckedFunction, id: usize) -> bool {
        for closure in &self.program.closures {
            if std::ptr::eq(fun, &closure.function) {
                return closure.parameters.contains(&id);
            }
            for method in &closure.methods {
                if std::ptr::eq(fun, &method.function) {
                    return method.parameters.contains(&id);
                }
            }
        }
        id < fun.parameter_count
    }
    pub(super) fn record_parameter_layout(&self,fun:&CheckedFunction,id:usize)->Option<crate::scoped_record_places::Layout> {
        let binding=fun.bindings.get(id)?;
        if binding.borrowed_element.is_none() && (!((binding.parameter && self.record_parameter_binding(fun,id)) || binding.capture_reference) || !matches!(binding.mode,Mode::Read|Mode::Mutate)) {return None;}
        let Type::Record(record)=binding.input_ty.unwrap_or(binding.ty) else{return None;};
        let declaration=self.program.scoped_records.declaration(record);
        if !self.borrowing_records.contains(&declaration) || self.program.records[declaration].fields.is_empty(){return None;}
        let mut layout=crate::scoped_record_places::Layout::for_record(declaration,&self.program.records)?;
        // The bridge keeps nominal identity, while each nullable leaf binds the
        // callback variables of this particular parameter occurrence.
        for leaf in &mut layout.leaves {
            let mut selected=Type::Record(record);
            for &field in &leaf.path {
                let Type::Record(parent)=selected else{return None;};
                selected=self.program.records[parent].fields[field].ty?;
            }
            if selected.union().is_some() { leaf.ty=selected; }
        }
        (!layout.leaves.is_empty()).then_some(layout)
    }
    /// Return an entry operation only when the nominal bridge has static leaves.
    pub(super) fn record_parameter_entry_view(&self, fun: &CheckedFunction, id: usize) -> Option<String> {
        self.record_parameter_layout(fun,id)?;
        let binding=&fun.bindings[id];
        let (mutable,method)=if binding.borrowed_element.is_none() && binding.mode==Mode::Mutate{("mut ","tsr_fields_mut")}else{("","tsr_fields")};
        Some(format!("let {mutable}record_fields{id}=value{id}.{method}();"))
    }
    pub(super) fn record_parameter_bound(&self, fun:&CheckedFunction,id:usize,function:usize)->Option<String> {
        self.record_parameter_bound_with(fun, id, |path| self.record_replacement_bounds(function,id,path))
    }
    pub(super) fn record_parameter_bound_with(&self, fun:&CheckedFunction,id:usize,mut replacements:impl FnMut(&[usize])->Vec<String>)->Option<String> {
        let layout=self.record_parameter_layout(fun,id)?;
        let binding=&fun.bindings[id];
        let record=layout.record;
        let fields=layout.leaves.iter().map(|leaf| {
            let name=leaf.path.iter().map(|index|format!("Field{index}")).collect::<String>();
            if matches!(leaf.ty,Type::Callback(_)) {
                let bounds=self.callback_bounds([leaf.ty],true);
                let bound=bounds.first().and_then(|bound|bound.split_once(':')).map(|(_,bound)|bound.trim()).expect("callback leaf bound");
                let replacements=replacements(&leaf.path);
                let replacements=if replacements.is_empty(){String::new()}else{format!(" + {}",replacements.join(" + "))};
                format!("{name}:{bound} + Sized{replacements}")
            }else{format!("{name}={}",self.native_type(leaf.ty))}
        }).collect::<Vec<_>>().join(",");
        let access=if binding.mode==Mode::Mutate{"Mutate"}else{"Read"};
        Some(format!("R{id}:Tsr{access}Record{record}<{fields}>"))
    }
    pub(super) fn record_parameter_borrowed_value(&self,fun:&CheckedFunction,id:usize,path:&[usize],mode:Mode)->Option<String> {
        if !matches!(mode,Mode::Read|Mode::Mutate){return None;}
        let mut record=self.record_parameter_layout(fun,id)?.record;
        for &field in path {let Type::Record(child)=self.program.records[record].fields.get(field)?.ty? else{return None;};record=child;}
        if self.program.records[record].fields.is_empty(){return None;}
        let mutable=mode==Mode::Mutate;
        Some(format!("&{}{}",if mutable{"mut "}else{""},self.record_view_from_fields(record,&mut path.to_vec(),mutable,&format!("record_fields{id}"))))
    }
    pub(super) fn record_view_from_fields(&self,record:usize,path:&mut Vec<usize>,mutable:bool,fields_name:&str)->String {
        let mut fields=self.program.records[record].fields.iter().enumerate().map(|(index,field)| {
            path.push(index);
            let value=if let Some(Type::Record(child))=field.ty && !self.program.records[child].fields.is_empty() {
                self.record_view_from_fields(child,path,mutable,fields_name)
            }else{
                let member=path.iter().map(|index|format!("field{index}")).collect::<Vec<_>>().join("_");
                format!("&{}*{fields_name}.{member}",if mutable{"mut "}else{""})
            };
            path.pop();format!("field{index}:{value}")
        }).collect::<Vec<_>>().join(",");
        if self.record_runtime.ordered.contains(&record){let _=write!(fields,",key_order:&{}*{fields_name}.{}",if mutable{"mut "}else{""},Emitter::record_order_member(path));}
        format!("Tsr{}View{record}{{{fields}}}",if mutable{"Mutate"}else{"Read"})
    }
    /// Static leaf access starts at the entry view, never at the whole carrier.
    pub(super) fn record_parameter_leaf_place(&self, fun: &CheckedFunction, id: usize, path: &[usize]) -> Option<String> {
        let layout=self.record_parameter_layout(fun,id)?;
        if !layout.leaves.iter().any(|leaf|leaf.path==path){return None;}
        let member=path.iter().map(|index|format!("field{index}")).collect::<Vec<_>>().join("_");
        Some(format!("*record_fields{id}.{member}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closure_parameter_layout_uses_checked_binding_identity() {
        let source = crate::format(r#"
            type Holder = { run: ReadOnly<Function<int>> | undefined; };
            function drive(outer: ReadOnly<Holder>): void {
                const prefix = [1];
                const holder: Holder = { run: undefined };
                const action = (target: Mutable<Holder>): void => {
                    console.log(prefix[0]);
                    target.run = (): int => prefix[0];
                };
                action(holder);
            }
            function main() {}
        "#).unwrap();
        assert_eq!(crate::format(&source).unwrap(), source);
        let checked = crate::check(&source).unwrap();
        let emitter = emitter_for(&checked.program, "");
        let closure = checked.program.closures.iter().find(|closure| {
            closure.parameters.iter().any(|&id| matches!(closure.function.bindings[id].input_ty.unwrap_or(closure.function.bindings[id].ty), Type::Record(_)))
        }).expect("record-taking closure");
        let id = closure.parameters[0];
        assert!(id >= closure.function.parameter_count, "witness must use an offset binding ID");
        assert!(closure.function.bindings[id].parameter);
        let mut outer_parameters = 0;
        for candidate in 0..closure.function.bindings.len() {
            let binding = &closure.function.bindings[candidate];
            if binding.parameter && !binding.capture_reference && !closure.parameters.contains(&candidate) {
                outer_parameters += 1;
                assert!(emitter.record_parameter_layout(&closure.function, candidate).is_none(), "outer parameter is not a closure parameter");
            }
        }
        assert!(outer_parameters > 0, "witness must retain an outer parameter binding");
        assert!(emitter.record_parameter_layout(&closure.function, id).is_some(), "checked closure parameter needs a record view");
    }
    #[test]
    fn ordinary_borrowed_record_parameters_use_one_entry_view() {
        let source=crate::format("type Holder={run:ReadOnly<Function<int>>;label:string;};function touch(holder:Mutable<Holder>):void{holder.label=\"changed\";}function inspect(holder:ReadOnly<Holder>):int{return holder.run();}function main(){}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        let rust=emit(&checked.program);
        assert!(rust.contains("let mut record_fields0=value0.tsr_fields_mut();"),"ordinary mutable entry is missing");
        assert!(rust.contains("let record_fields0=value0.tsr_fields();"),"ordinary shared entry is missing");
        crate::scoped_record_places::tests::native(&rust);
    }
    #[test]
    fn ordinary_parameter_routing_executes_nested_calls() {
        let source=crate::format("type Holder={run:ReadOnly<Function<int>>;label:string;};type Pair={left:Holder;right:Holder;};function touch(left:Mutable<Holder>,right:Mutable<Holder>):void{left.label=\"left!\";right.label=\"right!\";}function inspect(pair:ReadOnly<Pair>):int{return pair.left.run()+pair.right.run();}function relay(pair:Mutable<Pair>):void{touch(pair.left,pair.right);}function main(){const a=[3];const b=[11];const pair:Pair={left:{run:():int=>a[0],label:\"left\"},right:{run:():int=>b[0],label:\"right\"}};relay(pair);console.log(inspect(pair));console.log(pair.left.label,pair.right.label);}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        crate::scoped_record_places::tests::native_output(&emit(&checked.program),"14\nleft! right!\n");
        let source=source.replace("return pair.left.run() + pair.right.run();", "return 14;").replace("const a = [3];", "const a = \"abc\";").replace("const b = [11];", "const b = \"abcdefghijk\";").replace("a[0]", "a.length").replace("b[0]", "b.length").replace("touch(pair.left, pair.right);", "try { touch(pair.left, pair.right); } finally { pair.left.label = \"done\"; }");
        let source=crate::format(&source).unwrap();assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        crate::scoped_record_places::tests::native_output(&emit(&checked.program),"14\ndone right!\n");
    }
    #[test]
    fn scoped_parameter_can_borrow_a_plain_nested_record() {
        let source=crate::format("type Plain={label:string;};type Holder={run:ReadOnly<Function<int>>;payload:Plain;};function touch(value:Mutable<Plain>):void{value.label=\"changed\";}function read(value:ReadOnly<Plain>):int{return value.label.length;}function relay(holder:Mutable<Holder>):void{touch(holder.payload);console.log(read(holder.payload));}function main(){const text=\"x\";const holder:Holder={run:():int=>text.length,payload:{label:\"old\"}};relay(holder);console.log(holder.payload.label);}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        crate::scoped_record_places::tests::native_output(&emit(&checked.program),"7\nchanged\n");
        let nested=source.replace("payload: Plain", "payload: Branch").replace("type Holder", "type Branch = { leaf: Plain; }; type Holder").replace("holder.payload", "holder.payload.leaf").replace("payload: {", "payload: { leaf: {").replace("label: \"old\"", "label: \"old\" }");
        let nested=crate::format(&nested).unwrap();assert_eq!(crate::format(&nested).unwrap(),nested);
        let checked=crate::check(&nested).unwrap();
        crate::scoped_record_places::tests::native_output(&emit(&checked.program),"7\nchanged\n");
    }
    #[test]
    fn borrowed_plain_child_preserves_constructor_property_order() {
        let source=crate::format("type Plain={first:string;second:string;};type Holder={run:ReadOnly<Function<int>>;payload:Plain;};function keys(value:ReadOnly<Plain>):void{const names=Object.keys(value);console.log(names[0],names[1]);}function main(){const text=\"x\";const holder:Holder={run:():int=>text.length,payload:{second:\"b\",first:\"a\"}};keys(holder.payload);}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        crate::scoped_record_places::tests::native_output(&emit(&checked.program),"second first\n");
    }
    #[test]
    fn repeated_children_keep_separate_order_through_mutable_views() {
        let source=crate::format("type Plain={first:string;second:string;};type Holder={run:ReadOnly<Function<int>>;payload:Plain;};type Pair={left:Holder;right:Holder;};function keys(value:ReadOnly<Plain>):void{const names=Object.keys(value);console.log(names[0],names[1],value.first);}function change(value:Mutable<Plain>):void{value.first=\"z\";keys(value);}function relay(pair:Mutable<Pair>):void{change(pair.left.payload);keys(pair.right.payload);}function main(){const text=\"x\";const pair:Pair={left:{run:():int=>text.length,payload:{second:\"b\",first:\"a\"}},right:{run:():int=>text.length,payload:{first:\"c\",second:\"d\"}}};relay(pair);}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        crate::scoped_record_places::tests::native_output(&emit(&checked.program),"second first z\nfirst second c\n");
    }
    #[test]
    fn borrowed_ordered_child_values_and_entries_copy_owned_fields() {
        let source=crate::format("type Plain={first:string;second:string;};type Holder={run:ReadOnly<Function<int>>;payload:Plain;};function show(value:ReadOnly<Plain>):void{const values=Object.values(value);console.log(values[0],values[1]);const entries=Object.entries(value);console.log(entries[0][0],entries[0][1]);values[0]=\"copy\";entries[0][1]=\"entry\";console.log(value.second,values[0]);console.log(value.second,entries[0][1]);}function main(){const text=\"x\";const holder:Holder={run:():int=>text.length,payload:{second:\"b\",first:\"a\"}};show(holder.payload);}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        crate::scoped_record_places::tests::native_output(&emit(&checked.program),"b a\nsecond b\nb copy\nb entry\n");
    }
    #[test]
    fn borrowed_ordered_child_console_inspection_keeps_constructor_order() {
        let source=crate::format("type Plain={first:string;second:string;};type Holder={run:ReadOnly<Function<int>>;payload:Plain;};function show(value:ReadOnly<Plain>):void{console.log(value);}function main(){const text=\"x\";const holder:Holder={run:():int=>text.length,payload:{second:\"b\",first:\"a\"}};show(holder.payload);}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        crate::scoped_record_places::tests::native_output(&emit(&checked.program),"{ second: 'b', first: 'a' }\n");
    }
    #[test]
    fn borrowed_console_inspection_keeps_order_at_every_nested_node() {
        let source=crate::format("type Detail={first:string;second:string;};type Plain={detail:Detail;count:int;};type Holder={run:ReadOnly<Function<int>>;payload:Plain;};function show(value:ReadOnly<Plain>):void{console.log(value);}function main(){const text=\"x\";const holder:Holder={run:():int=>text.length,payload:{count:4,detail:{second:\"b\",first:\"a\"}}};show(holder.payload);}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        crate::scoped_record_places::tests::native_output(&emit(&checked.program),"{ count: 4, detail: { second: 'b', first: 'a' } }\n");
    }
    #[test]
    fn borrowed_ordered_child_spread_and_rest_keep_order() {
        let source=crate::format("type Plain={first:string;second:string;};type Holder={run:ReadOnly<Function<int>>;payload:Plain;};function show(value:ReadOnly<Plain>):void{const copy={...value};const {first,...tail}=value;const names=Object.keys(copy);const remaining=Object.keys(tail);console.log(names[0],names[1],copy.second);console.log(first,remaining[0],tail.second);}function main(){const text=\"x\";const holder:Holder={run:():int=>text.length,payload:{second:\"b\",first:\"a\"}};show(holder.payload);}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        crate::scoped_record_places::tests::native_output(&emit(&checked.program),"second first b\na second b\n");
    }
    #[test]
    fn borrowed_values_copy_nested_record_owners_and_their_order() {
        let source=crate::format("type Detail={first:string;second:string;};type Plain={detail:Detail;tag:string;};type Holder={run:ReadOnly<Function<int>>;payload:Plain;};function show(value:ReadOnly<Plain>):void{const values=Object.values(value);const copied=values[1];if(typeof copied===\"object\"){const names=Object.keys(copied);console.log(names[0],names[1],copied.second);}else{console.log(\"wrong value\");}}function main(){const text=\"x\";const holder:Holder={run:():int=>text.length,payload:{tag:\"label\",detail:{second:\"b\",first:\"a\"}}};show(holder.payload);}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        crate::scoped_record_places::tests::native_output(&emit(&checked.program),"second first b\n");
    }
    #[test]
    fn mutable_order_transport_borrows_metadata_in_place() {
        let source=crate::format("type Plain={first:string;second:string;};type Holder={run:ReadOnly<Function<int>>;payload:Plain;};function keys(value:ReadOnly<Plain>):string[]{return Object.keys(value);}function main(){}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();let mut rust=emit(&checked.program);
        let plain=checked.program.records.iter().position(|record|record.name.text=="Plain").unwrap();
        rust.push_str(&format!(r#"
            fn verify_order_transport(){{
                let mut value=Record{plain}{{field0:String::Owned("a".into()),field1:String::Owned("b".into()),key_order:[1,0]}};
                let pointer=value.key_order.as_ptr();
                {{let view=value.tsr_fields_mut();assert_eq!(view.key_order.as_ptr(),pointer);view.key_order.swap(0,1);}}
                assert_eq!(value.key_order,[0,1]);
            }}
        "#));
        rust=rust.replace("fn main() -> () {","fn main() -> () {verify_order_transport();");
        crate::scoped_record_places::tests::native(&rust);
    }
    #[test]
    fn array_join_registration_accepts_scoped_record_metadata() {
        let source=crate::format("type Holder={run:ReadOnly<Function<int>>;};function main(){const text=\"x\";const holder:Holder={run:():int=>text.length};console.log([\"a\",\"b\"].join(\",\"));console.log(holder.run());}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        crate::scoped_record_places::tests::native_output(&emit(&checked.program),"a,b\n1\n");
    }
    #[test]
    fn empty_record_children_keep_their_opaque_owned_abi() {
        let source=crate::format("type Empty={};type Holder={run:ReadOnly<Function<int>>;empty:Empty;};function marker(value:ReadOnly<Empty>):void{}function relay(holder:ReadOnly<Holder>):void{marker(holder.empty);}function main(){const text=\"x\";const holder:Holder={run:():int=>text.length,empty:{}};relay(holder);}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        let emitter=emitter_for(&checked.program,"");
        let marker=checked.program.functions.iter().find(|function|function.name=="marker").unwrap();
        assert!(emitter.record_parameter_entry_view(marker,0).is_none());
        crate::scoped_record_places::tests::native(&emit(&checked.program));
    }
    #[test]
    fn parameter_entry_preserves_existing_mode_boundaries() {
        let source=crate::format("type Holder={run:ReadOnly<Function<int>>;label:string;};type Plain={label:string;};function reader(value:ReadOnly<Holder>):void{}function owner(value:Owned<Holder>):void{}function plain(value:ReadOnly<Plain>):void{}function scalar(value:int):void{}function main(){}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();let emitter=emitter_for(&checked.program,"");
        for name in ["reader","owner","plain","scalar"] {
            let function=checked.program.functions.iter().find(|function|function.name==name).unwrap();
            let entry=emitter.record_parameter_entry_view(function,0);
            if name=="reader" {assert_eq!(entry.as_deref(),Some("let record_fields0=value0.tsr_fields();"));}
            else {assert!(entry.is_none(),"{name} must retain its existing ABI");}
        }
    }
    #[test]
    fn borrowed_parameter_entry_exposes_independent_mutable_leaves() {
        let source=crate::format("type Holder={run:ReadOnly<Function<int>>;label:string;};type Pair={left:Holder;right:Holder;};function relay(pair:Mutable<Pair>):void{}function reader(pair:ReadOnly<Pair>):void{}function main(){}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        let emitter=emitter_for(&checked.program,"");
        let function=checked.program.functions.iter().find(|function|function.name=="relay").unwrap();
        let entry=emitter.record_parameter_entry_view(function,0).expect("borrowed scoped record entry");
        assert_eq!(entry.matches("tsr_fields_mut()").count(),1);
        let reader=checked.program.functions.iter().find(|function|function.name=="reader").unwrap();
        let read_entry=emitter.record_parameter_entry_view(reader,0).expect("shared entry");
        let left=emitter.record_parameter_leaf_place(function,0,&[0,1]).expect("left leaf");
        let right=emitter.record_parameter_leaf_place(function,0,&[1,1]).expect("right leaf");
        assert!(emitter.record_parameter_leaf_place(function,0,&[0]).is_none(),"subtrees require their own view");
        assert!(emitter.record_parameter_leaf_place(function,0,&[2]).is_none(),"unknown paths stay rejected");
        let outer=checked.program.records.iter().position(|record|record.name.text=="Pair").unwrap();
        let Type::Record(inner)=checked.program.records[outer].fields[0].ty.unwrap() else{panic!("child")};
        let mut rust=emit(&checked.program);
        rust.push_str(&format!(r#"
            struct Action;
            impl ReadCallback<(),i64> for Action{{fn invoke_read(&self,_:())->i64{{3}}}}
            fn inspect<V:TsrReadRecord{outer}>(value0:&V)->i64 where V::Field0Field0:ReadCallback<(),i64>,V::Field1Field0:ReadCallback<(),i64>{{
                {read_entry}
                ReadCallback::invoke_read(record_fields0.field0_field0,())+ReadCallback::invoke_read(record_fields0.field1_field0,())
            }}
            fn update<V:TsrMutateRecord{outer}<Field0Field1=String,Field1Field1=String>>(value0:&mut V){{
                {entry}
                let left=&mut ({left});
                let right=&mut ({right});
                *left=String::Owned("left!".into());*right=String::Owned("right!".into());
            }}
            fn verify(){{
                let mut value=Record{outer}{{field0:Record{inner}{{field0:Action,field1:String::Owned("left".into())}},field1:Record{inner}{{field0:Action,field1:String::Owned("right".into())}}}};
                assert_eq!(inspect(&value),6);update(&mut value);assert_eq!(inspect(&value),6);
                assert_eq!(value.field0.field1.as_ref(),"left!");assert_eq!(value.field1.field1.as_ref(),"right!");
            }}
        "#));
        rust=rust.replace("fn main() -> () {","fn main() -> () {verify();");
        crate::scoped_record_places::tests::native(&rust);
    }
}
