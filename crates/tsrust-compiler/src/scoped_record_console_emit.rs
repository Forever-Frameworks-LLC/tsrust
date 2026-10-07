//! Console inspection accepts nominal owning records and borrowed record views.
use super::*;
impl Emitter<'_> {
    pub(super) fn borrowed_record_console_head(&self,id:usize)->Option<String> {
        let declaration=self.program.scoped_records.declaration(id);
        if !self.borrowing_records.contains(&declaration) || self.program.records[declaration].fields.is_empty(){return None;}
        let layout=crate::scoped_record_places::Layout::for_record(declaration,&self.program.records)?;
        let associated=layout.leaves.iter().map(|leaf|format!("{}={}",leaf.path.iter().map(|index|format!("Field{index}")).collect::<String>(),self.native_type(leaf.ty))).collect::<Vec<_>>().join(",");
        let view=self.record_view_from_fields(declaration,&mut Vec::new(),false,"record_fields");
        Some(format!("fn tsr_console_record_{id}<V:TsrReadRecord{declaration}<{associated}>>(value:&V,depth:usize)->TsrConsoleValue{{if depth>=128{{return TsrConsoleValue::Truncated;}}let record_fields=value.tsr_fields();let value={view};let mut fields=Vec::new();"))
    }
}
