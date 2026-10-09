//! Narrow arm-local storage and restore a common representation at fallthrough.
use super::*;
impl Analyzer {
    #[inline(never)]
    pub(super) fn scoped_map_branch_entries(&mut self,context:&mut Context,statement:&mut Stmt,results:&HashMap<usize,Type>,demand:&map_entry_demand::Demand,local:&std::collections::BTreeSet<crate::capture_loan_owners::Place>)->Result<(),Diagnostic> {
        let StmtKind::If{then_body,else_body,..}=&mut statement.kind else{return Ok(());};
        for (right,body) in [(false,then_body),(true,else_body)] {
            let mut arm_results=results.clone();
            self.scoped_map_entry_epochs_with_demand(context,body,&mut arm_results,demand,local)?;
            let mut restore=Vec::new();
            if !crate::completion::definitely_abrupt(body,0) {
                for place in local {
                    let (Some(input),Some(output))=(arm_results.get(&place.binding),results.get(&place.binding)) else{continue;};
                    if self.scoped_array_field_type(*input,&place.fields)==self.scoped_array_field_type(*output,&place.fields){continue;}
                    restore.push(storage_exit_availability::MapEntryEpoch{place:place.clone(),input:*input,output:*output,empty:false,before:false});
                }
            }
            context.storage_exits.record_map_branch_restore(statement.span,right,restore);
        }
        Ok(())
    }
}
