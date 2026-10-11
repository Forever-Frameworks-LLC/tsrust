mod support;
#[test]
fn jsonl_owned_reader_distinguishes_null_eof_and_errors() {
 let dir=support::TempDirectory::new();let data=dir.0.join("records.jsonl");std::fs::write(&data,"null\r\n42\n").unwrap();
 let source=format!(r#"import {{openJsonl,readJsonl,closeJsonl,kind}} from 'std:json';import {{strictEqual}} from 'std:testing';
 function main(){{match(openJsonl({path:?},{{maxBytes:64,maxDepth:4,maxEntries:8,maxNodes:16}},1)){{Ok(reader)=>{{
  match(readJsonl(reader)){{Ok(value)=>{{if(value===undefined){{throw 'unexpected EOF';}}strictEqual(kind(value),'null');}},Err(error)=>{{throw error.message;}}}}
  match(readJsonl(reader)){{Ok(value)=>{{if(value===undefined){{throw 'unexpected EOF';}}strictEqual(kind(value),'number');}},Err(error)=>{{throw error.message;}}}}
  match(readJsonl(reader)){{Ok(value)=>{{strictEqual(value===undefined,true);}},Err(error)=>{{throw error.message;}}}}
  match(closeJsonl(reader)){{Ok(_)=>{{}},Err(error)=>{{throw error.message;}}}}
 }},Err(error)=>{{throw error.message;}}}} console.log('JSONL passed');}}"#,path=data.to_str().unwrap());
 let path=dir.0.join("main.tsr");std::fs::write(&path,support::format_round_trip(&source)).unwrap();
 for release in [false,true] {let mut command=support::cli();command.arg("run").arg(&path);if release{command.arg("--release");}let output=support::run(&mut command,120);support::success(&output);assert_eq!(String::from_utf8(output.stdout).unwrap(),"JSONL passed\n");}
}

#[test]
fn jsonl_errors_are_terminal_and_limits_validate_before_open() {
 let dir=support::TempDirectory::new();let data=dir.0.join("bad.jsonl");std::fs::write(&data,"0\nsecret\n").unwrap();
 let source=format!(r#"import {{openJsonl,readJsonl,closeJsonl,JsonLimits}} from 'std:json';import {{strictEqual}} from 'std:testing';
 function main(){{const limits:JsonLimits={{maxBytes:16,maxDepth:4,maxEntries:8,maxNodes:16}};
 match(openJsonl({path:?},limits,2)){{Ok(reader)=>{{readJsonl(reader);
  match(readJsonl(reader)){{Ok(_)=>{{throw 'accepted bad record';}},Err(e)=>{{strictEqual(e.line,2);strictEqual(e.offset,2);strictEqual(e.code,'invalid-json');strictEqual(e.operation,'read');strictEqual(e.osCode===undefined,true);}}}}
  match(readJsonl(reader)){{Ok(_)=>{{throw 'lost failure';}},Err(e)=>{{strictEqual(e.line,2);strictEqual(e.offset,2);}}}}
  match(closeJsonl(reader)){{Ok(_)=>{{}},Err(e)=>{{throw e.message;}}}}
 }},Err(e)=>{{throw e.message;}}}}
 match(openJsonl('missing-never-opened',limits,0)){{Ok(_)=>{{throw 'bad capacity accepted';}},Err(e)=>{{strictEqual(e.code,'capacity-range');}}}}
 const invalid:JsonLimits={{maxBytes:16,maxDepth:65,maxEntries:8,maxNodes:16}};
 match(openJsonl('missing-never-opened',invalid)){{Ok(_)=>{{throw 'bad depth accepted';}},Err(e)=>{{strictEqual(e.code,'invalid-limits');}}}}
 match(openJsonl('missing-never-opened')){{Ok(_)=>{{throw 'missing file opened';}},Err(e)=>{{strictEqual(e.code,'not-found');strictEqual(e.operation,'open');}}}}
 console.log('errors passed');}}"#,path=data.to_str().unwrap());
 let path=dir.0.join("main.tsr");std::fs::write(&path,support::format_round_trip(&source)).unwrap();
 let output=support::run(support::cli().arg("run").arg(&path).current_dir(&dir.0),120);support::success(&output);assert_eq!(String::from_utf8(output.stdout).unwrap(),"errors passed\n");
}
#[test]
fn jsonl_reader_rejects_clone_construction_and_use_after_close() {
 let dir=support::TempDirectory::new();let path=dir.0.join("bad.tsr");
 for body in ["clone(reader);","closeJsonl(reader);readJsonl(reader);","const fake:JsonlReader={};"] {
  let source=format!("import {{JsonlReader,readJsonl,closeJsonl}} from 'std:json';function invalid(reader:JsonlReader){{{body}}}");
  std::fs::write(&path,support::format_round_trip(&source)).unwrap();let output=support::run(support::cli().arg("check").arg(&path),20);assert!(!output.status.success(),"accepted {body}");
 }
 // Type-only imports must emit their backend even without a read call.
 std::fs::write(&path,support::format_round_trip("import {JsonlReader,closeJsonl} from 'std:json';function release(reader:JsonlReader){closeJsonl(reader);}function main(){}" )).unwrap();
 support::success(&support::run(support::cli().arg("run").arg(&path),120));
}
