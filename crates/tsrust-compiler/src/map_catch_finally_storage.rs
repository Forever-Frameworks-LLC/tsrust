//! Keep body, handler and cleanup owners independent across catch-plus-finally.
use super::*;

impl Emitter<'_> {
    #[inline(never)]
    pub(super) fn map_catch_finally_storage_epoch(&self,output:&mut String,fun:&CheckedFunction,statement:&Stmt,region:&ExceptionRegion,declarations:&[Stmt],depth:usize)->bool {
        let (Some(handler),Some(cleanup))=(&region.catch,&region.finally) else{return false;};
        let statements=region.body.iter().chain(&handler.body).chain(cleanup).collect();
        let Some((places,native_places))=self.map_exception_storage_plan(fun,statements,declarations) else{return false;};
        let incoming=handler.error_contract.as_deref().expect("checked catch contract");
        if incoming.explicit.is_empty() && !incoming.runtime && !incoming.opaque{return false;}
        let values=format!("({},)",places.join(","));
        let pattern=format!("({},)",places.iter().map(|place|format!("mut {place}")).collect::<Vec<_>>().join(","));
        let storage_type=format!("({},)",places.iter().map(|place|native_places[place].as_str()).collect::<Vec<_>>().join(","));
        let (_,_,runtime)=self.exception_regions.current_transport().expect("enclosing catch completion");
        let first=self.completion_next.get();self.completion_next.set(first+5);
        let mut discovered=statement.clone();if let StmtKind::Try(region)=&mut discovered.kind{region.finally=None;}
        let outgoing=crate::error_effects::annotate_region_body(&mut [discovered],&self.program.error_effects);
        let mut cleanup_discovery=cleanup.clone();
        let cleanup_contract=crate::error_effects::annotate_region_body(&mut cleanup_discovery,&self.program.error_effects);
        let (in_decl,in_name)=self.completion_error_layout(&format!("StorageIncoming{first}"),incoming,runtime,statement.span);
        let (out_decl,out_name)=self.completion_error_layout(&format!("StorageOutgoing{first}"),&outgoing,runtime,statement.span);
        let (cleanup_decl,cleanup_name)=self.completion_error_layout(&format!("StorageCleanup{first}"),&cleanup_contract,runtime,statement.span);
        let boundary=region.body.first().map_or(statement.span.start,|statement|statement.span.start);
        let body_error=fun.storage_exits.map_completion_layout(boundary,crate::analysis::MapCompletionPhase::Body,Some(crate::completion::CompletionKind::Throw));
        let handler_error=fun.storage_exits.map_completion_layout(boundary,crate::analysis::MapCompletionPhase::Handled,Some(crate::completion::CompletionKind::Throw));
        let cleaned=fun.storage_exits.map_completion_layout(boundary,crate::analysis::MapCompletionPhase::Cleaned,None);
        let cleaned_error=fun.storage_exits.map_completion_layout(boundary,crate::analysis::MapCompletionPhase::Cleaned,Some(crate::completion::CompletionKind::Throw));
        let fallback=std::collections::BTreeMap::new();
        let return_carrier=self.map_completion_carrier(fun,format!("MapCatchControl{first}"),&places,&storage_type,boundary,crate::analysis::MapCompletionPhase::Handled,&fallback);
        let handler_throws=outgoing.runtime || outgoing.opaque || !outgoing.explicit.is_empty();
        let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(),&self.program.records);
        let setup=if let Some(binding)=handler.binding {
            let caught=exception_contract_emit::checked_catch_value("caught_error",&in_name,incoming,&unions,runtime,statement.span).expect("checked catch payload");
            format!("let {}value{binding}={caught};",if fun.bindings[binding].mutable{"mut "}else{""})
        }else{"drop(caught_error);".into()};
        let normal_finish=self.map_finish_cleanup(fun,fun,region,cleanup,declarations,first+2,&cleanup_name,&cleanup_contract,runtime,&places,&storage_type,&pattern,statement.span,cleaned,"storage","");
        let error_plan=fun.storage_exits.map_cleanup_plan(boundary,Some(crate::completion::CompletionKind::Throw)).map(|plan|plan.function(fun));
        let error_fun=error_plan.as_ref().unwrap_or(fun);let error_body=error_plan.as_ref().map_or(cleanup.as_slice(),|plan|plan.body.as_slice());
        let abrupt_cleanup=crate::completion::definitely_abrupt(cleanup,0);
        let resume_error=if abrupt_cleanup{"unreachable!(\"abrupt cleanup cannot resume pending error\")".into()}else{self.map_layout_storage_abrupt_dispatch(fun,"TsrCompletion::Throw(pending_error)",&out_name,&outgoing,runtime,statement.span,cleaned_error)};
        let normal=format!("let {pattern}=storage;{resume_error}");
        let throw_finish=if handler_throws{self.map_finish_cleanup(fun,error_fun,region,error_body,declarations,first+3,&cleanup_name,&cleanup_contract,runtime,&places,&storage_type,&pattern,statement.span,cleaned_error,&normal,"drop(pending_error);")}else{"match pending_error {}".into()};
        let result=self.native_type(fun.result);
        let (outer_error,_,_)=self.exception_regions.current_transport().expect("checked cleanup destination");
        let resumed=format!("match pending_exit{{TsrStorageExit::Return(value)=>TsrCompletion::<(),{result},{outer_error}>::Return(value),TsrStorageExit::Break(target)=>TsrCompletion::Break(target),TsrStorageExit::Continue(target)=>TsrCompletion::Continue(target)}}");
        let resume_control=if abrupt_cleanup{"unreachable!(\"abrupt cleanup cannot resume pending control\")".into()}else{format!("{};unreachable!()",self.map_cleanup_control_dispatch(fun,&resumed,statement.span))};
        let normal=format!("let {pattern}=storage;{resume_control}");
        let return_finish=self.map_finish_cleanup(fun,fun,region,cleanup,declarations,first+4,&cleanup_name,&cleanup_contract,runtime,&places,&storage_type,&pattern,statement.span,None,&normal,"drop(pending_exit);");
        let impossible="TsrCompletion::Break(_)|TsrCompletion::Continue(_)=>unreachable!(\"carried catch completion\")";
        let normal_arm="TsrCompletion::Normal(storage)=>(Some(storage),None,None)";
        let return_arm=if return_carrier.is_some(){"TsrCompletion::Return(control)=>(None,None,Some(control))"}else{"TsrCompletion::Return((exit,storage))=>(None,None,Some((exit,storage)))"};
        let handler_throw=if handler_throws{"TsrCompletion::Throw((error,storage))=>(None,Some((error,storage)),None)"}else{"TsrCompletion::Throw(error)=>match error {}"};
        let direct_return=return_carrier.as_ref().map(|carrier|{
            let precise=self.map_precise_return_cleanup(fun,region,declarations,first+4,carrier,&cleanup_name,&cleanup_contract,runtime,&places,&storage_type,&pattern,statement.span);
            self.map_direct_return_route(carrier,&pattern,precise,first)
        });
        *self.inline_map_return.borrow_mut()=direct_return.clone();
        let pending=self.map_return_phase_storage_completion_body(fun,&region.body,declarations,first,&in_name,incoming.clone(),runtime,&storage_type,&places,true,true,None,body_error,return_carrier.as_ref());
        *self.inline_map_return.borrow_mut()=direct_return;
        let handled=self.map_return_phase_storage_completion_body(fun,&handler.body,declarations,first+1,&out_name,outgoing.clone(),runtime,&storage_type,&places,handler_throws,true,None,handler_error,return_carrier.as_ref());
        let handler_split=format!("{{let completed={handled};match completed{{{normal_arm},{handler_throw},{return_arm},{impossible}}}}}");
        let body_split=format!("{{let completed={pending};match completed{{{normal_arm},TsrCompletion::Throw((error,storage))=>(None,Some((error,storage)),None),{return_arm},{impossible}}}}}");
        let error_storage=handler_error.map(|layout|self.map_error_storage_type(fun,&places,layout)).unwrap_or_else(||storage_type.clone());
        // Consume the body enum before catch, then consume every owner option
        // before cleanup. Inactive alternatives must not retain capture loans.
        let control_type=return_carrier.as_ref().map_or_else(||format!("(TsrStorageExit<{result}>,{storage_type})"),|carrier|carrier.native.clone());
        let carrier_declaration=return_carrier.as_ref().map_or(String::new(),|carrier|self.map_carrier_declaration(carrier));
        let selected_control=if let Some(carrier)=&return_carrier {
            let control_cleanups=self.map_precise_control_cleanups(fun,region,declarations,carrier,&cleanup_name,&cleanup_contract,runtime,&places,&storage_type,&pattern,statement.span);
            let precise=self.map_precise_return_cleanup(fun,region,declarations,first+4,carrier,&cleanup_name,&cleanup_contract,runtime,&places,&storage_type,&pattern,statement.span);
            let has_other=self.control_targets.snapshot().iter().any(|target|[crate::completion::CompletionKind::Break(Some(*target)),crate::completion::CompletionKind::Continue(Some(*target))].into_iter().any(|kind|fun.storage_exits.map_completion_layout(boundary,crate::analysis::MapCompletionPhase::Handled,Some(kind)).is_some()));
            let common=if has_other{format!("{{{return_finish}}}")}else{"unreachable!(\"no pending loop control\")".into()};
            self.map_select_control_owner(carrier,&pattern,precise,common,control_cleanups)
        }else{format!("let (pending_exit,{pattern})=control_owner.unwrap();{return_finish}")};
        let joined=format!("{{let (body_normal,body_error,body_control)={{let {pattern}={values};{body_split}}};let owners:(Option<{storage_type}>,Option<({out_name},{error_storage})>,Option<{control_type}>)=if body_normal.is_some(){{let storage=body_normal.unwrap();drop(body_error);drop(body_control);(Some(storage),None,None)}}else{{drop(body_normal);if body_error.is_some(){{let (caught_error,{pattern})=body_error.unwrap();drop(body_control);{setup}{handler_split}}}else{{drop(body_error);(None,None,body_control)}}}};owners}}");
        let indent="    ".repeat(depth);
        let _=writeln!(output,"{indent}{in_decl}{out_decl}{cleanup_decl}{carrier_declaration}let {pattern}={{let (normal_owner,error_owner,control_owner)={joined};if normal_owner.is_some(){{let {pattern}=normal_owner.unwrap();drop(error_owner);drop(control_owner);{normal_finish}}}else{{drop(normal_owner);if error_owner.is_some(){{let (pending_error,{pattern})=error_owner.unwrap();drop(control_owner);{throw_finish}}}else{{drop(error_owner);if control_owner.is_some(){{{selected_control}}}else{{unreachable!(\"exactly one pending cleanup owner\")}}}}}}}};");
        let boundary=places.iter().flat_map(|place|[self.map_storage_transfer_at_exit(fun,&region.body,place),self.map_storage_transfer_at_exit(fun,&handler.body,place),self.map_storage_transfer_at_exit(fun,cleanup,place)]).flatten().map(|span|span.end).max().unwrap_or(statement.span.end).max(statement.span.end);
        self.exception_regions.renew_storage(&places,boundary);
        true
    }
}
