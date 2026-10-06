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
pub(crate) fn definitely_abrupt(body:&[crate::syntax::Stmt],depth:usize)->bool {
    use crate::syntax::StmtKind;
    if depth>=128 {return false;}
    for statement in body {
        let abrupt=match &statement.kind {
            StmtKind::Return(_)|StmtKind::Throw(_)|StmtKind::Break|StmtKind::Continue=>true,
            StmtKind::Block(body) if statement.labels.is_empty()=>definitely_abrupt(body,depth+1),
            StmtKind::If{then_body,else_body,..}=>definitely_abrupt(then_body,depth+1) && definitely_abrupt(else_body,depth+1),
            StmtKind::Try(region)=>{
                region.finally.as_ref().is_some_and(|body|definitely_abrupt(body,depth+1)) ||
                (definitely_abrupt(&region.body,depth+1) && region.catch.as_ref().is_none_or(|handler|definitely_abrupt(&handler.body,depth+1)))
            }
            _=>false,
        };
        if abrupt {return true;}
    }
    false
}
