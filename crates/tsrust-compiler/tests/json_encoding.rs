mod support;
#[test]
fn json_encodes_ordinary_records_arrays_and_scalars_without_wrappers() {
    let source = r#"
import {strictEqual} from 'std:testing';
type Data={name:string;count:int;};
function main(){
 const data:Data={name:'hello',count:42};
 strictEqual(JSON.stringify(data),'{"count":42,"name":"hello"}');
 strictEqual(data.name,'hello');
 const values=[1,2,3];
 match(JSON.tryStringify(values)){Ok(text)=>{strictEqual(text,'[1,2,3]');},Err(error)=>{throw error.message;}}
 strictEqual(values[1],2);
 strictEqual(JSON.stringify('é\n'),'"é\\n"');strictEqual(JSON.stringify(true),'true');strictEqual(JSON.stringify(null),'null');
 match(JSON.tryStringify(Number.NaN)){Ok(text)=>{throw 'nonfinite accepted';},Err(error)=>{strictEqual(error.code,'number-nonfinite');}}
 match(JSON.tryStringify(data,1)){Ok(text)=>{throw 'limit ignored';},Err(error)=>{strictEqual(error.code,'output-limit');}}
}
"#;
    let dir = support::TempDirectory::new();
    let path = dir.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path), 90));
}

#[test]
fn json_encoding_preserves_recursive_records_and_rejects_undefined_positions() {
    let source = r#"
import {strictEqual} from 'std:testing';
type Tree={label:string;children:Tree[];next?:Tree;};
function main(){
 const tree=JSON.parse<Tree>('{"label":"root","children":[{"label":"leaf","children":[]}],"next":{"label":"next","children":[]}}');
 strictEqual(JSON.stringify(tree),'{"children":[{"children":[],"label":"leaf"}],"label":"root","next":{"children":[],"label":"next"}}');
 strictEqual(tree.label,'root');
 match(JSON.tryStringify(undefined)){Ok(text)=>{throw 'undefined root accepted';},Err(error)=>{strictEqual(error.code,'undefined-value');}}
 const values:(int|undefined)[]=[1,undefined];
 match(JSON.tryStringify(values)){Ok(text)=>{throw 'undefined entry accepted';},Err(error)=>{strictEqual(error.code,'undefined-value');}}
 const dynamic=JSON.parse('{"values":[1,2]}');
 strictEqual(JSON.stringify(dynamic),'{"values":[1,2]}');
}
"#;
    let dir = support::TempDirectory::new();
    let path = dir.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path), 90));
}

#[test]
fn json_encoding_rejects_callbacks_and_collections_with_public_diagnostics() {
    let dir = support::TempDirectory::new();
    for value in [
        "()=>1",
        "new Map<string,int>()",
        "new Set<int>()",
        "JSON.tryParse('null')",
    ] {
        let source = format!("function main(){{const value={value};JSON.stringify(value);}}");
        let diagnostic = tsrust_compiler::check(&source).err().unwrap();
        assert!(diagnostic.span.end <= source.len(), "{diagnostic:?}");
        let path = dir.source(&support::format_round_trip(&source));
        let output = support::run(support::cli().arg("check").arg(path), 30);
        assert!(!output.status.success(), "accepted {source}");
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(error.contains("json-type"), "{error}");
        assert!(
            !error.contains("@json") && !error.contains("native#"),
            "{error}"
        );
    }
}
