//! Eligibility of storage-carrying loop bodies, using resolved control targets.
use crate::syntax::{ControlTarget,Stmt,StmtKind};

pub(super) fn supports(body:&[Stmt],_label:usize)->bool {
    let mut pending=body.iter().collect::<Vec<_>>();
    while let Some(statement)=pending.pop() {
        match &statement.kind {
            StmtKind::Block(body) if statement.labels.is_empty()=>pending.extend(body),
            StmtKind::Try(region)=>pending.extend(region.statements()),
            StmtKind::If{then_body,else_body,..}=>pending.extend(then_body.iter().chain(else_body)),
            StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>pending.extend(body),
            StmtKind::Loop{body,update,tail_reachable,..}=>{
                pending.extend(body);
                if *tail_reachable {pending.extend(update);}
            },
            StmtKind::Let{..}|StmtKind::Expr(_)|StmtKind::Assign{..}|StmtKind::Return(_)|StmtKind::Throw(_)=>{},
            // Checked loop targets may name an ancestor. Its carried body
            // receives the current owner directly; exited inner scopes never
            // read that owner again. Block targets need separate transport.
            StmtKind::Break|StmtKind::Continue if matches!(statement.control_target,Some(ControlTarget::Loop(_)|ControlTarget::Tail(_)))=>{},
            _=>return false,
        }
    }
    true
}

/// Whether a direct control leaves this loop for an enclosing loop.
pub(super) fn has_nonlocal_control(body:&[Stmt],label:usize)->bool {
    let mut pending=body.iter().collect::<Vec<_>>();
    while let Some(statement)=pending.pop(){match &statement.kind {
        StmtKind::Block(body)=>pending.extend(body),
        StmtKind::If{then_body,else_body,..}=>pending.extend(then_body.iter().chain(else_body)),
        StmtKind::Break|StmtKind::Continue if matches!(statement.control_target,Some(ControlTarget::Loop(target)|ControlTarget::Tail(target)) if target!=label)=>return true,
        _=>{},
    }}
    false
}
