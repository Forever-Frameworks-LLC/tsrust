//! Capture release proofs at a control statement's normal completion join.
use super::*;

impl Analyzer {
    #[inline(never)]
    pub(super) fn record_map_completion_release(&mut self,context:&mut Context,left:&mut [Stmt],right:&mut [Stmt],span:Span)->Result<(),Diagnostic> {
        if !context.storage_exits.has_map_entry_survivors() {return Ok(());}
        let mut places=std::collections::BTreeMap::<crate::capture_loan_owners::Place,HashSet<usize>>::new();
        for body in [left,right] {
            crate::source_walk::body(body,&mut |_|Ok(()),&mut |value| {
                if let Some(state)=context.storage_exits.map_entry_survivors(value.span) && state.consistent {places.entry(state.place.clone()).or_default().extend(state.released.iter().copied());}
                Ok(())
            })?;
        }
        for (place,released) in places {
            let binding=&context.bindings[place.binding];
            if binding.parameter || binding.capture_reference || binding.transferred.is_some(){continue;}
            let Some(input @ Type::Map(_))=self.scoped_array_field_type(binding.ty,&place.fields) else {continue;};
            if released.is_empty(){continue;}
            // Join reaching payloads, rather than taking either arm's local
            // snapshot. Reinserted and conditionally retained entries stay live.
            let payloads=context.capture_loan_owners.map_payloads(&place);
            context.storage_exits.record_map_completion_survivors(span,storage_exit_availability::MapEntrySurvivors{place,input,payloads,released,consistent:true});
        }
        Ok(())
    }
}
