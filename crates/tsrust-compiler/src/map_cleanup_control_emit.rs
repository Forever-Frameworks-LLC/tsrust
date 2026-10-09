//! Resume cleanup controls through the enclosing carried iteration tuple.
use super::*;
impl Emitter<'_> {
    #[inline(never)]
    pub(super) fn map_cleanup_control_dispatch(&self,fun:&CheckedFunction,pending:&str,span:crate::Span)->String {
        let active=self.control_targets.snapshot();
        let mut remaining=active.clone();
        let mut arms=String::new();
        for target in active {
            if let ControlTarget::Block(label)=target {
                if !self.exception_regions.local_control_target(target){continue;}
                let epochs=fun.storage_exits.map_block_epochs(label);
                if epochs.is_empty(){continue;}
                remaining.retain(|candidate|*candidate!=target);
                let mut conversions=String::new();
                for epoch in epochs {
                    let conversion=crate::analysis::MapEntryEpoch{input:fun.bindings[epoch.place.binding].ty,..epoch.clone()};
                    if conversion.input!=conversion.output{conversions.push_str(&self.map_entry_epoch_code(fun,&conversion));}
                }
                let names=epochs.iter().map(|epoch|self.map_entry_epoch_place(fun,epoch)).collect::<Vec<_>>().join(",");
                let _=write!(arms,"TsrCompletion::Break(TsrControlTarget::Block({label}))=>{{{conversions}break 'map_block_{label} ({names},);}},");
                continue;
            }
            let label=match target {ControlTarget::Loop(label)|ControlTarget::Tail(label)=>label,_=>continue};
            // An enclosing finally must receive the control first. Only the
            // region owning this loop may yield its carried body tuple.
            if !self.exception_regions.local_control_target(target){continue;}
            let epochs=fun.storage_exits.map_iteration_epochs_at_label(label);
            if epochs.is_empty(){continue;}
            remaining.retain(|candidate|*candidate!=target);
            let mut conversions=String::new();
            for epoch in epochs {
                let conversion=crate::analysis::MapEntryEpoch{place:epoch.entry.place.clone(),input:fun.bindings[epoch.entry.place.binding].ty,output:epoch.body.output,empty:false,before:false};
                if conversion.input!=conversion.output {conversions.push_str(&self.map_entry_epoch_code(fun,&conversion));}
            }
            let names=epochs.iter().map(|epoch|self.map_entry_epoch_place(fun,&epoch.entry)).collect::<Vec<_>>().join(",");
            for (kind,stop) in [("Break",true),("Continue",false)] {
                let _=write!(arms,"TsrCompletion::{kind}(TsrControlTarget::Loop({label}))=>{{{conversions}break 'map_body_{label} ({stop},{names},);}},");
            }
        }
        if arms.is_empty(){return self.exception_regions.dispatch(pending,&self.control_targets.snapshot(),span).expect("cleanup control destination");}
        let fallback=self.exception_regions.dispatch("other",&remaining,span).expect("cleanup control destination");
        format!("match {pending} {{{arms}other=>{{{fallback};unreachable!(\"cleanup control cannot complete normally\")}}}}")
    }
}
