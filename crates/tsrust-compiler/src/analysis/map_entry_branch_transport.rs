//! Restore a common representation after an arm's scheduled storage analysis.
use super::*;
impl Analyzer {
    pub(super) fn finish_map_branch_entries(&mut self,context:&mut Context,span:Span,right:bool,body:&[Stmt],results:&HashMap<usize,Type>,arm_results:&HashMap<usize,Type>,local:&std::collections::BTreeSet<crate::capture_loan_owners::Place>) {
            let mut restore=Vec::new();
            if !crate::completion::definitely_abrupt(body) {
                for place in local {
                    let (Some(input),Some(output))=(arm_results.get(&place.binding),results.get(&place.binding)) else{continue;};
                    if self.scoped_array_field_type(*input,&place.fields)==self.scoped_array_field_type(*output,&place.fields){continue;}
                    restore.push(storage_exit_availability::MapEntryEpoch{place:place.clone(),input:*input,output:*output,empty:false,before:false});
                }
            }
            context.storage_exits.record_map_branch_restore(span,right,restore);
    }
}
