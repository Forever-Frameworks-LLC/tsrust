//! Source-level abrupt completion identities, independent of native execution.
//! Payload types and ownership state are carried by the checking/lowering phase.
use crate::syntax::ControlTarget;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CompletionKind {
    Return,
    Throw,
    Break(Option<ControlTarget>),
    Continue(Option<ControlTarget>),
}
impl CompletionKind {
    pub(crate) fn targets(self, target:ControlTarget)->bool {
        matches!(self,Self::Break(Some(actual))|Self::Continue(Some(actual)) if actual==target)
    }
    pub(crate) fn is_continue(self)->bool {matches!(self,Self::Continue(_))}
}

/// A conservative source proof that cleanup cannot preserve a pending exit.
/// Loops and labeled blocks remain unproven: their targeted exits can reach
/// normal fallthrough. This proof does not depend on native unwinding.
#[inline(never)]
pub(crate) fn definitely_abrupt(body:&[crate::syntax::Stmt])->bool {
    use crate::syntax::{Stmt,StmtKind};
    enum Work<'a> { Body(&'a [Stmt]), Statement(&'a Stmt), Any(usize), All(usize) }
    let mut pending=vec![Work::Body(body)];
    let mut results=Vec::new();
    while let Some(work)=pending.pop() {
        match work {
            Work::Body(body)=>{
                pending.push(Work::Any(body.len()));
                pending.extend(body.iter().rev().map(Work::Statement));
            }
            Work::Any(count)=>{
                let start=results.len()-count;
                let found=results[start..].iter().any(|value|*value);
                results.truncate(start);results.push(found);
            }
            Work::All(count)=>{
                let start=results.len()-count;
                let found=results[start..].iter().all(|value|*value);
                results.truncate(start);results.push(found);
            }
            Work::Statement(statement)=>match &statement.kind {
                StmtKind::Return(_)|StmtKind::Throw(_)|StmtKind::Break|StmtKind::Continue=>results.push(true),
                StmtKind::Block(body) if statement.labels.is_empty()=>pending.push(Work::Body(body)),
                StmtKind::If{then_body,else_body,..}=>pending.extend([Work::All(2),Work::Body(else_body),Work::Body(then_body)]),
                StmtKind::Try(region)=>{
                    pending.push(Work::Any(1+usize::from(region.finally.is_some())));
                    pending.push(Work::All(1+usize::from(region.catch.is_some())));
                    if let Some(handler)=&region.catch {pending.push(Work::Body(&handler.body));}
                    pending.push(Work::Body(&region.body));
                    if let Some(body)=&region.finally {pending.push(Work::Body(body));}
                }
                _=>results.push(false),
            }
        }
    }
    results.pop().expect("root completion proof")
}
