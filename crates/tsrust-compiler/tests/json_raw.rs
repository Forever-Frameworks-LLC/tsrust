mod support;

#[test]
fn validated_raw_fragments_borrow_input_and_release_failed_loans() {
 let source=r#"
import {borrowRaw,JsonRaw,JsonError} from 'std:json';
import {strictEqual} from 'std:testing';
function validate(text:ReadOnly<string>):Result<JsonRaw,JsonError>{return borrowRaw(text);}
function main(){
 let input='{"value":7}';
 {match(validate(input)){Ok(fragment)=>{},Err(error)=>{throw error.message;}}}
 input='null';strictEqual(input,'null');
 let invalid='[';
 match(borrowRaw(invalid)){Ok(fragment)=>{throw 'invalid JSON accepted';},Err(error)=>{strictEqual(error.code,'unexpected-end');}}
 invalid='{}';strictEqual(invalid,'{}');
}
"#;
 let directory=support::TempDirectory::new();
 let path=directory.source(&support::format_round_trip(source));
 support::success(&support::run(support::cli().arg("run").arg(path),90));
}

#[test]
fn raw_fragments_reject_source_invalidation_clone_and_escape() {
 for (body,expected) in [
  ("function main(){let source='null';match(borrowRaw(source)){Ok(raw)=>{source='1';consume(raw);},Err(e)=>{}}}","native-view-conflict"),
  ("function main(){const source='null';match(borrowRaw(source)){Ok(raw)=>{clone(raw);},Err(e)=>{}}}","clone-type"),
  ("function bad(){const source='null';return borrowRaw(source);}function main(){}","native-view-escape"),
 ] {
  let source=format!("import {{borrowRaw,JsonRaw}} from 'std:json';function consume(value:ReadOnly<JsonRaw>):void{{}}{body}");
  let directory=support::TempDirectory::new();let path=directory.source(&source);
  let output=support::run(support::cli().arg("check").arg(path),60);
  let stderr=String::from_utf8_lossy(&output.stderr);
  assert!(!output.status.success()&&stderr.contains(&format!("error[{expected}]")),"{stderr}");
 }
}

#[test]
fn raw_validation_enforces_native_limits_and_strict_syntax() {
 let source=r#"
import {borrowRaw,JsonLimits} from 'std:json';
import {strictEqual} from 'std:testing';
function rejects(text:string,limits:JsonLimits,code:string):void{
 match(borrowRaw(text,limits)){Ok(raw)=>{throw 'invalid raw input accepted';},Err(error)=>{strictEqual(error.code,code);}}
}
function main(){
 rejects('null',{maxBytes:3,maxDepth:64,maxEntries:100,maxNodes:100},'byte-limit');
 rejects('[]',{maxBytes:100,maxDepth:0,maxEntries:100,maxNodes:100},'depth-limit');
 rejects('[0,1]',{maxBytes:100,maxDepth:64,maxEntries:1,maxNodes:100},'entry-limit');
 rejects('[0]',{maxBytes:100,maxDepth:64,maxEntries:100,maxNodes:1},'node-limit');
 rejects('null',{maxBytes:-1,maxDepth:64,maxEntries:100,maxNodes:100},'invalid-limits');
 rejects('null',{maxBytes:100,maxDepth:65,maxEntries:100,maxNodes:100},'invalid-limits');
 rejects('null true',{maxBytes:100,maxDepth:64,maxEntries:100,maxNodes:100},'trailing-data');
 rejects('/*comment*/null',{maxBytes:100,maxDepth:64,maxEntries:100,maxNodes:100},'invalid-json');
 const source=' \n{"n":18446744073709551615,"text":"\\u0061"} ';
 match(borrowRaw(source)){Ok(raw)=>{},Err(error)=>{throw error.message;}}
 strictEqual(source,' \n{"n":18446744073709551615,"text":"\\u0061"} ');
}
"#;
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
 support::success(&support::run(support::cli().arg("run").arg(path),90));
}
