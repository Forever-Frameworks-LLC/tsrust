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
        ("function main(){JSON.tryParse('null');}",true),
        ("type Local={tryParse:int;};function main(){const JSON:Local={tryParse:7};function read():int{return JSON.tryParse;}console.log(read());}",false),
        ("function main(){JSON.stringify(JSON.parse('null'));}",true),
        ("type Utility={tryParse:Function<int,[text:ReadOnly<string>]>;};function main(){const JSON:Utility={tryParse:(text:string):int=>text.length};console.log(JSON.tryParse('local'));}",false),
        ("type Utility={tryParse:Function<int,[text:ReadOnly<string>]>;};function use(JSON:Utility):int{return JSON.tryParse('local');}function main(){const local:Utility={tryParse:(text:string):int=>text.length};console.log(use(local));}",false),
    ] {
        std::fs::write(&entry,support::format_round_trip(source)).unwrap();
        let output=support::run(support::cli().arg("emit").arg(&entry),30);
        support::success(&output);
        let emitted=String::from_utf8(output.stdout).unwrap();
        assert_eq!(emitted.contains("mod tsrust_native_json"),needed,"{source}");
    }
}
