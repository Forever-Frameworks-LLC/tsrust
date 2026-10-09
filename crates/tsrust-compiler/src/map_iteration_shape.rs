//! Eligibility of storage-carrying loop bodies, using resolved control targets.
use crate::syntax::{ControlTarget,Stmt,StmtKind};

pub(super) fn supports(body:&[Stmt],label:usize)->bool {
    let mut pending=body.iter().map(|statement|(statement,false)).collect::<Vec<_>>();
    let mut loops=std::collections::HashSet::from([label]);
    while let Some((statement,completion))=pending.pop() {
        match &statement.kind {
            StmtKind::Block(body) if statement.labels.is_empty()=>pending.extend(body.iter().map(|statement|(statement,completion))),
            StmtKind::Try(region)=>{
                pending.extend(region.statements().map(|statement|(statement,true)));
            },
            StmtKind::If{then_body,else_body,..}=>pending.extend(then_body.iter().chain(else_body).map(|statement|(statement,completion))),
            StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>{
                loops.insert(statement.span.start);
                pending.extend(body.iter().map(|statement|(statement,completion)));
            },
            StmtKind::Loop{body,update,tail_reachable,..}=>{
                loops.insert(statement.span.start);
                pending.extend(body.iter().map(|statement|(statement,completion)));
                if *tail_reachable {pending.extend(update.iter().map(|statement|(statement,completion)));}
            },
            StmtKind::Let{..}|StmtKind::Expr(_)|StmtKind::Assign{..}|StmtKind::Return(_)|StmtKind::Throw(_)=>{},
            StmtKind::Break|StmtKind::Continue=>{
                let Some(ControlTarget::Loop(target)|ControlTarget::Tail(target))=statement.control_target else{return false;};
                // Nonlocal controls inside try leave through its completion
                // transport, so every enclosing finally runs before dispatch.
                if !completion && !loops.contains(&target){return false;}
            },
            _=>return false,
        }
    }
    true
}
