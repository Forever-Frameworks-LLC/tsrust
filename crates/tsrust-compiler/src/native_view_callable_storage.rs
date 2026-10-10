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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Alternative {
    pub(crate) origin: usize,
    pub(crate) slots: Vec<Slot>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Family {
    pub(crate) signature: crate::callable_contracts::ContractId,
    pub(crate) invocation: Mode,
    pub(crate) alternatives: Vec<Alternative>,
}

pub(crate) fn family_slots(
    contract: &crate::analysis::CallbackContract,
    signatures: &crate::callable_contracts::Contracts,
    origins: impl IntoIterator<Item=usize>,
    closures: &[CheckedClosure],
    span: crate::Span,
) -> Result<Option<Family>,crate::Diagnostic> {
    let Some(storage)=crate::callable_storage::StorageContract::checked(signatures,contract.structural) else {return Ok(None);};
    let invocation=storage.invocation(signatures);
    let mut origins=origins.into_iter().collect::<Vec<_>>();
    origins.sort_unstable();origins.dedup();
    let mut alternatives=Vec::with_capacity(origins.len());
    let mut carries_view=false;
    for origin in origins {
        let closure=closures.get(origin).ok_or_else(||crate::Diagnostic::new("callable-layout","Unknown callable environment origin.",span))?;
        if invocation!=Mode::Consume && closure.invocation==Mode::Consume {
            return Err(crate::Diagnostic::new("callback-once","A reusable callback cannot consume its captures when invoked.",span)
                .related(closure.span,"Single-use closure is created here."));
        }
        if closure.polymorphic || closure.function.template {return Ok(None);}
        if closure.parameters.len()!=contract.params.len() || closure.function.result!=contract.result {return Ok(None);}
        for (&binding,expected) in closure.parameters.iter().zip(&contract.params) {
            let actual=&closure.function.bindings[binding];
            if actual.input_ty.unwrap_or(actual.ty)!=expected.ty || actual.mode!=expected.mode {return Ok(None);}
        }
        let slots=match carried_slots(closure) {
            Some(slots)=>{carries_view=true;slots},
            None if !closure.borrowed && closure.lifetime_count==0=>Vec::new(),
            None=>return Ok(None),
        };
        alternatives.push(Alternative{origin,slots});
    }
    Ok(carries_view.then_some(Family{signature:contract.structural,invocation,alternatives}))
}

/// Check backend eligibility before exposing a scoped family to code generation.
/// Ownership admission alone does not prove invocation/error ABI support.
pub(crate) fn validate_abi(
    layout: &Family,
    contract: &crate::analysis::CallbackContract,
    closures: &[CheckedClosure],
    _effects: &crate::error_effects::ErrorEffects,
    span: crate::Span,
) -> Result<(), crate::Diagnostic> {
    let unsupported = |message: &str| crate::Diagnostic::new("native-view-family-emission", message, span);
    if layout.invocation != Mode::Consume || layout.alternatives.is_empty() || contract.structural != layout.signature {
        return Err(unsupported("This native family ABI requires a matching finite one-shot callback contract."));
    }
    for alternative in &layout.alternatives {
        let closure = closures.get(alternative.origin).ok_or_else(|| unsupported("Unknown native callable family environment."))?;
        if carried_slots(closure).unwrap_or_default() != alternative.slots {
            return Err(unsupported("Native callable family capture paths and access must match the checked environment."));
        }
        for loan in closure.function.native_returns.as_deref().into_iter().flatten() {
            if closure.parameters.contains(&loan.root) {
                return Err(unsupported("Native family results borrowing an invocation argument require argument lifetime planning."));
            }
            if matches!(loan.source, LifetimeSource::Storage) {
                return Err(unsupported("A consuming native family cannot return a borrow of destroyed capture storage."));
            }
        }
    }
    Ok(())
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

    #[test]
    fn native_view_owning_family_preserves_alternatives_and_once_invocation() {
        let source="type Window={};function first(view:Window){return ()=>view;}function second(view:Window){return ()=>view;}function main(){}";
        let formatted=crate::formatting::format(source).unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
        let mut parsed=crate::syntax::parse(&formatted).unwrap();
        parsed.records[0].native_type=Some(NativeTypeId::ByteView);
        let mut checked=crate::analysis::analyze(parsed).unwrap();
        let result=checked.closures[0].function.result;
        let contract_id=checked.callable_shapes.intern([],result,Some(crate::syntax::CallbackKind::OwnedOnce));
        let contract=crate::analysis::CallbackContract {
            structural:contract_id,parameter_names:Vec::new(),params:Vec::new(),result,kind:crate::syntax::CallbackKind::OwnedOnce,
        };
        let layout=family_slots(&contract,&checked.callable_shapes,[1,0,1],&checked.closures,crate::Span::default()).unwrap().unwrap();
        assert_eq!(layout.signature,contract_id);
        assert_eq!(layout.invocation,Mode::Consume);
        assert_eq!(layout.alternatives.iter().map(|alternative|alternative.origin).collect::<Vec<_>>(),vec![0,1]);
        assert!(layout.alternatives.iter().all(|alternative|alternative.slots.len()==1));
    }

    #[test]
    fn native_view_owning_family_rejects_reuse_unknown_origins_and_physical_borrows() {
        let mut checked=checked(false);
        let result=checked.closures[0].function.result;
        let once=checked.callable_shapes.intern([],result,Some(crate::syntax::CallbackKind::OwnedOnce));
        let mut contract=crate::analysis::CallbackContract {
            structural:once,parameter_names:Vec::new(),params:Vec::new(),result,kind:crate::syntax::CallbackKind::OwnedOnce,
        };
        let span=crate::Span::new(4,9);
        let error=family_slots(&contract,&checked.callable_shapes,[99],&checked.closures,span).unwrap_err();
        assert_eq!(error.code,"callable-layout");
        assert_eq!(error.span,span);
        let reusable=checked.callable_shapes.intern([],result,Some(crate::syntax::CallbackKind::Owned));
        contract.structural=reusable;contract.kind=crate::syntax::CallbackKind::Owned;
        let error=family_slots(&contract,&checked.callable_shapes,[0],&checked.closures,span).unwrap_err();
        assert_eq!(error.code,"callback-once");
        assert_eq!(error.related[0].0,checked.closures[0].span);
        contract.structural=once;contract.kind=crate::syntax::CallbackKind::OwnedOnce;
        checked.closures[0].captures[0].storage=Mode::Read;
        assert!(family_slots(&contract,&checked.callable_shapes,[0],&checked.closures,span).unwrap().is_none());
    }

}
