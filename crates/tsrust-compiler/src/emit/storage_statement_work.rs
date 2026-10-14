//! Storage projections resume after child bodies without re-entering emission.
use super::*;
use super::statement_work::Work;
use std::rc::Rc;

pub(super) enum StorageWork<'a> {
    Renew(Vec<String>, usize),
    EmptyLoopTail(&'a Stmt, usize),
    BlockEnd(&'a Stmt, Vec<String>, Vec<String>),
    Branch(&'a Stmt, Rc<Vec<String>>, Rc<Vec<&'a Stmt>>, usize, Option<usize>, bool),
    BranchEnd(&'a Stmt, Rc<Vec<String>>, bool),
    IfEnd(&'a Stmt, Rc<Vec<String>>),
}

impl Emitter<'_> {
    pub(super) fn schedule_storage_block<'a, 's>(
        &'s self, output: &mut String, fun: &CheckedFunction, statement: &'a Stmt,
        visible: Rc<Vec<&'a Stmt>>, depth: usize, label: Option<usize>, pending: &mut Vec<Work<'a, 's>>,
    ) -> bool {
        let StmtKind::Block(body) = &statement.kind else { return false; };
        let arrays = fun.storage_exits.array_block_epochs(statement.span.start);
        let maps = fun.storage_exits.map_block_epochs(statement.span.start);
        if arrays.is_empty() && maps.is_empty() { return false; }
        let mut names = arrays.iter().chain(maps).map(|epoch| self.map_entry_epoch_place(fun, epoch)).collect::<Vec<_>>();
        let ordinary = if arrays.is_empty() { Vec::new() } else {
            self.map_block_return_places(fun, Declarations::Visible(&visible), body)
        };
        for name in &ordinary { if !names.contains(name) { names.push(name.clone()); } }
        let pattern = names.iter().map(|name| format!("mut {name}")).collect::<Vec<_>>().join(",");
        if arrays.is_empty() {
            pending.push(Work::Scope(self.control_targets.enter(ControlTarget::Block(statement.span.start))));
            let _ = writeln!(output, "let ({pattern},) = 'map_block_{}: {{", statement.span.start);
        } else { let _ = writeln!(output, "let ({pattern},) = {{"); }
        pending.push(Work::Storage(StorageWork::BlockEnd(statement, names, ordinary)));
        pending.push(Work::Body(body, visible, depth + 1, label));
        true
    }

    pub(super) fn schedule_storage_if<'a, 's>(
        &'s self, output: &mut String, fun: &CheckedFunction, statement: &'a Stmt,
        visible: Rc<Vec<&'a Stmt>>, depth: usize, label: Option<usize>, pending: &mut Vec<Work<'a, 's>>,
    ) -> bool {
        let StmtKind::If { condition, then_body, else_body } = &statement.kind else { return false; };
        let declarations = Declarations::Visible(&visible);
        let mut places = self.map_block_return_places(fun, declarations, then_body);
        for place in self.map_block_return_places(fun, declarations, else_body) {
            if !places.contains(&place) { places.push(place); }
        }
        for right in [false, true] {
            for epoch in fun.storage_exits.map_branch_restore(statement.span, right) {
                let place = self.map_entry_epoch_place(fun, epoch);
                if !places.contains(&place) { places.push(place); }
            }
        }
        if places.is_empty() { return false; }
        places.sort();
        let pattern = places.iter().map(|place| format!("mut {place}")).collect::<Vec<_>>().join(",");
        let condition = self.expr(fun, condition, Mode::Copy);
        let _ = writeln!(output, "let ({pattern},) = if {condition} {{");
        let places = Rc::new(places);
        pending.push(Work::Post(statement, depth));
        pending.push(Work::Storage(StorageWork::IfEnd(statement, places.clone())));
        pending.push(Work::Storage(StorageWork::Branch(statement, places.clone(), visible.clone(), depth, label, true)));
        pending.push(Work::Storage(StorageWork::Branch(statement, places, visible, depth, label, false)));
        true
    }

    pub(super) fn resume_storage_statement<'a, 's>(
        &'s self, output: &mut String, fun: &CheckedFunction, work: StorageWork<'a>, pending: &mut Vec<Work<'a, 's>>,
    ) {
        match work {
            StorageWork::Renew(names, boundary) => self.exception_regions.renew_storage(&names, boundary),
            StorageWork::EmptyLoopTail(statement, depth) => {
                let StmtKind::Loop { condition, update, post_test, tail_reachable, .. } = &statement.kind else { unreachable!() };
                let _ = writeln!(output, "    }}");
                pending.push(Work::Text("}\n".into()));
                if *tail_reachable {
                    if *post_test && let Some(condition) = condition { pending.push(Work::LoopTest(condition, statement.span.start)); }
                    pending.push(Work::Body(update, Rc::new(update.iter().collect()), depth + 1, None));
                }
            },
            StorageWork::BlockEnd(statement, names, ordinary) => {
                let StmtKind::Block(body) = &statement.kind else { unreachable!() };
                for epoch in fun.storage_exits.map_block_epochs(statement.span.start) {
                    if epoch.input != epoch.output { let _ = writeln!(output, "    {}", self.map_entry_epoch_code(fun, epoch)); }
                }
                let values = names.iter().map(|name| if ordinary.contains(name) {
                    self.map_branch_exit_value(fun, body, name)
                } else { name.clone() }).collect::<Vec<_>>().join(",");
                let _ = writeln!(output, "    ({values},)\n}};");
                self.exception_regions.renew_storage(&names, statement.span.end);
            },
            StorageWork::Branch(statement, places, visible, depth, label, right) => {
                let StmtKind::If { then_body, else_body, .. } = &statement.kind else { unreachable!() };
                // Enter only when this branch starts: its sibling must first
                // restore the exact incoming native-storage availability.
                pending.push(Work::StorageScope(self.exception_regions.storage_branch()));
                pending.push(Work::Storage(StorageWork::BranchEnd(statement, places, right)));
                pending.push(Work::Body(if right { else_body } else { then_body }, visible, depth + 1, label));
            },
            StorageWork::BranchEnd(statement, places, right) => {
                let StmtKind::If { then_body, else_body, .. } = &statement.kind else { unreachable!() };
                for epoch in fun.storage_exits.map_branch_restore(statement.span, right) {
                    let _ = writeln!(output, "    {}", self.map_entry_epoch_code(fun, epoch));
                }
                let body = if right { else_body } else { then_body };
                let values = places.iter().map(|place| self.map_branch_exit_value(fun, body, place)).collect::<Vec<_>>().join(",");
                let suffix = if right { "};" } else { "} else {" };
                let _ = writeln!(output, "    ({values},)\n{suffix}");
            },
            StorageWork::IfEnd(statement, places) => {
                let StmtKind::If { then_body, else_body, .. } = &statement.kind else { unreachable!() };
                let boundary = places.iter().flat_map(|place| [self.map_storage_transfer_at_exit(fun, then_body, place), self.map_storage_transfer_at_exit(fun, else_body, place)])
                    .flatten().map(|span| span.end).max().unwrap_or(statement.span.end).max(statement.span.end);
                self.exception_regions.renew_storage(&places, boundary);
            },
        }
    }
}
