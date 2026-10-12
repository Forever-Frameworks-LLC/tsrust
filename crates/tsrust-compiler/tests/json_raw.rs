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

#[test]
fn raw_record_fields_preserve_source_borrows_through_returns() {
 let source=r#"
import {borrowRaw,JsonRaw} from 'std:json';
import {strictEqual} from 'std:testing';
type Envelope={payload:JsonRaw;tag:int;};
function wrap(source:ReadOnly<string>):Envelope{
 match(borrowRaw(source)){Ok(raw)=>{return {payload:raw,tag:7};},Err(error)=>{throw error.message;}}
}
function inspect(value:ReadOnly<Envelope>):int{return value.tag;}
function main(){
 let source='{"value":7}';
 {const value=wrap(source);strictEqual(inspect(value),7);}
 source='null';strictEqual(source,'null');
}
"#;
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
 support::success(&support::run(support::cli().arg("run").arg(path),90));
}

#[test]
fn raw_record_fields_reject_owner_invalidation_and_escape() {
 for (body,expected) in [
  ("function main(){let source='null';const value=wrap(source);source='1';inspect(value);}","native-view-conflict"),
  ("function bad():Envelope{const source='null';return wrap(source);}function main(){}","native-view-escape"),
  ("function main(){const source='null';const value=wrap(source);clone(value);}","clone-type"),
 ] {
  let source=format!("import {{borrowRaw,JsonRaw}} from 'std:json';type Envelope={{payload:JsonRaw;tag:int;}};function wrap(source:ReadOnly<string>):Envelope{{match(borrowRaw(source)){{Ok(raw)=>{{return {{payload:raw,tag:7}};}},Err(error)=>{{throw error.message;}}}}}}function inspect(value:ReadOnly<Envelope>):int{{return value.tag;}}{body}");
  let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(&source));
  let output=support::run(support::cli().arg("check").arg(path),60);
  let stderr=String::from_utf8_lossy(&output.stderr);
  assert!(!output.status.success()&&stderr.contains(&format!("error[{expected}]")),"{stderr}");
 }
}

#[test]
fn typed_json_decodes_borrowed_raw_fields() {
 let source=r#"
import {JsonRaw} from 'std:json';
import {strictEqual} from 'std:testing';
type Envelope={payload:JsonRaw;tag:int;};
function main(){
 const source='{"payload":{"exact":18446744073709551615},"tag":7}';
 const value=JSON.parse<Envelope>(source);
 strictEqual(value.tag,7);
}
"#;
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
 support::success(&support::run(support::cli().arg("run").arg(path),90));
}

#[test]
fn typed_raw_fields_keep_owner_provenance() {
 for (body,expected) in [
  ("function main(){let source='{\"payload\":null}';const value=JSON.parse<Envelope>(source);source='{}';consume(value);}","native-view-conflict"),
  ("function bad():Envelope{const source='{\"payload\":null}';return JSON.parse<Envelope>(source);}function main(){}","native-view-escape"),
  ("function main(){const source='{\"payload\":null}';const value=JSON.parse<Envelope>(source);clone(value);}","clone-type"),
 ] {
  let source=format!("import {{JsonRaw}} from 'std:json';type Envelope={{payload:JsonRaw;}};function consume(value:ReadOnly<Envelope>){{}}{body}");
  let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(&source));
  let output=support::run(support::cli().arg("check").arg(path),60);let stderr=String::from_utf8_lossy(&output.stderr);
  assert!(!output.status.success()&&stderr.contains(&format!("error[{expected}]")),"{stderr}");
 }
}

#[test]
fn typed_raw_arrays_optional_fields_and_failed_loans_execute() {
 let source=r#"
import {JsonRaw} from 'std:json';
import {strictEqual} from 'std:testing';
type Envelope={payload:JsonRaw[];extra?:JsonRaw;tag:int;};
function read(source:ReadOnly<string>):Envelope{return JSON.parse<Envelope>(source);}
function main(){
 let source='{"payload":[null,{"n":7}],"tag":9}';
 {const value=read(source);strictEqual(value.tag,9);strictEqual(value.payload.length,2);}
 source='{}';
 match(JSON.tryParse<Envelope>(source)){Ok(value)=>{throw 'missing fields accepted';},Err(error)=>{strictEqual(error.code,'shape-mismatch');}}
 source='null';strictEqual(source,'null');
}
"#;
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
 support::success(&support::run(support::cli().arg("run").arg(path),90));
}
