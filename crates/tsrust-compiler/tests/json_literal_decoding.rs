mod support;
#[test]
fn closed_literal_json_preserves_exact_values_nullability_duplicates_and_limits(){
 let source=r#"
import {strictEqual} from 'std:testing';
import {parseJsonc,openJsonl,readJsonl,closeJsonl} from 'std:json';
type Status='draft'|'published';
type Item={status:Status;single:'draft';optional?:Status;maybe:Status|null;};
type Big=9007199254740993|9007199254740994;
function main(){
 const item=JSON.parse<Item>('{"status":"published","single":"draft","maybe":null}');
 strictEqual(item.status==='published',true);strictEqual(item.single==='draft',true);strictEqual(item.optional===undefined,true);strictEqual(item.maybe===null,true);
 const strings=JSON.parse<Status[]>('["draft","published","draft"]');strictEqual(strings.length,3);
 strictEqual(JSON.parse<true>('true')===true,true);strictEqual(JSON.parse<false>('false')===false,true);
 strictEqual(JSON.parse<7>('7')===7,true);strictEqual(JSON.parse<2.5>('2.5')===2.5,true);
 const big=JSON.parse<Big>('9007199254740993');strictEqual(big===9007199254740993,true);
 strictEqual(JSON.stringify(JSON.parse<0.0>('-0.0')),'-0.0');
 for(const input of ['"bad"','null','42','true']){match(JSON.tryParse<Status>(input)){Ok(value)=>{throw 'invalid literal accepted';},Err(error)=>{strictEqual(error.code,'shape-mismatch');}}}
 match(JSON.tryParse<7>('8')){Ok(value)=>{throw 'wrong int literal';},Err(error)=>{strictEqual(error.code,'shape-mismatch');}}
 match(JSON.tryParse<2.5>('9007199254740993')){Ok(value)=>{throw 'integer rounded';},Err(error)=>{strictEqual(error.code,'number-precision');}}
 match(JSON.tryParse<Item>('{"status":"bad","status":"draft","single":"draft","maybe":null}')){Ok(value)=>{throw 'invalid duplicate ignored';},Err(error)=>{strictEqual(error.code,'shape-mismatch');}}
 match(JSON.tryParse<Status[]>('["draft","published"]',{maxBytes:1024,maxDepth:4,maxEntries:4,maxNodes:2})){Ok(value)=>{throw 'budget bypassed';},Err(error)=>{strictEqual(error.code,'node-limit');}}
 strictEqual(unwrap(parseJsonc<Status>('/* header */"published"'))==='published',true);
 const reader=unwrap(openJsonl('records.jsonl'));const record=unwrap(readJsonl<Status>(reader));
 if(record!==undefined){strictEqual(record==='draft',true);}else{throw 'missing JSONL record';}
 strictEqual(unwrap(readJsonl<Status>(reader))===undefined,true);unwrap(closeJsonl(reader));
 console.log('Literal JSON passed');
}
"#;
 let dir=support::TempDirectory::new();std::fs::write(dir.0.join("records.jsonl"),"\"draft\"\n").unwrap();let path=dir.source(&support::format_round_trip(source));let output=support::run(support::cli().arg("run").arg(path).current_dir(&dir.0),120);support::success(&output);assert_eq!(output.stdout,b"Literal JSON passed\n");
}
#[test]
fn json_literal_support_does_not_admit_ambiguous_shape_unions(){
 for ty in ["string|int","'draft'|7","First|Second"]{
  let source=support::format_round_trip(&format!("interface First{{value:int;}}interface Second{{value:string;}}type Value={ty};function main(){{const value=JSON.parse<Value>('null');}}"));
  let dir=support::TempDirectory::new();let path=dir.source(&source);let output=support::run(support::cli().arg("check").arg(path),30);assert!(!output.status.success(),"{ty}");assert!(String::from_utf8_lossy(&output.stderr).contains("json-type"),"{ty}: {}",String::from_utf8_lossy(&output.stderr));
 }
}
