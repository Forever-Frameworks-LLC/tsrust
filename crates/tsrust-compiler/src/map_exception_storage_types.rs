//! Normal exception transport restores the checked common storage family.
use super::*;

impl Analyzer {
    #[inline(never)]
    pub(super) fn plan_map_exception_storage_exit(&mut self,context:&Context,region:&ExceptionRegion,results:&mut HashMap<usize,Type>,local:&std::collections::BTreeSet<crate::capture_loan_owners::Place>) {
        let mut places=std::collections::BTreeSet::new();
        let mut pending=region.statements().collect::<Vec<_>>();
        while let Some(statement)=pending.pop() {
            for epoch in context.storage_exits.map_entry_epochs(statement.span) {
                if local.contains(&epoch.place){places.insert(epoch.place.clone());}
            }
            match &statement.kind {
                StmtKind::Block(body)|StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>pending.extend(body),
                StmtKind::Loop{body,update,tail_reachable,..}=>{pending.extend(body);if *tail_reachable{pending.extend(update);}},
                StmtKind::If{then_body,else_body,..}=>{pending.extend(then_body);pending.extend(else_body);},
                StmtKind::Try(region)=>pending.extend(region.statements()),
                _=>{},
            }
        }
        for place in places {
            let Some(root)=results.get(&place.binding).copied() else{continue;};
            let original=context.bindings[place.binding].ty;
            let normal=region.body.first().and_then(|statement|context.storage_exits.map_completion_layout(statement.span.start,map_completion_payloads::Phase::Handled,None));
            let leaf=if region.finally.is_none() {normal.and_then(|types|types.get(&place)).copied()}else{None}
                .or_else(||self.scoped_array_field_type(original,&place.fields));
            let Some(leaf)=leaf else{continue;};
            let output=self.scoped_array_replace_field(root,&place.fields,leaf).expect("checked transported Map path");
            results.insert(place.binding,output);
        }
    }
}
