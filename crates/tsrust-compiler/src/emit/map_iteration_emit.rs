//! Move carried survivors through resumable iteration emission.
use super::*;
use statement_work::Work;
use std::rc::Rc;

pub(super) enum IterationWork<'a> {
    FirstEnd(&'a Stmt, Rc<Vec<&'a Stmt>>, usize),
    BodyEnd(&'a Stmt, Rc<Vec<&'a Stmt>>, usize, bool),
    Back(&'a Stmt, bool),
}
impl Emitter<'_> {
    pub(super) fn schedule_map_iteration<'a, 's:'a>(&'s self, output: &mut String, fun: &'a CheckedFunction, statement: &'a Stmt, visible: Rc<Vec<&'a Stmt>>, depth: usize, pending: &mut Vec<Work<'a, 's>>) -> bool {
        let epochs = fun.storage_exits.map_iteration_epochs(statement.span);
        if epochs.is_empty() { return false; }
        if matches!(statement.kind, StmtKind::ForOf { .. }) {
            pending.push(Work::Post(statement, depth));
            self.schedule_for_of(output, fun, statement, visible, depth, &[], pending);
            return true;
        }
        let post_test = match &statement.kind {
            StmtKind::While { .. } => false,
            StmtKind::Loop { condition: Some(_), post_test, .. } => *post_test,
            _ => return false,
        };
        let label = statement.span.start;
        let target = if matches!(statement.kind, StmtKind::Loop { .. }) { ControlTarget::Tail(label) } else { ControlTarget::Loop(label) };
        pending.push(Work::Scope(self.control_targets.enter(target)));
        if post_test {
            let pattern = epochs.iter().map(|epoch| format!("mut {}", self.map_entry_epoch_place(fun, &epoch.entry))).collect::<Vec<_>>().join(",");
            let _ = writeln!(output, "let (mut map_stop_{label},{pattern},) = {{");
            pending.push(Work::Iteration(IterationWork::FirstEnd(statement, visible.clone(), depth)));
            self.schedule_map_iteration_work(output, fun, statement, visible, depth + 1, false, pending);
        } else { self.schedule_map_iteration_loop(output, fun, statement, visible, depth, false, pending); }
        true
    }
    fn schedule_map_iteration_loop<'a, 's:'a>(&'s self, output: &mut String, fun: &'a CheckedFunction, statement: &'a Stmt, visible: Rc<Vec<&'a Stmt>>, depth: usize, post_test: bool, pending: &mut Vec<Work<'a, 's>>) {
        let condition = match &statement.kind { StmtKind::While { condition, .. } | StmtKind::Loop { condition: Some(condition), .. } => condition, _ => unreachable!() };
        let label = statement.span.start;
        let names = fun.storage_exits.map_iteration_epochs(statement.span).iter().map(|epoch| self.map_entry_epoch_place(fun, &epoch.entry)).collect::<Vec<_>>().join(",");
        if !post_test { let _ = writeln!(output, "let mut map_stop_{label}=false;"); }
        let _ = writeln!(output, "'loop_{label}: while !map_stop_{label} && ({}) {{", self.expr(fun, condition, Mode::Copy));
        let _ = writeln!(output, "    (map_stop_{label},{names},) = {{");
        pending.push(Work::Post(statement, depth));
        pending.push(Work::Text(format!("        (map_stop_{label},{names},)\n    }};\n}}\n")));
        self.schedule_map_iteration_work(output, fun, statement, visible, depth + 2, true, pending);
    }
    fn schedule_map_iteration_work<'a, 's:'a>(&'s self, output: &mut String, fun: &'a CheckedFunction, statement: &'a Stmt, visible: Rc<Vec<&'a Stmt>>, depth: usize, widen: bool, pending: &mut Vec<Work<'a, 's>>) {
        let epochs = fun.storage_exits.map_iteration_epochs(statement.span);
        let label = statement.span.start;
        for epoch in epochs {
            let working = crate::analysis::MapEntryEpoch { place: epoch.entry.place.clone(), input: if widen { epoch.entry.output } else { epoch.entry.input }, output: epoch.working, empty: if widen { epoch.carrier_empty } else { epoch.entry.empty }, before: true };
            if working.input != working.output { let _ = writeln!(output, "{}", self.map_entry_epoch_code(fun, &working)); }
        }
        let body = match &statement.kind { StmtKind::While { body, .. } | StmtKind::Loop { body, .. } | StmtKind::ForOf { body, .. } => body, _ => unreachable!() };
        let pattern = epochs.iter().map(|epoch| format!("mut {}", self.map_entry_epoch_place(fun, &epoch.entry))).collect::<Vec<_>>().join(",");
        let exits = crate::completion::definitely_abrupt(body);
        let annotation = if exits {
            let types = epochs.iter().map(|epoch| self.native_type(self.map_storage_leaf_type(epoch.body.output, &epoch.entry.place.fields).expect("checked iteration owner"))).collect::<Vec<_>>().join(",");
            format!(":(bool,{types},)")
        } else { String::new() };
        let _ = writeln!(output, "let (map_stop_{label},{pattern},){annotation} = 'map_body_{label}: {{");
        pending.push(Work::Iteration(IterationWork::BodyEnd(statement, visible.clone(), depth, exits)));
        pending.push(Work::Body(body, visible, depth + 1, Some(label)));
    }
    pub(super) fn schedule_map_for_of_body<'a, 's:'a>(&'s self, output: &mut String, fun: &'a CheckedFunction, statement: &'a Stmt, visible: Rc<Vec<&'a Stmt>>, depth: usize, pending: &mut Vec<Work<'a, 's>>) -> bool {
        let epochs = fun.storage_exits.map_iteration_epochs(statement.span);
        if epochs.is_empty() { return false; }
        let label = statement.span.start;
        let names = epochs.iter().map(|epoch| self.map_entry_epoch_place(fun, &epoch.entry)).collect::<Vec<_>>().join(",");
        let _ = writeln!(output, "let map_stop_{label};\n(map_stop_{label},{names},) = {{");
        pending.push(Work::Text(format!("    (map_stop_{label},{names},)\n}};\nif map_stop_{label} {{break 'loop_{label};}}\n")));
        self.schedule_map_iteration_work(output, fun, statement, visible, depth + 1, true, pending);
        true
    }
    pub(super) fn resume_map_iteration<'a, 's:'a>(&'s self, output: &mut String, fun: &'a CheckedFunction, work: IterationWork<'a>, pending: &mut Vec<Work<'a, 's>>) {
        match work {
            IterationWork::FirstEnd(statement, visible, depth) => {
                let label = statement.span.start;
                let names = fun.storage_exits.map_iteration_epochs(statement.span).iter().map(|epoch| self.map_entry_epoch_place(fun, &epoch.entry)).collect::<Vec<_>>().join(",");
                let _ = writeln!(output, "    (map_stop_{label},{names},)\n}};");
                if matches!(statement.kind, StmtKind::Loop { tail_reachable: false, .. }) {
                    if fun.storage_exits.map_loop_has_break(statement.span) { pending.push(Work::Post(statement, depth)); }
                } else { self.schedule_map_iteration_loop(output, fun, statement, visible, depth, true, pending); }
            },
            IterationWork::BodyEnd(statement, visible, depth, exits) => {
                let epochs = fun.storage_exits.map_iteration_epochs(statement.span);
                let label = statement.span.start;
                let places = epochs.iter().map(|epoch| self.map_entry_epoch_place(fun, &epoch.entry)).collect::<Vec<_>>();
                let names = places.join(",");
                let abrupt = fun.storage_exits.has_map_iteration_controls(label);
                if exits { let _ = writeln!(output, "    unreachable!(\"checked abrupt iteration body\");\n}};"); }
                else {
                    if abrupt { for epoch in epochs {
                        if epoch.body.input != epoch.body.output { let _ = writeln!(output, "    {}", self.map_entry_epoch_code(fun, &epoch.body)); }
                    } }
                    let _ = writeln!(output, "    (false,{names},)\n}};");
                }
                if abrupt {
                    let pattern = places.iter().map(|place| format!("mut {place}")).collect::<Vec<_>>().join(",");
                    let _ = writeln!(output, "let ({pattern},) = if !map_stop_{label} {{");
                }
                pending.push(Work::Iteration(IterationWork::Back(statement, abrupt)));
                if let StmtKind::Loop { update, tail_reachable: true, .. } = &statement.kind {
                    pending.push(Work::Body(update, visible, depth + usize::from(abrupt), Some(label)));
                }
            },
            IterationWork::Back(statement, abrupt) => {
                let epochs = fun.storage_exits.map_iteration_epochs(statement.span);
                let indent = if abrupt { "    " } else { "" };
                for epoch in epochs {
                    if epoch.back.input != epoch.back.output { let _ = writeln!(output, "{indent}{}", self.map_entry_epoch_code(fun, &epoch.back)); }
                }
                if !abrupt { return; }
                let names = epochs.iter().map(|epoch| self.map_entry_epoch_place(fun, &epoch.entry)).collect::<Vec<_>>().join(",");
                let _ = writeln!(output, "    ({names},)\n}} else {{");
                for epoch in epochs {
                    let stopped = crate::analysis::MapEntryEpoch { place: epoch.entry.place.clone(), input: epoch.body.output, output: epoch.back.output, empty: epoch.break_empty, before: false };
                    if stopped.input != stopped.output { let _ = writeln!(output, "    {}", self.map_entry_epoch_code(fun, &stopped)); }
                }
                let _ = writeln!(output, "    ({names},)\n}};");
            },
        }
    }
    pub(super) fn map_iteration_control_code(&self,fun:&CheckedFunction,statement:&Stmt)->Option<String> {
        let (label,continuing,epochs)=fun.storage_exits.map_iteration_control(statement.span)?;
        let mut code=String::new();
        for epoch in epochs {if epoch.input!=epoch.output{code.push_str(&self.map_entry_epoch_code(fun,epoch));}}
        let names=epochs.iter().map(|epoch|self.map_entry_epoch_place(fun,epoch)).collect::<Vec<_>>().join(",");
        if matches!(statement.control_target,Some(ControlTarget::Block(_))) {
            code.push_str(&format!("break 'map_block_{label} ({names},);"));
        } else {code.push_str(&format!("break 'map_body_{label} ({},{names},);",!continuing));}
        Some(code)
    }

}
