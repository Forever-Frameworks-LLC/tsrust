//! Preserve renewed Map owners through finally when no catch handles the error.
use super::*;

impl Emitter<'_> {
    #[inline(never)]
    pub(super) fn map_finally_storage_epoch(
        &self,
        output: &mut String,
        fun: &CheckedFunction,
        statement: &Stmt,
        region: &ExceptionRegion,
        declarations: &[Stmt],
        depth: usize,
    ) -> bool {
        let Some(cleanup) = &region.finally else {
            return false;
        };
        let statements=region.body.iter().chain(cleanup).collect();
        let Some((places,native_places))=self.map_exception_storage_plan(fun,statements,declarations) else{return false;};
        let values = format!("({},)", places.join(","));
        let pattern = format!(
            "({},)",
            places
                .iter()
                .map(|place| format!("mut {place}"))
                .collect::<Vec<_>>()
                .join(",")
        );
        let storage_type = format!(
            "({},)",
            places
                .iter()
                .map(|place| native_places[place].as_str())
                .collect::<Vec<_>>()
                .join(",")
        );
        let (_, _, runtime) = self
            .exception_regions
            .current_transport()
            .expect("checked enclosing completion");
        let first = self.completion_next.get();
        self.completion_next.set(first + 4);
        let mut body = region.body.clone();
        let incoming =
            crate::error_effects::annotate_region_body(&mut body, &self.program.error_effects);
        let mut cleanup_body = cleanup.clone();
        let cleanup_contract = crate::error_effects::annotate_region_body(
            &mut cleanup_body,
            &self.program.error_effects,
        );
        let (incoming_decl, incoming_name) = self.completion_error_layout(
            &format!("FinallyIncoming{first}"),
            &incoming,
            runtime,
            statement.span,
        );
        let (cleanup_decl, cleanup_name) = self.completion_error_layout(
            &format!("FinallyCleanup{first}"),
            &cleanup_contract,
            runtime,
            statement.span,
        );
        let error=region.body.first().and_then(|statement|fun.storage_exits.map_completion_layout(statement.span.start,crate::analysis::MapCompletionPhase::Body,Some(crate::completion::CompletionKind::Throw)));
        let fallback=std::collections::BTreeMap::new();
        let boundary=region.body.first().map_or(statement.span.start,|statement|statement.span.start);
        let return_carrier=self.map_completion_carrier(fun,format!("MapControl{first}"),&places,&storage_type,boundary,crate::analysis::MapCompletionPhase::Body,&fallback);
        let cleaned=fun.storage_exits.map_completion_layout(boundary,crate::analysis::MapCompletionPhase::Cleaned,None);
        let cleaned_error=fun.storage_exits.map_completion_layout(boundary,crate::analysis::MapCompletionPhase::Cleaned,Some(crate::completion::CompletionKind::Throw));
        let field_plan=self.completion_field_plan(fun,statement,region);
        let input_fields=field_plan.as_ref().map(|plan|field_completion_transport::tuple(&plan.input_fields));
        let output_fields=field_plan.as_ref().map(|plan|field_completion_transport::tuple(&plan.output_fields));
        let (input_pattern,input_restore)=field_plan.as_ref().map_or_else(||(pattern.clone(),String::new()),|plan|{
            let (fields,restore)=field_completion_transport::restoration(&plan.input_fields,&format!("map_input_field{first}"));(format!("({pattern},{fields})"),restore)
        });
        let (output_pattern,output_restore)=field_plan.as_ref().map_or_else(||(pattern.clone(),String::new()),|plan|{
            let (fields,restore)=field_completion_transport::restoration(&plan.output_fields,&format!("map_output_field{first}"));(format!("({pattern},{fields})"),restore)
        });
        let normal_finish=self.map_finish_field_cleanup(fun,fun,region,cleanup,declarations,first+1,&cleanup_name,&cleanup_contract,runtime,&places,&storage_type,&pattern,statement.span,cleaned,"storage","",output_fields.as_deref());
        let error_plan=fun.storage_exits.map_cleanup_plan(boundary,Some(crate::completion::CompletionKind::Throw)).map(|plan|plan.function(fun));
        let error_fun=error_plan.as_ref().unwrap_or(fun);let error_body=error_plan.as_ref().map_or(cleanup.as_slice(),|plan|plan.body.as_slice());
        let throw_finish=if incoming.runtime || incoming.opaque || !incoming.explicit.is_empty(){
            let resume=if crate::completion::definitely_abrupt(cleanup,0){"unreachable!(\"abrupt cleanup cannot resume pending error\")".into()}else{self.map_layout_storage_abrupt_dispatch(fun,"TsrCompletion::Throw(pending_error)",&incoming_name,&incoming,runtime,statement.span,cleaned_error)};
            let normal=format!("let {pattern}=storage;{resume}");
            self.map_finish_cleanup(fun,error_fun,region,error_body,declarations,first+2,&cleanup_name,&cleanup_contract,runtime,&places,&storage_type,&pattern,statement.span,cleaned_error,&normal,"drop(pending_error);")
        }else{"match pending_error {}".into()};
        let result = self.native_type(fun.result);
        let (outer_error, _, _) = self
            .exception_regions
            .current_transport()
            .expect("checked enclosing completion");
        let resumed = format!(
            "match pending_exit {{TsrStorageExit::Return(value)=>TsrCompletion::<(),{result},{outer_error}>::Return(value),TsrStorageExit::Break(target)=>TsrCompletion::Break(target),TsrStorageExit::Continue(target)=>TsrCompletion::Continue(target)}}"
        );
        let resume_control = self.map_cleanup_control_dispatch(fun,&resumed,statement.span);
        let resume_control = if crate::completion::definitely_abrupt(cleanup, 0) {
            "unreachable!(\"abrupt cleanup cannot resume pending control\")".into()
        } else {
            format!("{resume_control};unreachable!()")
        };
        let normal=format!("let {pattern}=storage;{resume_control}");
        let return_finish=self.map_finish_cleanup(fun,fun,region,cleanup,declarations,first+3,&cleanup_name,&cleanup_contract,runtime,&places,&storage_type,&pattern,statement.span,None,&normal,"drop(pending_exit);");
        let indent = "    ".repeat(depth);
        // Finish the completion enum's scope before cleanup borrows captures.
        // Each optional owner keeps its own native type; exactly one is live.
        let carrier_declaration=return_carrier.as_ref().map_or(String::new(),|carrier|self.map_carrier_declaration(carrier));
        let carried=if return_carrier.is_some(){"TsrCompletion::Return(control)=>(None,None,Some(control))"}else{"TsrCompletion::Return((exit,storage))=>(None,None,Some((exit,storage)))"};
        let selected_control=if let Some(carrier)=&return_carrier {
            let control_cleanups=self.map_precise_control_cleanups(fun,region,declarations,carrier,&cleanup_name,&cleanup_contract,runtime,&places,&storage_type,&pattern,statement.span);
            let precise=self.map_precise_return_cleanup(fun,region,declarations,first+3,carrier,&cleanup_name,&cleanup_contract,runtime,&places,&storage_type,&pattern,statement.span);
            let has_other=region.body.first().is_some_and(|statement|self.control_targets.snapshot().iter().any(|target|[crate::completion::CompletionKind::Break(Some(*target)),crate::completion::CompletionKind::Continue(Some(*target))].into_iter().any(|kind|fun.storage_exits.map_completion_layout(statement.span.start,crate::analysis::MapCompletionPhase::Body,Some(kind)).is_some())));
            let common=if has_other{format!("{{{return_finish}}}")}else{"unreachable!(\"no pending loop control\")".into()};
            self.map_select_control_owner(carrier,&pattern,precise,common,control_cleanups)
        }else{format!("let (pending_exit,{pattern})=control_owner.unwrap();{return_finish}")};
        if field_plan.is_none() && exception_contract_emit::has_partial_return(fun,&region.body) && let Some(carrier)=&return_carrier {
            let precise=self.map_precise_return_cleanup(fun,region,declarations,first+3,carrier,&cleanup_name,&cleanup_contract,runtime,&places,&storage_type,&pattern,statement.span);
            *self.inline_map_return.borrow_mut()=Some(self.map_direct_return_route(carrier,&pattern,precise,first));
        }
        let pending = self.map_field_phase_storage_completion_body(
            fun,
            &region.body,
            declarations,
            first,
            &incoming_name,
            incoming.clone(),
            runtime,
            &storage_type,
            &places,
            true,
            true,
            None,
            error,
            return_carrier.as_ref(),
            input_fields.as_deref(),
        );
        let split=format!("{{let completed={pending};match completed{{TsrCompletion::Normal(storage)=>(Some(storage),None,None),TsrCompletion::Throw((error,storage))=>(None,Some((error,storage)),None),{carried},TsrCompletion::Break(_)|TsrCompletion::Continue(_)=>unreachable!(\"carried finally completion\")}}}}");
        let _=writeln!(output,"{indent}{incoming_decl} {cleanup_decl} {carrier_declaration} let {output_pattern}={{let (normal_owner,error_owner,control_owner)={{let {pattern}={values};{split}}};if normal_owner.is_some(){{let {input_pattern}=normal_owner.unwrap();{input_restore}drop(error_owner);drop(control_owner);{normal_finish}}}else{{drop(normal_owner);if error_owner.is_some(){{let (pending_error,{pattern})=error_owner.unwrap();drop(control_owner);{throw_finish}}}else{{drop(error_owner);if control_owner.is_some(){{{selected_control}}}else{{unreachable!(\"exactly one pending finally owner\")}}}}}}}};{output_restore}");
        let boundary = places
            .iter()
            .flat_map(|place| {
                [
                    self.map_storage_transfer_at_exit(fun, &region.body, place),
                    self.map_storage_transfer_at_exit(fun, cleanup, place),
                ]
            })
            .flatten()
            .map(|span| span.end)
            .max()
            .unwrap_or(statement.span.end)
            .max(statement.span.end);
        self.exception_regions.renew_storage(&places, boundary);
        true
    }
    #[inline(never)]
    pub(super) fn map_layout_carried_cleanup_dispatch(&self,fun:&CheckedFunction,pending:&str,error_name:&str,contract:&crate::error_effects::FailureSet,runtime:Option<Type>,places:&[String],span:crate::Span,layout:Option<&std::collections::BTreeMap<crate::capture_loan_owners::Place,Type>>)->String {
        let pattern = format!(
            "({},)",
            places
                .iter()
                .map(|place| format!("mut {place}"))
                .collect::<Vec<_>>()
                .join(",")
        );
        let error = self.map_layout_storage_abrupt_dispatch(
            fun,
            "TsrCompletion::Throw(pending_error)",
            error_name,
            contract,
            runtime,
            span,
            layout,
        );
        let result = self.native_type(fun.result);
        let (outer_error, _, _) = self
            .exception_regions
            .current_transport()
            .expect("checked cleanup destination");
        let resumed = format!(
            "match pending_exit{{TsrStorageExit::Return(value)=>TsrCompletion::<(),{result},{outer_error}>::Return(value),TsrStorageExit::Break(target)=>TsrCompletion::Break(target),TsrStorageExit::Continue(target)=>TsrCompletion::Continue(target)}}"
        );
        let control = self.map_cleanup_control_dispatch(fun,&resumed,span);
        format!(
            "match {pending}{{TsrCompletion::Throw((pending_error,{pattern}))=>{{{error}}},TsrCompletion::Return((pending_exit,{pattern}))=>{{{control};unreachable!()}},_=>unreachable!(\"checked carried cleanup exit\")}}"
        )
    }
}
