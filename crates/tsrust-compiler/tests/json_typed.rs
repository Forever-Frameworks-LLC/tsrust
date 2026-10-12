mod support;
#[test]
fn typed_json_decodes_records_arrays_and_checked_numbers() {
    let source = r#"
import {strictEqual} from 'std:testing';
type Item={name:string;values:int[];active:boolean;};
function main(){
 const item=JSON.parse<Item>('{"name":"hello","values":[1,2],"active":true}');
 strictEqual(item.name,'hello');strictEqual(item.values[1],2);strictEqual(item.active,true);
 match(JSON.tryParse<int[]>('[1,2,3]')){Ok(values)=>{strictEqual(values.length,3);},Err(error)=>{throw error.message;}}
 match(JSON.tryParse<number>('9007199254740993')){Ok(value)=>{throw 'rounded';},Err(error)=>{strictEqual(error.code,'number-precision');}}
 match(JSON.tryParse<Item>('{"name":42,"values":[],"active":true}')){Ok(value)=>{throw 'wrong shape';},Err(error)=>{strictEqual(error.code,'shape-mismatch');}}
 strictEqual(typeof JSON.parse('42'),'number');
}
"#;
    let dir = support::TempDirectory::new();
    let path = dir.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path), 90));
}

#[test]
fn typed_json_preserves_recursive_shapes_optional_fields_and_limits() {
    let source = r#"
import {strictEqual} from 'std:testing';
type Tree={label:string;children:Tree[];next?:Tree;maybe:string|null;};
function read<T>(text:ReadOnly<string>):T{return JSON.parse<T>(text);}
function main(){
 strictEqual(read<int>('42'),42);
 const text='{"label":"old","label":"root","children":[{"label":"leaf","children":[],"maybe":null}],"maybe":"present","ignored":{"any":[1,2]}}';
 const tree=JSON.parse<Tree>(text);
 strictEqual(text.length>0,true);strictEqual(tree.label,'root');strictEqual(tree.next===undefined,true);
 strictEqual(tree.children[0].label,'leaf');strictEqual(tree.children[0].maybe===null,true);
 const chain=JSON.parse<Tree>('{"label":"a","children":[],"maybe":null,"next":{"label":"b","children":[],"maybe":null}}');
 if(chain.next!==undefined){strictEqual(chain.next.label,'b');}else{throw 'missing link';}
 match(JSON.tryParse<Tree>('{"label":42,"label":"later","children":[],"maybe":null}')){Ok(value)=>{throw 'invalid duplicate accepted';},Err(error)=>{strictEqual(error.code,'shape-mismatch');}}
 const pair=JSON.parse<[string,int]>('["value",1.0]');strictEqual(pair[0],'value');strictEqual(pair[1],1);
 const nested=JSON.parse<number[][]>('[[1,2],[3]]');strictEqual(nested[1][0],3.0);
 match(JSON.tryParse<Tree>('{"label":"a","children":[],"maybe":null,"ignored":[1,2,3]}',{maxBytes:1024,maxDepth:8,maxEntries:4,maxNodes:6})){
  Ok(value)=>{throw 'ignored values bypassed limits';},Err(error)=>{strictEqual(error.code,'node-limit');}
 }
 match(JSON.tryParse<Tree>('{"label":"a","children":[]}')){Ok(value)=>{throw 'missing required field';},Err(error)=>{strictEqual(error.code,'shape-mismatch');}}
 match(JSON.tryParse<int>('1.5')){Ok(value)=>{throw 'fraction accepted';},Err(error)=>{strictEqual(error.code,'integer-fractional');}}
 match(JSON.tryParse<int>('18446744073709551615')){Ok(value)=>{throw 'overflow accepted';},Err(error)=>{strictEqual(error.code,'integer-range');}}
 let caught=false;try {JSON.parse<boolean>('42');}catch(error){strictEqual(error.code,'shape-mismatch');caught=true;}strictEqual(caught,true);
 strictEqual(JSON.parse<int>('9223372036854775807'),9223372036854775807);
 strictEqual(JSON.parse<string>('"\\ud83d\\ude00"'),'😀');
}
"#;
    let dir = support::TempDirectory::new();
    let path = dir.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path), 90));
}

#[test]
fn typed_json_rejects_non_data_targets_and_wrong_arity() {
    for (source, code) in [
        ("function main(){JSON.parse<void>('null');}", "error[type]"),
        (
            "function main(){JSON.parse<ReadOnly<Function<int>>>('null');}",
            "callback-escape",
        ),
        ("function main(){JSON.parse<string|int>('1');}", "json-type"),
        (
            "import {Bytes} from 'std:bytes';function main(){JSON.parse<Bytes>('null');}",
            "json-type",
        ),
        ("function main(){JSON.parse<int,string>('1');}", "arity"),
    ] {
        let dir = support::TempDirectory::new();
        let path = dir.source(&support::format_round_trip(source));
        let output = support::run(support::cli().arg("check").arg(path), 60);
        assert!(!output.status.success());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains(code), "{error}");
    }
}
#[test]
fn typed_json_catalog_examples_execute() {
    let catalog: serde_json::Value =
        serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
    let mut sources = std::collections::HashSet::new();
    for entry in catalog["JSON"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| matches!(entry["name"].as_str(), Some("parse" | "tryParse")))
    {
        for example in entry["examples"].as_array().unwrap() {
            let source = example["source"].as_str().unwrap();
            if !sources.insert(source) {
                continue;
            }
            let dir = support::TempDirectory::new();
            let path = dir.source(&support::format_round_trip(source));
            let output = support::run(support::cli().arg("run").arg(path), 90);
            support::success(&output);
            assert_eq!(
                String::from_utf8(output.stdout).unwrap(),
                example["output"].as_str().unwrap()
            );
        }
    }
}

#[test]
fn typed_json_emits_direct_visitors_without_dynamic_parse_bridge() {
    let source = "type Item={text:string;values:int[];};function main(){const item=JSON.parse<Item>('{\"text\":\"ok\",\"values\":[1]}');console.log(item.text);}";
    let dir = support::TempDirectory::new();
    let path = dir.source(&support::format_round_trip(source));
    let output = support::run(support::cli().arg("emit").arg(path), 60);
    support::success(&output);
    let emitted = String::from_utf8(output.stdout).unwrap();
    assert!(emitted.contains("tsrust_native_json::typed::decode::<Decode0>"));
    assert!(emitted.contains("serde::de::MapAccess") && emitted.contains("serde::de::SeqAccess"));
    assert!(!emitted.contains("fn tsrust_std_json_parse("));
}
