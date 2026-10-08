//! Owning record carrier conversion evaluates its input once and moves leaves.
use super::*;
impl Emitter<'_>{
    pub(super) fn record_carrier_conversion(&self,source:Type,target:Type,value:&str)->String{
        if source==target{return value.into();}
        match(source,target){
            (Type::Array(source),Type::Array(target))=>{
                let converted=self.record_carrier_conversion(source.ty(),target.ty(),"record_array_item");
                format!("({value}).into_iter().map(|record_array_item|{{{converted}}}).collect::<{}>()",self.native_type(Type::Array(target)))
            }
            (Type::Record(source),Type::Record(target))=>{
                let fields=self.program.records[source].fields.iter().zip(&self.program.records[target].fields).enumerate().map(|(index,(source,target))|format!("field{index}:{}",self.record_carrier_conversion(source.ty.unwrap(),target.ty.unwrap(),&format!("({value}).field{index}")))).collect::<Vec<_>>().join(",");
                let metadata=if self.record_runtime.ordered.contains(&target){format!(",key_order:({value}).key_order")}else{String::new()};
                format!("Record{}{{{fields}{metadata}}}",self.program.scoped_records.declaration(target))
            }
            (Type::Closure(origin),Type::BorrowedCallable(family))=>self.stored_callable_constructor(family,origin,value),
            (Type::BorrowedCallable(source),Type::BorrowedCallable(target))=>{
                let origins=self.program.stored_contracts.origins(source).unwrap();
                let arms=origins.iter().enumerate().map(|(index,&origin)|format!("BorrowedCallable{source}::{}(payload)=>{}",crate::borrowed_callable_dispatch::variant(index,origins.len()),self.stored_callable_constructor(target,origin,"payload"))).collect::<Vec<_>>().join(",");
                format!("match {value}{{{arms}}}")
            }
            _=>panic!("unchecked record carrier conversion {source:?} to {target:?}"),
        }
    }
}
