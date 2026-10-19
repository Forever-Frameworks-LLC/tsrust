mod support;
use tsrust_compiler::check;
#[test]
fn generic_aliases_substitute_defaults_and_preserve_ownership_and_precision() {
    let entry = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/generic-type-aliases/main.tsr");
    let output = support::run(support::cli().arg("run").arg(entry), 90);
    support::success(&output);
    assert_eq!(output.stdout, b"Generic type aliases passed\n");
    support::execute(
        r#"
type Values<T=string>=T[];
type Choice<A,B=A>=A|B;
type Pair<A=int,B=Values<A>>=[A,B];
type Reader<T>=Function<int,[value:ReadOnly<T>]>;
type OwnedReader<T>=Owned<Function<int,[value:ReadOnly<T>]>>;
type Global=string;type Shadow<Global>=Global[];
function sum(values:ReadOnly<Values<int>>):int{return values[0]+values[1];}
function main(){const words:Values=['first','second'];console.log(words.join(','));const pair:Pair=[7,[2,3]];console.log(pair[0],sum(pair[1]));const value:Choice<int>=9;console.log(value);const shadow:Shadow<int>=[3];console.log(shadow[0]);const reader:Reader<int>=(value:int):int=>value+1;console.log(reader(4));const owned:OwnedReader<int>=(value:int):int=>value+2;console.log(owned(5));}
"#,
        "first,second\n7 5\n9\n3\n5\n7\n",
        None,
    );
}
#[test]
fn generic_alias_failures_have_specific_diagnostics() {
    for (source, code) in [
        ("type A<T>=T[];function main(){const x:A=[];}", "arity"),
        (
            "type A<T>=T[];function main(){const x:A<int,string>=[];}",
            "arity",
        ),
        ("type A<T=int,U>=T;function main(){}", "type-parameter"),
        ("type A<T=T>=T;function main(){}", "type-parameter"),
        ("type A<T=U,U=int>=T;function main(){}", "type-parameter"),
        ("type A<T,T>=T;function main(){}", "duplicate"),
        ("type A<true>=int;function main(){}", "type-parameter"),
        (
            "type A<T>=B<T>;type B<U>=A<U>;function main(){}",
            "recursive-alias",
        ),
        (
            "type A<T>=T[];function main(){const values:A<int>=['wrong'];}",
            "type",
        ),
        ("type A<T=int>={value:T;};function main(){}", "unsupported"),
    ] {
        let formatted = support::format_round_trip(source);
        let error = check(&formatted).err().expect(source);
        assert_eq!(error.code, code, "{source}: {error:?}");
    }
}
#[test]
fn alias_editor_query_uses_the_same_substitution_and_default_rules() {
    let catalog = tsrust_compiler::TypeAliasCatalog::parse(
        "type Values<T=string>=T[];type Pair<A=int,B=Values<A>>=[A,B];",
    )
    .unwrap();
    assert_eq!(catalog.expand("Values").unwrap(), "string[]");
    assert_eq!(catalog.expand("Pair").unwrap(), "[int, int[]]");
    assert_eq!(
        catalog.expand("Values<int|string>").unwrap(),
        "(int | string)[]"
    );
    assert!(catalog.expand("Pair<int,string,boolean>").is_err());
    let callbacks =
        tsrust_compiler::TypeAliasCatalog::parse("type Callback<T>=Once<Function<T>>;").unwrap();
    assert_eq!(
        callbacks.expand("Callback<string>").unwrap(),
        "Once<Function<string>>"
    );
    assert!(
        catalog
            .declarations()
            .iter()
            .any(|i| i.signature.contains("T = string"))
    );
}

#[test]
fn aliases_bound_expansion_before_allocating_repeated_substitutions() {
    let mut source = String::from("type A0<T>=[T,T];");
    for n in 1..20 {
        source.push_str(&format!("type A{n}<T> = A{}<A{}<T>>;", n - 1, n - 1));
    }
    source.push_str("function main(){}");
    let error = check(&source).err().unwrap();
    assert_eq!(error.code, "unsupported");
    assert!(error.message.contains("annotation nodes"), "{error:?}");
}

#[test]
fn generic_aliases_never_clone_or_resurrect_resources() {
    for (body, code) in [
        (
            "const value:Identity<TrackedResource>=trackedResource();const copy=clone(value);",
            "clone-type",
        ),
        (
            "const value:Identity<TrackedResource>=trackedResource();closeResource(value);resourceId(value);",
            "transferred",
        ),
    ] {
        let directory = support::TempDirectory::new();
        let source = support::format_round_trip(&format!(
            "import {{TrackedResource,trackedResource,closeResource,resourceId}} from 'std:testing';type Identity<T>=T;function main(){{{body}}}"
        ));
        let path = directory.source(&source);
        for command in ["check", "emit"] {
            let output = support::run(support::cli().arg(command).arg(&path), 30);
            assert!(!output.status.success(), "{source}");
            assert!(
                String::from_utf8_lossy(&output.stderr).contains(&format!("error[{code}]")),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
