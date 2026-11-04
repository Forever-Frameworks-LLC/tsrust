mod support;
#[test]
fn closed_literal_json_preserves_exact_values_nullability_duplicates_and_limits(){
 let source=r#"
import {strictEqual} from 'std:testing';
import {parseJsonc,openJsonl,readJsonl,closeJsonl} from 'std:json';
type Status='draft'|'published';
type Item={status:Status;single:'draft';optional?:Status;maybe:Status|null;};
type Big=9007199254740993|9007199254740994;type Flag=true|false;type Ratio=2.5|7.5;
function main(){
 const item=JSON.parse<Item>('{"status":"published","single":"draft","maybe":null}');
 strictEqual(item.status==='published',true);strictEqual(item.single==='draft',true);strictEqual(item.optional===undefined,true);strictEqual(item.maybe===null,true);
 const strings=JSON.parse<Status[]>('["draft","published","draft"]');strictEqual(strings.length,3);
 strictEqual(JSON.parse<true>('true')===true,true);strictEqual(JSON.parse<false>('false')===false,true);
 strictEqual(JSON.parse<7>('7')===7,true);strictEqual(JSON.parse<2.5>('2.5')===2.5,true);
 const flag=JSON.parse<Flag>('false');strictEqual(flag===false,true);const ratio=JSON.parse<Ratio>('7.5');strictEqual(ratio===7.5,true);
 const big=JSON.parse<Big>('9007199254740993');strictEqual(big===9007199254740993,true);
 strictEqual(JSON.stringify(JSON.parse<0.0>('-0.0')),'-0.0');
 for(const input of ['"bad"','null','42','true']){match(JSON.tryParse<Status>(input)){Ok(value)=>{throw 'invalid literal accepted';},Err(error)=>{strictEqual(error.code,'shape-mismatch');}}}
 match(JSON.tryParse<7>('8')){Ok(value)=>{throw 'wrong int literal';},Err(error)=>{strictEqual(error.code,'shape-mismatch');}}
 match(JSON.tryParse<2.5>('9007199254740993')){Ok(value)=>{throw 'integer rounded';},Err(error)=>{strictEqual(error.code,'number-precision');}}
 match(JSON.tryParse<Item>('{"status":"bad","status":"draft","single":"draft","maybe":null}')){Ok(value)=>{throw 'invalid duplicate ignored';},Err(error)=>{strictEqual(error.code,'shape-mismatch');}}
 match(JSON.tryParse<Status[]>('["draft","published"]',{maxBytes:1024,maxDepth:4,maxEntries:4,maxNodes:2})){Ok(value)=>{throw 'budget bypassed';},Err(error)=>{strictEqual(error.code,'node-limit');}}
 match(parseJsonc<Status>('/* header */"published"')){Ok(value)=>{strictEqual(value==='published',true);},Err(error)=>{throw error.message;}}
 const reader=unwrap(openJsonl('records.jsonl'));
 match(readJsonl<Status>(reader)){Ok(value)=>{if(value!==undefined){strictEqual(value==='draft',true);}else{throw 'missing JSONL record';}},Err(error)=>{throw error.message;}}
 match(readJsonl<Status>(reader)){Ok(value)=>{strictEqual(value===undefined,true);},Err(error)=>{throw error.message;}}unwrap(closeJsonl(reader));
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

#[test]
fn typed_json_keys_and_literal_values_survive_escapes_duplicates_and_source_drop() {
 let source=r#"
import {strictEqual} from 'std:testing';
type Tag='ready'|'é😀\0';
type Item={tag:Tag;single:'fixed';text:string;n:int;};
function read(text:string):Item{return JSON.parse<Item>(text);}
function main(){
 let text='{"t\\u0061g":"r\\u0065ady","single":"fi\\u0078ed","text":"owned","n":2}';
 const item=read(clone(text));text='replaced';
 strictEqual(item.tag==='ready',true);strictEqual(item.single==='fixed',true);strictEqual(item.text,'owned');strictEqual(item.n,2);
 const unicode=read('{"tag":"é\\ud83d\\ude00\\u0000","single":"fixed","text":"independent","n":3}');
 strictEqual(unicode.tag==='é😀\0',true);strictEqual(unicode.text,'independent');
 const duplicate=read('{"tag":"ready","tag":"é\\ud83d\\ude00\\u0000","single":"fixed","text":"old","te\\u0078t":"last","n":4,"unknown":{"ignored":[true,null]},"\\u0000key":0}');
 strictEqual(duplicate.tag==='é😀\0',true);strictEqual(duplicate.text,'last');
 const nested=JSON.parse<[Tag,Item]>('["r\\u0065ady",{"tag":"ready","single":"fixed","text":"tuple","n":5}]');
 strictEqual(nested[0]==='ready',true);strictEqual(nested[1].text,'tuple');
 const arrays=JSON.parse<Tag[]>('["ready","é\\ud83d\\ude00\\u0000"]');strictEqual(arrays[1]==='é😀\0',true);
 for(const invalid of ['{"tag":"wrong","single":"fixed","text":"x","n":1}','{"tag":"wrong","tag":"ready","single":"fixed","text":"x","n":1}','{"tag":"ready","single":"wrong","text":"x","n":1}','{"tag":"ready","single":"fixed","text":"x","n":1,"\\ud800":0}']){
  match(JSON.tryParse<Item>(invalid)){Ok(value)=>{throw 'invalid JSON literal or key accepted';},Err(error)=>{strictEqual(error.code,'shape-mismatch');}}
 }
 console.log('Owned literal JSON passed');
}
"#;
 let dir=support::TempDirectory::new();let path=dir.source(&support::format_round_trip(source));
 for release in [false,true] {
  let mut cli=support::cli();cli.arg("run").arg(&path);if release {cli.arg("--release");}
  let output=support::run(&mut cli,120);support::success(&output);assert_eq!(output.stdout,b"Owned literal JSON passed\n");
 }
}
