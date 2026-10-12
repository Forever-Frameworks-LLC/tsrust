//! Eligible local Map storage follows explicit moves into record fields.
use super::*;
impl Analyzer {
    pub(super) fn move_map_entry_places(&self,statement:&Stmt,local:&mut std::collections::BTreeSet<crate::capture_loan_owners::Place>) {
        let StmtKind::Let{binding:Some(destination),value,..}=&statement.kind else{return;};
        let mut pending=vec![(value,Vec::<String>::new())];
        while let Some((value,path))=pending.pop() {
            match &value.kind {
                ExprKind::Unary{op:UnaryOp::RecordCarrier,value,..}=>pending.push((value,path)),
                ExprKind::Record(fields)=>for (name,value) in fields {
                    let mut nested=path.clone();nested.push(name.text.clone());pending.push((value,nested));
                },
                _ if matches!(value.ty,Some(Type::Map(_)|Type::Record(_)))=>{
                    let Some(source)=Self::root_binding(value) else{continue;};
                    let Some((_,fields))=Self::direct_fields(value) else{continue;};
                    let moved=local.iter().filter(|place|place.binding==source && place.fields.starts_with(&fields)).cloned().collect::<Vec<_>>();
                    for place in moved {
                        local.remove(&place);
                        let mut target=path.clone();target.extend_from_slice(&place.fields[fields.len()..]);
                        local.insert(crate::capture_loan_owners::Place{binding:*destination,fields:target});
                    }
                },
                _=>{},
            }
        }
    }
}
