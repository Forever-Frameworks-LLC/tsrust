//! A reusable capture must be initialized at every observable completion.
use std::collections::BTreeSet;

/// Monotone proof over checked completion snapshots. Missing exits cannot prove
/// restoration, and one unavailable exit permanently rejects that capture.
#[derive(Default)]
pub(super) struct Proof {
    initialized: Option<BTreeSet<usize>>,
}
impl Proof {
    pub(super) fn observe(&mut self, slots: impl IntoIterator<Item=(usize,bool)>) {
        let initialized=slots.into_iter().filter_map(|(slot,initialized)|initialized.then_some(slot)).collect::<BTreeSet<_>>();
        if let Some(previous)=&mut self.initialized {previous.retain(|slot|initialized.contains(slot));}
        else {self.initialized=Some(initialized);}
    }
    pub(super) fn permits(&self, slot: usize) -> bool {
        self.initialized.as_ref().is_some_and(|slots|slots.contains(&slot))
    }
}
#[derive(Clone)]
pub(super) struct Tracker {
    slots: Vec<usize>,
    proof: std::rc::Rc<std::cell::RefCell<Proof>>,
}
impl Tracker {
    pub(super) fn new(slots: Vec<usize>) -> Self {Self {slots,proof:Default::default()}}
    pub(super) fn observe(&self,bindings:&[super::Binding]) {
        self.proof.borrow_mut().observe(self.slots.iter().map(|&slot| {
            let binding=&bindings[slot];
            (slot,binding.transferred.is_none() && binding.moved_fields.is_empty())
        }));
    }
    pub(super) fn permits(&self,slot:usize)->bool {self.proof.borrow().permits(slot)}
}
pub(super) fn record(context:&super::Context) {
    if let Some(tracker)=&context.capture_restoration {tracker.observe(&context.bindings);}
}
/// Refined implicit failure effects are not yet attached during capture checking.
/// Keep restoration conservative for operations that may leave storage empty.
pub(super) fn bounded_body(body:&mut [crate::syntax::Stmt])->bool {
    let mut bounded=true;
    crate::source_walk::body(body,&mut |_|Ok(()),&mut |expression| {
        if matches!(expression.kind,crate::syntax::ExprKind::Call(..)|crate::syntax::ExprKind::Index(..)|crate::syntax::ExprKind::Binary {..}|crate::syntax::ExprKind::Unary {..}) {bounded=false;}
        Ok(())
    }).expect("read-only checked body walk");
    bounded
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_and_failed_completions_cannot_certify_reusable_storage() {
        let mut proof=Proof::default();assert!(!proof.permits(1));
        proof.observe([(1,true),(2,true)]);assert!(proof.permits(1));assert!(!proof.permits(99));
        proof.observe([(1,false),(2,true)]);assert!(!proof.permits(1));assert!(proof.permits(2));
        proof.observe([(1,true),(2,true)]);assert!(!proof.permits(1),"a later restored exit cannot erase an earlier empty exit");
    }
    #[test]
    fn completion_order_does_not_change_capture_restoration() {
        let completions=[[(1,true),(2,false)],[(1,false),(2,true)],[(1,true),(2,true)]];
        let mut forward=Proof::default();for snapshot in completions {forward.observe(snapshot);}
        let mut reverse=Proof::default();for snapshot in completions.into_iter().rev() {reverse.observe(snapshot);}
        for slot in [1,2] {assert_eq!(forward.permits(slot),reverse.permits(slot));assert!(!forward.permits(slot));}
    }
    #[test]
    fn checked_source_proves_only_complete_capture_restoration() {
        for (body,expected) in [
            ("const previous=action;action=identity;return previous;",true),
            ("const previous=action;if(flag){action=identity;}return previous;",false),
            ("const previous=action;if(flag){throw 1;}action=identity;return previous;",false),
            ("const previous=action;if(flag){action=identity;}else{action=identity;}return previous;",true),
        ] {
            let source=format!("function identity<T>(value:T):T{{return value;}}function make(){{let action=identity;return (flag:boolean)=>{{{body}}};}}function main(){{const take=make();const saved=take(false);console.log(saved<int>(3));}}");
            let formatted=crate::format(&source).unwrap();assert_eq!(crate::format(&formatted).unwrap(),formatted);
            let checked=crate::check(&formatted).unwrap_or_else(|error|panic!("{}",error.render("input.tsr",&formatted)));
            let captures=checked.program.closures.iter().flat_map(|closure|closure.captures.iter().map(move |capture|(closure,capture))).filter(|(closure,capture)|closure.function.bindings[capture.inner].name=="action").collect::<Vec<_>>();
            assert!(!captures.is_empty(),"{body}");
            assert!(captures.iter().all(|(_,capture)|capture.restored==expected),"{body}: incorrect completion proof");
        }
    }

}
