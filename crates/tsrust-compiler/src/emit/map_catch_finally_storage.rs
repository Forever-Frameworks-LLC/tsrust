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
        let mut field_plan=self.completion_field_plan_inner(fun,statement,region,true);
        let body_fields=field_plan.as_ref().and_then(|_|{
            let mut fields=field_completion_transport::partial_places(fun,&region.body);
            field_completion_transport::normalize(fun,statement.span.start,&mut fields);
            use crate::analysis::MapCompletionPhase::Body;
            Some((self.control_completion_fields(fun,boundary,Body,None,&fields)?,self.control_completion_fields(fun,boundary,Body,Some(crate::completion::CompletionKind::Throw),&fields)?))
        });
        if body_fields.is_none(){field_plan=None;}
        let (body_normal_fields,body_error_fields)=body_fields.unwrap_or_default();
        let body_values=field_plan.as_ref().map(|_|field_completion_transport::tuple(&body_normal_fields.iter().map(|(name,_)|name.clone()).collect::<Vec<_>>()));
        let (body_pattern,body_restore)=if field_plan.is_some(){
            let (fields,restore)=field_completion_transport::restoration(&body_normal_fields.iter().map(|(name,_)|name.clone()).collect::<Vec<_>>(),&format!("map_body_normal{first}"));(format!("(storage,{fields})"),restore)
        }else{("storage".into(),String::new())};
        let caught_names=body_error_fields.iter().enumerate().map(|(index,_)|format!("map_caught_field{first}_{index}")).collect::<Vec<_>>();
        let caught_pattern=map_error_storage_emit::extend_tuple(&pattern,&caught_names);
        let caught_restore=body_error_fields.iter().zip(&caught_names).map(|((field,_),value)|format!("{field}={value};")).collect::<String>();
        let input_fields=field_plan.as_ref().map(|plan|field_completion_transport::tuple(&plan.input_fields));
        let output_fields=field_plan.as_ref().map(|plan|field_completion_transport::tuple(&plan.output_fields));
        let (input_pattern,input_restore)=field_plan.as_ref().map_or_else(||(pattern.clone(),String::new()),|plan|{
            let (fields,restore)=field_completion_transport::restoration(&plan.input_fields,&format!("map_input_field{first}"));(format!("({pattern},{fields})"),restore)
        });
        let (output_pattern,output_restore)=field_plan.as_ref().map_or_else(||(pattern.clone(),String::new()),|plan|{
            let (fields,restore)=field_completion_transport::restoration(&plan.output_fields,&format!("map_output_field{first}"));(format!("({pattern},{fields})"),restore)
        });
        let normal_finish=self.map_finish_field_cleanup(fun,fun,region,cleanup,declarations,first+2,&cleanup_name,&cleanup_contract,runtime,&places,&storage_type,&pattern,statement.span,cleaned,"storage","",output_fields.as_deref());
        let error_plan=fun.storage_exits.map_cleanup_plan(boundary,Some(crate::completion::CompletionKind::Throw)).map(|plan|plan.function(fun));
        let error_fun=error_plan.as_ref().unwrap_or(fun);let error_body=error_plan.as_ref().map_or(cleanup.as_slice(),|plan|plan.body.as_slice());
        let abrupt_cleanup=crate::completion::definitely_abrupt(cleanup);
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
        let direct_return=return_carrier.as_ref().filter(|_|field_plan.is_none() && (exception_contract_emit::has_partial_return(fun,&region.body)
            || exception_contract_emit::has_partial_return(fun,&handler.body))).map(|carrier|{
            let precise=self.map_precise_return_cleanup(fun,region,declarations,first+4,carrier,&cleanup_name,&cleanup_contract,runtime,&places,&storage_type,&pattern,statement.span);
            self.map_direct_return_route(carrier,&pattern,precise,first)
        });
        *self.inline_map_return.borrow_mut()=direct_return.clone();
        let pending=self.map_field_phase_storage_completion_body(fun,&region.body,declarations,first,&in_name,incoming.clone(),runtime,&storage_type,&places,true,true,None,body_error,return_carrier.as_ref(),body_values.as_deref(),&body_error_fields);
        *self.inline_map_return.borrow_mut()=direct_return;
        let handled=self.map_field_phase_storage_completion_body(fun,&handler.body,declarations,first+1,&out_name,outgoing.clone(),runtime,&storage_type,&places,handler_throws,true,None,handler_error,return_carrier.as_ref(),input_fields.as_deref(),&[]);
        // An empty handler has no statement input epoch. Project its caught
        // owners explicitly into the normal cleanup representation.
        let handled=if handler.body.is_empty(){
            let caught=places.iter().map(|place|{
                let (binding,fields)=self.map_storage_binding(fun,place).expect("caught Map place");
                let key=crate::capture_loan_owners::Place{binding,fields:fields.clone()};
                let target=self.map_storage_leaf_type(fun.bindings[binding].ty,&fields).expect("caught Map type");
                let source=body_error.and_then(|layout|layout.get(&key)).copied().unwrap_or(target);
                let value=self.map_typed_storage_leaf_value(place,None,Some(source),target);
                value.live.unwrap_or(value.place)
            }).collect::<Vec<_>>();
            let values=field_completion_transport::tuple(&caught);
            let values=input_fields.as_ref().map_or_else(||values.clone(),|fields|format!("({values},{fields})"));
            format!("TsrCompletion::<_,{},{out_name}>::Normal({values})",return_carrier.as_ref().map_or_else(||format!("(TsrStorageExit<{}>,{storage_type})",self.native_type(fun.result)),|carrier|carrier.native.clone()))
        }else{handled};
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
        // Each selected path restores its own fields before entering handler or cleanup.
        let body_normal=input_fields.as_ref().map_or_else(||"storage".to_owned(),|fields|format!("(storage,{fields})"));
        let body_normal_select=if fun.storage_exits.has_completion_owners(boundary,crate::analysis::MapCompletionPhase::Body,None){
            format!("let {body_pattern}=body_normal.unwrap();{body_restore}drop(body_error);drop(body_control);(Some({body_normal}),None,None)")
        }else{"unreachable!(\"no normal Map body completion\")".into()};
        let joined=format!("{{let (body_normal,body_error,body_control)={{let {pattern}={values};{body_split}}};let owners:(Option<_>,Option<({out_name},{error_storage})>,Option<{control_type}>)=if body_normal.is_some(){{{body_normal_select}}}else{{drop(body_normal);if body_error.is_some(){{let (caught_error,{caught_pattern})=body_error.unwrap();{caught_restore}drop(body_control);{setup}{handler_split}}}else{{drop(body_error);(None,None,body_control)}}}};owners}}");
        let indent="    ".repeat(depth);
        let _=writeln!(output,"{indent}{in_decl}{out_decl}{cleanup_decl}{carrier_declaration}let {output_pattern}={{let (normal_owner,error_owner,control_owner)={joined};if normal_owner.is_some(){{let {input_pattern}=normal_owner.unwrap();{input_restore}drop(error_owner);drop(control_owner);{normal_finish}}}else{{drop(normal_owner);if error_owner.is_some(){{let (pending_error,{pattern})=error_owner.unwrap();drop(control_owner);{throw_finish}}}else{{drop(error_owner);if control_owner.is_some(){{{selected_control}}}else{{unreachable!(\"exactly one pending cleanup owner\")}}}}}}}};{output_restore}");
        let boundary=places.iter().flat_map(|place|[self.map_storage_transfer_at_exit(fun,&region.body,place),self.map_storage_transfer_at_exit(fun,&handler.body,place),self.map_storage_transfer_at_exit(fun,cleanup,place)]).flatten().map(|span|span.end).max().unwrap_or(statement.span.end).max(statement.span.end);
        self.exception_regions.renew_storage(&places,boundary);
        true
    }
}
