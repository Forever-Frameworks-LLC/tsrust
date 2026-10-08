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
