//! Carry renewed Map owners separately on normal and caught exception paths.
use super::*;

impl Emitter<'_> {
    #[inline(never)]
    pub(super) fn map_try_storage_epoch(&self, output: &mut String, fun: &CheckedFunction, statement: &Stmt, declarations: &[Stmt], depth: usize) -> bool {
        if !statement.labels.is_empty() { return false; }
        let StmtKind::Try(region) = &statement.kind else { return false; };
        let Some(handler) = &region.catch else { return false; };
        if region.finally.is_some() { return false; }
        // Structured exits require their own point-sensitive storage payloads.
        // Start with straight-line regions; fallible expressions remain allowed.
        let simple = |body: &[Stmt]| body.iter().all(|item| item.labels.is_empty() && matches!(item.kind, StmtKind::Let { .. } | StmtKind::Expr(_) | StmtKind::Assign { .. } | StmtKind::Throw(_)));
        if !simple(&region.body) || !simple(&handler.body) { return false; }
        let mut places = Vec::new();
        for item in region.body.iter().chain(&handler.body) {
            if let Some((place, _)) = self.local_map_clear_storage(fun, declarations, item)
                && !places.contains(&place) { places.push(place); }
        }
        if places.is_empty() { return false; }
        // A consumed owner needs an exit-specific tombstone, handled separately.
        if places.iter().any(|place| !self.map_storage_live_at_exit(fun, &region.body, place) || !self.map_storage_live_at_exit(fun, &handler.body, place)) { return false; }
        places.sort();
        let values = format!("({},)", places.join(","));
        let pattern = format!("({},)", places.iter().map(|place| format!("mut {place}")).collect::<Vec<_>>().join(","));
        let (_, _, runtime) = self.exception_regions.current_transport().expect("checked enclosing completion");
        let first = self.completion_next.get(); self.completion_next.set(first + 2);
        let incoming = handler.error_contract.as_deref().expect("checked catch contract");
        if incoming.explicit.is_empty() && !incoming.runtime && !incoming.opaque { return false; }
        let mut discovery = vec![statement.clone()];
        let outgoing = crate::error_effects::annotate_region_body(&mut discovery, &self.program.error_effects);
        let (in_decl, in_name) = self.completion_error_layout(&format!("StorageIncoming{first}"), incoming, runtime, statement.span);
        let (out_decl, out_name) = self.completion_error_layout(&format!("StorageOutgoing{first}"), &outgoing, runtime, statement.span);
        let pending = self.map_storage_completion_body(fun, &region.body, declarations, first, &in_name, incoming.clone(), runtime, &values, true);
        let handled = self.map_storage_completion_body(fun, &handler.body, declarations, first + 1, &out_name, outgoing.clone(), runtime, &values, false);
        let unions = crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
        let setup = if let Some(binding) = handler.binding {
            let value = exception_contract_emit::checked_catch_value("caught_error", &in_name, incoming, &unions, runtime, statement.span).expect("checked catch payload");
            format!("let {}value{binding}={value};", if fun.bindings[binding].mutable { "mut " } else { "" })
        } else { "drop(caught_error);".into() };
        let (outer_name, outer_contract, _) = self.exception_regions.current_transport().unwrap();
        let abrupt = exception_contract_emit::checked_completion_conversion("abrupt", &out_name, &outer_name, &outgoing, &outer_contract, &unions, runtime, statement.span).expect("checked outgoing storage completion");
        let dispatch = self.exception_regions.dispatch(&abrupt, &self.control_targets.snapshot(), statement.span).expect("checked enclosing destination");
        let indent = "    ".repeat(depth);
        let result = self.native_type(fun.result);
        let _ = writeln!(output, "{indent}{in_decl} {out_decl} let {pattern} = {{ let {pattern} = {values}; match {pending} {{ TsrCompletion::Normal(storage)=>storage, TsrCompletion::Throw((caught_error,{pattern}))=>{{ {setup} match {handled} {{ TsrCompletion::Normal(storage)=>storage, other=>{{ let abrupt:TsrCompletion<(),{result},{out_name}>=match other {{ TsrCompletion::Return(value)=>TsrCompletion::Return(value), TsrCompletion::Throw(error)=>TsrCompletion::Throw(error), TsrCompletion::Break(target)=>TsrCompletion::Break(target), TsrCompletion::Continue(target)=>TsrCompletion::Continue(target), TsrCompletion::Normal(_)=>unreachable!() }}; {dispatch}; unreachable!() }} }} }}, TsrCompletion::Return(_)|TsrCompletion::Break(_)|TsrCompletion::Continue(_)=>unreachable!(\"straight-line storage region\") }} }};");
        true
    }

    #[inline(never)]
    fn map_storage_completion_body(&self, fun: &CheckedFunction, body: &[Stmt], declarations: &[Stmt], id: usize, error_name: &str, contract: crate::error_effects::FailureSet, runtime: Option<Type>, values: &str, carry_error: bool) -> String {
        let _region = self.exception_regions.enter_typed(id, Vec::new(), error_name.into(), contract, runtime);
        if carry_error { self.exception_regions.attach_storage(values.into()); }
        let mut statements = String::new();
        self.control_block_epochs(&mut statements, fun, body, 1, None, Some(declarations));
        let result = self.native_type(fun.result);
        let error = if carry_error { "_" } else { error_name };
        format!("{{ let completion:TsrCompletion<_,{result},{error}>='exception_region{id}: {{ {statements} TsrCompletion::Normal({values}) }}; completion }}")
    }
}
