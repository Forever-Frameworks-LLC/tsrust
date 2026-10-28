use tsrust_compiler::check;

#[test]
fn unused_generic_defaults_reject_known_implicit_void_results() {
    for (source, initializer) in [
        (
            "function empty(){}function bad<T>(value:T=empty()):T{return value;}function main(){}",
            "empty()",
        ),
        (
            "function empty(){}function bad<T>(action:ReadOnly<Function<T>>=empty):T{return action();}function main(){}",
            "empty",
        ),
        (
            "function empty(value:int){}function bad<T>(value:T=empty(7)):T{return value;}function main(){}",
            "empty(7)",
        ),
    ] {
        let error = check(source)
            .err()
            .expect("void is not every unconstrained type");
        assert_eq!(error.code, "type");
        assert!(error.message.contains("every type permitted"));
        assert_eq!(&source[error.span.start..error.span.end], initializer);
    }
}

#[test]
fn implicit_void_functions_and_function_values_keep_existing_calls() {
    let source = "function empty(){}function apply<T>(fallback:T):T{empty();return fallback;}function main(){const action=empty;action();console.log(apply<int>(7));}";
    let checked =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("void.tsr", source)));
    assert!(!checked.emit_rust().contains("unsafe"));
}

#[test]
fn generic_default_scope_preserves_linked_native_imports() {
    use tsrust_compiler::project::{SourceId, SourceProvider, check_graph, load_project};
    struct Single<'a>(&'a str);
    impl SourceProvider for Single<'_> {
        fn resolve(&self, _: Option<&SourceId>, path: &str) -> Result<SourceId, String> {
            (path == "main.tsr")
                .then(|| SourceId(path.into()))
                .ok_or_else(|| "unexpected source".into())
        }
        fn read(&self, id: &SourceId) -> Result<String, String> {
            if id.0 != "main.tsr" { return Err(format!("Missing source {}",id.0)); }
            Ok(self.0.into())
        }
    }
    for (invalid, initializer) in [
        (
            "import {basename} from 'std:path';function bad<T>(value:T=basename('/tmp/report.txt')):T{return value;}function main(){}",
            "basename('/tmp/report.txt')",
        ),
        (
            "import {basename} from 'std:path';function bad<T>(value:T=basename):T{return value;}function main(){}",
            "basename",
        ),
    ] {
        let error = check_graph(load_project(&Single(invalid), "main.tsr").unwrap())
            .err()
            .expect("native string is not arbitrary T");
        assert_eq!(error.code, "type");
        assert!(error.message.contains("every type permitted"));
        assert_eq!(error.location.source.0, "main.tsr");
        assert_eq!(
            &invalid[error.location.span.start..error.location.span.end],
            initializer
        );
    }
    let valid = "import {basename} from 'std:path';function name<T>(fallback:T,value:string=basename('/tmp/report.txt')):T{return fallback;}function main(){console.log(name<int>(7));}";
    let checked = check_graph(load_project(&Single(valid), "main.tsr").unwrap())
        .unwrap_or_else(|error| panic!("{error:?}"));
    assert!(checked.emit_rust().contains("tsrust_native_path"));
}
