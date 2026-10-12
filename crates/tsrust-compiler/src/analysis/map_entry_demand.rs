//! Conservative later-use proof for native Map environment narrowing.
use super::*;

#[derive(Default)]
pub(super) struct Demand {
    last_use: HashMap<usize,usize>,
    opaque_call_end: usize,
}
impl Demand {
    pub(super) fn checked(body:&mut [Stmt]) -> Result<Self,Diagnostic> {
        let mut demand=Self::default();
        crate::source_walk::body(body,&mut |_|Ok(()),&mut |value| {
            if matches!(value.kind,ExprKind::Name(_)) && let Some(binding)=value.binding {
                demand.last_use.entry(binding).and_modify(|end|*end=(*end).max(value.span.end)).or_insert(value.span.end);
            }
            // Named calls can access lexical captures
            // without a source Name at the call site. Keep their boundaries.
            // Stored aggregate calls retain their environment through checked
            // loan owners; a released loan cannot remain in those environments.
            if matches!(value.kind,ExprKind::Call(_, _)) && matches!(value.target,Some(CallTarget::Function(_)|CallTarget::ForwardFunction(_))) {
                demand.opaque_call_end=demand.opaque_call_end.max(value.span.end);
            }
            Ok(())
        })?;
        Ok(demand)
    }
    pub(super) fn after(&self,end:usize,released:&HashSet<usize>) -> bool {
        self.opaque_call_end>end || released.iter().any(|binding|self.last_use.get(binding).is_some_and(|last|*last>end))
    }
}

/// Whether a loop can write, move or call through one of the released capture
/// owners. Indexed and field reads share a released loan; they never require a
/// narrower Map environment on the backedge.
pub(super) fn mutates(body:&[Stmt],released:&HashSet<usize>) -> Result<bool,Diagnostic> {
    fn root(mut value:&Expr)->&Expr {
        while let ExprKind::Field(base,_)|ExprKind::Index(base,_)=&value.kind {value=base;}
        value
    }
    let mut written=HashSet::new();
    let mut pending=body.iter().collect::<Vec<_>>();
    while let Some(statement)=pending.pop() {
        match &statement.kind {
            StmtKind::Assign{target,..}=>{written.insert(root(target).span.start);}
            StmtKind::Block(body)|StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>pending.extend(body),
            StmtKind::If{then_body,else_body,..}=>{pending.extend(then_body);pending.extend(else_body);}
            StmtKind::Loop{body,update,..}=>{pending.extend(body);pending.extend(update);}
            StmtKind::Try(region)=>pending.extend(region.statements()),
            // A lifted declaration can reach lexical captures without a source name.
            StmtKind::LocalFunction(_)=>return Ok(true),
            _=>{}
        }
    }
    let mut body=body.to_vec();
    let mut uses=Vec::new();
    let mut reads=HashSet::new();
    let mut opaque=false;
    crate::source_walk::body(&mut body,&mut |_|Ok(()),&mut |value| {
        match &value.kind {
            ExprKind::Name(_) if value.binding.is_some_and(|binding|released.contains(&binding))=>uses.push(value.span.start),
            ExprKind::Field(base,_)|ExprKind::Index(base,_) if matches!(base.kind,ExprKind::Name(_))=>{reads.insert(base.span.start);}
            ExprKind::Call(callee,_)=>{
                written.insert(root(callee).span.start);
                if matches!(value.target,Some(CallTarget::Function(_)|CallTarget::ForwardFunction(_))) {opaque=true;}
            }
            ExprKind::Assign{target,..}|ExprKind::CompoundAssign{target,..}|ExprKind::CoalesceAssign{target,..}|ExprKind::LogicalAssign{target,..}|ExprKind::Update{target,..}=>{written.insert(root(target).span.start);}
            _=>{}
        }
        Ok(())
    })?;
    Ok(opaque || uses.iter().any(|start|!reads.contains(start) || written.contains(start)))
}
