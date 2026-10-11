mod support;
use tsrust_compiler::project::{FileSourceProvider,load_project,check_graph};
use tsrust_compiler::project_session::ProjectSession;
fn write(root:&std::path::Path,name:&str,source:&str){std::fs::write(root.join(name),support::format_round_trip(source)).unwrap();}
fn compare(session:&mut ProjectSession,entry:&str) {
 let cached=session.check(&FileSourceProvider,entry);
 let clean=load_project(&FileSourceProvider,entry).and_then(check_graph);
 match (cached,clean) {
  (Ok(a),Ok(b))=>assert_eq!(a.emit_rust(),b.emit_rust()),
  (Err(a),Err(b))=>{assert_eq!(a.code,b.code);assert_eq!(a.message,b.message);assert_eq!(a.location,b.location);},
  _=>panic!("cached and clean checking disagree"),
 }
}
#[test]
fn unchanged_modules_reuse_syntax_and_dependents_invalidate() {
 let dir=support::TempDirectory::new();let root=&dir.0;
 write(root,"left.tsr","export function left():int{return 1;}");
 write(root,"right.tsr","export function right():int{return 2;}");
 write(root,"main.tsr","import {left} from './left.tsr';import {right} from './right.tsr';function main(){console.log(left()+right());}");
 let entry=root.join("main.tsr");let entry=entry.to_str().unwrap();let mut session=ProjectSession::default();
 compare(&mut session,entry);assert_eq!(session.stats().parsed,3);
 compare(&mut session,entry);assert_eq!(session.stats().reused,3);
 write(root,"left.tsr","export function left():int{return 100;}");
 compare(&mut session,entry);assert_eq!(session.stats().parsed,2);assert_eq!(session.stats().reused,1);
 write(root,"left.tsr","export function left():int{return 'wrong';}");compare(&mut session,entry);
 write(root,"left.tsr","export function left():int{return 3;}");compare(&mut session,entry);
 write(root,"main.tsr","import {right} from './right.tsr';function main(){console.log(right());}");
 std::fs::remove_file(root.join("left.tsr")).unwrap();compare(&mut session,entry);assert_eq!(session.stats().retained,2);
}
#[test]
fn exported_types_and_default_imports_match_clean_builds_after_edits() {
 let dir=support::TempDirectory::new();let root=&dir.0;
 write(root,"dep.tsr","export type Value = int;export default function value():Value{return 7;}");
 write(root,"main.tsr","import value,{Value} from './dep.tsr';function main(){const x:Value=value();console.log(x);}");
 let entry=root.join("main.tsr");let entry=entry.to_str().unwrap();let mut session=ProjectSession::default();compare(&mut session,entry);compare(&mut session,entry);
 write(root,"dep.tsr","export type Value = string;export default function value():Value{return 'seven';}");compare(&mut session,entry);assert_eq!(session.stats().parsed,2);
 write(root,"dep.tsr","export function value():int{return 9;}");compare(&mut session,entry);
}
#[test]
fn cache_preserves_recursive_views_resource_cleanup_and_test_registration() {
 for relative in ["tests/fixtures/native-byte-views/recursive-arrays.tsr","tests/fixtures/native-files/cleanup.tsr"] {
  let entry=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);let entry=entry.to_str().unwrap();
  let mut session=ProjectSession::default();assert!(session.check(&FileSourceProvider,entry).is_ok());compare(&mut session,entry);compare(&mut session,entry);assert!(session.stats().reused>0);
 }
 let dir=support::TempDirectory::new();let root=&dir.0;
 write(root,"sample.test.tsr","import {test,strictEqual} from 'std:testing';test('value',()=>{strictEqual(1,1);});");
 let entry=root.join("sample.test.tsr");let entry=entry.to_str().unwrap();let mut session=ProjectSession::default();assert!(session.check(&FileSourceProvider,entry).is_ok());compare(&mut session,entry);compare(&mut session,entry);assert!(session.stats().reused>0);
}
