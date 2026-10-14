//! Discover storage conversions shared by body, handler and cleanup emission.
use super::*;

impl Emitter<'_> {
    #[inline(never)]
    pub(super) fn map_exception_storage_plan(&self,fun:&CheckedFunction,mut statements:Vec<&Stmt>,declarations: Declarations<'_, '_>)->Option<(Vec<String>,std::collections::HashMap<String,String>)> {
        let mut places=Vec::new();
        let mut native_places=std::collections::HashMap::new();
        while let Some(item)=statements.pop() {
            if !item.labels.is_empty() && !matches!(item.kind,StmtKind::Break|StmtKind::Continue|StmtKind::While{..}|StmtKind::Loop{..}|StmtKind::ForOf{..}) && !(matches!(item.kind,StmtKind::Block(_)) && !fun.storage_exits.map_block_epochs(item.span.start).is_empty()) {return None;}
            match &item.kind {
                StmtKind::Block(body)|StmtKind::ForOf{body,..}=>statements.extend(body),
                StmtKind::If{then_body,else_body,..}=>{statements.extend(then_body);statements.extend(else_body);},
                StmtKind::Try(region)=>statements.extend(region.statements()),
                StmtKind::While{body,..} if !fun.storage_exits.map_iteration_epochs(item.span).is_empty()=>statements.extend(body),
                StmtKind::Loop{body,update,tail_reachable,..} if !fun.storage_exits.map_iteration_epochs(item.span).is_empty()=>{
                    statements.extend(body);
                    if *tail_reachable {statements.extend(update);}
                },
                StmtKind::Let{..}|StmtKind::Expr(_)|StmtKind::Assign{..}|StmtKind::Throw(_)|StmtKind::Return(_)|StmtKind::Break|StmtKind::Continue=>{},
                _=>return None,
            }
            for epoch in fun.storage_exits.map_entry_epochs(item.span).iter().chain(fun.storage_exits.map_block_epochs(item.span.start)) {
                let place=self.map_entry_epoch_place(fun,epoch);
                if !places.contains(&place) {
                    let original=fun.bindings[epoch.place.binding].ty;
                    let ty=self.map_storage_leaf_type(original,&epoch.place.fields).expect("checked Map path");
                    native_places.insert(place.clone(),self.native_type(ty));
                    places.push(place);
                }
            }
            if let Some((place,native))=self.local_map_clear_storage(fun,declarations,item) && !places.contains(&place) {
                native_places.insert(place.clone(),native);
                places.push(place);
            }
        }
        if places.is_empty(){return None;}
        places.sort();
        Some((places,native_places))
    }
}
