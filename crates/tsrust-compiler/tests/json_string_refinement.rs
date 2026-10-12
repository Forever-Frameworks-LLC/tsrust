mod support;
#[test]
fn json_string_guard_enables_borrowed_length_and_search() {
 let source=r#"
import {strictEqual} from 'std:testing';
import {get} from 'std:json';
function main(){
 const value=JSON.parse('"é🦀é"');
 if(typeof value==='string'){
  strictEqual(value.length,8);
  strictEqual(value.startsWith('é'),true);
  strictEqual(value.endsWith('é'),true);
  strictEqual(value.includes('🦀'),true);
  strictEqual(value.indexOf('é',2),6);
  strictEqual(value.lastIndexOf('é'),6);
  const needle=JSON.parse('"é"');if(typeof needle==='string'){strictEqual(value.includes(needle),true);}
 }
 const owner=JSON.parse('{"text":"hello"}');
 match(get(owner,'text')){Ok(child)=>{if(typeof child==='string'){strictEqual(child.length,5);strictEqual(child.includes('ell'),true);}},Err(error)=>{throw error.message;}}
 strictEqual(JSON.stringify(value),'"é🦀é"');
}
"#;
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));support::success(&support::run(support::cli().arg("run").arg(path),90));
}

#[test]
fn json_string_reads_reject_missing_or_invalidated_proofs() {
 for body in [
  "value.includes('x');",
  "if(typeof value==='number'){value.includes('x');}",
  "if(typeof value==='string'){value=JSON.parse('42');value.includes('x');}",
  "if(typeof value==='string'){value.length=0;}",
  "if(typeof value==='string'){while(true){value.includes('x');value=JSON.parse('42');}}",
 ] {
  let source=format!("function main(){{let value=JSON.parse('\"hello\"');{body}}}");
  let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(&source));let output=support::run(support::cli().arg("check").arg(path),60);let diagnostics=String::from_utf8_lossy(&output.stderr);
  assert!(!output.status.success()&&diagnostics.contains("error[")&&!diagnostics.contains("panicked"),"{body}: {diagnostics}");
 }
}
