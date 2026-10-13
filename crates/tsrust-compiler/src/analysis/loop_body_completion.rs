//! Join loop completion state after scheduled body analysis has returned.
use super::*;

impl Analyzer {
    #[inline(never)]
    pub(super) fn finish_loop_body(&mut self,context:&mut Context,body:&mut [Stmt],span:Span,entry_owners:crate::capture_loan_owners::Owners,baseline:Vec<Availability>,zero_flow:crate::flow::Facts,exits:bool)->Result<bool,Diagnostic> {
        let frame = context.loops.pop().expect("matching loop frame");
        if !exits {context.storage_exits.record_map_loop_edge(span,storage_exit_availability::MapLoopEdge::Back,&context.capture_loan_owners);}
        if let Some(owners)=&frame.continue_owners {context.storage_exits.record_map_loop_edge(span,storage_exit_availability::MapLoopEdge::Back,owners);}
        if let Some(owners)=&frame.break_owners {context.storage_exits.record_map_loop_edge(span,storage_exit_availability::MapLoopEdge::Break,owners);}
        context.capture_loan_owners = crate::capture_loan_owners::Owners::reaching(
            Some(entry_owners), (!exits).then(||context.capture_loan_owners.clone()));
        if let Some(owners) = &frame.continue_owners { context.capture_loan_owners.join(owners); }
        if let Some(owners) = &frame.break_owners { context.capture_loan_owners.join(owners); }
        Self::restore_live_capture_loans(context);
        exception_state::consume_target(context,frame.target);
        context.flow = crate::flow::join(&zero_flow, &context.flow);
        if let Some(breaks) = frame.break_flow {
            context.flow = crate::flow::join(&context.flow, &breaks);
        }
        let break_seen = frame.break_seen;
        for ((id, _), (condition_state, break_state)) in frame
            .entry
            .into_iter()
            .zip(baseline.into_iter().zip(frame.breaks))
        {
            condition_state
                .join(break_state)
                .restore(&mut context.bindings[id]);
        }
        self.record_map_completion_release(context,body,&mut [],span)?;
        Ok(break_seen)
    }
}
