//! Move proven surviving payloads into a narrower native environment.
use super::*;

impl Emitter<'_> {
    pub(super) fn map_entry_epoch_code(&self,fun:&CheckedFunction,epoch:&crate::analysis::MapEntryEpoch)->String {
        let mut source=epoch.input;
        let mut target=epoch.output;
        let mut indices=Vec::new();
        for name in &epoch.place.fields {
            let (Type::Record(input),Type::Record(output))=(source,target) else {unreachable!("checked record Map path")};
            let index=self.program.records[input].fields.iter().position(|field|field.name.text==*name).expect("checked Map field");
            indices.push(index);
            source=self.program.records[input].fields[index].ty.unwrap();
            target=self.program.records[output].fields[index].ty.unwrap();
        }
        let place=if indices.is_empty(){format!("value{}",epoch.place.binding)}else{self.owned_record_leaf_place(fun,epoch.place.binding,&indices).expect("owning Map leaf")};
        let (Type::Map(input),Type::Map(output))=(source,target) else {unreachable!("checked Map epoch")};
        let source=self.program.records[input].map_definition.as_ref().unwrap().value;
        let target=self.program.records[output].map_definition.as_ref().unwrap().value;
        let native=self.native_type(Type::Map(output));
        if epoch.empty{return format!("drop({place});let mut {place}: {native}=<{native}>::new();");}
        let converted=self.map_entry_payload_projection(source,target,"entry_value");
        format!("let mut {place}: {native}=<{native}>::from_values(({place}).tsr_owned_items().map(|entry|{{let (entry_key,entry_value)=TsrMapEntry::into_pair(entry);(entry_key,{converted})}}));")
    }

    pub(super) fn map_entry_payload_projection(&self, source:Type,target:Type,value:&str) -> String {
        if source==target {return value.into();}
        match (source,target) {
            (Type::Record(source),Type::Record(target)) => {
                let fields=self.program.records[source].fields.iter().zip(&self.program.records[target].fields).enumerate().map(|(index,(source,target))|format!("field{index}:{}",self.map_entry_payload_projection(source.ty.unwrap(),target.ty.unwrap(),&format!("({value}).field{index}")))).collect::<Vec<_>>().join(",");
                let metadata=if self.record_runtime.ordered.contains(&target){format!(",key_order:({value}).key_order")}else{String::new()};
                format!("Record{}{{{fields}{metadata}}}",self.program.scoped_records.declaration(target))
            }
            (Type::BorrowedCallable(source),Type::BorrowedCallable(target)) => {
                let targets=self.program.stored_contracts.origins(target).unwrap();
                let arms=self.program.stored_contracts.origins(source).unwrap().iter().enumerate().map(|(index,&origin)|{
                    let result=if targets.contains(&origin){self.stored_callable_constructor(target,origin,"payload")}else{"unreachable!(\"excluded Map callback environment\")".into()};
                    format!("BorrowedCallable{source}::{}(payload)=>{result}",crate::borrowed_callable_dispatch::variant(index,self.program.stored_contracts.origins(source).unwrap().len()))
                }).collect::<Vec<_>>().join(",");
                format!("match {value}{{{arms}}}")
            }
            (Type::BorrowedCallable(source),Type::Closure(target)) => {
                let origins=self.program.stored_contracts.origins(source).unwrap();
                let arms=origins.iter().enumerate().map(|(index,&origin)|format!("BorrowedCallable{source}::{}(payload)=>{}",crate::borrowed_callable_dispatch::variant(index,origins.len()),if origin==target {"payload"}else{"unreachable!(\"excluded Map callback environment\")"})).collect::<Vec<_>>().join(",");
                format!("match {value}{{{arms}}}")
            }
            _=>self.record_carrier_conversion(source,target,value),
        }
    }
}
