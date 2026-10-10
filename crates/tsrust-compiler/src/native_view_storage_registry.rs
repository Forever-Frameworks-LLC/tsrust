//! Scoped owning identity retains finite environments without certifying static storage.
use super::*;

impl Registry {
    pub(crate) fn scoped_native_family(&self,family:usize)->Option<&crate::native_view_callable_storage::Family> {
        if !self.scoped_native.contains(&family) {return None;}
        self.native_views.as_ref()?.get(family)
    }

    // Source construction retains a distinct lifetime-bearing family identity.
    pub(crate) fn intern_scoped_native(
        &mut self,
        signatures:&Contracts,
        callback:&crate::analysis::CallbackContract,
        declaration:usize,
        origins:impl IntoIterator<Item=usize>,
        closures:&[CheckedClosure],
        span:Span,
    )->Result<Option<crate::syntax::Type>,Diagnostic> {
        let Some(layout)=crate::native_view_callable_storage::family_slots(callback,signatures,origins,closures,span)? else {return Ok(None);};
        let origins=layout.alternatives.iter().map(|alternative|alternative.origin).collect::<Vec<_>>();
        let contract=StorageContract::ScopedOwned(callback.structural,origins.clone());
        let ty=self.intern_contract(contract,declaration);
        let crate::syntax::Type::StoredCallable(family)=ty else {unreachable!()};
        self.origins[family]=origins;
        self.scoped_native.insert(family);
        self.native_views.get_or_insert_with(Default::default).prepare(family,layout);
        Ok(Some(ty))
    }
}
