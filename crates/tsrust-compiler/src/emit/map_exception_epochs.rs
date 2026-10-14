//! Carry renewed Map owners separately on normal and caught exception paths.
use super::*;
use statement_work::Work;
use std::rc::Rc;

impl Emitter<'_> {
    #[inline(never)]
    pub(super) fn schedule_map_try_storage<'a,'s:'a>(&'s self, fun: &'a CheckedFunction, statement: &'a Stmt, declarations: Declarations<'a, '_>,work:&mut Vec<Work<'a,'s>>) -> bool {
        if !statement.labels.is_empty() { return false; }
        let StmtKind::Try(region) = &statement.kind else { return false; };
        let Some(handler) = &region.catch else { return self.schedule_map_finally(fun,statement,region,declarations,work); };
        if region.finally.is_some(){return self.schedule_map_catch_finally(fun,statement,region,declarations,work);}
        let statements=region.body.iter().chain(&handler.body).chain(region.finally.iter().flatten()).collect();
        let Some((places,native_places))=self.map_exception_storage_plan(fun,statements,declarations) else{return false;};
        let values = format!("({},)", places.join(","));
        let storage_type = format!("({},)", places.iter().map(|place| native_places[place].as_str()).collect::<Vec<_>>().join(","));
        let pattern = format!("({},)", places.iter().map(|place| format!("mut {place}")).collect::<Vec<_>>().join(","));
        let (_, _, runtime) = self.exception_regions.current_transport().expect("checked enclosing completion");
        let first = self.completion_next.get(); self.completion_next.set(first + 5);
        let incoming = handler.error_contract.as_deref().expect("checked catch contract");
        if incoming.explicit.is_empty() && !incoming.runtime && !incoming.opaque { return false; }
        let mut pending_statement = statement.clone();
        if let StmtKind::Try(pending_region) = &mut pending_statement.kind { pending_region.finally = None; }
        let mut discovery = vec![pending_statement];
        let outgoing = crate::error_effects::annotate_region_body(&mut discovery, &self.program.error_effects);
        let (in_decl, in_name) = self.completion_error_layout(&format!("StorageIncoming{first}"), incoming, runtime, statement.span);
        let (out_decl, out_name) = self.completion_error_layout(&format!("StorageOutgoing{first}"), &outgoing, runtime, statement.span);
        let normal=if region.finally.is_none(){region.body.first().and_then(|statement|fun.storage_exits.map_completion_layout(statement.span.start,crate::analysis::MapCompletionPhase::Handled,None))}else{None};
        let error=region.body.first().and_then(|statement|fun.storage_exits.map_completion_layout(statement.span.start,crate::analysis::MapCompletionPhase::Body,Some(crate::completion::CompletionKind::Throw)));
        let handler_throws = outgoing.runtime || outgoing.opaque || !outgoing.explicit.is_empty();
        let handler_error=region.body.first().and_then(|statement|fun.storage_exits.map_completion_layout(statement.span.start,crate::analysis::MapCompletionPhase::Handled,Some(crate::completion::CompletionKind::Throw)));
        let phases=[(&region.body,first,in_name.clone(),incoming.clone(),true,error),(&handler.body,first+1,out_name.clone(),outgoing.clone(),handler_throws,handler_error)];
        let phase_storage=storage_type.clone();let phase_places=Rc::new(places.clone());let phase_declarations=Rc::new(declarations.iter().collect::<Vec<_>>());
        work.push(Work::Resume(Box::new(move |output,_,results| {
        let handled=results.pop().expect("completed Map handler");
        let pending=results.pop().expect("completed Map body");
        let unions = crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
        let setup = if let Some(binding) = handler.binding {
            let value = exception_contract_emit::checked_catch_value("caught_error", &in_name, incoming, &unions, runtime, statement.span).expect("checked catch payload");
            format!("let {}value{binding}={value};", if fun.bindings[binding].mutable { "mut " } else { "" })
        } else { "drop(caught_error);".into() };
        let result_type=self.native_type(fun.result);
        let (outer_error,_,_)=self.exception_regions.current_transport().expect("checked enclosing handler completion");
        let resumed=format!("match pending_exit{{TsrStorageExit::Return(value)=>TsrCompletion::<(),{result_type},{outer_error}>::Return(value),TsrStorageExit::Break(target)=>TsrCompletion::Break(target),TsrStorageExit::Continue(target)=>TsrCompletion::Continue(target)}}");
        let resume_control=self.map_cleanup_control_dispatch(fun,&resumed,statement.span);
        let returned_finish=format!("{resume_control};unreachable!()");
        let thrown_finish=self.map_layout_storage_abrupt_dispatch(fun,"TsrCompletion::Throw(pending_error)",&out_name,&outgoing,runtime,statement.span,handler_error);
        let throw_arm=if handler_throws{format!("TsrCompletion::Throw((pending_error,{pattern}))=>{{{thrown_finish}}}")}else{"TsrCompletion::Throw(error)=>match error {}".into()};
        let return_arm=format!("TsrCompletion::Return((pending_exit,{pattern}))=>{{{returned_finish}}}");
        let split=format!("{{let completed={pending};match completed{{TsrCompletion::Normal(storage)=>(Some(storage),None,None),TsrCompletion::Throw((error,storage))=>(None,Some((error,storage)),None),TsrCompletion::Return((exit,storage))=>(None,None,Some((exit,storage))),TsrCompletion::Break(_)|TsrCompletion::Continue(_)=>unreachable!(\"carried handler completion\")}}}}");

        let _=writeln!(output,"{in_decl} {out_decl} let {pattern}={{let (normal_owner,error_owner,control_owner)={{let {pattern}={values};{split}}};if normal_owner.is_some(){{let storage=normal_owner.unwrap();drop(error_owner);drop(control_owner);storage}}else{{drop(normal_owner);if error_owner.is_some(){{let (caught_error,{pattern})=error_owner.unwrap();drop(control_owner);{setup}match {handled}{{TsrCompletion::Normal(storage)=>storage,{throw_arm},{return_arm},TsrCompletion::Break(_)|TsrCompletion::Continue(_)=>unreachable!(\"carried handler control\")}}}}else{{drop(error_owner);if control_owner.is_some(){{let (pending_exit,{pattern})=control_owner.unwrap();{returned_finish}}}else{{unreachable!(\"exactly one pending handler owner\")}}}}}}}};");
        let boundary=places.iter().flat_map(|place|[
            self.map_storage_transfer_at_exit(fun,&region.body,place),
            self.map_storage_transfer_at_exit(fun,&handler.body,place),
            region.finally.as_ref().and_then(|body|self.map_storage_transfer_at_exit(fun,body,place)),
        ]).flatten().map(|span|span.end).max().unwrap_or(statement.span.end).max(statement.span.end);
        self.exception_regions.renew_storage(&places,boundary);
        })));
        for (body,id,name,contract,carry_error,error_layout) in phases.into_iter().rev(){
            let storage=phase_storage.clone();let places=phase_places.clone();let declarations=phase_declarations.clone();
            work.push(Work::Captured);
            work.push(Work::Resume(Box::new(move |_,work,_| {
                self.schedule_map_phase_body(fun,body,Declarations::Visible(&declarations),id,&name,contract,runtime,&storage,&places,carry_error,true,normal,error_layout,None,None,&[],work);
            })));
            work.push(Work::Capture);
        }

        true
    }

    #[inline(never)]
    pub(super) fn map_layout_storage_abrupt_dispatch(&self, fun: &CheckedFunction, pending: &str, error_name: &str, contract: &crate::error_effects::FailureSet, runtime: Option<Type>, span: crate::Span, layout:Option<&std::collections::BTreeMap<crate::capture_loan_owners::Place,Type>>) -> String {
        let (outer_name, outer_contract, _) = self.exception_regions.current_transport().expect("checked enclosing completion");
        let unions = crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
        let abrupt = exception_contract_emit::checked_completion_conversion("abrupt", error_name, &outer_name, contract, &outer_contract, &unions, runtime, span).expect("checked outgoing storage completion");
        let error_owner=self.map_outgoing_error_storage(fun,span,layout);
        let dispatch = if layout.is_some() {self.exception_regions.dispatch_error(&abrupt,span,error_owner.as_deref().unwrap_or("()")).expect("outgoing error destination")}else{self.map_cleanup_control_dispatch(fun,&abrupt,span)};
        let result = self.native_type(fun.result);
        let pending = if pending == "TsrCompletion::Throw(pending_error)" {
            format!("TsrCompletion::<(),{result},{error_name}>::Throw(pending_error)")
        } else if pending == "TsrCompletion::Return(pending_return)" {
            format!("TsrCompletion::<(),{result},{error_name}>::Return(pending_return)")
        } else { pending.to_owned() };
        format!("let abrupt:TsrCompletion<(),{result},{error_name}>=match {pending} {{ TsrCompletion::Return(value)=>TsrCompletion::Return(value), TsrCompletion::Throw(error)=>TsrCompletion::Throw(error), TsrCompletion::Break(target)=>TsrCompletion::Break(target), TsrCompletion::Continue(target)=>TsrCompletion::Continue(target), TsrCompletion::Normal(_)=>unreachable!() }}; {dispatch}; unreachable!()")
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn schedule_map_phase_body<'a,'s:'a>(&'s self,fun:&'a CheckedFunction,body:&'a [Stmt],declarations:Declarations<'a,'_>,id:usize,error_name:&str,contract:crate::error_effects::FailureSet,runtime:Option<Type>,storage_type:&str,places:&[String],carry_error:bool,carry_control:bool,normal:Option<&'a std::collections::BTreeMap<crate::capture_loan_owners::Place,Type>>,error_layout:Option<&std::collections::BTreeMap<crate::capture_loan_owners::Place,Type>>,returned:Option<&map_return_storage::Carrier<'_>>,normal_fields:Option<&str>,error_fields:&[(String,Type)],work:&mut Vec<Work<'a,'s>>) {
        // Loops inside this phase own their controls. Forward only exits that
        // actually leave the phase to its enclosing cleanup.
        let locals=exception_contract_emit::local_control_targets(body);
        let scope = self.exception_regions.enter_typed(id, locals, error_name.into(), contract, runtime);
        {
            let points=fun.storage_exits.starts().map(|start| (start,places.iter().map(|place|self.map_storage_value_before(fun,start,place)).collect())).collect();
            let expressions=fun.storage_exits.expression_points().map(|point| (point,places.iter().map(|place|self.map_storage_value_at_expression(fun,point,place)).collect())).collect();
            let boundary=body.first().map_or(0,|statement|statement.span.start);
            let initial=places.iter().map(|place|self.map_storage_value_before(fun,boundary,place)).collect();
            self.exception_regions.attach_storage(initial,points,expressions,carry_error,boundary);
        }
        if error_layout.is_some() || !error_fields.is_empty(){
            let fallback=std::collections::BTreeMap::new();
            self.attach_map_error_storage(fun,body,places,error_layout.unwrap_or(&fallback),error_fields);
        }
        if carry_control {self.exception_regions.carry_storage_on_control_exit();}
        if let Some(carrier)=returned{self.attach_map_return_storage(fun,body,places,carrier);self.attach_map_control_storage(fun,body,places,carrier);}
        if let Some(route)=self.inline_map_return.borrow_mut().take(){self.exception_regions.set_inline_carried_return(route);}
        let result = self.native_type(fun.result);
        let result = if let Some(carrier)=returned{carrier.native.clone()}else if carry_control {format!("(TsrStorageExit<{result}>,{storage_type})")} else {result};
        let error_storage=error_layout.map(|layout|self.map_error_storage_type(fun,places,layout)).unwrap_or_else(||storage_type.to_owned());
        let error_storage=map_error_storage_emit::extend_tuple(&error_storage,&error_fields.iter().map(|(_,ty)|self.native_type(*ty)).collect::<Vec<_>>());
        let error = if carry_error { format!("({error_name},{error_storage})") } else { error_name.to_owned() };

        let places=places.to_vec();let normal_fields=normal_fields.map(str::to_owned);
        work.push(Work::RestoreFunction);
        work.push(Work::Region(scope));
        work.push(Work::Resume(Box::new(move |output,_,results| {
            let statements=results.pop().expect("completed Map storage phase");
        let normal_values = format!("({},)", places.iter().map(|place| self.map_phase_completion_exit_value(fun, body, place, normal)).collect::<Vec<_>>().join(","));
        let normal_values=normal_fields.as_deref().map_or_else(||normal_values.clone(),|fields|format!("({normal_values},{fields})"));
        output.push_str(&format!("{{ let completion:TsrCompletion<_,{result},{error}>='exception_region{id}: {{ {statements} TsrCompletion::Normal({normal_values}) }}; completion }}"));
        })));
        work.push(Work::Captured);
        work.push(Work::Body(body,Rc::new(declarations.iter().collect()),1,None));
        work.push(Work::Capture);
        work.push(Work::Function(fun));
    }

}
