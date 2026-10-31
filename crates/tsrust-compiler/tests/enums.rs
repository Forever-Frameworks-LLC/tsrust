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

#[test]
fn string_conversion_borrows_string_members_and_formats_whole_numbers_exactly() {
    let source = support::format_round_trip("enum Label{Short=\"s\",Long=\"long\"}enum Phase{Ready,Busy=4}function pick(flag:boolean):Label|undefined{return flag?Label.Long:undefined;}function main(){const label=Label.Long;console.log(String(label),String(pick(true)),String(Number(Phase.Busy)));console.log(String(-5),String(9007199254740991),String(9007199254740992.0),String(-0.0),String(2.5),String(1e21));}");
    let rust = tsrust_compiler::check(&source).unwrap().emit_rust();
    assert!(rust.contains("String::Borrowed(builtin_argument_0.tsr_value())"), "string enum conversion should borrow its static text");
    support::execute(&source, "long long 4\n-5 9007199254740991 9007199254740992 0 2.5 1e+21\n", None);
}
