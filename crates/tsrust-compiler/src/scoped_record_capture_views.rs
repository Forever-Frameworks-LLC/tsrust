//! Record captures retain borrowing views of the original independent leaves.
use super::*;
impl Emitter<'_> {
    pub(super) fn record_capture_is_view(&self,capture:&crate::analysis::Capture,closure:&crate::analysis::CheckedClosure)->bool {
        matches!(capture.storage,Mode::Read|Mode::Mutate) && matches!(closure.function.bindings[capture.inner].ty,Type::Record(record) if self.borrowing_records.contains(&self.program.scoped_records.declaration(record)) && !self.program.records[record].fields.is_empty())
    }
    pub(super) fn record_capture_view_type(&self,ty:Type,mutable:bool,lifetime:&str,lifetimes:&[String])->String {
        let Type::Record(record)=ty else{unreachable!()};
        let layout=crate::scoped_record_places::Layout::for_record(record,&self.program.records).expect("validated record capture layout");
        let mut next=0;let mut args=vec![lifetime.to_owned()];
        for leaf in layout.leaves {
            if !self.scoped_record_native_parameter(leaf.ty){continue;}
            let count=crate::environment_lifetimes::count(leaf.ty,&self.program.records,|id|self.program.unions[id.index()].members.clone(),|ty|match ty{Type::Closure(id)=>self.program.closures[id].lifetime_count,Type::BorrowedCallable(id)=>self.program.stored_contracts.origins(id).unwrap().iter().map(|&id|self.program.closures[id].lifetime_count).sum(),_=>0});
            args.push(self.named_record_environment_type(leaf.ty,&lifetimes[next..next+count]));next+=count;
        }
        format!("Tsr{}View{}<{}>",if mutable{"Mutate"}else{"Read"},self.program.scoped_records.declaration(record),args.join(","))
    }
    pub(super) fn record_capture_value(&self,fun:&CheckedFunction,id:usize,mode:Mode)->String {
        if let Some(value)=self.owned_record_value(fun,id,&[],mode).or_else(||self.record_parameter_borrowed_value(fun,id,&[],mode)) {
            return value.strip_prefix(if mode==Mode::Mutate{"&mut ("}else{"&("}).and_then(|v|v.strip_suffix(')')).unwrap_or_else(||value.strip_prefix(if mode==Mode::Mutate{"&mut "}else{"&"}).unwrap()).to_owned();
        }
        let Type::Record(record)=fun.bindings[id].ty else{unreachable!()};
        let declaration=self.program.scoped_records.declaration(record);
        let receiver=self.binding(fun,id,mode);let method=if mode==Mode::Mutate{"tsr_fields_mut"}else{"tsr_fields"};
        let view=self.record_view_from_fields(declaration,&mut Vec::new(),mode==Mode::Mutate,"capture_fields");
        format!("{{let capture_fields=({receiver}).{method}();{view}}}")
    }
}
