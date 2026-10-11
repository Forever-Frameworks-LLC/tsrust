mod support;

#[test]
fn json_dependencies_follow_native_values_not_namespace_spelling() {
    let directory=support::TempDirectory::new();
    let entry=directory.0.join("main.tsr");
    for (source,needed) in [
        ("function main(){console.log('plain');}",false),
        ("type Utility={parse:Function<int,[text:ReadOnly<string>]>;};function main(){const JSON:Utility={parse:(text:string):int=>text.length};console.log(JSON.parse('local'));}",false),
        ("import {JsonError} from 'std:json';function main(){const error:JsonError={code:'local',message:'local',offset:0};console.log(error.code);}",false),
        ("import {parse} from 'std:json';function main(){parse('null');}",true),
    ] {
        std::fs::write(&entry,support::format_round_trip(source)).unwrap();
        let output=support::run(support::cli().arg("emit").arg(&entry),30);
        support::success(&output);
        let emitted=String::from_utf8(output.stdout).unwrap();
        assert_eq!(emitted.contains("mod tsrust_native_json"),needed,"{source}");
    }
}
