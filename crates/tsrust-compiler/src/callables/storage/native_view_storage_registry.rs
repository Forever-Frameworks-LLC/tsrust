//! Scoped owning identity retains finite environments without certifying static storage.
use super::*;

impl Registry {
    pub(crate) fn scoped_native_families(&self)->impl Iterator<Item=(usize,&crate::native_view_callable_storage::Family)> {
        self.scoped_native.iter().filter_map(|&id|self.scoped_native_family(id).map(|family|(id,family)))
    }

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
    /// Recursive fields have one shared lifetime and one stable family identity.
    /// Each factory contributes alternatives without changing nonrecursive families.
    pub(crate) fn intern_recursive_scoped_native(
        &mut self,
        signatures: &Contracts,
        callbacks: &[crate::analysis::CallbackContract],
        source: usize,
        record: usize,
        field: usize,
        closures: &[CheckedClosure],
        span: Span,
    ) -> Result<crate::syntax::Type, Diagnostic> {
        let declaration = self.declaration(source).expect("checked scoped declaration");
        let callback = &callbacks[declaration];
        let contract = StorageContract::RecursiveScopedOwned(callback.structural, record, field);
        let mut origins = self.origins(source).expect("checked scoped origins").to_vec();
        if let Some(&existing) = self.identities.get(&contract) {
            origins.extend(self.origins[existing].iter().copied());
        }
        // Validate the combined layout before mutating the admitted identity.
        let layout = crate::native_view_callable_storage::family_slots(callback, signatures, origins, closures, span)?
            .ok_or_else(|| Diagnostic::new("native-view-callable-storage", "Recursive callback storage requires a checked view-carrying environment.", span))?;
        let ty = self.intern_contract(contract, declaration);
        let crate::syntax::Type::StoredCallable(family) = ty else { unreachable!() };
        self.origins[family] = layout.alternatives.iter().map(|alternative| alternative.origin).collect();
        self.scoped_native.insert(family);
        self.native_views.get_or_insert_with(Default::default).prepare(family, layout);
        Ok(ty)
    }

}

#[cfg(test)]
mod recursive_tests {
    use super::*;
    #[test]
    fn recursive_fields_union_origins_without_widening_independent_families() {
        let source = "type Window={};function first(view:Window){return ()=>view;}function second(view:Window){return ()=>view;}function main(){}";
        let mut parsed = crate::syntax::parse(source).unwrap();
        parsed.records[0].native_type = Some(crate::native_signatures::NativeTypeId::ByteView);
        let mut checked = crate::analysis::analyze(parsed).unwrap();
        let result = checked.closures[0].function.result;
        let structural = checked.callable_shapes.intern([], result, Some(CallbackKind::OwnedOnce));
        let callbacks = [crate::analysis::CallbackContract { structural, parameter_names:vec![], params:vec![], result, kind:CallbackKind::OwnedOnce }];
        let mut registry = Registry::default();
        let mut families = Vec::new();
        for origin in [0,1] {
            let crate::syntax::Type::StoredCallable(id) = registry.intern_scoped_native(&checked.callable_shapes, &callbacks[0], 0, [origin], &checked.closures, Span::default()).unwrap().unwrap() else { unreachable!() };
            families.push(id);
        }
        let merged = registry.intern_recursive_scoped_native(&checked.callable_shapes,&callbacks,families[0],10,0,&checked.closures,Span::default()).unwrap();
        assert_eq!(registry.intern_recursive_scoped_native(&checked.callable_shapes,&callbacks,families[1],10,0,&checked.closures,Span::default()).unwrap(),merged);
        let crate::syntax::Type::StoredCallable(id) = merged else { unreachable!() };
        assert_eq!(registry.origins(id),Some(&[0,1][..]));
        assert_eq!(registry.origins(families[0]),Some(&[0][..]));
        assert_eq!(registry.origins(families[1]),Some(&[1][..]));
        assert_ne!(registry.intern_recursive_scoped_native(&checked.callable_shapes,&callbacks,families[0],10,1,&checked.closures,Span::default()).unwrap(),merged);
        let before = registry.scoped_native_family(id).unwrap().clone();
        checked.closures[1].captures[0].storage = Mode::Read;
        assert!(registry.intern_recursive_scoped_native(&checked.callable_shapes,&callbacks,families[1],10,0,&checked.closures,Span::default()).is_err());
        assert_eq!(registry.scoped_native_family(id),Some(&before));
        registry.remap_origins(&[Some(1),Some(0)]).unwrap();
        assert_eq!(registry.origins(id),Some(&[0,1][..]));
        assert_eq!(registry.origins(families[0]),Some(&[1][..]));
    }
}
