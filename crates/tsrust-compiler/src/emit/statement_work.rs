//! Shared statement continuations; child emission returns to one heap scheduler.
use super::*;
use std::rc::Rc;

pub(super) enum Work<'a, 's> {
    Body(&'a [Stmt], Rc<Vec<&'a Stmt>>, usize, Option<usize>),
    Statement(&'a Stmt, &'a [Stmt], Rc<Vec<&'a Stmt>>, usize, Option<usize>),
    Text(String),
    Function(&'a CheckedFunction),
    RestoreFunction,
    Capture,
    Captured,
    Resume(Box<dyn FnOnce(&mut String,&mut Vec<Work<'a,'s>>,&mut Vec<String>)+'a>),
    Region(exception_contract_emit::RegionScope<'s>),
    Scope(exception_contract_emit::ControlScope<'s>),
    StorageScope(native_storage_availability::Scope<'s>),
    Storage(storage_statement_work::StorageWork<'a>),
    Iteration(map_iteration_emit::IterationWork<'a>),
    Post(&'a Stmt, usize),
    BlockReturn(&'a Stmt, Vec<String>),
    LoopTail(&'a Stmt, usize),
    LoopTest(&'a Expr, usize),
}
