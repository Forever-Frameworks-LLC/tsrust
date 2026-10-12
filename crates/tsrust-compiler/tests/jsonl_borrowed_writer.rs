mod support;
#[test]
fn borrowed_jsonl_writer_flushes_and_leaves_owner_usable() {
 let dir=support::TempDirectory::new();let output=dir.0.join("out.jsonl");
 let source=format!(r#"import {{create,bufferWriter,writeAll,close,BufferedWriter}} from 'std:fs';import {{fromUtf8}} from 'std:bytes';import {{JsonlWriterView,JsonlError,borrowJsonlWriter,writeJsonl as write,flushJsonl,closeJsonlWriter}} from 'std:json';import {{strictEqual}} from 'std:testing';
 function writerFrom(output:Mutable<BufferedWriter>):Result<JsonlWriterView,JsonlError>{{return borrowJsonlWriter(output,64);}}
 type Item={{name:string;}};function main(){{const item:Item={{name:'hello'}};
 match(create({path:?})){{Ok(file)=>{{match(bufferWriter(file,64)){{Ok(output)=>{{
  {{match(writerFrom(output)){{Ok(writer)=>{{match(write(writer,item)){{Ok(_)=>{{}},Err(e)=>{{throw e.message;}}}}match(write<int>(writer,7)){{Ok(_)=>{{}},Err(e)=>{{throw e.message;}}}}match(flushJsonl(writer)){{Ok(_)=>{{}},Err(e)=>{{throw e.message;}}}}match(closeJsonlWriter(writer)){{Ok(_)=>{{}},Err(e)=>{{throw e.message;}}}}}},Err(e)=>{{throw e.message;}}}}}}
  strictEqual(item.name,'hello');match(fromUtf8('tail\n')){{Ok(bytes)=>{{writeAll(output,bytes);}},Err(e)=>{{throw e.message;}}}}close(output);
 }},Err(e)=>{{throw e.message;}}}}}},Err(e)=>{{throw e.message;}}}}}}"#,path=output.to_str().unwrap());
 let path=dir.source(&support::format_round_trip(&source));support::success(&support::run(support::cli().arg("run").arg(path),90));assert_eq!(std::fs::read_to_string(output).unwrap(),"{\"name\":\"hello\"}\n7\ntail\n");
}
#[test]
fn borrowed_jsonl_writer_bounds_terminal_errors_and_drop_flush() {
 let dir=support::TempDirectory::new();let output=dir.0.join("out.jsonl");
 let source=format!(r#"import {{create,bufferWriter,writeAll,readFile,close}} from 'std:fs';import {{fromUtf8,toUtf8}} from 'std:bytes';import {{borrowJsonlWriter,writeJsonl,closeJsonlWriter}} from 'std:json';import {{strictEqual}} from 'std:testing';
 function main(){{match(create({path:?})){{Ok(file)=>{{match(bufferWriter(file,64)){{Ok(output)=>{{
  {{match(borrowJsonlWriter(output,-1)){{Ok(_)=>{{throw 'invalid limit accepted';}},Err(e)=>{{strictEqual(e.code,'invalid-limits');}}}}}}
  {{match(borrowJsonlWriter(output,1)){{Ok(writer)=>{{writeJsonl(writer,1);}},Err(e)=>{{throw e.message;}}}}}}
  match(readFile({path:?})){{Ok(bytes)=>{{match(toUtf8(bytes)){{Ok(text)=>{{strictEqual(text,'1\n');}},Err(e)=>{{throw e.message;}}}}}},Err(e)=>{{throw e.message;}}}}
  {{match(borrowJsonlWriter(output,2)){{Ok(writer)=>{{
   match(writeJsonl(writer,'long')){{Ok(_)=>{{throw 'overlimit accepted';}},Err(e)=>{{strictEqual(e.code,'output-limit');strictEqual(e.line,1);strictEqual(e.offset,0);}}}}
   match(writeJsonl(writer,1)){{Ok(_)=>{{throw 'terminal error lost';}},Err(e)=>{{strictEqual(e.code,'output-limit');}}}}
   match(closeJsonlWriter(writer)){{Ok(_)=>{{throw 'close lost failure';}},Err(e)=>{{strictEqual(e.code,'output-limit');}}}}
  }},Err(e)=>{{throw e.message;}}}}}}
  match(fromUtf8('tail\n')){{Ok(bytes)=>{{writeAll(output,bytes);}},Err(e)=>{{throw e.message;}}}}close(output);
 }},Err(e)=>{{throw e.message;}}}}}},Err(e)=>{{throw e.message;}}}}}}"#,path=output.to_str().unwrap());
 let path=dir.source(&support::format_round_trip(&source));support::success(&support::run(support::cli().arg("run").arg(path),90));assert_eq!(std::fs::read_to_string(output).unwrap(),"1\ntail\n");
}
#[test]
fn borrowed_jsonl_writer_rejects_owner_conflicts_escape_clone_and_reuse() {
 let dir=support::TempDirectory::new();let path=dir.0.join("bad.tsr");
 for body in ["flush(output);writeJsonl(writer,1);","close(output);writeJsonl(writer,1);","clone(writer);","closeJsonlWriter(writer);writeJsonl(writer,1);","const fake:JsonlWriterView={};"] {
  let source=format!("import {{BufferedWriter,flush,close}} from 'std:fs';import {{JsonlWriterView,borrowJsonlWriter,writeJsonl,closeJsonlWriter}} from 'std:json';function invalid(output:BufferedWriter){{match(borrowJsonlWriter(output)){{Ok(writer)=>{{{body}}},Err(e)=>{{throw e.message;}}}}}}");
  std::fs::write(&path,support::format_round_trip(&source)).unwrap();let output=support::run(support::cli().arg("check").arg(&path),20);assert!(!output.status.success(),"accepted {body}");assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));
 }
 let source="import {create,bufferWriter} from 'std:fs';import {JsonlWriterView,JsonlError,borrowJsonlWriter} from 'std:json';function escape():Result<JsonlWriterView,JsonlError>{match(create('out.jsonl')){Ok(file)=>{match(bufferWriter(file)){Ok(output)=>{return borrowJsonlWriter(output);},Err(e)=>{throw e.message;}}},Err(e)=>{throw e.message;}}}";
 std::fs::write(&path,support::format_round_trip(source)).unwrap();let output=support::run(support::cli().arg("check").arg(&path),20);assert!(!output.status.success(),"accepted local-owner escape");
}
#[test]
fn borrowed_jsonl_writer_catalog_example_executes() {
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();let entry=catalog["std:json"].as_array().unwrap().iter().find(|entry|entry["name"]=="borrowJsonlWriter").unwrap();let example=&entry["examples"][0];
 let dir=support::TempDirectory::new();let path=dir.source(&support::format_round_trip(example["source"].as_str().unwrap()));let output=support::run(support::cli().arg("run").arg(path).current_dir(&dir.0),90);support::success(&output);assert!(output.stdout.is_empty());assert_eq!(std::fs::read(dir.0.join("borrowed-jsonl-output.jsonl")).unwrap(),b"7\n");
}
