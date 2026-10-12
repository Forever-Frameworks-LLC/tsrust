mod support;
#[test]
fn guarded_json_strings_infer_borrowed_locals_and_returns() {
 let source=r#"
import {JsonValue,StringView,get} from 'std:json';
import {strictEqual} from 'std:testing';
function text(value:ReadOnly<JsonValue>):StringView{if(typeof value==='string'){return value;}throw 'string required';}
function main(){const owner=JSON.parse('"hello"');if(typeof owner==='string'){const local=owner;strictEqual(local.length,5);strictEqual(String(local),'hello');}const returned=text(owner);strictEqual(returned.includes('ell'),true);strictEqual(JSON.stringify(owner),'"hello"');const object=JSON.parse('{"text":"nested"}');match(get(object,'text')){Ok(child)=>{if(typeof child==='string'){const projected=child;strictEqual(projected.length,6);}},Err(error)=>{throw error.message;}}}
"#;
 let dir=support::TempDirectory::new();let path=dir.source(&support::format_round_trip(source));support::success(&support::run(support::cli().arg("run").arg(path),90));
}
#[test]
fn guarded_json_string_projection_tracks_owner_and_invalidated_proofs() {
 for body in [
 "function bad(){const owner=JSON.parse('\"text\"');if(typeof owner==='string'){return owner;}throw 'wrong';}function main(){}",
 "function main(){let owner=JSON.parse('\"text\"');if(typeof owner==='string'){const view=owner;owner=JSON.parse('null');console.log(view.length);}}",
 "function main(){let owner=JSON.parse('\"text\"');if(typeof owner==='string'){owner=JSON.parse('42');const text:StringView=owner;}}",
 "function main(){let owner=JSON.parse('\"text\"');if(typeof owner==='string'){while(true){const view=owner;console.log(view.length);owner=JSON.parse('42');}}}",
 "function main(){let owner=JSON.parse('\"text\"');if(typeof owner==='string'){}const view:StringView=owner;}",
 ] {let dir=support::TempDirectory::new();let source=format!("import {{StringView}} from 'std:json';{body}");let path=dir.source(&support::format_round_trip(&source));let output=support::run(support::cli().arg("check").arg(path),60);assert!(!output.status.success(),"accepted: {body}");}
}

#[test]
fn string_projection_needs_no_explicit_json_import() {
 let source="function main(){const owner=JSON.parse('\"hello\"');if(typeof owner==='string'){const text=owner;console.log(text.length);}}";
 let dir=support::TempDirectory::new();let path=dir.source(&support::format_round_trip(source));let output=support::run(support::cli().arg("run").arg(path),90);support::success(&output);assert_eq!(output.stdout,b"5\n");
}
