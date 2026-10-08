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
        let Some(previous) = index
            .checked_sub(1)
            .and_then(|index| declarations.get(index))
        else {
            return false;
        };
        let StmtKind::Let {
            binding: Some(binding),
            value,
            ..
        } = &previous.kind
        else {
            return false;
        };
        if !matches!(&value.kind, ExprKind::MapConstructor(arguments, _) if arguments.is_empty())
            || !statement.labels.is_empty()
            || map_reference(condition, *binding)
        {
            return false;
        }
        let mut reset = None;
        let mut cleaned = false;
        for item in body {
            match &item.kind {
                StmtKind::Try(region) if !cleaned && region.catch.is_none() => {
                    let Some(cleanup) = &region.finally else {
                        return false;
                    };
                    let Some((position, place)) =
                        cleanup.iter().enumerate().find_map(|(position, item)| {
                            self.local_map_clear_storage(fun, declarations, item)
                                .filter(|(place, _)| place == &format!("value{binding}"))
                                .map(|place| (position, place))
                        })
                    else {
                        return false;
                    };
                    // A direct clear must be reached on every cleanup path. No
                    // earlier cleanup exit and no later refill can retain an entry.
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
                        .any(|item| !map_independent_statement(item, *binding))
                    {
                        return false;
                    }
                    reset = Some(place);
                    cleaned = true;
                }
                StmtKind::Let { .. } | StmtKind::Expr(_) | StmtKind::Assign { .. }
                    if !cleaned || map_independent_statement(item, *binding) => {}
                _ => return false,
            }
        }
        let Some((place, native)) = reset else {
            return false;
        };
        let indent = "    ".repeat(depth);
        let label = statement.span.start;
        let _scope = self.control_targets.enter(ControlTarget::Loop(label));
        // Keep the original empty owner for zero iterations and outer errors.
        // Cleanup empties each iteration before any exit, so no payload crosses
        // epochs; only loop-local owners ever contain callbacks.
        let _ = writeln!(
            output,
            "{indent}'loop_{label}: while {} {{\n{indent}    let mut {place}: {native} = <{native}>::new();",
            self.expr(fun, condition, Mode::Copy)
        );
        let mut nested = declarations.to_vec();
        nested.extend(body.iter().cloned());
        self.control_block_epochs(output, fun, body, depth + 1, Some(label), Some(&nested));
        let _ = writeln!(output, "{indent}}}");
        self.exception_regions
            .renew_storage(&[place], statement.span.end);
        true
    }
}

fn map_independent_statement(statement: &Stmt, binding: usize) -> bool {
    match &statement.kind {
        StmtKind::Let { value, .. } | StmtKind::Expr(value) => !map_reference(value, binding),
        StmtKind::Assign { target, value, .. } => {
            !map_reference(target, binding) && !map_reference(value, binding)
        }
        StmtKind::Return(value) => value
            .as_ref()
            .is_none_or(|value| !map_reference(value, binding)),
        StmtKind::Throw(value) => !map_reference(value, binding),
        StmtKind::Break | StmtKind::Continue => true,
        _ => false,
    }
}

// Unknown lowered expressions conservatively retain the original loop path.
fn map_reference(expression: &Expr, binding: usize) -> bool {
    let mut pending = vec![expression];
    while let Some(value) = pending.pop() {
        if matches!(&value.kind,ExprKind::Field(base,name) if name == "size" && base.binding == Some(binding) && matches!(base.kind,ExprKind::Name(_)))
        {
            continue;
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
