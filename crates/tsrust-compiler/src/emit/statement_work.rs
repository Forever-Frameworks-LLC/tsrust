//! Shared statement continuations; child emission returns to one heap scheduler.
use super::*;
use std::rc::Rc;

pub(super) enum Work<'a, 's> {
    Body(&'a [Stmt], Rc<Vec<&'a Stmt>>, usize, Option<usize>),
    Statement(&'a Stmt, &'a [Stmt], Rc<Vec<&'a Stmt>>, usize, Option<usize>),
    Text(String),
    Scope(exception_contract_emit::ControlScope<'s>),
    StorageScope(native_storage_availability::Scope<'s>),
    Storage(storage_statement_work::StorageWork<'a>),
    Iteration(map_iteration_emit::IterationWork<'a>),
    Post(&'a Stmt, usize),
    BlockReturn(&'a Stmt, Vec<String>),
    LoopTail(&'a Stmt, usize),
    LoopTest(&'a Expr, usize),
}
