use super::*;

impl Analyzer {
    #[inline(never)]
    pub(super) fn validate_control_labels(context:&Context,labels:&[Token])->Result<(),Diagnostic>{
        for (index,label) in labels.iter().enumerate(){
            if labels[..index].iter().any(|before|before.text==label.text) || context.loops.iter().any(|frame|frame.labels.iter().any(|before|before.text==label.text)) {
                return Err(Diagnostic::new("duplicate",format!("Label {:?} is already active.",label.text),label.span));
            }
        }
        Ok(())
    }

    #[inline(never)]
    pub(super) fn control_exit(&mut self,context:&mut Context,labels:&[Token],target:&mut Option<ControlTarget>,continuing:bool,span:Span)->Result<(),Diagnostic>{
        let index=if let Some(label)=labels.first(){
            context.loops.iter().rposition(|frame|frame.labels.iter().any(|name|name.text==label.text))
        } else {context.loops.iter().rposition(|frame|!matches!(frame.target,ControlTarget::Block(_)))};
        let Some(index)=index else {
            return Err(Diagnostic::new("loop-control","break and continue require an enclosing target in the same function or closure.",labels.first().map_or(span,|label|label.span)));
        };
        if continuing && matches!(context.loops[index].target,ControlTarget::Block(_)) {
            return Err(Diagnostic::new("loop-control","continue requires a label on an enclosing loop.",labels.first().map_or(span,|label|label.span)));
        }
        if context.region_loop_depth.is_some_and(|depth|index<depth) {
            *target=Some(context.loops[index].target);
            return Ok(());
        }
        *target=Some(context.loops[index].target);
        Self::commit_control_at(context,index,continuing)
    }

    #[inline(never)]
    fn commit_control_at(context:&mut Context,index:usize,continuing:bool)->Result<(),Diagnostic>{
        let flow=context.flow.clone();
        let mut owners=context.capture_loan_owners.clone();
        let frame=&mut context.loops[index];
        let _=owners.leave_scope(frame.scope_start);
        let owner_slot=if continuing {&mut frame.continue_owners}else{&mut frame.break_owners};
        *owner_slot=Some(crate::capture_loan_owners::Owners::reaching(owner_slot.take(),Some(owners)));
        let slot=if continuing {&mut frame.continue_flow}else{&mut frame.break_flow};
        *slot=Some(slot.as_ref().map_or_else(||flow.clone(),|before|crate::flow::join(before,&flow)));
        if continuing {
            if frame.defer_continue {
                let states=frame.entry.iter().map(|(id,_)|Availability::of(&context.bindings[*id])).collect::<Vec<_>>();
                frame.continues=Some(match frame.continues.take(){Some(before)=>before.into_iter().zip(states).map(|(a,b)|a.join(b)).collect(),None=>states});
            }else{Self::check_loop_back_at(context,index)?;}
        }else{
            frame.break_seen=true;
            for ((id,_),state) in frame.entry.iter().zip(&mut frame.breaks){*state=state.clone().join(Availability::of(&context.bindings[*id]));}
        }
        Ok(())
    }

    /// Commit a cleaned exit in isolation, preserving the normal continuation.
    #[inline(never)]
    pub(super) fn commit_region_control(&mut self,context:&mut Context,target:ControlTarget,continuing:bool,state:exception_state::ExitState,span:Span)->Result<bool,Diagnostic>{
        let Some(index)=context.loops.iter().rposition(|frame|frame.target==target) else {
            return Err(Diagnostic::new("loop-control","Completion destination is no longer active.",span));
        };
        if context.region_loop_depth.is_some_and(|depth|index<depth) {return Ok(false);}
        let mut path=context.clone();
        state.restore(&mut path);
        Self::commit_control_at(&mut path,index,continuing)?;
        context.loops[index]=path.loops.remove(index);
        Ok(true)
    }

    #[inline(never)]
    pub(super) fn labeled_block(&mut self,context:&mut Context,body:&mut [Stmt],result_type:Type,span:Span,labels:&[Token])->Result<bool,Diagnostic>{
        Self::validate_control_labels(context,labels)?;
        let mut ids=context.names.values().copied().collect::<Vec<_>>();ids.sort_unstable();
        let entry=ids.into_iter().map(|id|(id,Availability::of(&context.bindings[id]))).collect::<Vec<_>>();
        context.loops.push(LoopFrame{labels:labels.to_vec(),target:ControlTarget::Block(span.start),flow_entry:context.flow.clone(),flow_required:HashMap::new(),break_flow:None,continue_flow:None,breaks:vec![Availability::default();entry.len()],break_seen:false, break_owners:None, continue_owners:None,scope_start:context.bindings.len(),defer_continue:false,continues:None,entry,span});
        let exits=self.block(context,body,result_type)?;
        let frame=context.loops.pop().expect("matching label frame");
        exception_state::consume_target(context,frame.target);
        context.capture_loan_owners=crate::capture_loan_owners::Owners::reaching(
            (!exits).then(||context.capture_loan_owners.clone()),frame.break_owners);
        Self::restore_live_capture_loans(context);
        if frame.break_seen {
            for ((id,_),state) in frame.entry.into_iter().zip(frame.breaks){
                let state=if exits {state}else{state.join(Availability::of(&context.bindings[id]))};
                state.restore(&mut context.bindings[id]);
            }
            if let Some(flow)=frame.break_flow{context.flow=if exits {flow}else{crate::flow::join(&context.flow,&flow)};}
        }
        // Preserve the normal/break owner join for later native storage planning.
        // A deletion on one path cannot eliminate entries retained on another.
        self.record_map_completion_release(context,body,&mut [],span)?;
        Ok(exits && !frame.break_seen)
    }
}
