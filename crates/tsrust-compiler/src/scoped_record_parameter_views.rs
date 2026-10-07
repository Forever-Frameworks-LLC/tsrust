//! One whole-record transport at borrowed parameter entry.
use super::*;

impl Emitter<'_> {
    /// Return an entry operation only when the nominal bridge has static leaves.
    pub(super) fn record_parameter_entry_view(&self, fun: &CheckedFunction, id: usize) -> Option<String> {
        let binding=fun.bindings.get(id)?;
        if !binding.parameter || !matches!(binding.mode,Mode::Read|Mode::Mutate) {return None;}
        let Type::Record(record)=binding.input_ty.unwrap_or(binding.ty) else{return None;};
        if !self.scoped_record_native_parameter(Type::Record(record)){return None;}
        let declaration=self.program.scoped_records.declaration(record);
        let layout=crate::scoped_record_places::Layout::for_record(declaration,&self.program.records)?;
        if layout.leaves.is_empty(){return None;}
        let (mutable,method)=if binding.mode==Mode::Mutate{("mut ","tsr_fields_mut")}else{("","tsr_fields")};
        Some(format!("let {mutable}record_fields{id}=value{id}.{method}();"))
    }
    /// Static leaf access starts at the entry view, never at the whole carrier.
    pub(super) fn record_parameter_leaf_place(&self, fun: &CheckedFunction, id: usize, path: &[usize]) -> Option<String> {
        self.record_parameter_entry_view(fun,id)?;
        let binding=&fun.bindings[id];
        let Type::Record(record)=binding.input_ty.unwrap_or(binding.ty) else{return None;};
        let declaration=self.program.scoped_records.declaration(record);
        let layout=crate::scoped_record_places::Layout::for_record(declaration,&self.program.records)?;
        if !layout.leaves.iter().any(|leaf|leaf.path==path){return None;}
        let member=path.iter().map(|index|format!("field{index}")).collect::<Vec<_>>().join("_");
        Some(format!("*record_fields{id}.{member}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
