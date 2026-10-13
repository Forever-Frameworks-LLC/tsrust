//! One completion scheduler spans ordinary statements, catch and repeated cleanup.
use super::*;
use super::exception_assignment::{join_normal, merge};
use crate::syntax::CatchClause;

fn entry(assigned: bool) -> Flow { Flow { normal: Some(assigned), exits: Vec::new() } }
struct Cleanup<'a> {
    body: &'a [Stmt],
    pending: std::vec::IntoIter<(Option<CompletionKind>, bool)>,
    result: Flow,
}
enum Work<'a> {
    Body(&'a [Stmt], Flow),
    Append { rest: &'a [Stmt], prior: Flow, before: bool, failed: bool },
    Block(Option<ControlTarget>),
    IfYes(&'a [Stmt], bool), IfNo(Flow),
    LoopExit(ControlTarget, bool),
    LoopBody { update: &'a [Stmt], condition: Option<&'a Expr>, post: bool, before: bool, target: ControlTarget },
    LoopTail { prior: Flow, condition: Option<&'a Expr>, post: bool },
    Catch(Option<&'a CatchClause>, Option<&'a [Stmt]>),
    Caught(Flow, Option<&'a [Stmt]>),
    Finally(Option<&'a [Stmt]>),
    Cleanup(Cleanup<'a>),
    Cleaned(Cleanup<'a>, Option<CompletionKind>, bool),
}
fn apply_cleanup(state: &mut Cleanup<'_>, mut cleanup: Flow, kind: Option<CompletionKind>) {
    if let Some(kind) = kind {
        if let Some(assigned) = cleanup.normal.take() { cleanup.exits.push(Exit { kind, assigned }); }
    }
    merge(&mut state.result, cleanup);
}

pub(super) fn run(statements: &[Stmt], variable: Variable, assigned: bool) -> Result<Flow, Diagnostic> {
    let mut work = vec![Work::Body(statements, entry(assigned))];
    let mut done = entry(assigned);
    // Immutable cleanup roots have only two input states per variable query.
    // Share successful results across parent re-entry as well as sibling exits.
    let mut cleanup_cache = std::collections::HashMap::<(*const Stmt, usize, bool), Flow>::new();
    while let Some(next) = work.pop() {
        match next {
            Work::Body(body, prior) => {
                let Some(before) = prior.normal else { done = prior; continue };
                let Some((statement, rest)) = body.split_first() else { done = prior; continue };
                let failed = crate::error_effects::statement_entry_may_fail(statement);
                work.push(Work::Append { rest, prior, before, failed });
                done = entry(before);
                match &statement.kind {
                    StmtKind::LocalFunction(_) => unreachable!("local declarations must be resolved before checking"),
                    StmtKind::Try(region) => {
                        work.push(Work::Catch(region.catch.as_ref(), region.finally.as_deref()));
                        work.push(Work::Body(&region.body, entry(before)));
                    },
                    StmtKind::Throw(value) => {
                        if !before { require_clear(value, variable)?; }
                        done.normal = None; done.exits.push(Exit { kind: CompletionKind::Throw, assigned: before });
                    },
                    StmtKind::Assign { target, value, operator: None } if variable.matches(target) => {
                        if !before { require_clear(value, variable)?; }
                        done.normal = Some(true);
                    },
                    StmtKind::Assign { target, value, .. } => if !before { require_clear(target, variable)?; require_clear(value, variable)?; },
                    StmtKind::Let { name, value, .. } if variable.initialized_by(name) => {
                        if !before { require_clear(value, variable)?; }
                        done.normal = Some(!value.is_deferred());
                    },
                    StmtKind::Let { value, .. } | StmtKind::Expr(value) => if !before { require_clear(value, variable)?; },
                    StmtKind::Block(body) => {
                        work.push(Work::Block((!statement.labels.is_empty()).then_some(ControlTarget::Block(statement.span.start))));
                        work.push(Work::Body(body, entry(before)));
                    },
                    StmtKind::If { condition, then_body, else_body } => {
                        if !before { require_clear(condition, variable)?; }
                        work.push(Work::IfYes(else_body, before)); work.push(Work::Body(then_body, entry(before)));
                    },
                    StmtKind::While { condition, body } => {
                        if !before { require_clear(condition, variable)?; }
                        work.push(Work::LoopExit(ControlTarget::Loop(statement.span.start), before)); work.push(Work::Body(body, entry(before)));
                    },
                    StmtKind::ForOf { name, iterable, body, .. } => {
                        if !before { require_clear(iterable, variable)?; }
                        work.push(Work::LoopExit(ControlTarget::Loop(statement.span.start), before));
                        work.push(Work::Body(body, entry(before || variable.initialized_by(name))));
                    },
                    StmtKind::Loop { condition, body, update, post_test, .. } => {
                        if !before && !post_test { if let Some(condition) = condition { require_clear(condition, variable)?; } }
                        work.push(Work::LoopBody { update, condition: condition.as_ref(), post: *post_test, before, target: ControlTarget::Tail(statement.span.start) });
                        work.push(Work::Body(body, entry(before)));
                    },
                    StmtKind::Return(value) => {
                        if !before { if let Some(value) = value { require_clear(value, variable)?; } }
                        done.normal = None; done.exits.push(Exit { kind: CompletionKind::Return, assigned: before });
                    },
                    StmtKind::Break | StmtKind::Continue => {
                        done.normal = None;
                        done.exits.push(Exit { kind: if matches!(statement.kind, StmtKind::Continue) { CompletionKind::Continue(statement.control_target) } else { CompletionKind::Break(statement.control_target) }, assigned: before });
                    },
                }
            },
            Work::Append { rest, mut prior, before, failed } => {
                if failed { done.exits.push(Exit { kind: CompletionKind::Throw, assigned: before }); }
                prior.normal = done.normal; prior.exits.append(&mut done.exits);
                work.push(Work::Body(rest, prior));
            },
            Work::Block(target) => if let Some(target) = target {
                done.exits.retain(|exit| {
                    if exit.kind.targets(target) { done.normal = join_normal(done.normal, Some(exit.assigned)); false } else { true }
                });
            },
            Work::IfYes(body, before) => { work.push(Work::IfNo(done)); work.push(Work::Body(body, entry(before))); done = entry(before); },
            Work::IfNo(mut yes) => { yes.normal = join_normal(yes.normal, done.normal); yes.exits.append(&mut done.exits); done = yes; },
            Work::LoopExit(target, before) => { done.normal = Some(before); done.exits.retain(|exit| !exit.kind.targets(target)); },
            Work::LoopBody { update, condition, post, before, target } => {
                let mut continuing = done.normal;
                let mut prior = entry(before);
                for exit in done.exits.drain(..) {
                    if exit.kind.targets(target) {
                        if exit.kind.is_continue() { continuing = join_normal(continuing, Some(exit.assigned)); }
                    } else { prior.exits.push(exit); }
                }
                if let Some(assigned) = continuing {
                    work.push(Work::LoopTail { prior, condition, post }); work.push(Work::Body(update, entry(assigned)));
                } else { done = prior; }
            },
            Work::LoopTail { mut prior, condition, post } => {
                if post && !done.normal.unwrap_or(true) { if let Some(condition) = condition { require_clear(condition, variable)?; } }
                if post && condition.is_some_and(|value| value.may_fail) {
                    if let Some(assigned) = done.normal { prior.exits.push(Exit { kind: CompletionKind::Throw, assigned }); }
                }
                prior.exits.append(&mut done.exits); done = prior;
            },
            Work::Catch(handler, finally) => {
                if let Some(handler) = handler {
                    let mut caught = None;
                    done.exits.retain(|exit| {
                        if exit.kind == CompletionKind::Throw { caught = join_normal(caught, Some(exit.assigned)); false } else { true }
                    });
                    if let Some(assigned) = caught {
                        work.push(Work::Caught(done, finally));
                        work.push(Work::Body(&handler.body, entry(assigned || handler.name.as_ref().is_some_and(|name| variable.initialized_by(name)))));
                        done = entry(assigned); continue;
                    }
                }
                work.push(Work::Finally(finally));
            },
            Work::Caught(mut prior, finally) => { merge(&mut prior, done); done = prior; work.push(Work::Finally(finally)); },
            Work::Finally(body) => if let Some(body) = body {
                let mut pending = Vec::new();
                if let Some(assigned) = done.normal { pending.push((None, assigned)); }
                pending.extend(done.exits.drain(..).map(|exit| (Some(exit.kind), exit.assigned)));
                work.push(Work::Cleanup(Cleanup { body, pending: pending.into_iter(), result: Flow { normal: None, exits: Vec::new() } }));
            },
            Work::Cleanup(mut state) => {
                if let Some((kind, assigned)) = state.pending.next() {
                    if let Some(cleanup) = cleanup_cache.get(&(state.body.as_ptr(), state.body.len(), assigned)).cloned() {
                        apply_cleanup(&mut state, cleanup, kind); work.push(Work::Cleanup(state));
                    } else {
                        let body = state.body;
                        work.push(Work::Cleaned(state, kind, assigned)); work.push(Work::Body(body, entry(assigned)));
                    }
                } else { done = state.result; }
            },
            Work::Cleaned(mut state, kind, assigned) => {
                cleanup_cache.insert((state.body.as_ptr(), state.body.len(), assigned), done.clone());
                apply_cleanup(&mut state, done, kind); work.push(Work::Cleanup(state)); done = entry(assigned);
            },
        }
    }
    Ok(done)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deep_blocks_branches_and_cleanup_preserve_assignment_and_leaf_diagnostics() {
        for (opening, closing) in [("{", "}"), ("if(true){", "}else{x=2;}"), ("try{", "}finally{}"), ("try{}finally{", "}")] {
            let depth = 16_384;
            for (leaf, succeeds) in [("x=1;", true), ("console.log(x);", false)] {
                let source = format!("function main(){{let x:int;{}{}{}console.log(x);}}", opening.repeat(depth), leaf, closing.repeat(depth));
                let mut program = crate::syntax::parse_unbounded(&source).unwrap();
                let result = super::super::check(&mut program.functions[0].body, &[]);
                if succeeds { result.unwrap(); }
                else {
                    let error = result.unwrap_err();
                    assert_eq!(error.code, "unassigned");
                    assert_eq!(error.span.start, source.find("console.log(x)").unwrap() + "console.log(".len());
                }
                drop(program);
            }
        }
    }

    #[test]
    fn deep_cleanup_reentry_reuses_each_boolean_input_without_rewalking_descendants() {
        let depth = 16_384;
        let source = format!("function main(){{let x:int;{}x=2;{}}}", "try{if(true){x=1;return;}else{throw 1;}}finally{".repeat(depth), "}".repeat(depth));
        let program = crate::syntax::parse_unbounded(&source).unwrap();
        let result = run(&program.functions[0].body[1..], Variable { name: "x", binding: None, captures: &[], named: None }, false).unwrap();
        assert_eq!(result.normal, None);
        assert_eq!(result.exits.len(), 2);
        assert!(result.exits.iter().all(|exit| exit.assigned));
        drop(program);
    }

    #[test]
    fn cleanup_keeps_distinct_pending_exits_and_replaces_them_only_when_abrupt() {
        let source = "function main(){let x:int;try{if(true){x=1;return;}else{throw 1;}}finally{x=2;}}";
        let program = crate::syntax::parse_unbounded(source).unwrap();
        let variable = Variable { name: "x", binding: None, captures: &[], named: None };
        let result = run(&program.functions[0].body[1..], variable, false).unwrap();
        assert_eq!(result.normal, None);
        assert_eq!(result.exits.iter().map(|exit| (exit.kind, exit.assigned)).collect::<Vec<_>>(), [(CompletionKind::Return, true), (CompletionKind::Throw, true)]);
        let source = "function main(){let x:int;try{if(true){x=1;return;}else{throw 1;}}finally{return;}}";
        let program = crate::syntax::parse_unbounded(source).unwrap();
        let result = run(&program.functions[0].body[1..], variable, false).unwrap();
        assert_eq!(result.exits.len(), 1);
        assert_eq!(result.exits[0].kind, CompletionKind::Return);
        assert!(!result.exits[0].assigned);
    }
}
