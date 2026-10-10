//! Candidate carried-view slots do not certify a lifetime-free owning family.
use crate::analysis::CheckedClosure;
use crate::analysis::native_view_loans::{LifetimeSource, Segment, VariantGuard};
use crate::native_return_borrows::BorrowAccess;
use crate::Mode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Slot {
    pub(crate) capture: usize,
    pub(crate) path: Vec<Segment>,
    pub(crate) guards: Vec<VariantGuard>,
    pub(crate) access: BorrowAccess,
}

pub(crate) fn carried_slots(closure: &CheckedClosure) -> Option<Vec<Slot>> {
    let mut slots = Vec::new();
    for capture in &closure.captures {
        if matches!(capture.storage, Mode::Read | Mode::Mutate) {
            return None;
        }
        for loan in closure.function.bindings[capture.inner]
            .native_view_loans.as_deref().into_iter().flatten()
        {
            if loan.root != capture.inner || matches!(loan.source, LifetimeSource::Storage) {
                return None;
            }
            let (path, guards) = match &loan.source {
                LifetimeSource::Carried(path) => (path.clone(), Vec::new()),
                LifetimeSource::VariantCarried { path, guards } => (path.clone(), guards.clone()),
                LifetimeSource::Storage => unreachable!("rejected physical storage"),
            };
            let slot = Slot { capture: capture.inner, path, guards, access: loan.access };
            if !slots.contains(&slot) { slots.push(slot); }
        }
    }
    (!slots.is_empty()).then_some(slots)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_signatures::NativeTypeId;

    fn checked(mutable: bool) -> crate::analysis::CheckedProgram {
        let source="type Window={};function save(view:Window){return ()=>view;}function main(){}";
        let formatted=crate::formatting::format(source).unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
        let mut parsed=crate::syntax::parse(&formatted).unwrap();
        parsed.records[0].native_type=Some(if mutable {NativeTypeId::ByteViewMut}else{NativeTypeId::ByteView});
        crate::analysis::analyze(parsed).unwrap()
    }

    #[test]
    fn native_view_owned_callable_slots_keep_access_and_capture_identity() {
        for mutable in [false,true] {
            let checked=checked(mutable);
            let closure=&checked.closures[0];
            let slots=carried_slots(closure).expect("owned carried views need lifetime slots");
            assert_eq!(slots.len(),1);
            assert_eq!(slots[0].capture,closure.captures[0].inner);
            assert!(slots[0].path.is_empty());
            assert!(slots[0].guards.is_empty());
            assert_eq!(slots[0].access,if mutable {BorrowAccess::Exclusive}else{BorrowAccess::ReadOnly});
            assert_eq!(closure.invocation,Mode::Consume);
        }
    }

    #[test]
    fn native_view_owned_callable_slots_reject_physical_reference_captures() {
        let mut checked=checked(false);
        checked.closures[0].captures[0].storage=Mode::Read;
        assert!(carried_slots(&checked.closures[0]).is_none());
        checked.closures[0].captures[0].storage=Mode::Mutate;
        assert!(carried_slots(&checked.closures[0]).is_none());
    }
    #[test]
    fn native_view_owned_callable_slots_keep_nested_fields_and_result_guards() {
        for (shape, guarded) in [("type Payload={first:Window;second:Window;};",false),
                                 ("type Payload=Result<Window,int>;",true)] {
            let source=format!("type Window={{}};{shape}function save(input:Payload){{return ()=>input;}}function main(){{}}");
            let formatted=crate::formatting::format(&source).unwrap();
            assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
            let mut parsed=crate::syntax::parse(&formatted).unwrap();
            parsed.records.iter_mut().find(|record|record.name.text=="Window").unwrap().native_type=Some(NativeTypeId::ByteView);
            let checked=crate::analysis::analyze(parsed).unwrap();
            let slots=carried_slots(&checked.closures[0]).unwrap();
            if guarded {
                assert_eq!(slots.len(),1);
                assert_eq!(slots[0].path,vec![Segment::Field("payload".into())]);
                assert_eq!(slots[0].guards.len(),1);
                assert!(slots[0].guards[0].path.is_empty());
            } else {
                assert_eq!(slots.len(),2);
                assert!(slots.iter().any(|slot|slot.path==vec![Segment::Field("first".into())]));
                assert!(slots.iter().any(|slot|slot.path==vec![Segment::Field("second".into())]));
                assert!(slots.iter().all(|slot|slot.guards.is_empty()));
            }
        }
    }

    #[test]
    fn native_view_owned_callable_metadata_does_not_bypass_storage_guard() {
        let checked=checked(false);
        let closure=&checked.closures[0];
        let mut signatures=crate::callable_contracts::Contracts::default();
        let id=signatures.intern([],closure.function.result,Some(crate::syntax::CallbackKind::OwnedOnce));
        let contract=crate::callable_storage::StorageContract::checked(&signatures,id).unwrap();
        let error=contract.environment(&signatures,closure,crate::Span::default()).err().expect("metadata does not certify lifetime-free storage");
        assert_eq!(error.code,"native-view-callable-storage");
        assert_eq!(error.related[0].0,closure.span);
        assert!(error.message.contains("not implemented"));
    }

    #[test]
    fn native_view_owned_callable_slots_reject_foreign_roots_and_storage_loans() {
        let mut checked=checked(false);
        let inner=checked.closures[0].captures[0].inner;
        let loans=checked.closures[0].function.bindings[inner].native_view_loans.as_mut().unwrap();
        loans[0].root=inner+1;
        assert!(carried_slots(&checked.closures[0]).is_none());
        let loans=checked.closures[0].function.bindings[inner].native_view_loans.as_mut().unwrap();
        loans[0].root=inner;
        loans[0].source=LifetimeSource::Storage;
        assert!(carried_slots(&checked.closures[0]).is_none());
    }

}
