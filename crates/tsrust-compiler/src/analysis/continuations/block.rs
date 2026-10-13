//! Block scope state retained across statement completion.
use super::super::*;

pub(super) struct BlockFrame {
    pub(super) result_type: Type,
    keep_names: bool,
    first_local: usize,
    entry_loans: crate::capture_loan_identity::Boundary,
    shadowed: Vec<(String, usize)>,
    native_shadow_boundary: usize,
    control_boundary: usize,
    pub(super) exited: bool,
    nested_stores: HashSet<String>,
    whole_stores: HashSet<String>,
    declared: usize,
    next: usize,
}

impl Analyzer {
    pub(in crate::analysis) fn block_with_scope(&mut self, context: &mut Context, body: &mut [Stmt], result_type: Type, keep_names: bool) -> Result<bool, Diagnostic> {
        super::driver::check_block(self, context, body, result_type, keep_names)
    }

    /// Advance only before dispatch. A suspended statement resumes after this
    /// boundary, so lexical visibility and storage observations are not replayed.
    pub(super) fn next_block_statement(&mut self, context: &mut Context, body: &mut [Stmt], frame: &mut BlockFrame) -> Result<Option<usize>, Diagnostic> {
        while frame.next < body.len() {
            let index = frame.next;
            frame.next += 1;
            let stmt = &mut body[index];
            // Only diagnostic replay sets these keys; executable analysis never does.
            // Keep declarations in the lexical prepass so shadowed outer names cannot
            // become visible after an invalid initializer is skipped.
            if !self.program.diagnostic_skips.is_empty() && self.skip_diagnostic_statement(stmt) {continue;}

            if frame.exited {
                return Err(Diagnostic::new(
                    "unsupported",
                    "Statement is unreachable after all preceding paths exit this block.",
                    stmt.span,
                ));
            }
            for id in frame.declared..context.bindings.len() {
                let binding = &context.bindings[id];
                if !context.lexical_scopes.last().is_some_and(|names| names.contains(&binding.name)) { continue; }
                if frame.nested_stores.contains(&binding.name)
                    || frame.whole_stores.contains(&binding.name) && !self.nullable_record_field_paths(binding.ty).is_empty()
                { context.control_stored.insert(id); }
            }
            frame.declared = context.bindings.len();
            context.storage_exits.record_before(stmt.span, &context.bindings);
            return Ok(Some(index));
        }
        Ok(None)
    }

    /// Compatibility dispatch while child expression and control-flow requests
    /// are migrated into the shared driver. Block children bypass this.
    pub(super) fn check_block_statement(&mut self, context: &mut Context, stmt: &mut Stmt, frame: &mut BlockFrame) -> Result<(), Diagnostic> {
            match &mut stmt.kind {
                StmtKind::LocalFunction(_) => unreachable!("local declarations must be resolved before checking"),
                StmtKind::Throw(value) if context.completion_frame.is_some() => {
                    self.check_throw_value(context,value)?;
                    frame.exited=true;
                }
                StmtKind::Try(region) if context.exception_environment.is_some() && context.completion_frame.is_some() => {
                    frame.exited=self.check_nested_exception_region(context,region,frame.result_type,stmt.span)?;
                }
                StmtKind::Try(_) | StmtKind::Throw(_) => return Err(Diagnostic::new("unsupported","Exception syntax is parsed; error propagation and completion analysis are not implemented yet.",stmt.span)),
                StmtKind::ForOf { .. } | StmtKind::While { .. } => unreachable!("loop children are scheduled by the driver"),
                StmtKind::Block(_) => unreachable!("block children are scheduled by the driver"),
                StmtKind::Loop { .. } => unreachable!("loop tail is scheduled by the driver"),
                StmtKind::Break | StmtKind::Continue => {
                    self.control_exit(context, &stmt.labels, &mut stmt.control_target, matches!(stmt.kind,StmtKind::Continue), stmt.span)?;
                    let kind=if matches!(stmt.kind,StmtKind::Continue) {
                        crate::completion::CompletionKind::Continue(stmt.control_target)
                    } else {crate::completion::CompletionKind::Break(stmt.control_target)};
                    exception_state::record_completion(context,kind,stmt.span);
                    frame.exited=true;
                }
                StmtKind::If { .. } => unreachable!("if children are scheduled by the driver"),
                StmtKind::Let { .. } => unreachable!("declaration children are scheduled by the driver"),
                StmtKind::Assign { .. } => unreachable!("assignment children are scheduled by the driver"),
                StmtKind::Return(value) => {
                    self.return_statement(context, value, frame.result_type, stmt.span)?;
                    context.storage_exits.record_expression(stmt.span, &context.bindings);
                    capture_restoration::record(context);
                    frame.exited = true;
                }
                StmtKind::Expr(_) => unreachable!("expression statements are scheduled by the driver"),
            }
        Ok(())
    }

    #[inline(never)]
    pub(super) fn enter_block(&mut self, context: &mut Context, body: &mut [Stmt], result_type: Type, keep_names: bool) -> Result<Box<BlockFrame>, Diagnostic> {
        if self.depth >= 128 {
            return Err(Diagnostic::new(
                "unsupported",
                "Combined analysis nesting exceeds the prototype limit of 128.",
                body.first().map_or(Span::default(), |stmt| stmt.span),
            ));
        }
        context.lexical_scopes.push(
            body.iter()
                .filter_map(|stmt| match &stmt.kind {
                    StmtKind::Let { name, .. } => Some(name.text.clone()),
                    _ => None,
                })
                .collect(),
        );
        // Remove only names actually shadowed by this block. Their typed slots
        // and source identities remain available to captured named bodies.
        let mut shadowed = Vec::new();
        for name in context.lexical_scopes.last().expect("entered lexical scope") {
            if let Some(&slot) = context.names.get(name)
                && (context.lexical_scopes.len() != context.callable_scope_depth + 1 || context.is_capture(slot)) {
                shadowed.push((name.clone(), slot));
            }
        }
        let native_shadow_boundary = context.native_shadowed_bindings.as_ref().map_or(0, |slots| slots.len());
        if self.native_borrow_types && !shadowed.is_empty() {
            context.native_shadowed_bindings.get_or_insert_with(Default::default)
                .extend(shadowed.iter().map(|(_, slot)| *slot));
        }
        for (name, _) in &shadowed { context.names.remove(name); }
        self.depth += 1;
        let control_boundary = context.loops.len();
        let first_local = context.bindings.len();
        let entry_loans = crate::capture_loan_identity::Boundary::capture(&context.capture_loans);
        let exited = false;
        let (nested_stores, whole_stores) = nullable_record_fields::control_stores(body);
        let declared = first_local;
        Ok(Box::new(BlockFrame { result_type, keep_names, first_local, entry_loans,
            shadowed, native_shadow_boundary, control_boundary, exited, nested_stores, whole_stores, declared, next: 0 }))
    }

    pub(super) fn finish_block(&mut self, context: &mut Context, body: &mut [Stmt], frame: Box<BlockFrame>) -> Result<bool, Diagnostic> {
        let result = (|| {
            self.record_sequence_carriers(context,body)?;
            context.storage_exits.record(body, &context.bindings[..frame.first_local]);
            if !frame.keep_names { Self::check_retained_scope_borrows(context, frame.first_local)?; }
            Ok(frame.exited)
        })();
        self.leave_block(context, frame, result.is_err());
        result
    }

    /// Release scope state on both successful completion and child failure.
    /// Failed function bodies must not keep their partial declaration visibility.
    pub(super) fn leave_block(&mut self, context: &mut Context, frame: Box<BlockFrame>, failed: bool) {
        let BlockFrame { keep_names, first_local, entry_loans, shadowed, native_shadow_boundary, control_boundary, .. } = *frame;
        let keep_names = keep_names && !failed;
        while context.loops.len() > control_boundary {
            let abandoned = context.loops.pop().expect("nested control frame");
            exception_state::consume_target(context, abandoned.target);
        }
        context.lexical_scopes.pop();
        if !keep_names {
            entry_loans.unwind(&mut context.capture_loans);
            let _ = context.capture_loan_owners.leave_scope(first_local);
            Self::restore_live_capture_loans(context);
            context.mutable_match_borrows.retain(|id| *id < first_local);
            for (id, binding) in context.bindings.iter().enumerate().skip(first_local) {
                // A nested block may already have restored an outer binding with
                // this name. Remove only the local slot still visible here.
                if context.names.get(&binding.name) == Some(&id) {
                    context.names.remove(&binding.name);
                }
            }
        }
        if !keep_names {
            context.names.extend(shadowed);
        }
        if let Some(slots) = context.native_shadowed_bindings.as_mut() {
            slots.truncate(native_shadow_boundary);
        }
        self.depth -= 1;
    }

}
