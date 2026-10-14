//! Consume cleanup results before forwarding their independently typed owners.
use super::*;
use statement_work::Work;
type Layout=std::collections::BTreeMap<crate::capture_loan_owners::Place,Type>;
impl Emitter<'_> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn schedule_map_finish<'a,'s:'a>(&'s self,fun:&'a CheckedFunction,planned:&'a CheckedFunction,region:&'a ExceptionRegion,body:&'a [Stmt],declarations:Declarations<'a,'_>,id:usize,error_name:&str,contract:&crate::error_effects::FailureSet,runtime:Option<Type>,places:&[String],storage:&str,pattern:&str,span:crate::Span,normal:Option<&'a Layout>,normal_code:&str,override_drop:&str,normal_fields:Option<&str>,work:&mut Vec<Work<'a,'s>>) {
        let boundary=region.body.first().map_or(span.start,|statement|statement.span.start);
        let fallback=map_return_storage::empty_layout();
        // A preserved pending return is not a return performed by cleanup.
        // Only cleanup's own return/control exits need this specialized carrier.
        let has_controls=!exception_contract_emit::escaping_controls(body).is_empty();
        let source_control=exception_contract_emit::has_source_return(body) || has_controls;
        let carrier=if source_control{
            self.map_completion_carrier(fun,format!("MapCleanupControl{id}"),places,storage,boundary,crate::analysis::MapCompletionPhase::Cleaned,fallback)
        }else{None};
        let error=fun.storage_exits.map_completion_layout(boundary,crate::analysis::MapCompletionPhase::Cleaned,Some(crate::completion::CompletionKind::Throw));
        let returned=fun.storage_exits.map_completion_layout(boundary,crate::analysis::MapCompletionPhase::Cleaned,Some(crate::completion::CompletionKind::Return));

        // Body setup is deferred until its preceding siblings have restored scopes.
        let body_carrier=carrier;
        let error_name=error_name.to_owned();let contract=contract.clone();let places=places.to_vec();
        let storage=storage.to_owned();let pattern=pattern.to_owned();let normal_code=normal_code.to_owned();let override_drop=override_drop.to_owned();
        // Carrier metadata is immutable; the continuation owns it across body emission.
        let carrier=std::rc::Rc::new(body_carrier);let finish_carrier=carrier.clone();
        let finish_name=error_name.clone();let finish_contract=contract.clone();let finish_places=places.clone();let finish_storage=storage.clone();
        work.push(Work::Resume(Box::new(move |output,_,results| {
            let body=results.pop().expect("completed Map cleanup");
            let carrier=&*finish_carrier;let error_name=finish_name.as_str();let contract=&finish_contract;let places=&finish_places;let storage=finish_storage.as_str();let pattern=pattern.as_str();
        let thrown=if contract.runtime || contract.opaque || !contract.explicit.is_empty(){self.map_layout_storage_abrupt_dispatch(fun,"TsrCompletion::Throw(pending_error)",error_name,contract,runtime,span,error)}else{"match pending_error {}".into()};
        let legacy=self.map_layout_carried_cleanup_dispatch(fun,"legacy",error_name,contract,runtime,places,span,error);
        let legacy_error=error.map(|layout|self.map_error_storage_type(fun,places,layout)).unwrap_or_else(||storage.to_owned());
        let legacy_type=format!("TsrCompletion<(),(TsrStorageExit<{}>,{storage}),({error_name},{legacy_error})>",self.native_type(fun.result));
        let values=format!("({},)",places.join(","));
        let common=if has_controls || carrier.is_none(){format!("let legacy:{legacy_type}=TsrCompletion::Return((pending_exit,{values}));{legacy}")}else{"unreachable!(\"cleanup has no loop control\")".into()};
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
        output.push_str(&format!("{{{declaration}let (normal_owner,error_owner,control_owner)={{let completed={body};match completed{{TsrCompletion::Normal(storage)=>(Some(storage),None,None),TsrCompletion::Throw((error,storage))=>(None,Some((error,storage)),None),{carried},_=>unreachable!(\"carried cleanup completion\")}}}};if normal_owner.is_some(){{let storage=normal_owner.unwrap();drop(error_owner);drop(control_owner);{normal_code}}}else{{drop(normal_owner);{override_drop}if error_owner.is_some(){{let (pending_error,{pattern})=error_owner.unwrap();drop(control_owner);{thrown}}}else{{drop(error_owner);if control_owner.is_some(){{{selected}}}else{{unreachable!(\"one cleanup completion owner\")}}}}}}}}"));
        })));
        work.push(Work::Captured);
        self.schedule_map_phase_body(planned,body,declarations,id,&error_name,contract,runtime,&storage,&places,true,true,normal,error,carrier.as_ref().as_ref(),normal_fields,&[],work);
        work.push(Work::Capture);
    }

}
