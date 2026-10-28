//! Explicit record snapshots copy leaves rather than cloning a borrowing view.
use super::*;
impl Emitter<'_> {
    fn parameter_record_source(&self,fun:&CheckedFunction,value:&Expr)->Option<(usize,usize,Vec<usize>)> {
        let mut root=value;
        loop {match &root.kind {
            ExprKind::Name(_)=>break,
            ExprKind::Field(base,_) if matches!(base.ty,Some(Type::Record(_)))=>root=base,
            _=>return None,
        }}
        let(root,steps)=self.storage_steps(value);let id=root.binding?;
        let mut record=self.record_parameter_layout(fun,id)?.record;
        let mut path=Vec::new();
        for step in steps {let StorageStep::Field(field)=step else{return None;};let Type::Record(child)=self.program.records[record].fields.get(field)?.ty? else{return None;};path.push(field);record=child;}
        Some((id,record,path))
    }
    pub(super) fn record_parameter_order_access(&self,fun:&CheckedFunction,value:&Expr)->Option<String> {
        let(id,record,path)=self.parameter_record_source(fun,value)?;
        if self.record_borrow_leaf(record){return None;}
        if !self.record_runtime.ordered.contains(&record){return None;}
        Some(format!("record_fields{id}.{}",Self::record_order_member(&path)))
    }
    pub(super) fn record_parameter_owned_clone(&self,fun:&CheckedFunction,value:&Expr)->Option<String> {
        if self.record_parameter_borrow_leaf(fun,value){return Some(format!("Clone::clone({})",self.expr(fun,value,Mode::Read)));}
        self.record_parameter_fields_clone(fun,value)
    }
    pub(super) fn record_parameter_fields_clone(&self,fun:&CheckedFunction,value:&Expr)->Option<String> {
        if !matches!(value.ty,Some(Type::Record(_))){return None;}
        let(id,record,mut path)=self.parameter_record_source(fun,value)?;
        if !self.record_runtime.cloneable(record){return None;}
        if self.record_borrow_leaf(record){return None;}
        Some(self.record_clone_from_fields(id,record,&mut path))
    }
    pub(super) fn record_parameter_borrow_leaf(&self,fun:&CheckedFunction,value:&Expr)->bool{
        self.parameter_record_source(fun,value).is_some_and(|(_,record,_)|self.record_runtime.cloneable(record)&&self.record_borrow_leaf(record))
    }
    fn record_clone_from_fields(&self,id:usize,record:usize,path:&mut Vec<usize>)->String {
        let mut fields=self.program.records[record].fields.iter().enumerate().map(|(index,field)| {
            path.push(index);
            let copied=if let Some(Type::Record(child))=field.ty && !self.program.records[child].fields.is_empty() && !self.record_borrow_leaf(child){
                self.record_clone_from_fields(id,child,path)
            }else{
                let member=path.iter().map(|index|format!("field{index}")).collect::<Vec<_>>().join("_");
                format!("Clone::clone(&*record_fields{id}.{member})")
            };
            path.pop();format!("field{index}:{copied}")
        }).collect::<Vec<_>>().join(",");
        if self.record_runtime.ordered.contains(&record){let _=write!(fields,",key_order:*record_fields{id}.{}",Self::record_order_member(path));}
        format!("Record{record}{{{fields}}}")
    }
}
