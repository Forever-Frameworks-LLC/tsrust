//! Finite generic environments have storage identity independent of invocation arity.
use crate::analysis::CheckedClosure;
use crate::{Diagnostic, Span};
use std::collections::{BTreeMap, BTreeSet};

/// Storage connectivity is monotone. It does not describe which origin reaches a call.
#[derive(Default)]
pub(crate) struct Registry {
    parents: BTreeMap<usize, usize>,
}

impl Registry {
    pub(crate) fn observe(&mut self, previous: usize, replacement: usize) {
        self.parents.entry(previous).or_insert(previous);
        self.parents.entry(replacement).or_insert(replacement);
        let previous = self.root(previous).unwrap();
        let replacement = self.root(replacement).unwrap();
        self.parents.insert(previous.max(replacement), previous.min(replacement));
    }

    fn root(&self, origin: usize) -> Option<usize> {
        let mut root = *self.parents.get(&origin)?;
        while let Some(&parent) = self.parents.get(&root) {
            if parent == root { return Some(root); }
            root = parent;
        }
        None
    }

    fn members(&self, origin: usize) -> Option<Vec<usize>> {
        let root = self.root(origin)?;
        Some(self.parents.keys().copied().filter(|&member| self.root(member) == Some(root)).collect())
    }

    /// Declaration proofs select concrete environments only from this storage.
    /// Repeated references to one declaration retain their distinct payloads.
    pub(crate) fn reachable_origins(
        &self, origin: usize, declarations: &[Span],
        declaration: impl Fn(usize) -> Option<Span>, span: Span,
    ) -> Result<Vec<usize>, Diagnostic> {
        if declarations.is_empty() {
            return Err(Diagnostic::new("callable-layout", "A generic invocation has no reachable declaration.", span));
        }
        let members = self.members(origin).unwrap_or_else(|| vec![origin]);
        let mut observed = Vec::new();
        let mut selected = Vec::new();
        for member in members {
            let source = declaration(member).ok_or_else(|| Diagnostic::new("callable-layout", "Generic storage has an environment without declaration provenance.", span))?;
            if declarations.contains(&source) {
                selected.push(member);
                observed.push(source);
            }
        }
        for &source in declarations {
            if !observed.contains(&source) {
                return Err(Diagnostic::new("callable-layout", "A reachable declaration has no environment in this generic storage family.", span)
                    .related(source, "Reachable callable declared here."));
            }
        }
        Ok(selected)
    }

    pub(crate) fn remap(&self, ids: &[Option<usize>], span: Span) -> Result<Self, Diagnostic> {
        let mut result = Self::default();
        let mut seen = BTreeSet::new();
        for &origin in self.parents.keys() {
            let mapped = ids.get(origin).copied().flatten().ok_or_else(|| Diagnostic::new("callable-layout", "Generic storage lost an environment origin during compaction.", span))?;
            if !seen.insert(mapped) {
                return Err(Diagnostic::new("callable-layout", "Distinct generic environments cannot merge during compaction.", span));
            }
            let root = ids[self.root(origin).expect("registered root")].expect("registered root mapping");
            result.observe(root, mapped);
        }
        Ok(result)
    }

    pub(crate) fn validate(&self, closures: &[CheckedClosure], span: Span) -> Result<(), Diagnostic> {
        if self.parents.is_empty() { return Ok(()); }
        checked_origins(self.parents.keys().copied(), closures, span).map(|_| ())
    }

    pub(crate) fn families(&self, closures: &[CheckedClosure], span: Span) -> Result<Vec<Family>, Diagnostic> {
        self.parents.keys().copied().filter(|&origin| self.root(origin) == Some(origin))
            .map(|origin| self.checked(origin, closures, span)).collect()
    }

    fn checked(&self, origin: usize, closures: &[CheckedClosure], span: Span) -> Result<Family, Diagnostic> {
        let members = self.members(origin).ok_or_else(|| Diagnostic::new("callable-layout", "Generic storage origin has not been registered.", span))?;
        Family::checked(*members.first().expect("registered member"), members, closures, span)
    }
}

pub(crate) struct Family {
    pub(crate) id: usize,
    pub(crate) origins: Vec<usize>,
}

fn checked_origins(origins: impl IntoIterator<Item = usize>, closures: &[CheckedClosure], span: Span) -> Result<Vec<usize>, Diagnostic> {
        let mut origins = origins.into_iter().collect::<Vec<_>>();
        origins.sort_unstable();
        origins.dedup();
        if origins.is_empty() {
            return Err(Diagnostic::new("callable-layout", "Generic storage requires at least one environment origin.", span));
        }
        for &origin in &origins {
            let closure = closures.get(origin).ok_or_else(|| Diagnostic::new("callable-layout", "Generic storage refers to an unknown environment origin.", span))?;
            if !closure.polymorphic || closure.function.template {
                return Err(Diagnostic::new("callable-layout", "Generic storage requires checked polymorphic environment layouts.", span).related(closure.span, "Environment is defined here."));
            }
        }
        Ok(origins)
}

impl Family {
    fn checked(id: usize, origins: impl IntoIterator<Item = usize>, closures: &[CheckedClosure], span: Span) -> Result<Self, Diagnostic> {
        Ok(Self { id, origins: checked_origins(origins, closures, span)? })
    }

    pub(crate) fn declaration(&self, parameters: &[String], environment_type: impl Fn(usize) -> String) -> String {
        let parameters = if parameters.is_empty() { String::new() } else { format!("<{}>", parameters.join(", ")) };
        let variants = self.origins.iter().enumerate().map(|(variant, &origin)| format!("V{variant}({})", environment_type(origin))).collect::<Vec<_>>().join(", ");
        format!("enum TsrGenericFamily{}{parameters} {{ {variants} }}\n", self.id)
    }

    #[cfg(test)]
    fn inject(&self, origin: usize, value: &str, span: Span) -> Result<String, Diagnostic> {
        let variant = self.origins.binary_search(&origin).map_err(|_| Diagnostic::new("callable-layout", "Environment origin is not a member of this generic storage family.", span))?;
        Ok(format!("TsrGenericFamily{}::V{variant}({value})", self.id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn environments() -> crate::Analysis {
        crate::check("function first<T>(value:T):T{return value;}function second<T>(value:T,next:T):T{return next;}function main(){const one=first;const two=second;one<int>(3);two<int>(4,7);}").unwrap()
    }

    #[test]
    fn storage_keeps_different_arities_and_canonical_origin_identity() {
        let checked = environments();
        let family = Family::checked(4, [1, 0, 1], &checked.program.closures, Span::default()).unwrap();
        assert_eq!(family.origins, vec![0, 1]);
        assert_ne!(checked.program.closures[0].methods[0].parameters.len(), checked.program.closures[1].methods[0].parameters.len());
        assert_eq!(family.declaration(&[], |id| format!("Closure{id}")), "enum TsrGenericFamily4 { V0(Closure0), V1(Closure1) }\n");
        assert_eq!(family.inject(1, "make_second()", Span::default()).unwrap(), "TsrGenericFamily4::V1(make_second())");
        assert_eq!(family.inject(2, "missing", Span::new(3, 4)).unwrap_err().span, Span::new(3, 4));
        assert_eq!(Family::checked(4, [], &checked.program.closures, Span::default()).err().unwrap().code, "callable-layout");
        assert!(Family::checked(4, [usize::MAX], &checked.program.closures, Span::default()).is_err());
    }

    #[test]
    fn storage_rejects_unchecked_templates_and_monomorphic_environments() {
        let mut checked = environments();
        checked.program.closures[0].function.template = true;
        let error = Family::checked(4, [0], &checked.program.closures, Span::default()).err().unwrap();
        assert_eq!(error.code, "callable-layout");
        assert!(!error.related.is_empty());
        checked.program.closures[0].function.template = false;
        checked.program.closures[0].polymorphic = false;
        assert!(Family::checked(4, [0], &checked.program.closures, Span::default()).is_err());
    }

    #[test]
    fn replacement_connectivity_is_transitive_without_merging_independent_storage() {
        let mut registry = Registry::default();
        registry.observe(7, 3);
        registry.observe(9, 11);
        registry.observe(5, 7);
        registry.observe(3, 7);
        assert_eq!(registry.members(5), Some(vec![3, 5, 7]));
        assert_eq!(registry.members(11), Some(vec![9, 11]));
        assert_eq!(registry.members(0), None);
        registry.observe(11, 5);
        assert_eq!(registry.members(7), Some(vec![3, 5, 7, 9, 11]));
        let checked = environments();
        let mut actual = Registry::default();
        actual.observe(1, 0);
        assert_eq!(actual.checked(1, &checked.program.closures, Span::default()).unwrap().origins, vec![0, 1]);
    }

    #[test]
    fn compaction_preserves_distinct_environments_and_rejects_missing_origins() {
        let mut registry = Registry::default();
        registry.observe(0, 2);
        registry.observe(1, 1);
        let remapped = registry.remap(&[Some(2), Some(0), Some(1)], Span::default()).unwrap();
        assert_eq!(remapped.members(2), Some(vec![1, 2]));
        assert_eq!(remapped.members(0), Some(vec![0]));
        assert!(registry.remap(&[Some(0), Some(1), None], Span::default()).is_err());
        assert!(registry.remap(&[Some(0), Some(1), Some(0)], Span::default()).is_err());
        assert!(registry.remap(&[Some(0)], Span::default()).is_err());
    }

    #[test]
    fn invocation_provenance_keeps_all_matching_environments_within_storage() {
        let mut registry = Registry::default();
        registry.observe(0, 1);
        registry.observe(1, 2);
        registry.observe(3, 3);
        let first = Span::new(10, 15);
        let second = Span::new(20, 26);
        let declarations = [first, first, second, first];
        let resolve = |origin| declarations.get(origin).copied();
        let call = Span::new(40, 50);
        assert_eq!(registry.reachable_origins(0, &[first], resolve, call).unwrap(), vec![0, 1]);
        assert_eq!(registry.reachable_origins(1, &[second, first, first], resolve, call).unwrap(), vec![0, 1, 2]);
        assert_eq!(registry.reachable_origins(2, &[second], resolve, call).unwrap(), vec![2]);
        assert_eq!(registry.reachable_origins(3, &[first], resolve, call).unwrap(), vec![3]);
        assert_eq!(registry.reachable_origins(4, &[first], |_| Some(first), call).unwrap(), vec![4]);
        for proof in [vec![], vec![Span::new(70, 75)]] {
            assert_eq!(registry.reachable_origins(0, &proof, resolve, call).unwrap_err().span, call);
        }
        assert!(registry.reachable_origins(0, &[first], |origin| if origin == 2 { None } else { resolve(origin) }, call).is_err());
    }

    #[test]
    fn checked_reassignments_retain_fresh_origins_without_changing_receiver_type() {
        let checked = crate::check("function identity<T>(value:T):T{return value;}function main(){let action=identity;const saved=action;action=identity;action<int>(7);action=identity;action<string>('owned');saved<int>(3);}").unwrap();
        let main = checked.program.functions.iter().find(|function| function.name == "main").unwrap();
        let action = main.bindings.iter().find(|binding| binding.name == "action").unwrap();
        let crate::syntax::Type::Closure(origin) = action.ty else { panic!("generic receiver"); };
        let members = checked.program.polymorphic_storage.members(origin).expect("checked assignment origins");
        assert_eq!(members.len(), 3);
        assert!(members.contains(&origin));
        assert!(members.iter().all(|&id| checked.program.closures[id].polymorphic && checked.program.closures[id].captures.is_empty()));
        assert_eq!(checked.program.closures[origin].methods.len(), 2);
    }

    #[test]
    fn unused_template_assignments_do_not_publish_provisional_storage_origins() {
        let checked = crate::check("function identity<T>(value:T):T{return value;}function unused<T>(){let action=identity;action=identity;}function main(){}").unwrap();
        assert!(checked.program.polymorphic_storage.parents.is_empty());
    }

    #[test]
    fn concrete_specializations_keep_registered_origins_after_template_compaction() {
        let checked = crate::check("function identity<T>(value:T):T{return value;}function unused<T>(){let action=identity;action=identity;}function replace<T>(value:T):T{let action=identity;action=identity;return action<T>(value);}function main(){replace<int>(7);replace<string>('owned');}").unwrap();
        let registry = &checked.program.polymorphic_storage;
        assert!(registry.parents.len() >= 2);
        assert!(registry.parents.keys().all(|&origin| !checked.program.closures[origin].function.template));
        let mut receivers = 0;
        for function in checked.program.functions.iter().filter(|function| !function.template) {
            for binding in &function.bindings {
                if binding.name != "action" { continue; }
                let crate::syntax::Type::Closure(origin) = binding.ty else { panic!("concrete generic receiver"); };
                let family = registry.members(origin).expect("remapped receiver storage");
                assert!(family.len() >= 2);
                assert!(family.contains(&origin));
                receivers += 1;
            }
        }
        assert_eq!(receivers, 2);
    }

    fn native(source: &str) {
        let directory = std::env::temp_dir().join(format!("tsrust-polymorphic-carrier-{}-{:?}", std::process::id(), std::thread::current().id()));
        std::fs::create_dir_all(&directory).unwrap();
        let input = directory.join("carrier.rs");
        let output = directory.join("carrier");
        std::fs::write(&input, source).unwrap();
        let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let compiled = std::process::Command::new(rustc).arg("--edition=2024").arg(&input).arg("-o").arg(&output).output().unwrap();
        assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
        let executed = std::process::Command::new(&output).output().unwrap();
        assert!(executed.status.success(), "{}", String::from_utf8_lossy(&executed.stderr));
        assert!(executed.stdout.is_empty());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn native_replacement_moves_nonclone_environments_and_drops_each_once() {
        let checked = environments();
        let family = Family::checked(4, [0, 1], &checked.program.closures, Span::default()).unwrap();
        let declaration = family.declaration(&[], |id| format!("Closure{id}"));
        let first = family.inject(0, "{ evaluations += 1; Closure0(Payload(3)) }", Span::default()).unwrap();
        let second = family.inject(1, "{ evaluations += 1; Closure1(Payload(7)) }", Span::default()).unwrap();
        native(&format!(r#"
            use std::sync::atomic::{{AtomicUsize, Ordering}};
            static DROPS: AtomicUsize = AtomicUsize::new(0);
            struct Payload(i64);
            impl Drop for Payload {{ fn drop(&mut self) {{ DROPS.fetch_add(1, Ordering::SeqCst); }} }}
            struct Closure0(Payload);
            struct Closure1(Payload);
            {declaration}
            fn read(value: &TsrGenericFamily4) -> i64 {{ match value {{ TsrGenericFamily4::V0(env) => env.0.0, TsrGenericFamily4::V1(env) => env.0.0 }} }}
            fn main() {{
                let mut evaluations = 0;
                let mut action = {first};
                let saved = action;
                action = {second};
                assert_eq!((read(&saved), read(&action), evaluations), (3,7,2));
                assert_eq!(DROPS.load(Ordering::SeqCst), 0);
                action = {first};
                assert_eq!(DROPS.load(Ordering::SeqCst), 1);
                assert_eq!((read(&saved), read(&action), evaluations), (3,3,3));
                drop(action); drop(saved);
                assert_eq!(DROPS.load(Ordering::SeqCst), 3);
            }}
        "#));
    }

    #[test]
    fn native_borrowed_carrier_preserves_explicit_lifetime_parameters() {
        let checked = environments();
        let family = Family::checked(4, [0, 1], &checked.program.closures, Span::default()).unwrap();
        let declaration = family.declaration(&["'a".into(), "'b".into()], |id| format!("Closure{id}<{}>", if id == 0 { "'a" } else { "'b" }));
        native(&format!(r#"
            struct Closure0<'a>(&'a str);
            struct Closure1<'b>(&'b mut i64);
            {declaration}
            fn main() {{
                let text = String::from("retained");
                let mut count = 0;
                let value = TsrGenericFamily4::V0(Closure0(&text));
                match value {{ TsrGenericFamily4::V0(env) => assert_eq!(env.0, "retained"), _ => unreachable!() }}
                let mut value = TsrGenericFamily4::V1(Closure1(&mut count));
                match &mut value {{ TsrGenericFamily4::V1(env) => *env.0 += 1, _ => unreachable!() }}
                drop(value);
                assert_eq!(count, 1);
                assert_eq!(text, "retained");
            }}
        "#));
    }
}
