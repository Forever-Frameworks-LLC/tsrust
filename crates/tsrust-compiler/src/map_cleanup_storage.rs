//! Consume cleanup results before forwarding their independently typed owners.
use super::*;
type Layout=std::collections::BTreeMap<crate::capture_loan_owners::Place,Type>;
impl Emitter<'_> {
    #[inline(never)]
    pub(super) fn map_finish_cleanup(&self,fun:&CheckedFunction,planned:&CheckedFunction,region:&ExceptionRegion,body:&[Stmt],declarations:&[Stmt],id:usize,error_name:&str,contract:&crate::error_effects::FailureSet,runtime:Option<Type>,places:&[String],storage:&str,pattern:&str,span:crate::Span,normal:Option<&Layout>,normal_code:&str,override_drop:&str)->String {
        let boundary=region.body.first().map_or(span.start,|statement|statement.span.start);
        let fallback=std::collections::BTreeMap::new();
        // A preserved pending return is not a return performed by cleanup.
        // Only cleanup's own return/control exits need this specialized carrier.
        let source_control=exception_contract_emit::has_source_return(body) || !exception_contract_emit::escaping_controls(body).is_empty();
        let carrier=if source_control{
            self.map_completion_carrier(fun,format!("MapCleanupControl{id}"),places,storage,boundary,crate::analysis::MapCompletionPhase::Cleaned,&fallback)
        }else{None};
        let error=fun.storage_exits.map_completion_layout(boundary,crate::analysis::MapCompletionPhase::Cleaned,Some(crate::completion::CompletionKind::Throw));
        let returned=fun.storage_exits.map_completion_layout(boundary,crate::analysis::MapCompletionPhase::Cleaned,Some(crate::completion::CompletionKind::Return));
        let body=self.map_return_phase_storage_completion_body(planned,body,declarations,id,error_name,contract.clone(),runtime,storage,places,true,true,normal,error,carrier.as_ref());
        let thrown=self.map_layout_storage_abrupt_dispatch(fun,"TsrCompletion::Throw(pending_error)",error_name,contract,runtime,span,error);
        let legacy=self.map_layout_carried_cleanup_dispatch(fun,"legacy",error_name,contract,runtime,places,span,error);
        let legacy_error=error.map(|layout|self.map_error_storage_type(fun,places,layout)).unwrap_or_else(||storage.to_owned());
        let legacy_type=format!("TsrCompletion<(),(TsrStorageExit<{}>,{storage}),({error_name},{legacy_error})>",self.native_type(fun.result));
        let values=format!("({},)",places.join(","));
        let common=format!("let legacy:{legacy_type}=TsrCompletion::Return((pending_exit,{values}));{legacy}");
        let (declaration,carried,selected)=if let Some(carrier)=&carrier {
            let owner=self.map_outgoing_return_storage(fun,span,returned);
            let resumed=self.exception_regions.dispatch_return("pending_return",span,owner.as_deref());
            let controls=carrier.controls.iter().map(|(kind,layout)|self.map_selected_control_dispatch(fun,*kind,Some(layout),span)).collect();
            (self.map_carrier_declaration(carrier),"TsrCompletion::Return(control)=>(None,None,Some(control))".to_owned(),self.map_select_control_owner(carrier,pattern,resumed,common,controls))
        }else{
            (String::new(),"TsrCompletion::Return(control)=>(None,None,Some(control))".to_owned(),format!("let (pending_exit,{pattern})=control_owner.unwrap();{common}"))
        };
        let selected=if source_control{selected}else{"unreachable!(\"cleanup has no source control exit\")".into()};
        // End the result enum's scope before dispatch. Inactive alternatives
        // must not retain capture loans while the selected owner is restored.
        format!("{{{declaration}let (normal_owner,error_owner,control_owner)={{let completed={body};match completed{{TsrCompletion::Normal(storage)=>(Some(storage),None,None),TsrCompletion::Throw((error,storage))=>(None,Some((error,storage)),None),{carried},_=>unreachable!(\"carried cleanup completion\")}}}};if normal_owner.is_some(){{let storage=normal_owner.unwrap();drop(error_owner);drop(control_owner);{normal_code}}}else{{drop(normal_owner);{override_drop}if error_owner.is_some(){{let (pending_error,{pattern})=error_owner.unwrap();drop(control_owner);{thrown}}}else{{drop(error_owner);if control_owner.is_some(){{{selected}}}else{{unreachable!(\"one cleanup completion owner\")}}}}}}}}")
    }
}
