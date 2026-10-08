//! Finite generic environments have storage identity independent of invocation arity.
use crate::analysis::CheckedClosure;
use crate::{Diagnostic, Span};

struct Family {
    id: usize,
    origins: Vec<usize>,
}

impl Family {
    fn checked(id: usize, origins: impl IntoIterator<Item = usize>, closures: &[CheckedClosure], span: Span) -> Result<Self, Diagnostic> {
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
        Ok(Self { id, origins })
    }

    fn declaration(&self, parameters: &[String], environment_type: impl Fn(usize) -> String) -> String {
        let parameters = if parameters.is_empty() { String::new() } else { format!("<{}>", parameters.join(", ")) };
        let variants = self.origins.iter().enumerate().map(|(variant, &origin)| format!("V{variant}({})", environment_type(origin))).collect::<Vec<_>>().join(", ");
        format!("enum TsrGenericFamily{}{parameters} {{ {variants} }}\n", self.id)
    }

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
