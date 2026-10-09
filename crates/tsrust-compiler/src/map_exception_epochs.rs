//! Carry renewed Map owners separately on normal and caught exception paths.
use super::*;

impl Emitter<'_> {
    #[inline(never)]
    pub(super) fn map_try_storage_epoch(&self, output: &mut String, fun: &CheckedFunction, statement: &Stmt, declarations: &[Stmt], depth: usize) -> bool {
        if !statement.labels.is_empty() { return false; }
        let StmtKind::Try(region) = &statement.kind else { return false; };
        let Some(handler) = &region.catch else { return self.map_finally_storage_epoch(output,fun,statement,region,declarations,depth); };
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
        let pending = self.map_error_phase_storage_completion_body(fun, &region.body, declarations, first, &in_name, incoming.clone(), runtime, &storage_type, &places, true, true, normal, error);
        let handler_throws = outgoing.runtime || outgoing.opaque || !outgoing.explicit.is_empty();
        let handler_error=if region.finally.is_none(){region.body.first().and_then(|statement|fun.storage_exits.map_completion_layout(statement.span.start,crate::analysis::MapCompletionPhase::Handled,Some(crate::completion::CompletionKind::Throw)))}else{None};
        let handled = self.map_error_phase_storage_completion_body(fun, &handler.body, declarations, first + 1, &out_name, outgoing.clone(), runtime, &storage_type, &places, handler_throws, true, normal, handler_error);
        let unions = crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
        let setup = if let Some(binding) = handler.binding {
            let value = exception_contract_emit::checked_catch_value("caught_error", &in_name, incoming, &unions, runtime, statement.span).expect("checked catch payload");
            format!("let {}value{binding}={value};", if fun.bindings[binding].mutable { "mut " } else { "" })
        } else { "drop(caught_error);".into() };
        let result_type=self.native_type(fun.result);
        let (outer_error,_,_)=self.exception_regions.current_transport().expect("checked enclosing handler completion");
        let resumed=format!("match pending_exit{{TsrStorageExit::Return(value)=>TsrCompletion::<(),{result_type},{outer_error}>::Return(value),TsrStorageExit::Break(target)=>TsrCompletion::Break(target),TsrStorageExit::Continue(target)=>TsrCompletion::Continue(target)}}");
        let resume_control=self.map_cleanup_control_dispatch(fun,&resumed,statement.span);
        let mut returned_finish=format!("{resume_control};unreachable!()");
        let mut cleanup_declaration = String::new();
        let mut normal_finish = values.clone();
        let mut thrown_finish = if region.finally.as_ref().is_some_and(|body| crate::completion::definitely_abrupt(body, 0)) {
            "unreachable!(\"abrupt cleanup cannot resume pending error\")".into()
        } else { self.map_layout_storage_abrupt_dispatch(fun, "TsrCompletion::Throw(pending_error)", &out_name, &outgoing, runtime, statement.span, handler_error) };
        if let Some(cleanup) = &region.finally {
            let mut cleanup_discovery = cleanup.clone();
            let cleanup_contract = crate::error_effects::annotate_region_body(&mut cleanup_discovery, &self.program.error_effects);
            let (declaration, cleanup_name) = self.completion_error_layout(&format!("StorageCleanup{first}"), &cleanup_contract, runtime, statement.span);
            cleanup_declaration = declaration;
            let cleaned=region.body.first().and_then(|statement|fun.storage_exits.map_completion_layout(statement.span.start,crate::analysis::MapCompletionPhase::Cleaned,None));
            let normal_cleanup = self.map_phase_storage_completion_body(fun, cleanup, declarations, first + 2, &cleanup_name, cleanup_contract.clone(), runtime, &storage_type, &places, true, true, cleaned);
            let thrown_cleanup = self.map_storage_completion_body(fun, cleanup, declarations, first + 3, &cleanup_name, cleanup_contract.clone(), runtime, &storage_type, &places, true, true);
            let returned_cleanup = self.map_storage_completion_body(fun, cleanup, declarations, first + 4, &cleanup_name, cleanup_contract.clone(), runtime, &storage_type, &places, true, true);
            let cleanup_abrupt = self.map_carried_cleanup_dispatch(fun, "other", &cleanup_name, &cleanup_contract, runtime, &places, statement.span);
            let resume = if crate::completion::definitely_abrupt(cleanup,0) { "unreachable!(\"abrupt cleanup cannot resume pending control\")".to_owned() }else{format!("{resume_control};unreachable!()")};
            returned_finish=format!("match {returned_cleanup}{{TsrCompletion::Normal(storage)=>{{let {pattern}=storage;{resume}}},other=>{{drop(pending_exit);{cleanup_abrupt}}}}}");
            normal_finish = format!("{{ match {normal_cleanup} {{ TsrCompletion::Normal(storage)=>storage, other=>{{ {cleanup_abrupt} }} }} }}");
            let resume = if crate::completion::definitely_abrupt(cleanup, 0) { "unreachable!(\"abrupt cleanup cannot resume pending error\")".into() }
                else { self.map_storage_abrupt_dispatch(fun, "TsrCompletion::Throw(pending_error)", &out_name, &outgoing, runtime, statement.span) };
            thrown_finish = format!("{{ match {thrown_cleanup} {{ TsrCompletion::Normal(storage)=>{{ let {pattern}=storage; {resume} }}, other=>{{ drop(pending_error); {cleanup_abrupt} }} }} }}");
        }
        let cleanup_releases_all=region.finally.as_ref().is_some_and(|cleanup| places.iter().all(|place|cleanup.iter().any(|item|self.local_map_clear_storage(fun,declarations,item).is_some_and(|(cleared,_)| &cleared==place))));
        if cleanup_releases_all {
            // Finish the pending and handler owner scopes before entering cleanup.
            // The joined control carries no callback storage; storage leaves separately.
            let joined_throw: String = if handler_throws {
                "TsrCompletion::Throw((error,storage))=>(TsrCompletion::Throw(error),storage)".into()
            } else { "TsrCompletion::Throw(error)=>match error {}".into() };
            let joined_return="TsrCompletion::Return((exit,storage))=>(TsrCompletion::Return(exit),storage)";
            let joined_normal="TsrCompletion::Normal(storage)=>(TsrCompletion::Normal(()),storage)";
            let impossible="TsrCompletion::Break(_)|TsrCompletion::Continue(_)=>unreachable!(\"carried storage control\")";
            let joined=format!("{{let {pattern}={values};let outcome:(TsrCompletion<(),TsrStorageExit<{result_type}>,{out_name}>,{storage_type})=match {pending}{{{joined_normal},TsrCompletion::Throw((caught_error,{pattern}))=>{{{setup} match {handled}{{{joined_normal},{joined_throw},{joined_return},{impossible}}}}},{joined_return},{impossible}}};outcome}}");
            let indent = "    ".repeat(depth);
            let _ = writeln!(output, "{indent}{in_decl} {out_decl} {cleanup_declaration} let {pattern} = {{ let (pending_control,{pattern})={joined}; match pending_control {{ TsrCompletion::Normal(())=>{normal_finish}, TsrCompletion::Throw(pending_error)=>{{{thrown_finish}}}, TsrCompletion::Return(pending_exit)=>{{{returned_finish}}},TsrCompletion::Break(_)|TsrCompletion::Continue(_)=>unreachable!(\"joined storage control\") }} }};");
        } else {
            normal_finish=if region.finally.is_some() {format!("{{let {pattern}=storage;{normal_finish}}}")}else{"storage".into()};
            let throw_arm = if handler_throws {
                format!("TsrCompletion::Throw((pending_error,{pattern}))=>{{ {thrown_finish} }}")
            } else { "TsrCompletion::Throw(error)=>match error {}".into() };
            let return_arm=format!("TsrCompletion::Return((pending_exit,{pattern}))=>{{{returned_finish}}}");
            let indent = "    ".repeat(depth);
            if region.finally.is_none() {
                let split=format!("{{let completed={pending};match completed{{TsrCompletion::Normal(storage)=>(Some(storage),None,None),TsrCompletion::Throw((error,storage))=>(None,Some((error,storage)),None),TsrCompletion::Return((exit,storage))=>(None,None,Some((exit,storage))),TsrCompletion::Break(_)|TsrCompletion::Continue(_)=>unreachable!(\"carried handler completion\")}}}}");
                let _=writeln!(output,"{indent}{in_decl} {out_decl} let {pattern}={{let (normal_owner,error_owner,control_owner)={{let {pattern}={values};{split}}};if normal_owner.is_some(){{let storage=normal_owner.unwrap();drop(error_owner);drop(control_owner);{normal_finish}}}else{{drop(normal_owner);if error_owner.is_some(){{let (caught_error,{pattern})=error_owner.unwrap();drop(control_owner);{setup}match {handled}{{TsrCompletion::Normal(storage)=>{normal_finish},{throw_arm},{return_arm},TsrCompletion::Break(_)|TsrCompletion::Continue(_)=>unreachable!(\"carried handler control\")}}}}else{{drop(error_owner);if control_owner.is_some(){{let (pending_exit,{pattern})=control_owner.unwrap();{returned_finish}}}else{{unreachable!(\"exactly one pending handler owner\")}}}}}}}};");
            }else if normal.is_none() && (self.map_phase_has_entry_narrowing(fun,&handler.body) || region.finally.as_ref().is_some_and(|body|self.map_phase_has_entry_narrowing(fun,body))) {
            let split=self.map_split_pending_storage(fun,&pending,&in_name,&storage_type);
            let outer_return=format!("TsrCompletion::Return(pending_exit)=>{{{returned_finish}}}");
            let _ = writeln!(output, "{indent}{in_decl} {out_decl} {cleanup_declaration} let {pattern} = {{ let (pending_control,{pattern})={{let {pattern}={values};{split}}};match pending_control {{ TsrCompletion::Normal(())=>{{let storage={values};{normal_finish}}}, TsrCompletion::Throw(caught_error)=>{{ {setup} match {handled} {{ TsrCompletion::Normal(storage)=>{normal_finish}, {throw_arm}, {return_arm},TsrCompletion::Break(_)|TsrCompletion::Continue(_)=>unreachable!(\"carried handler control\") }} }}, {outer_return},TsrCompletion::Break(_)|TsrCompletion::Continue(_)=>unreachable!(\"carried storage control\") }} }};");
            }else {
                let _ = writeln!(output, "{indent}{in_decl} {out_decl} {cleanup_declaration} let {pattern} = {{ let {pattern} = {values}; match {pending} {{ TsrCompletion::Normal(storage)=>{normal_finish}, TsrCompletion::Throw((caught_error,{pattern}))=>{{ {setup} match {handled} {{ TsrCompletion::Normal(storage)=>{normal_finish}, {throw_arm}, {return_arm},TsrCompletion::Break(_)|TsrCompletion::Continue(_)=>unreachable!(\"carried handler control\") }} }}, {return_arm},TsrCompletion::Break(_)|TsrCompletion::Continue(_)=>unreachable!(\"carried storage control\") }} }};");
            }
        }
        let boundary=places.iter().flat_map(|place|[
            self.map_storage_transfer_at_exit(fun,&region.body,place),
            self.map_storage_transfer_at_exit(fun,&handler.body,place),
            region.finally.as_ref().and_then(|body|self.map_storage_transfer_at_exit(fun,body,place)),
        ]).flatten().map(|span|span.end).max().unwrap_or(statement.span.end).max(statement.span.end);
        self.exception_regions.renew_storage(&places,boundary);
        true
    }

    pub(super) fn map_phase_has_entry_narrowing(&self,fun:&CheckedFunction,body:&[Stmt])->bool {
        let mut pending=body.iter().collect::<Vec<_>>();
        while let Some(statement)=pending.pop() {
            if fun.storage_exits.map_entry_epochs(statement.span).iter().any(|epoch|!epoch.before && !epoch.empty){return true;}
            match &statement.kind {
                StmtKind::Block(body)|StmtKind::While{body,..}|StmtKind::Loop{body,..}|StmtKind::ForOf{body,..}=>pending.extend(body),
                StmtKind::If{then_body,else_body,..}=>{pending.extend(then_body);pending.extend(else_body);},
                StmtKind::Try(region)=>{pending.extend(&region.body);if let Some(handler)=&region.catch{pending.extend(&handler.body);}pending.extend(region.finally.iter().flatten());},
                _=>{},
            }
        }
        false
    }

    /// Finish the owner-bearing completion scope before executing its next phase.
    pub(super) fn map_split_pending_storage(&self,fun:&CheckedFunction,pending:&str,error:&str,storage:&str)->String {
        let result=self.native_type(fun.result);
        format!("{{let pending_completion={pending};let separated:(TsrCompletion<(),TsrStorageExit<{result}>,{error}>,{storage})=match pending_completion{{TsrCompletion::Normal(storage)=>(TsrCompletion::Normal(()),storage),TsrCompletion::Throw((error,storage))=>(TsrCompletion::Throw(error),storage),TsrCompletion::Return((exit,storage))=>(TsrCompletion::Return(exit),storage),TsrCompletion::Break(_)|TsrCompletion::Continue(_)=>unreachable!(\"carried phase completion\")}};separated}}")
    }

    #[inline(never)]
    pub(super) fn map_storage_abrupt_dispatch(&self, fun: &CheckedFunction, pending: &str, error_name: &str, contract: &crate::error_effects::FailureSet, runtime: Option<Type>, span: crate::Span) -> String {
        self.map_layout_storage_abrupt_dispatch(fun,pending,error_name,contract,runtime,span,None)
    }
    pub(super) fn map_layout_storage_abrupt_dispatch(&self, fun: &CheckedFunction, pending: &str, error_name: &str, contract: &crate::error_effects::FailureSet, runtime: Option<Type>, span: crate::Span, layout:Option<&std::collections::BTreeMap<crate::capture_loan_owners::Place,Type>>) -> String {
        let (outer_name, outer_contract, _) = self.exception_regions.current_transport().expect("checked enclosing completion");
        let unions = crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
        let abrupt = exception_contract_emit::checked_completion_conversion("abrupt", error_name, &outer_name, contract, &outer_contract, &unions, runtime, span).expect("checked outgoing storage completion");
        let error_owner=self.map_outgoing_error_storage(fun,span,layout);
        let dispatch = if let Some(owner)=error_owner {self.exception_regions.dispatch_error(&abrupt,span,&owner).expect("outgoing error destination")}else{self.map_cleanup_control_dispatch(fun,&abrupt,span)};
        let result = self.native_type(fun.result);
        let pending = if pending == "TsrCompletion::Throw(pending_error)" {
            format!("TsrCompletion::<(),{result},{error_name}>::Throw(pending_error)")
        } else if pending == "TsrCompletion::Return(pending_return)" {
            format!("TsrCompletion::<(),{result},{error_name}>::Return(pending_return)")
        } else { pending.to_owned() };
        format!("let abrupt:TsrCompletion<(),{result},{error_name}>=match {pending} {{ TsrCompletion::Return(value)=>TsrCompletion::Return(value), TsrCompletion::Throw(error)=>TsrCompletion::Throw(error), TsrCompletion::Break(target)=>TsrCompletion::Break(target), TsrCompletion::Continue(target)=>TsrCompletion::Continue(target), TsrCompletion::Normal(_)=>unreachable!() }}; {dispatch}; unreachable!()")
    }

    #[inline(never)]
    pub(super) fn map_storage_completion_body(&self, fun: &CheckedFunction, body: &[Stmt], declarations: &[Stmt], id: usize, error_name: &str, contract: crate::error_effects::FailureSet, runtime: Option<Type>, storage_type: &str, places: &[String], carry_error: bool, carry_control: bool) -> String {
        self.map_phase_storage_completion_body(fun,body,declarations,id,error_name,contract,runtime,storage_type,places,carry_error,carry_control,None)
    }
    #[inline(never)]
    pub(super) fn map_phase_storage_completion_body(&self,fun:&CheckedFunction,body:&[Stmt],declarations:&[Stmt],id:usize,error_name:&str,contract:crate::error_effects::FailureSet,runtime:Option<Type>,storage_type:&str,places:&[String],carry_error:bool,carry_control:bool,normal:Option<&std::collections::BTreeMap<crate::capture_loan_owners::Place,Type>>)->String {
        self.map_error_phase_storage_completion_body(fun,body,declarations,id,error_name,contract,runtime,storage_type,places,carry_error,carry_control,normal,None)
    }
    #[inline(never)]
    pub(super) fn map_error_phase_storage_completion_body(&self,fun:&CheckedFunction,body:&[Stmt],declarations:&[Stmt],id:usize,error_name:&str,contract:crate::error_effects::FailureSet,runtime:Option<Type>,storage_type:&str,places:&[String],carry_error:bool,carry_control:bool,normal:Option<&std::collections::BTreeMap<crate::capture_loan_owners::Place,Type>>,error_layout:Option<&std::collections::BTreeMap<crate::capture_loan_owners::Place,Type>>)->String {
        // Loops inside this phase own their controls. Forward only exits that
        // actually leave the phase to its enclosing cleanup.
        let locals=exception_contract_emit::local_control_targets(body);
        let _region = self.exception_regions.enter_typed(id, locals, error_name.into(), contract, runtime);
        {
            let points=fun.storage_exits.starts().map(|start| (start,places.iter().map(|place|self.map_storage_value_before(fun,start,place)).collect())).collect();
            let expressions=fun.storage_exits.expression_points().map(|point| (point,places.iter().map(|place|self.map_storage_value_at_expression(fun,point,place)).collect())).collect();
            let boundary=body.first().map_or(0,|statement|statement.span.start);
            let initial=places.iter().map(|place|self.map_storage_value_before(fun,boundary,place)).collect();
            self.exception_regions.attach_storage(initial,points,expressions,carry_error,boundary);
        }
        if let Some(layout)=error_layout {self.attach_map_error_storage(fun,body,places,layout);}
        if carry_control {self.exception_regions.carry_storage_on_control_exit();}
        let mut statements = String::new();
        self.control_block_epochs(&mut statements, fun, body, 1, None, Some(declarations));
        let normal_values = format!("({},)", places.iter().map(|place| self.map_phase_completion_exit_value(fun, body, place, normal)).collect::<Vec<_>>().join(","));
        let result = self.native_type(fun.result);
        let result = if carry_control {format!("(TsrStorageExit<{result}>,{storage_type})")} else {result};
        let error_storage=error_layout.map(|layout|self.map_error_storage_type(fun,places,layout)).unwrap_or_else(||storage_type.to_owned());
        let error = if carry_error { format!("({error_name},{error_storage})") } else { error_name.to_owned() };
        format!("{{ let completion:TsrCompletion<_,{result},{error}>='exception_region{id}: {{ {statements} TsrCompletion::Normal({normal_values}) }}; completion }}")
    }
}
