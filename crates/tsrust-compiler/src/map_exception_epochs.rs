//! Carry renewed Map owners separately on normal and caught exception paths.
use super::*;

impl Emitter<'_> {
    #[inline(never)]
    pub(super) fn map_try_storage_epoch(&self, output: &mut String, fun: &CheckedFunction, statement: &Stmt, declarations: &[Stmt], depth: usize) -> bool {
        if !statement.labels.is_empty() { return false; }
        let StmtKind::Try(region) = &statement.kind else { return false; };
        let Some(handler) = &region.catch else { return false; };
        let mut places = Vec::new();
        let mut native_places = std::collections::HashMap::new();
        let mut statements = region.body.iter().chain(&handler.body).chain(region.finally.iter().flatten()).collect::<Vec<_>>();
        while let Some(item) = statements.pop() {
            if !item.labels.is_empty() { return false; }
            match &item.kind {
                StmtKind::Block(body) => statements.extend(body),
                StmtKind::If { then_body, else_body, .. } => {
                    statements.extend(then_body); statements.extend(else_body);
                },
                StmtKind::Let { .. } | StmtKind::Expr(_) | StmtKind::Assign { .. } | StmtKind::Throw(_) => {},
                // Loops and control exits need their own storage snapshots.
                _ => return false,
            }
            if let Some((place, native)) = self.local_map_clear_storage(fun, declarations, item)
                && !places.contains(&place) { native_places.insert(place.clone(), native); places.push(place); }
        }
        if places.is_empty() { return false; }
        places.sort();
        let values = format!("({},)", places.join(","));
        let storage_type = format!("({},)", places.iter().map(|place| native_places[place].as_str()).collect::<Vec<_>>().join(","));
        let pattern = format!("({},)", places.iter().map(|place| format!("mut {place}")).collect::<Vec<_>>().join(","));
        let (_, _, runtime) = self.exception_regions.current_transport().expect("checked enclosing completion");
        let first = self.completion_next.get(); self.completion_next.set(first + 4);
        let incoming = handler.error_contract.as_deref().expect("checked catch contract");
        if incoming.explicit.is_empty() && !incoming.runtime && !incoming.opaque { return false; }
        let mut pending_statement = statement.clone();
        if let StmtKind::Try(pending_region) = &mut pending_statement.kind { pending_region.finally = None; }
        let mut discovery = vec![pending_statement];
        let outgoing = crate::error_effects::annotate_region_body(&mut discovery, &self.program.error_effects);
        let (in_decl, in_name) = self.completion_error_layout(&format!("StorageIncoming{first}"), incoming, runtime, statement.span);
        let (out_decl, out_name) = self.completion_error_layout(&format!("StorageOutgoing{first}"), &outgoing, runtime, statement.span);
        let pending = self.map_storage_completion_body(fun, &region.body, declarations, first, &in_name, incoming.clone(), runtime, &values, &storage_type, &places, true);
        let handler_throws = outgoing.runtime || outgoing.opaque || !outgoing.explicit.is_empty();
        let handled = self.map_storage_completion_body(fun, &handler.body, declarations, first + 1, &out_name, outgoing.clone(), runtime, &values, &storage_type, &places, handler_throws);
        let unions = crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
        let setup = if let Some(binding) = handler.binding {
            let value = exception_contract_emit::checked_catch_value("caught_error", &in_name, incoming, &unions, runtime, statement.span).expect("checked catch payload");
            format!("let {}value{binding}={value};", if fun.bindings[binding].mutable { "mut " } else { "" })
        } else { "drop(caught_error);".into() };
        let mut cleanup_declaration = String::new();
        let mut normal_finish = "storage".to_owned();
        let mut thrown_finish = if region.finally.as_ref().is_some_and(|body| crate::completion::definitely_abrupt(body, 0)) {
            "unreachable!(\"abrupt cleanup cannot resume pending error\")".into()
        } else { self.map_storage_abrupt_dispatch(fun, "TsrCompletion::Throw(pending_error)", &out_name, &outgoing, runtime, statement.span) };
        if let Some(cleanup) = &region.finally {
            let mut cleanup_discovery = cleanup.clone();
            let cleanup_contract = crate::error_effects::annotate_region_body(&mut cleanup_discovery, &self.program.error_effects);
            let (declaration, cleanup_name) = self.completion_error_layout(&format!("StorageCleanup{first}"), &cleanup_contract, runtime, statement.span);
            cleanup_declaration = declaration;
            let normal_cleanup = self.map_storage_completion_body(fun, cleanup, declarations, first + 2, &cleanup_name, cleanup_contract.clone(), runtime, &values, &storage_type, &places, false);
            let thrown_cleanup = self.map_storage_completion_body(fun, cleanup, declarations, first + 3, &cleanup_name, cleanup_contract.clone(), runtime, &values, &storage_type, &places, false);
            let cleanup_abrupt = self.map_storage_abrupt_dispatch(fun, "other", &cleanup_name, &cleanup_contract, runtime, statement.span);
            normal_finish = format!("{{ let {pattern}=storage; match {normal_cleanup} {{ TsrCompletion::Normal(storage)=>storage, other=>{{ {cleanup_abrupt} }} }} }}");
            let resume = if crate::completion::definitely_abrupt(cleanup, 0) { "unreachable!(\"abrupt cleanup cannot resume pending error\")".into() }
                else { self.map_storage_abrupt_dispatch(fun, "TsrCompletion::Throw(pending_error)", &out_name, &outgoing, runtime, statement.span) };
            thrown_finish = format!("{{ match {thrown_cleanup} {{ TsrCompletion::Normal(_)=>{{ {resume} }}, other=>{{ drop(pending_error); {cleanup_abrupt} }} }} }}");
        }
        let throw_arm = if handler_throws {
            format!("TsrCompletion::Throw((pending_error,{pattern}))=>{{ {thrown_finish} }}")
        } else { "TsrCompletion::Throw(error)=>match error {}".into() };
        let indent = "    ".repeat(depth);
        let _ = writeln!(output, "{indent}{in_decl} {out_decl} {cleanup_declaration} let {pattern} = {{ let {pattern} = {values}; match {pending} {{ TsrCompletion::Normal(storage)=>{normal_finish}, TsrCompletion::Throw((caught_error,{pattern}))=>{{ {setup} match {handled} {{ TsrCompletion::Normal(storage)=>{normal_finish}, {throw_arm}, TsrCompletion::Return(_)|TsrCompletion::Break(_)|TsrCompletion::Continue(_)=>unreachable!(\"straight-line handler\") }} }}, TsrCompletion::Return(_)|TsrCompletion::Break(_)|TsrCompletion::Continue(_)=>unreachable!(\"straight-line storage region\") }} }};");
        true
    }

    #[inline(never)]
    fn map_storage_abrupt_dispatch(&self, fun: &CheckedFunction, pending: &str, error_name: &str, contract: &crate::error_effects::FailureSet, runtime: Option<Type>, span: crate::Span) -> String {
        let (outer_name, outer_contract, _) = self.exception_regions.current_transport().expect("checked enclosing completion");
        let unions = crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
        let abrupt = exception_contract_emit::checked_completion_conversion("abrupt", error_name, &outer_name, contract, &outer_contract, &unions, runtime, span).expect("checked outgoing storage completion");
        let dispatch = self.exception_regions.dispatch(&abrupt, &self.control_targets.snapshot(), span).expect("checked enclosing destination");
        let result = self.native_type(fun.result);
        let pending = if pending == "TsrCompletion::Throw(pending_error)" {
            format!("TsrCompletion::<(),{result},{error_name}>::Throw(pending_error)")
        } else { pending.to_owned() };
        format!("let abrupt:TsrCompletion<(),{result},{error_name}>=match {pending} {{ TsrCompletion::Return(value)=>TsrCompletion::Return(value), TsrCompletion::Throw(error)=>TsrCompletion::Throw(error), TsrCompletion::Break(target)=>TsrCompletion::Break(target), TsrCompletion::Continue(target)=>TsrCompletion::Continue(target), TsrCompletion::Normal(_)=>unreachable!() }}; {dispatch}; unreachable!()")
    }

    #[inline(never)]
    fn map_storage_completion_body(&self, fun: &CheckedFunction, body: &[Stmt], declarations: &[Stmt], id: usize, error_name: &str, contract: crate::error_effects::FailureSet, runtime: Option<Type>, values: &str, storage_type: &str, places: &[String], carry_error: bool) -> String {
        let _region = self.exception_regions.enter_typed(id, Vec::new(), error_name.into(), contract, runtime);
        {
            let points=fun.storage_exits.starts().map(|start| {
                let payload=format!("({},)",places.iter().map(|place| if self.map_storage_live_before(fun,start,place) {place.clone()} else {self.map_empty_storage_value(fun,place)}).collect::<Vec<_>>().join(","));
                (start,payload)
            }).collect();
            let expressions=fun.storage_exits.expression_points().map(|point| {
                let payload=format!("({},)",places.iter().map(|place| if self.map_storage_live_at_expression(fun,point,place) {place.clone()} else {self.map_empty_storage_value(fun,place)}).collect::<Vec<_>>().join(","));
                (point,payload)
            }).collect();
            self.exception_regions.attach_storage(values.into(), points, expressions, carry_error);
        }
        let mut statements = String::new();
        self.control_block_epochs(&mut statements, fun, body, 1, None, Some(declarations));
        let normal_values = format!("({},)", places.iter().map(|place| self.map_branch_exit_value(fun, body, place)).collect::<Vec<_>>().join(","));
        let result = self.native_type(fun.result);
        let error = if carry_error { format!("({error_name},{storage_type})") } else { error_name.to_owned() };
        format!("{{ let completion:TsrCompletion<_,{result},{error}>='exception_region{id}: {{ {statements} TsrCompletion::Normal({normal_values}) }}; completion }}")
    }
}
