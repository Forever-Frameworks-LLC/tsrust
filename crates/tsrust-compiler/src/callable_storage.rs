//! Callable family identity distinguishes owning storage from scoped borrowed sums.
use crate::analysis::CheckedClosure;
use crate::callable_contracts::{ContractId, Contracts};
use crate::syntax::CallbackKind;
use crate::{Diagnostic, Mode, Span};

/// Owned/shared construction proves structural storage eligibility. Borrowed
/// identities additionally retain concrete alternatives and never certify ownership.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum StorageContract { Owned(ContractId), SharedRead(ContractId), Borrowed(ContractId, Vec<usize>) }

impl StorageContract {
    pub(crate) fn checked(signatures: &Contracts, signature: ContractId) -> Option<Self> {
        signatures
            .get(signature)
            .kind
            .filter(|kind| matches!(kind, CallbackKind::Owned | CallbackKind::OwnedOnce))
            .map(|_| Self::Owned(signature))
    }

    /// Shared handles own the environment but expose only readonly invocation.
    /// Ordinary borrowed Read callbacks are not admitted by checked().
    pub(crate) fn checked_shared(signatures: &Contracts, signature: ContractId) -> Option<Self> {
        (signatures.get(signature).kind == Some(CallbackKind::SharedRead)).then_some(Self::SharedRead(signature))
    }

    pub(crate) fn environment<'a>(
        &self,
        signatures: &Contracts,
        closure: &'a CheckedClosure,
        span: Span,
    ) -> Result<OwnedEnvironment<'a>, Diagnostic> {
        if matches!(self,Self::Borrowed(..)) {
            return Err(Diagnostic::new("borrowed-callable-layout","A scoped borrowed family cannot certify owned environment storage.",span));
        }
        if self.invocation(signatures) != Mode::Consume && closure.invocation == Mode::Consume {
            return Err(Diagnostic::new(
                "callback-once",
                "A reusable callback cannot consume its captures when invoked.",
                span,
            )
            .related(closure.span, "Single-use closure is created here."));
        }
        if closure.borrowed {
            return Err(Diagnostic::new(
                "callback-escape",
                "An owned callback requires an environment without borrowed captures.",
                span,
            )
            .related(closure.span, "Borrowed environment is created here."));
        }
        if matches!(self, Self::SharedRead(_)) && closure.invocation != Mode::Read {
            return Err(Diagnostic::new("callback-access", "Shared callback storage requires readonly invocation; mutable captures cannot be shared.", span)
                .related(closure.span, "Mutable environment is created here."));
        }
        Ok(OwnedEnvironment(closure))
    }

    pub(crate) fn invocation(&self, signatures: &Contracts) -> Mode {
        let Self::Owned(signature) = self else { return Mode::Read; };
        match signatures.get(*signature).kind {
            Some(CallbackKind::Owned) => Mode::Mutate,
            Some(CallbackKind::OwnedOnce) => Mode::Consume,
            _ => unreachable!("checked owned storage contract"),
        }
    }
}

/// A checked environment reference; no native capture value is copied or shared.
pub(crate) struct OwnedEnvironment<'a>(&'a CheckedClosure);
impl<'a> OwnedEnvironment<'a> {
    pub(crate) fn closure(self) -> &'a CheckedClosure {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::Type;

    #[test]
    fn borrowed_and_unresolved_signatures_cannot_be_storage_contracts() {
        let mut signatures = Contracts::default();
        for kind in [None, Some(CallbackKind::Reusable), Some(CallbackKind::Once)] {
            let id = signatures.intern([], Type::Int, kind);
            assert!(StorageContract::checked(&signatures, id).is_none());
        }
    }

    #[test]
    fn identity_retains_invocation_access_and_nominal_result() {
        let mut signatures = Contracts::default();
        for (kind, invocation) in [
            (CallbackKind::Owned, Mode::Mutate),
            (CallbackKind::OwnedOnce, Mode::Consume),
        ] {
            let id =
                signatures.intern([(Type::Record(3), Mode::Read)], Type::Record(7), Some(kind));
            let storage = StorageContract::checked(&signatures, id).unwrap();
            assert_eq!(storage.invocation(&signatures), invocation);
            let equal =
                signatures.intern([(Type::Record(3), Mode::Read)], Type::Record(7), Some(kind));
            assert_eq!(Some(storage.clone()), StorageContract::checked(&signatures, equal));
            for (parameter, access, result) in [
                (Type::Record(4), Mode::Read, Type::Record(7)),
                (Type::Record(3), Mode::Consume, Type::Record(7)),
                (Type::Record(3), Mode::Read, Type::Record(8)),
            ] {
                let other = signatures.intern([(parameter, access)], result, Some(kind));
                assert_ne!(Some(storage.clone()), StorageContract::checked(&signatures, other));
            }
        }
    }
}

#[cfg(test)]
#[test]
fn checked_source_paths_share_storage_identity_without_merging_owners() {
    let checked = crate::check("function first(action: Function<int>): int { return action(); } function second(other: Function<int>): int { return other(); } function main() { console.log(first((): int => { return 1; })); console.log(second((): int => { return 2; })); }").unwrap_or_else(|error| panic!("{error:?}"));
    let callbacks = &checked.program.callback_contracts;
    assert!(callbacks.len() >= 2);
    assert_eq!(callbacks[0].parameter_names, callbacks[1].parameter_names);
    let shapes = &checked.program.callable_shapes;
    let first = StorageContract::checked(shapes, callbacks[0].structural).unwrap();
    let second = StorageContract::checked(shapes, callbacks[1].structural).unwrap();
    assert_eq!(first, second);
    assert_ne!(
        checked.program.functions[0].bindings[0].ty,
        checked.program.functions[1].bindings[0].ty
    );
}

#[cfg(test)]
#[test]
fn environment_admission_preserves_origin_and_rejects_borrowed_or_consuming_reuse() {
    use crate::syntax::Type;
    let owned = crate::check("function main() { const text = \"Pat\"; const action: Once<Function<string>> = (): string => { return text; }; console.log(action()); }").unwrap_or_else(|error| panic!("{error:?}"));
    let borrowed = crate::check("function main() { const text = \"Pat\"; const action = (): int => { return text.length; }; console.log(action()); }").unwrap_or_else(|error| panic!("{error:?}"));
    let mut signatures = Contracts::default();
    let reusable_id = signatures.intern([], Type::String, Some(CallbackKind::Owned));
    let once_id = signatures.intern([], Type::String, Some(CallbackKind::OwnedOnce));
    let reusable = StorageContract::checked(&signatures, reusable_id).unwrap();
    let once = StorageContract::checked(&signatures, once_id).unwrap();
    let origin = &owned.program.closures[0];
    let admitted = once
        .environment(&signatures, origin, Span::new(1, 2))
        .unwrap()
        .closure();
    assert!(std::ptr::eq(origin, admitted));
    for (contract, closure, code) in [
        (reusable.clone(), origin, "callback-once"),
        (reusable, &borrowed.program.closures[0], "callback-escape"),
        (once, &borrowed.program.closures[0], "callback-escape"),
    ] {
        let span = Span::new(3, 8);
        let error = contract
            .environment(&signatures, closure, span)
            .err()
            .expect("ineligible environment");
        assert_eq!(error.code, code);
        assert_eq!(error.span, span);
        assert_eq!(error.related[0].0, closure.span);
    }
}

/// Canonical storage families retain a representative source signature separately
/// from their structural identity. Capture instances are never stored here.
#[derive(Default)]
pub(crate) struct Registry {
    identities: std::collections::HashMap<StorageContract, usize>,
    declarations: Vec<usize>,
    origins: Vec<Vec<usize>>,
    borrowed: std::collections::HashSet<usize>,
    nested_results: std::collections::HashSet<(usize,usize)>,
}
impl Registry {
    pub(crate) fn note_nested_result(&mut self, actual:usize, expected:usize) {
        self.nested_results.insert((actual.min(expected),actual.max(expected)));
    }
    pub(crate) fn equivalent_result(&self,actual:crate::syntax::Type,expected:crate::syntax::Type)->bool {
        if actual==expected {return true;}
        let (crate::syntax::Type::Closure(a),crate::syntax::Type::Closure(b))=(actual,expected) else {return false;};
        self.nested_results.contains(&(a.min(b),a.max(b)))
    }
    pub(crate) fn intern(
        &mut self,
        signatures: &Contracts,
        callback: &crate::analysis::CallbackContract,
        declaration: usize,
    ) -> Option<crate::syntax::Type> {
        let contract = StorageContract::checked(signatures, callback.structural)?;
        Some(self.intern_contract(contract, declaration))
    }
    #[allow(dead_code)] // Checked compiler bridge integration follows storage acceptance.
    pub(crate) fn intern_shared(
        &mut self,
        signatures: &Contracts,
        callback: &crate::analysis::CallbackContract,
        declaration: usize,
    ) -> Option<crate::syntax::Type> {
        let contract = StorageContract::checked_shared(signatures, callback.structural)?;
        Some(self.intern_contract(contract, declaration))
    }
    pub(crate) fn intern_borrowed(&mut self,signature:ContractId,declaration:usize,origins:impl IntoIterator<Item=usize>)->usize {
        let mut origins=origins.into_iter().collect::<Vec<_>>();origins.sort();origins.dedup();
        let crate::syntax::Type::StoredCallable(id)=self.intern_contract(StorageContract::Borrowed(signature,origins.clone()),declaration) else {unreachable!()};
        self.origins[id]=origins.to_vec();self.borrowed.insert(id);id
    }
    pub(crate) fn is_borrowed(&self,id:usize)->bool {
        self.borrowed.contains(&id)
    }
    fn intern_contract(&mut self, contract: StorageContract, declaration: usize) -> crate::syntax::Type {
        let id = if let Some(&id) = self.identities.get(&contract) {
            id
        } else {
            let id = self.declarations.len();
            self.declarations.push(declaration);
            self.origins.push(Vec::new());
            self.identities.insert(contract, id);
            id
        };
        crate::syntax::Type::StoredCallable(id)
    }
    pub(crate) fn register_origin(
        &mut self,
        family: usize,
        origin: usize,
        callbacks: &[crate::analysis::CallbackContract],
        signatures: &Contracts,
        closures: &[CheckedClosure],
        span: Span,
    ) -> Result<(), Diagnostic> {
        let declaration = self
            .declaration(family)
            .and_then(|id| callbacks.get(id))
            .ok_or_else(|| {
                Diagnostic::new("callable-layout", "Unknown callable storage family.", span)
            })?;
        if self.is_borrowed(family) {
            let origins=self.origins(family).expect("known borrowed family");
            if !origins.contains(&origin) || origins.len()<2 {
                return Err(Diagnostic::new("borrowed-callable-layout","A borrowed family must retain its checked environment origins.",span));
            }
            for &other in &origins[1..]{crate::borrowed_callable_contracts::Family::checked(closures,[origins[0],other],span)?;}
            return Ok(());
        }
        crate::callable_dispatch::Family::checked_results(declaration,signatures,[origin],closures,span,|actual,expected|self.equivalent_result(actual,expected))?;
        let origins = &mut self.origins[family];
        if let Err(index) = origins.binary_search(&origin) {
            origins.insert(index, origin);
        }
        Ok(())
    }

    /// Records an origin for error-effect lookup in a speculative access
    /// pass. Layout is not validated: access modes are not final there, and
    /// the speculative tree is discarded. The definitive pass uses
    /// `register_origin`.
    #[inline(never)]
    pub(crate) fn note_speculative_origin(&mut self, family: usize, origin: usize) {
        let Some(origins) = self.origins.get_mut(family) else { return };
        if let Err(index) = origins.binary_search(&origin) {
            origins.insert(index, origin);
        }
    }

    pub(crate) fn singleton_origin(&self, family: usize, origin: usize) -> Option<bool> {
        let origins = self.origins.get(family)?;
        origins.binary_search(&origin).ok()?;
        Some(origins.len() == 1)
    }

    pub(crate) fn origins(&self, family: usize) -> Option<&[usize]> {
        self.origins.get(family).map(Vec::as_slice)
    }

    /// Keep family identities stable when definitive checking compacts closures.
    /// Validate the whole mapping before changing any registered origin set.
    pub(crate) fn remap_origins(&mut self, ids: &[Option<usize>]) -> Result<(), Diagnostic> {
        if self.origins.iter().all(Vec::is_empty) { return Ok(()); }
        let count = ids.iter().filter(|id| id.is_some()).count();
        let mut seen = vec![false; count];
        for id in ids.iter().flatten().copied() {
            if id >= count || std::mem::replace(&mut seen[id], true) {
                return Err(Diagnostic::new("callable-layout", "Callable environment compaction requires distinct retained origin IDs.", Span::default()));
            }
        }
        for origin in self.origins.iter().flatten() {
            if ids.get(*origin).copied().flatten().is_none() {
                return Err(Diagnostic::new("callable-layout", "Callable storage retains an unresolved environment origin.", Span::default()));
            }
        }
        let nested_results=self.nested_results.iter().map(|&(a,b)|{
            let a=ids.get(a).copied().flatten().ok_or_else(||Diagnostic::new("callable-layout","Nested result lost its environment origin.",Span::default()))?;
            let b=ids.get(b).copied().flatten().ok_or_else(||Diagnostic::new("callable-layout","Nested result lost its environment origin.",Span::default()))?;
            Ok((a.min(b),a.max(b)))
        }).collect::<Result<std::collections::HashSet<_>,Diagnostic>>()?;
        self.nested_results=nested_results;
        self.identities=std::mem::take(&mut self.identities).into_iter().map(|(contract,id)|{
            let contract=if let StorageContract::Borrowed(signature,origins)=contract {
                let mut origins=origins.into_iter().map(|origin|ids[origin].expect("validated borrowed origin")).collect::<Vec<_>>();origins.sort();
                StorageContract::Borrowed(signature,origins)
            }else{contract};(contract,id)
        }).collect();
        for origins in &mut self.origins {
            for origin in origins.iter_mut() {
                *origin = ids[*origin].expect("validated retained storage origin");
            }
            origins.sort_unstable();
        }
        Ok(())
    }

    pub(crate) fn populated(&self) -> impl Iterator<Item = (usize, usize, &[usize])> {
        self.families().filter(|(_, _, origins)| !origins.is_empty())
    }

    pub(crate) fn families(&self) -> impl Iterator<Item = (usize, usize, &[usize])> {
        self.origins
            .iter()
            .enumerate()
            .map(|(id, origins)| (id, self.declarations[id], origins.as_slice()))
    }

    /// Resolve callback protocol metadata without changing storage ownership.
    pub(crate) fn callback_declaration(&self, ty: crate::syntax::Type) -> Option<usize> {
        match ty {
            crate::syntax::Type::Callback(id) => Some(id),
            crate::syntax::Type::StoredCallable(id) | crate::syntax::Type::BorrowedCallable(id) => self.declaration(id),
            _ => None,
        }
    }

    pub(crate) fn declaration(&self, family: usize) -> Option<usize> {
        self.declarations.get(family).copied()
    }
}

#[cfg(test)]
#[test]
fn family_origin_compaction_preserves_identity_and_commits_only_valid_maps() {
    let mut registry = Registry::default();
    registry.declarations = vec![7, 9, 11];
    registry.origins = vec![vec![1, 3], Vec::new(), vec![3]];
    registry.note_nested_result(1,3);
    let original = registry.origins.clone();
    for invalid in [
        vec![None, Some(0), None, None],
        vec![None, Some(0)],
        vec![None, Some(0), None, Some(0)],
        vec![None, Some(0), None, Some(8)],
    ] {
        assert_eq!(registry.remap_origins(&invalid).unwrap_err().code, "callable-layout");
        assert_eq!(registry.origins, original);
        assert!(registry.equivalent_result(crate::syntax::Type::Closure(1),crate::syntax::Type::Closure(3)));
        assert_eq!(registry.declarations, [7, 9, 11]);
    }
    registry.remap_origins(&[None, Some(1), None, Some(0)]).unwrap();
    assert!(registry.equivalent_result(crate::syntax::Type::Closure(0),crate::syntax::Type::Closure(1)));
    assert!(!registry.equivalent_result(crate::syntax::Type::Closure(0),crate::syntax::Type::Closure(3)));
    assert_eq!(registry.origins, [vec![0, 1], Vec::new(), vec![0]]);
    assert_eq!(registry.declaration(0), Some(7));
    assert_eq!(registry.declaration(1), Some(9));
    assert_eq!(registry.declaration(2), Some(11));
    assert_eq!(registry.singleton_origin(0, 1), Some(false));
    assert_eq!(registry.singleton_origin(2, 0), Some(true));
}

#[cfg(test)]
#[test]
fn origin_registration_is_sorted_deduplicated_and_commits_only_valid_candidates() {
    let mut checked = crate::check("function left(start: int): () => int { return (): int => { return start; }; } function right(start: int): () => int { return (): int => { return start + 1; }; } function sink(action: Function<int>): int { return action(); } function main() {}").unwrap_or_else(|error| panic!("{error:?}"));
    let program = &mut checked.program;
    let ids = program.functions[..2]
        .iter()
        .map(|function| match function.result {
            crate::syntax::Type::Closure(id) => id,
            _ => panic!("concrete factory"),
        })
        .collect::<Vec<_>>();
    assert_eq!(program.stored_contracts.populated().count(), 0);
    for &origin in &[ids[1], ids[0], ids[1]] {
        program
            .stored_contracts
            .register_origin(
                0,
                origin,
                &program.callback_contracts,
                &program.callable_shapes,
                &program.closures,
                Span::default(),
            )
            .unwrap();
    }
    let before = program.stored_contracts.origins.clone();
    let mut expected = ids.clone();
    expected.sort_unstable();
    assert_eq!(before[0], expected);
    let span = Span::new(3, 9);
    for (family, origin) in [(99, ids[0]), (0, program.closures.len())] {
        let error = program
            .stored_contracts
            .register_origin(
                family,
                origin,
                &program.callback_contracts,
                &program.callable_shapes,
                &program.closures,
                span,
            )
            .unwrap_err();
        assert_eq!(error.code, "callable-layout");
        assert_eq!(error.span, span);
        assert_eq!(program.stored_contracts.origins, before);
    }
    program.closures[ids[0]].borrowed = true;
    assert_eq!(
        program
            .stored_contracts
            .register_origin(
                0,
                ids[0],
                &program.callback_contracts,
                &program.callable_shapes,
                &program.closures,
                span
            )
            .unwrap_err()
            .code,
        "callback-escape"
    );
    assert_eq!(program.stored_contracts.origins, before);
}

#[cfg(test)]
#[test]
fn shared_storage_admits_owned_read_environments_and_rejects_mutation_consumption_and_borrows() {
    use crate::syntax::Type;
    let owned=crate::check("function reader(actual:int[]):Function<int>{return():int=>{return actual.length;};}function main(){}").unwrap();
    let mutable=crate::check("function writer(actual:int[]):Function<void>{return():void=>{actual.push(1);};}function main(){}").unwrap();
    let consuming=crate::check("function take(actual:string):Once<Function<string>>{return():string=>{return actual;};}function main(){}").unwrap();
    let borrowed=crate::check("function main(){const values=[1];const reader=():int=>{return values.length;};console.log(reader());}").unwrap();
    let mut signatures=Contracts::default();
    let read=signatures.intern([],Type::Int,Some(CallbackKind::SharedRead));
    assert!(StorageContract::checked(&signatures,read).is_none());
    let shared=StorageContract::checked_shared(&signatures,read).unwrap();
    assert_eq!(shared.invocation(&signatures),Mode::Read);
    assert!(std::ptr::eq(shared.environment(&signatures,&owned.program.closures[0],Span::default()).unwrap().closure(),&owned.program.closures[0]));
    for (closure,code) in [(&mutable.program.closures[0],"callback-access"),(&consuming.program.closures[0],"callback-once"),(&borrowed.program.closures[0],"callback-escape")] {
        assert_eq!(shared.environment(&signatures,closure,Span::default()).err().unwrap().code,code);
    }
    for kind in [None,Some(CallbackKind::Read),Some(CallbackKind::Reusable),Some(CallbackKind::Once),Some(CallbackKind::Owned),Some(CallbackKind::OwnedOnce)] {
        let id=signatures.intern([],Type::Int,kind);assert!(StorageContract::checked_shared(&signatures,id).is_none());
    }
}

#[cfg(test)]
#[test]
fn shared_registry_preserves_contract_identity_and_rejects_invalid_origins_atomically() {
    use crate::syntax::Type;
    let checked=crate::check("function reader(actual:int[]):Function<int>{return():int=>{return actual.length;};}function writer(actual:int[]):Function<int>{return():int=>{actual.push(1);return actual.length;};}function main(){const values=[1];const local=():int=>{return values.length;};console.log(local());}").unwrap();
    let mut signatures=Contracts::default();
    let callbacks=[
        crate::analysis::CallbackContract{structural:signatures.intern([],Type::Int,Some(CallbackKind::SharedRead)),parameter_names:vec![],params:vec![],result:Type::Int,kind:CallbackKind::SharedRead},
        crate::analysis::CallbackContract{structural:signatures.intern([],Type::Int,Some(CallbackKind::Owned)),parameter_names:vec![],params:vec![],result:Type::Int,kind:CallbackKind::Owned},
    ];
    let mut registry=Registry::default();
    assert!(registry.intern(&signatures,&callbacks[0],0).is_none());
    let Type::StoredCallable(shared)=registry.intern_shared(&signatures,&callbacks[0],0).unwrap() else{unreachable!()};
    let Type::StoredCallable(owned)=registry.intern(&signatures,&callbacks[1],1).unwrap() else{unreachable!()};
    assert_ne!(shared,owned);
    assert_eq!(registry.intern_shared(&signatures,&callbacks[0],1),Some(Type::StoredCallable(shared)));
    assert_eq!(registry.declaration(shared),Some(0));
    registry.register_origin(shared,0,&callbacks,&signatures,&checked.program.closures,Span::default()).unwrap();
    for (origin,code) in [(1,"callback-access"),(2,"callback-escape"),(100,"callable-layout")] {
        assert_eq!(registry.register_origin(shared,origin,&callbacks,&signatures,&checked.program.closures,Span::default()).unwrap_err().code,code);
        assert_eq!(registry.origins(shared),Some([0].as_slice()));
    }
    registry.register_origin(shared,0,&callbacks,&signatures,&checked.program.closures,Span::default()).unwrap();
    assert_eq!(registry.origins(shared),Some([0].as_slice()));
}

#[cfg(test)]
#[test]
fn borrowed_family_identity_retains_distinct_origins_after_compaction() {
    let mut shapes=Contracts::default();
    let signature=shapes.intern([],crate::syntax::Type::Int,Some(CallbackKind::Read));
    let mut registry=Registry::default();
    let first=registry.intern_borrowed(signature,0,[1,3]);
    assert_eq!(registry.intern_borrowed(signature,1,[3,1]),first);
    let second=registry.intern_borrowed(signature,0,[0,3]);
    assert_ne!(first,second);assert!(registry.is_borrowed(first));
    registry.remap_origins(&[Some(2),Some(1),None,Some(0)]).unwrap();
    assert_eq!(registry.origins(first),Some(&[0,1][..]));
    assert_eq!(registry.intern_borrowed(signature,2,[1,0]),first);
    assert_eq!(registry.intern_borrowed(signature,2,[2,0]),second);
    assert!(crate::syntax::ArrayElement::from_value_type(crate::syntax::Type::BorrowedCallable(first)).is_none());
    assert!(!crate::syntax::Type::BorrowedCallable(first).is_copy());
}
