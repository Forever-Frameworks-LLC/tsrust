//! Native-bearing instances determine stable generic positions on nominal records.
use super::*;

#[derive(Default)]
pub(super) struct Index {
    fields:HashSet<(usize,usize)>,
}
impl Index {
    pub(super) fn new(program:&CheckedProgram,views:&native_view_emit::Shapes)->Self {
        let mut fields=HashSet::new();
        for (id,record) in program.records.iter().enumerate() {
            let base=program.scoped_records.declaration(id);
            for (position,field) in record.fields.iter().enumerate() {
                if views.contains(field.ty.expect("checked field")) {fields.insert((base,position));}
            }
        }
        Self{fields}
    }
    pub(super) fn contains(&self,record:usize,position:usize)->bool {
        self.fields.contains(&(record,position))
    }
}
impl Emitter<'_> {
    pub(super) fn scoped_record_native_field(&self,record:usize,position:usize,ty:Type)->bool {
        let base=self.program.scoped_records.declaration(record);
        self.native_record_fields.contains(base,position)
            || self.scoped_record_native_parameter(ty)
            || self.scoped_record_native_parameter(self.program.records[base].fields[position].ty.expect("checked declared field"))
    }
}
