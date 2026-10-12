mod support;
#[test]
fn borrowed_jsonl_reader_preserves_owner_and_unread_records() {
 let dir=support::TempDirectory::new();let data=dir.0.join("records.jsonl");
 std::fs::write(&data,b"1\n2\n3\n").unwrap();
 let source=format!(r#"import {{open,bufferReader,close,read,BufferedReader}} from 'std:fs';
 import {{borrowJsonl,readJsonl,closeJsonl,asInt,JsonlReaderView,JsonlError}} from 'std:json';
 import {{toUtf8}} from 'std:bytes';import {{strictEqual}} from 'std:testing';
 function readerFrom(input:Mutable<BufferedReader>):Result<JsonlReaderView,JsonlError>{{return borrowJsonl(input);}}
 function main(){{match(open({path:?})){{Ok(file)=>{{match(bufferReader(file,8)){{Ok(input)=>{{
  {{match(readerFrom(input)){{Ok(reader)=>{{
   match(readJsonl(reader)){{Ok(value)=>{{if(value!==undefined){{match(asInt(value)){{Ok(n)=>{{strictEqual(n,1);}},Err(e)=>{{throw e.message;}}}}}}else{{throw 'early EOF';}}}},Err(e)=>{{throw e.message;}}}}
   closeJsonl(reader);
  }},Err(e)=>{{throw e.message;}}}}}}
  match(read(input,2)){{Ok(bytes)=>{{match(toUtf8(bytes)){{Ok(text)=>{{strictEqual(text,'2\n');}},Err(e)=>{{throw e.message;}}}}}},Err(e)=>{{throw e.message;}}}}
  close(input);
 }},Err(e)=>{{throw e.message;}}}}}},Err(e)=>{{throw e.message;}}}}console.log('borrowed passed');}}"#,path=data.to_str().unwrap());
 let path=dir.0.join("main.tsr");std::fs::write(&path,support::format_round_trip(&source)).unwrap();
 let output=support::run(support::cli().arg("run").arg(&path).current_dir(&dir.0),120);support::success(&output);assert_eq!(String::from_utf8(output.stdout).unwrap(),"borrowed passed\n");
}
#[test]
fn borrowed_jsonl_typed_reads_and_aliases_preserve_owner() {
 let dir=support::TempDirectory::new();let data=dir.0.join("records.jsonl");std::fs::write(&data,b"7\nnull\n").unwrap();
 let source=format!(r#"import {{open,bufferReader,close}} from 'std:fs';import {{borrowJsonl,readJsonl as next,closeJsonl}} from 'std:json';import {{strictEqual}} from 'std:testing';
 function main(){{match(open({path:?})){{Ok(file)=>{{match(bufferReader(file,1)){{Ok(input)=>{{
  {{match(borrowJsonl(input)){{Ok(reader)=>{{match(next<int>(reader)){{Ok(value)=>{{strictEqual(value,7);}},Err(e)=>{{throw e.message;}}}}match(next<null>(reader)){{Ok(value)=>{{strictEqual(value===null,true);}},Err(e)=>{{throw e.message;}}}}match(next<int>(reader)){{Ok(value)=>{{strictEqual(value===undefined,true);}},Err(e)=>{{throw e.message;}}}}closeJsonl(reader);}},Err(e)=>{{throw e.message;}}}}}}
  close(input);
 }},Err(e)=>{{throw e.message;}}}}}},Err(e)=>{{throw e.message;}}}}console.log('typed borrowed passed');}}"#,path=data.to_str().unwrap());
 let path=dir.0.join("main.tsr");std::fs::write(&path,support::format_round_trip(&source)).unwrap();
 let output=support::run(support::cli().arg("run").arg(&path).current_dir(&dir.0),120);support::success(&output);assert_eq!(String::from_utf8(output.stdout).unwrap(),"typed borrowed passed\n");
}
#[test]
fn borrowed_jsonl_rejects_owner_conflicts_escape_clone_and_reuse() {
 let dir=support::TempDirectory::new();let path=dir.0.join("bad.tsr");
 for body in ["read(input,1);readJsonl(reader);","close(input);readJsonl(reader);","clone(reader);","closeJsonl(reader);readJsonl(reader);","const fake:JsonlReaderView={};"] {
  let source=format!("import {{BufferedReader,read,close}} from 'std:fs';import {{JsonlReaderView,borrowJsonl,readJsonl,closeJsonl}} from 'std:json';function invalid(input:BufferedReader){{match(borrowJsonl(input)){{Ok(reader)=>{{{body}}},Err(e)=>{{throw e.message;}}}}}}");
  std::fs::write(&path,support::format_round_trip(&source)).unwrap();let output=support::run(support::cli().arg("check").arg(&path),20);assert!(!output.status.success(),"accepted {body}");assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));
 }
 let source="import {open,bufferReader} from 'std:fs';import {JsonlReaderView,JsonlError,borrowJsonl} from 'std:json';function escape():Result<JsonlReaderView,JsonlError>{match(open('input.jsonl')){Ok(file)=>{match(bufferReader(file)){Ok(input)=>{return borrowJsonl(input);},Err(e)=>{throw e.message;}}},Err(e)=>{throw e.message;}}}";
 std::fs::write(&path,support::format_round_trip(source)).unwrap();let output=support::run(support::cli().arg("check").arg(&path),20);assert!(!output.status.success(),"accepted local-owner escape");
}
#[test]
fn borrowed_jsonl_catalog_example_executes() {
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
 let entry=catalog["std:json"].as_array().unwrap().iter().find(|entry|entry["name"]=="borrowJsonl").unwrap();
 let example=&entry["examples"][0];let dir=support::TempDirectory::new();let path=dir.0.join("example.tsr");std::fs::write(&path,support::format_round_trip(example["source"].as_str().unwrap())).unwrap();
 let output=support::run(support::cli().arg("run").arg(&path).current_dir(&dir.0),120);support::success(&output);assert_eq!(String::from_utf8(output.stdout).unwrap(),example["output"].as_str().unwrap());
}
#[test]
fn borrowed_jsonl_positions_limits_and_terminal_errors_preserve_owner() {
 let dir=support::TempDirectory::new();let data=dir.0.join("records.jsonl");std::fs::write(&data,b"1\nbad\n3\n").unwrap();
 let source=format!(r#"import {{open,bufferReader,read,close}} from 'std:fs';import {{borrowJsonl,readJsonl,closeJsonl,JsonLimits}} from 'std:json';import {{toUtf8}} from 'std:bytes';import {{strictEqual}} from 'std:testing';
 function main(){{match(open({path:?})){{Ok(file)=>{{match(bufferReader(file,8)){{Ok(input)=>{{
  read(input,2);const invalid:JsonLimits={{maxBytes:64,maxDepth:65,maxEntries:8,maxNodes:8}};
  {{match(borrowJsonl(input,invalid)){{Ok(_)=>{{throw 'invalid limits accepted';}},Err(e)=>{{strictEqual(e.code,'invalid-limits');}}}}}}
  {{match(borrowJsonl(input)){{Ok(reader)=>{{
   match(readJsonl(reader)){{Ok(_)=>{{throw 'bad record accepted';}},Err(e)=>{{strictEqual(e.line,1);strictEqual(e.offset,0);strictEqual(e.code,'invalid-json');}}}}
   match(readJsonl(reader)){{Ok(_)=>{{throw 'terminal error lost';}},Err(e)=>{{strictEqual(e.line,1);strictEqual(e.offset,0);}}}}
   closeJsonl(reader);
  }},Err(e)=>{{throw e.message;}}}}}}
  match(read(input,2)){{Ok(bytes)=>{{match(toUtf8(bytes)){{Ok(text)=>{{strictEqual(text,'3\n');}},Err(e)=>{{throw e.message;}}}}}},Err(e)=>{{throw e.message;}}}}close(input);
 }},Err(e)=>{{throw e.message;}}}}}},Err(e)=>{{throw e.message;}}}}console.log('borrowed errors passed');}}"#,path=data.to_str().unwrap());
 let path=dir.0.join("main.tsr");std::fs::write(&path,support::format_round_trip(&source)).unwrap();let output=support::run(support::cli().arg("run").arg(&path).current_dir(&dir.0),120);support::success(&output);assert_eq!(String::from_utf8(output.stdout).unwrap(),"borrowed errors passed\n");
}
