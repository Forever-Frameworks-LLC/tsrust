mod support;

#[test]
fn exported_enums_keep_identity_across_aliases() {
    use tsrust_compiler::project::{SourceId, SourceProvider, load_project, check_graph};
    struct Memory;
    impl SourceProvider for Memory {
        fn resolve(&self, _: Option<&SourceId>, path: &str) -> Result<SourceId, String> { Ok(SourceId(path.trim_start_matches("./").into())) }
        fn read(&self, id: &SourceId) -> Result<String, String> {
            Ok(match id.0.as_str() {
                "main.tsr" => "import {Phase as State,ready} from './dep.tsr';function main(){const state:State=ready();const same=state===State.Ready;}",
                "dep.tsr" => "export enum Phase{Ready,Busy}export function ready():Phase{return Phase.Ready;}",
                _ => return Err("missing module".into()),
            }.into())
        }
    }
    check_graph(load_project(&Memory,"main.tsr").unwrap()).unwrap();
}

#[test]
fn enum_constant_flags_and_qualified_previous_members() {
    tsrust_compiler::check("enum Flag{Read=1<<0,Write=1<<1,Both=Flag.Read|Write,Inverse=~0}function main(){const flag:Flag=Flag.Both;}").unwrap();
}

#[test]
fn enum_values_keep_nominal_types_copy_access_and_native_behavior() {
    let directory = support::TempDirectory::new();
    let path = directory.source(include_str!("fixtures/enums/values.tsr"));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn enums_reject_invalid_members_and_implicit_primitive_or_cross_enum_values() {
    for source in [
        "enum E{A,A}function main(){}",
        "enum E{A=1,B='two'}function main(){}",
        "enum E{A='one',B}function main(){}",
        "enum E{A}function main(){const value=E.Missing;}",
        "enum E{A}function main(){const value:E=0;}",
        "enum E{A}enum F{A}function main(){const value:E=F.A;}",

    ] {
        let error = tsrust_compiler::check(source).err().expect("invalid enum accepted");
        assert!(matches!(error.code, "enum" | "duplicate" | "type" | "immutable" | "assignment"), "{}: {}", error.code, error.message);
    }
}

#[test]
fn enum_constants_stay_immutable_after_copy() {
    let source = "enum E{A}function main(){const value=E.A;value=E.A;}";
    let error = tsrust_compiler::check(source).err().unwrap();
    assert_eq!(error.code, "unsupported");
    assert!(error.message.contains("Only let"));
}
