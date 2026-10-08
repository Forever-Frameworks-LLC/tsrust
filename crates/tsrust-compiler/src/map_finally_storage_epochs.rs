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
        let mut places = Vec::new();
        let mut native_places = std::collections::HashMap::new();
        let mut statements = region.body.iter().chain(cleanup).collect::<Vec<_>>();
        while let Some(item) = statements.pop() {
            if !item.labels.is_empty() {
                return false;
            }
            match &item.kind {
                StmtKind::Block(body) => statements.extend(body),
                StmtKind::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    statements.extend(then_body);
                    statements.extend(else_body);
                }
                StmtKind::Let { .. }
                | StmtKind::Expr(_)
                | StmtKind::Assign { .. }
                | StmtKind::Throw(_)
                | StmtKind::Return(_)
                | StmtKind::Break
                | StmtKind::Continue => {}
                _ => return false,
            }
            if let Some((place, native)) = self.local_map_clear_storage(fun, declarations, item)
                && !places.contains(&place)
            {
                native_places.insert(place.clone(), native);
                places.push(place);
            }
        }
        if places.is_empty() {
            return false;
        }
        places.sort();
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
        let pending = self.map_storage_completion_body(
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
        );
        let normal_cleanup = self.map_storage_completion_body(
            fun,
            cleanup,
            declarations,
            first + 1,
            &cleanup_name,
            cleanup_contract.clone(),
            runtime,
            &storage_type,
            &places,
            false,
            false,
        );
        let thrown_cleanup = self.map_storage_completion_body(
            fun,
            cleanup,
            declarations,
            first + 2,
            &cleanup_name,
            cleanup_contract.clone(),
            runtime,
            &storage_type,
            &places,
            false,
            false,
        );
        let returned_cleanup = self.map_storage_completion_body(
            fun,
            cleanup,
            declarations,
            first + 3,
            &cleanup_name,
            cleanup_contract.clone(),
            runtime,
            &storage_type,
            &places,
            false,
            false,
        );
        let cleanup_abrupt = self.map_storage_abrupt_dispatch(
            fun,
            "other",
            &cleanup_name,
            &cleanup_contract,
            runtime,
            statement.span,
        );
        let normal_finish = format!(
            "{{let {pattern}=storage;match {normal_cleanup}{{TsrCompletion::Normal(storage)=>storage,other=>{{{cleanup_abrupt}}}}}}}"
        );
        let throw_arm = if incoming.runtime || incoming.opaque || !incoming.explicit.is_empty() {
            let resume = if crate::completion::definitely_abrupt(cleanup, 0) {
                "unreachable!(\"abrupt cleanup cannot resume pending error\")".into()
            } else {
                self.map_storage_abrupt_dispatch(
                    fun,
                    "TsrCompletion::Throw(pending_error)",
                    &incoming_name,
                    &incoming,
                    runtime,
                    statement.span,
                )
            };
            format!(
                "TsrCompletion::Throw((pending_error,{pattern}))=>{{match {thrown_cleanup}{{TsrCompletion::Normal(storage)=>{{drop(storage);{resume}}},other=>{{drop(pending_error);{cleanup_abrupt}}}}}}}"
            )
        } else {
            "TsrCompletion::Throw((error,_))=>match error {}".into()
        };
        let result = self.native_type(fun.result);
        let (outer_error, _, _) = self
            .exception_regions
            .current_transport()
            .expect("checked enclosing completion");
        let resumed = format!(
            "match pending_exit {{TsrStorageExit::Return(value)=>TsrCompletion::<(),{result},{outer_error}>::Return(value),TsrStorageExit::Break(target)=>TsrCompletion::Break(target),TsrStorageExit::Continue(target)=>TsrCompletion::Continue(target)}}"
        );
        let resume_control = self
            .exception_regions
            .dispatch(&resumed, &self.control_targets.snapshot(), statement.span)
            .expect("checked cleanup control destination");
        let resume_control = if crate::completion::definitely_abrupt(cleanup, 0) {
            "unreachable!(\"abrupt cleanup cannot resume pending control\")".into()
        } else {
            format!("{resume_control};unreachable!()")
        };
        let return_arm = format!(
            "TsrCompletion::Return((pending_exit,{pattern}))=>{{match {returned_cleanup}{{TsrCompletion::Normal(storage)=>{{drop(storage);{resume_control}}},other=>{{drop(pending_exit);{cleanup_abrupt}}}}}}}"
        );
        let indent = "    ".repeat(depth);
        let _ = writeln!(
            output,
            "{indent}{incoming_decl} {cleanup_decl} let {pattern}={{let {pattern}={values};match {pending}{{TsrCompletion::Normal(storage)=>{normal_finish},{throw_arm},{return_arm},TsrCompletion::Break(_)|TsrCompletion::Continue(_)=>unreachable!(\"straight-line finally storage\")}}}};"
        );
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
}
