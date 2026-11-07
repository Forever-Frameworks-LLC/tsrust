//! Array ownership moves into a concrete standard-library cursor.
use super::*;
impl Emitter<'_>{
    pub(super) fn iterator_record(&self,output:&mut String,id:usize)->bool{
        let record=&self.program.records[id];
        if record.native_type!=Some(crate::native_signatures::NativeTypeId::IteratorCursor){return false;}
        let storage=record.fields.iter().find(|field|field.name.text=="_items").and_then(|field|field.ty).expect("reserved owning cursor storage");
        let Type::Array(element)=storage else{unreachable!("cursor storage is an array")};
        let item=element.ty();
        let _=writeln!(output,"type Record{id}=TsrIteratorCursor<{}>;",self.native_type(item));true
    }
    pub(super) fn iterator_leaf(&self,expr:&Expr,source:&str)->String{
        if expr.target==Some(CallTarget::IteratorCursor){return format!("({source}).tsr_into_iterator_cursor()");}
        let Type::Union(union)=expr.ty.expect("checked iterator result") else{unreachable!("iterator result union")};
        let arm=|has_value:bool|{
            let (variant,&member)=self.program.unions[union.index()].members.iter().enumerate().find(|(_,ty)|matches!(ty,Type::Record(id) if self.program.records[*id].fields.iter().any(|field|field.name.text=="value")==has_value)).expect("iterator result alternative");
            let Type::Record(id)=member else{unreachable!()};
            let mut fields=self.program.records[id].fields.iter().enumerate().map(|(index,field)|format!("field{index}:{}",if field.name.text=="value"{"item"}else if has_value{"false"}else{"true"})).collect::<Vec<_>>();
            if self.record_runtime.ordered.contains(&id){fields.push(format!("key_order:[{}]",(0..fields.len()).map(|index|index.to_string()).collect::<Vec<_>>().join(",")));}
            let record=format!("Record{id}{{{}}}",fields.join(","));
            format!("Union{}::V{variant}({})",union.index(),self.union_store_payload(union,member,&record))
        };
        format!("match ({source}).next(){{Some(item)=>{},None=>{}}}",arm(true),arm(false))
    }
    pub(super) fn iterator_runtime(&self,output:&mut String,owned:bool,numeric:bool,record_arrays:&std::collections::HashSet<usize>){
        if !self.program.records.iter().any(|record|record.native_type==Some(crate::native_signatures::NativeTypeId::IteratorCursor)){return;}
        output.push_str(crate::stdlib::iterator::RUNTIME);
        if owned{output.push_str("impl<T> TsrIntoIteratorCursor<T> for Array<T>{fn tsr_into_iterator_cursor(self)->TsrIteratorCursor<T>{match self{Self::Heap(values)=>TsrIteratorCursor::Heap(values.into_iter()),Self::Small{values,..}=>TsrIteratorCursor::Inline(values.into_iter().flatten())}}}\n");}
        if numeric{output.push_str("impl<T:Copy+Default> TsrIntoIteratorCursor<T> for NumericArray<T>{fn tsr_into_iterator_cursor(self)->TsrIteratorCursor<T>{match self{Self::Heap(values)=>TsrIteratorCursor::Heap(values.into_iter()),Self::Small{values,len}=>TsrIteratorCursor::Numeric(values.into_iter().take(len))}}}\n");}
        for id in 0..self.program.records.len(){
            if !record_arrays.contains(&id){continue;}
            let _=writeln!(output,"impl TsrIntoIteratorCursor<Record{id}> for RecordArray{id}{{fn tsr_into_iterator_cursor(mut self)->TsrIteratorCursor<Record{id}>{{TsrIteratorCursor::Heap(std::mem::take(&mut self.0).into_iter())}}}}");
        }
    }
}
