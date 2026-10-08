//! End proven-empty scoped Map epochs at iteration boundaries.
use super::*;

impl Emitter<'_> {
    #[inline(never)]
    pub(super) fn map_empty_while_epoch(
        &self,
        output: &mut String,
        fun: &CheckedFunction,
        statement: &Stmt,
        declarations: &[Stmt],
        depth: usize,
    ) -> bool {
        let StmtKind::While { condition, body } = &statement.kind else {
            return false;
        };
        let Some(index) = declarations
            .iter()
            .position(|item| item.span == statement.span)
        else {
            return false;
        };
        if !statement.labels.is_empty() {
            return false;
        }
        // Consecutive empty declarations and each owning record leaf form an
        // independent storage epoch. Prove every selected leaf is cleared.
        let mut candidates = Vec::new();
        for previous in declarations[..index].iter().rev() {
            let StmtKind::Let {
                binding: Some(binding),
                value,
                ..
            } = &previous.kind
            else {
                break;
            };
            let mut found = false;
            if empty_map_initializer(value, &[]) {
                candidates.push((format!("value{binding}"), *binding, Vec::new()));
                found = true;
            }
            if let Some(layout) = self.owned_record_layout(fun, *binding) {
                for leaf in &layout.leaves {
                    if !matches!(leaf.ty, Type::Map(_)) {
                        continue;
                    }
                    let mut ty = fun.bindings[*binding].ty;
                    let mut names = Vec::new();
                    for index in &leaf.path {
                        let Type::Record(record) = ty else {
                            unreachable!("checked owning record path")
                        };
                        let field = &self.program.records[record].fields[*index];
                        names.push(field.name.text.clone());
                        ty = field.ty.unwrap();
                    }
                    if empty_map_initializer(value, &names) {
                        candidates.push((leaf.native_name(*binding), *binding, names));
                        found = true;
                    }
                }
            }
            if !found {
                break;
            }
        }
        if candidates.is_empty() {
            return false;
        }
        let mut resets = Vec::new();
        let mut cleaned = false;
        for item in body {
            match &item.kind {
                StmtKind::Try(region) if !cleaned => {
                    let Some(cleanup) = &region.finally else {
                        return false;
                    };
                    for (candidate, binding, path) in &candidates {
                        let Some((position, native)) =
                            cleanup.iter().enumerate().find_map(|(position, item)| {
                                self.local_map_clear_storage(fun, declarations, item)
                                    .filter(|(place, _)| place == candidate)
                                    .map(|(_, native)| (position, native))
                            })
                        else {
                            return false;
                        };
                        if cleanup[..position].iter().any(|item| {
                            !matches!(
                                item.kind,
                                StmtKind::Let { .. } | StmtKind::Expr(_) | StmtKind::Assign { .. }
                            )
                        }) {
                            return false;
                        }
                        if cleanup[position + 1..]
                            .iter()
                            .any(|item| !map_independent_statement(item, *binding, path))
                        {
                            return false;
                        }
                        resets.push((candidate.clone(), native, *binding, path.clone()));
                    }
                    cleaned = true;
                }
                StmtKind::Let { .. } | StmtKind::Expr(_) | StmtKind::Assign { .. }
                    if !cleaned
                        || resets.iter().all(|(_, _, binding, path)| {
                            map_independent_statement(item, *binding, path)
                        }) => {}
                _ => return false,
            }
        }
        if resets.is_empty()
            || resets
                .iter()
                .any(|(_, _, binding, path)| map_reference(condition, *binding, path))
        {
            return false;
        }
        let indent = "    ".repeat(depth);
        let label = statement.span.start;
        let _scope = self.control_targets.enter(ControlTarget::Loop(label));
        // Keep the original empty owner for zero iterations and outer errors.
        // Cleanup empties each iteration before any exit, so no payload crosses
        // epochs; only loop-local owners ever contain callbacks.
        let _ = writeln!(
            output,
            "{indent}'loop_{label}: while {} {{",
            self.expr(fun, condition, Mode::Copy)
        );
        for (place, native, _, _) in &resets {
            let _ = writeln!(
                output,
                "{indent}    let mut {place}: {native} = <{native}>::new();"
            );
        }
        let mut nested = declarations.to_vec();
        nested.extend(body.iter().cloned());
        self.control_block_epochs(output, fun, body, depth + 1, Some(label), Some(&nested));
        let _ = writeln!(output, "{indent}}}");
        self.exception_regions.renew_storage(
            &resets
                .iter()
                .map(|(place, _, _, _)| place.clone())
                .collect::<Vec<_>>(),
            statement.span.end,
        );
        true
    }
}

fn map_independent_statement(statement: &Stmt, binding: usize, path: &[String]) -> bool {
    match &statement.kind {
        StmtKind::Let { value, .. } | StmtKind::Expr(value) => !map_reference(value, binding, path),
        StmtKind::Assign { target, value, .. } => {
            !map_reference(target, binding, path) && !map_reference(value, binding, path)
        }
        StmtKind::Return(value) => value
            .as_ref()
            .is_none_or(|value| !map_reference(value, binding, path)),
        StmtKind::Throw(value) => !map_reference(value, binding, path),
        StmtKind::Break | StmtKind::Continue => true,
        _ => false,
    }
}

// Unknown lowered expressions conservatively retain the original loop path.
fn map_reference(expression: &Expr, binding: usize, path: &[String]) -> bool {
    let mut pending = vec![expression];
    while let Some(value) = pending.pop() {
        if value.target == Some(CallTarget::ObjectKeys)
            && let ExprKind::Call(_, arguments) = &value.kind
            && arguments.len() == 1
            && arguments[0].binding == Some(binding)
            && matches!(arguments[0].kind, ExprKind::Name(_))
        {
            continue;
        }
        let mut base = value;
        let mut fields = Vec::new();
        while let ExprKind::Field(parent, name) = &base.kind {
            fields.push(name.as_str());
            base = parent;
        }
        fields.reverse();
        if matches!(base.kind, ExprKind::Name(_)) && base.binding == Some(binding) {
            let selected = path.iter().map(String::as_str).collect::<Vec<_>>();
            if fields.starts_with(&selected) {
                if fields[selected.len()..] == ["size"] {
                    continue;
                }
                return true;
            }
            if selected.starts_with(&fields) {
                return true;
            }
            continue; // An independent sibling leaf.
        }
        if value.binding == Some(binding) {
            return true;
        }
        match &value.kind {
            ExprKind::Name(_)
            | ExprKind::String(_)
            | ExprKind::Integer(_)
            | ExprKind::Number(_)
            | ExprKind::Boolean(_)
            | ExprKind::Null
            | ExprKind::Undefined
            | ExprKind::EnumMember { .. } => {}
            ExprKind::Field(base, _)
            | ExprKind::Unary { value: base, .. }
            | ExprKind::Update { target: base, .. }
            | ExprKind::Inject { value: base, .. }
            | ExprKind::Project { value: base, .. }
            | ExprKind::TypeAssertion { value: base, .. } => pending.push(base),
            ExprKind::Index(left, right)
            | ExprKind::Binary { left, right, .. }
            | ExprKind::Assign {
                target: left,
                value: right,
                ..
            }
            | ExprKind::CompoundAssign {
                target: left,
                value: right,
                ..
            }
            | ExprKind::LogicalAssign {
                target: left,
                value: right,
                ..
            }
            | ExprKind::CoalesceAssign {
                target: left,
                value: right,
                ..
            } => {
                pending.push(left);
                pending.push(right);
            }
            ExprKind::Call(callee, args) => {
                pending.push(callee);
                pending.extend(args);
            }
            ExprKind::Sequence(items) | ExprKind::Template(items) | ExprKind::Array(items) => {
                pending.extend(items)
            }
            _ => return true,
        }
    }
    false
}

fn empty_map_initializer(mut value: &Expr, path: &[String]) -> bool {
    for name in path {
        while let ExprKind::Unary {
            op: UnaryOp::RecordCarrier,
            value: inner,
            ..
        } = &value.kind
        {
            value = inner;
        }
        let ExprKind::Record(fields) = &value.kind else {
            return false;
        };
        let Some((_, field)) = fields.iter().find(|(field, _)| field.text == *name) else {
            return false;
        };
        value = field;
    }
    while let ExprKind::Unary {
        op: UnaryOp::RecordCarrier,
        value: inner,
        ..
    } = &value.kind
    {
        value = inner;
    }
    matches!(&value.kind,ExprKind::MapConstructor(arguments,_) if arguments.is_empty())
}
