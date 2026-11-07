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
                if views.contains(field.ty.expect("checked field")) || contains_finite_owned(program,field.ty.expect("checked field")) {fields.insert((base,position));}
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
        // Recursive components have concrete declarations, including their
        // lifetime-free symbolic base. They cannot use expanding field generics.
        if self.native_views.cyclic(record) { return false; }
        let base=self.program.scoped_records.declaration(record);
        self.native_record_fields.contains(base,position)
            || self.scoped_record_native_parameter(ty)
            || self.scoped_record_native_parameter(self.program.records[base].fields[position].ty.expect("checked declared field"))
    }
}

// Owned finite families parameterize records without introducing view lifetimes.
pub(super) fn contains_finite_owned(program:&CheckedProgram,root:Type)->bool{
    let mut pending=vec![root];let mut seen=HashSet::new();
    while let Some(ty)=pending.pop(){
        if !seen.insert(ty){continue;}
        match ty{
            Type::StoredCallable(family) if program.stored_contracts.scoped_native_family(family).is_some_and(|layout|layout.lifetimes()==0)=>return true,
            Type::Callback(id) if program.callback_contracts[id].kind.is_owned() && program.absent_callback_payloads.contains(&id)=>return true,
            Type::Record(id)=>pending.extend(program.records[id].fields.iter().filter_map(|field|field.ty)),
            Type::Array(element)=>pending.push(element.ty()),
            Type::Union(id)=>pending.extend(&program.unions[id.index()].members),
            _=>{},
        }
    }false
}
