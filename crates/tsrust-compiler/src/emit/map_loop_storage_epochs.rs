//! End proven-empty scoped Map epochs at iteration boundaries.
use super::*;
use statement_work::Work;
use storage_statement_work::StorageWork;
use std::rc::Rc;

impl Emitter<'_> {
    #[inline(never)]
    pub(super) fn schedule_empty_map_loop<'a, 's>(
        &'s self,
        output: &mut String,
        fun: &CheckedFunction,
        statement: &'a Stmt,
        visible: Rc<Vec<&'a Stmt>>,
        depth: usize,
        pending: &mut Vec<Work<'a, 's>>,
    ) -> bool {
        let declarations = Declarations::Visible(&visible);
        let (condition, body, tail) = match &statement.kind {
            StmtKind::While { condition, body } => (Some(condition), body, None),
            StmtKind::Loop { condition, body, update, post_test, tail_reachable } =>
                (condition.as_ref(), body, Some((update, *post_test, *tail_reachable))),
            StmtKind::ForOf { iterable, body, .. } => (Some(iterable), body, None),
            _ => return false,
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
        let mut intervening = Vec::new();
        for previous in declarations.prefix(index).iter().rev() {
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
                if self.scoped_record_native_parameter(fun.bindings[*binding].ty) {
                    candidates.push((format!("value{binding}"), *binding, Vec::new()));
                }
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
                        if self.scoped_record_native_parameter(leaf.ty) {
                            candidates.push((leaf.native_name(*binding), *binding, names));
                        }
                        found = true;
                    }
                }
            }
            if !found {
                intervening.push(previous);
            }
        }
        if candidates.is_empty() {
            return false;
        }
        if intervening.iter().any(|item| candidates.iter().any(|(_, binding, path)|
            !map_independent_statement(item, *binding, path))) {
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
                .any(|(_, _, binding, path)| {
                    condition.is_some_and(|condition| map_reference(condition, *binding, path))
                    || tail.is_some_and(|(update, _, _)| update.iter().any(|item|
                        !map_independent_statement(item, *binding, path)))
                })
        {
            return false;
        }

        let label = statement.span.start;
        let names = resets.iter().map(|(place, _, _, _)| place.clone()).collect::<Vec<_>>();
        let mut nested = declarations.iter().collect::<Vec<_>>();
        nested.extend(body.iter());
        let nested = Rc::new(nested);
        pending.push(Work::Post(statement, depth));
        if matches!(statement.kind, StmtKind::ForOf { .. }) {
            let layouts = resets.iter().map(|(place, native, _, _)| (place.clone(), native.clone())).collect::<Vec<_>>();
            pending.push(Work::Storage(StorageWork::Renew(names, statement.span.end)));
            self.schedule_for_of(output, fun, statement, nested, depth, &layouts, pending);
            return true;
        }
        pending.push(Work::Scope(self.control_targets.enter(if tail.is_some() { ControlTarget::Tail(label) } else { ControlTarget::Loop(label) })));
        pending.push(Work::Storage(StorageWork::Renew(names, statement.span.end)));
        // Each iteration owns its fresh empty storage; zero iterations retain
        // the original owner. Tail work remains inside the loop control scope.
        if let Some((_, post_test, _)) = tail {
            let _ = writeln!(output, "'loop_{label}: loop {{");
            if !post_test && let Some(condition) = condition {
                let _ = writeln!(output, "    if !({}) {{ break 'loop_{label}; }}", self.expr(fun, condition, Mode::Copy));
            }
            let _ = writeln!(output, "    'body_{label}: {{");
            pending.push(Work::Storage(StorageWork::EmptyLoopTail(statement, depth)));
        } else {
            let _ = writeln!(output, "'loop_{label}: while {} {{", self.expr(fun, condition.unwrap(), Mode::Copy));
            pending.push(Work::Text("}\n".into()));
        }
        for (place, native, _, _) in &resets { let _ = writeln!(output, "let mut {place}: {native} = <{native}>::new();"); }
        pending.push(Work::Body(body, nested, depth + if tail.is_some() { 2 } else { 1 }, Some(label)));

        true
    }
}

fn map_independent_statement(statement: &Stmt, binding: usize, path: &[String]) -> bool {
    let mut pending = vec![statement];
    while let Some(item) = pending.pop() {
        match &item.kind {
            StmtKind::Let { value, .. } | StmtKind::Expr(value)
                if !map_reference(value, binding, path) => {}
            StmtKind::Assign { target, value, .. }
                if !map_reference(target, binding, path)
                    && !map_reference(value, binding, path) => {}
            StmtKind::Return(value)
                if value
                    .as_ref()
                    .is_none_or(|value| !map_reference(value, binding, path)) => {}
            StmtKind::Throw(value) if !map_reference(value, binding, path) => {}
            StmtKind::Break | StmtKind::Continue => {}
            StmtKind::Block(body) => pending.extend(body),
            StmtKind::If {
                condition,
                then_body,
                else_body,
            } if !map_reference(condition, binding, path) => {
                pending.extend(then_body);
                pending.extend(else_body);
            }
            _ => return false,
        }
    }
    true
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
            ExprKind::Sequence(items) | ExprKind::Template(items) | ExprKind::Array(items)
            | ExprKind::ArrayOf(items, _) | ExprKind::SetConstructor(items, _)
            | ExprKind::MapConstructor(items, _) | ExprKind::NativeConstructor(items, _) => {
                pending.extend(items)
            }
            ExprKind::Record(fields) => pending.extend(fields.iter().map(|(_, value)| value)),
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
    matches!(&value.kind,ExprKind::MapConstructor(arguments,_) | ExprKind::NativeConstructor(arguments,_) if arguments.is_empty() && matches!(value.ty,Some(Type::Map(_))))
}
