mod support;

#[test]
fn jsonc_raw_and_jsonl_defaults_accept_rust_sized_values_and_keep_explicit_limits(){
 let dir=support::TempDirectory::new();let input=dir.0.join("large.jsonl");let owned=dir.0.join("owned.jsonl");let borrowed=dir.0.join("borrowed.jsonl");
 let text="x".repeat(17*1024*1024);std::fs::write(&input,format!("\"{text}\"\n")).unwrap();
 let source=format!(r#"
import {{parseJsonc,borrowRaw,JsonLimits,openJsonl,readJsonl,closeJsonl,borrowJsonl,createJsonl,writeJsonl,closeJsonlWriter,borrowJsonlWriter}} from 'std:json';
import {{open,create,bufferReader,bufferWriter,close}} from 'std:fs';
import {{strictEqual}} from 'std:testing';
type Node={{next?:Node;}};
function main(){{
 const source='/* comment */'+ '{{"next":'.repeat(100)+'{{}}'+'}}'.repeat(100);
 match(parseJsonc(source)){{Ok(value)=>{{strictEqual(JSON.stringify(value).startsWith('{{"next":'),true);}},Err(error)=>{{throw error.message;}}}}
 match(parseJsonc<Node>(source,false,undefined)){{Ok(value)=>{{strictEqual(JSON.stringify(value).startsWith('{{"next":'),true);}},Err(error)=>{{throw error.message;}}}}
 const text='"'+'x'.repeat(17*1024*1024)+'"';
 match(parseJsonc<string>(text)){{Ok(value)=>{{strictEqual(value.length,17*1024*1024);}},Err(error)=>{{throw error.message;}}}}
 match(parseJsonc(text,false,undefined)){{Ok(value)=>{{strictEqual(JSON.stringify(value),text);}},Err(error)=>{{throw error.message;}}}}
 match(borrowRaw(text,undefined)){{Ok(raw)=>{{strictEqual(JSON.stringify(raw),text);}},Err(error)=>{{throw error.message;}}}}
 const deep='['.repeat(100)+'0'+']'.repeat(100);
 match(borrowRaw(deep)){{Ok(raw)=>{{strictEqual(JSON.stringify(raw),deep);}},Err(error)=>{{throw error.message;}}}}
 const limits:JsonLimits={{maxBytes:1000,maxDepth:65,maxEntries:1,maxNodes:66}};
 const exact='['.repeat(65)+'0'+']'.repeat(65);
 match(borrowRaw(exact,limits)){{Ok(raw)=>{{strictEqual(JSON.stringify(raw),exact);}},Err(error)=>{{throw error.message;}}}}
 match(parseJsonc(exact,false,limits)){{Ok(_)=>{{}},Err(error)=>{{throw error.message;}}}}
 match(parseJsonc(text,false,limits)){{Ok(_)=>{{throw 'byte limit ignored';}},Err(error)=>{{strictEqual(error.code,'byte-limit');strictEqual(error.offset,1000);}}}}
 const many='['+'0,'.repeat(1000000)+'0]';
 match(parseJsonc(many)){{Ok(_)=>{{}},Err(error)=>{{throw error.message;}}}}
 match(borrowRaw(many)){{Ok(_)=>{{}},Err(error)=>{{throw error.message;}}}}
 match(openJsonl({input:?},undefined,1048577)){{Ok(reader)=>{{
  match(readJsonl(reader)){{Ok(value)=>{{if(value===undefined){{throw 'missing dynamic record';}}strictEqual(JSON.stringify(value),text);}},Err(error)=>{{throw error.message;}}}}
  closeJsonl(reader);
 }},Err(error)=>{{throw error.message;}}}}
 match(open({input:?})){{Ok(file)=>{{match(bufferReader(file,1048577)){{Ok(input)=>{{
  {{match(borrowJsonl(input,undefined)){{Ok(reader)=>{{match(readJsonl<string>(reader)){{Ok(value)=>{{strictEqual(value!.length,17*1024*1024);}},Err(error)=>{{throw error.message;}}}}closeJsonl(reader);}},Err(error)=>{{throw error.message;}}}}}}
  close(input);
 }},Err(error)=>{{throw error.message;}}}}}},Err(error)=>{{throw error.message;}}}}
 const payload='x'.repeat(65*1024*1024);
 match(createJsonl({owned:?},undefined,1048577)){{Ok(writer)=>{{match(writeJsonl(writer,payload)){{Ok(_)=>{{}},Err(error)=>{{throw error.message;}}}}match(closeJsonlWriter(writer)){{Ok(_)=>{{}},Err(error)=>{{throw error.message;}}}}}},Err(error)=>{{throw error.message;}}}}
 match(create({borrowed:?})){{Ok(file)=>{{match(bufferWriter(file,1048577)){{Ok(output)=>{{
  {{match(borrowJsonlWriter(output,undefined)){{Ok(writer)=>{{match(writeJsonl(writer,payload)){{Ok(_)=>{{}},Err(error)=>{{throw error.message;}}}}match(closeJsonlWriter(writer)){{Ok(_)=>{{}},Err(error)=>{{throw error.message;}}}}}},Err(error)=>{{throw error.message;}}}}}}
  close(output);
 }},Err(error)=>{{throw error.message;}}}}}},Err(error)=>{{throw error.message;}}}}
}}
"#,input=input.to_str().unwrap(),owned=owned.to_str().unwrap(),borrowed=borrowed.to_str().unwrap());
 let path=dir.source(&support::format_round_trip(&source));support::success(&support::run(support::cli().arg("run").arg(path),180));
 let payload="x".repeat(65*1024*1024);let expected=format!("\"{payload}\"\n");assert_eq!(std::fs::read_to_string(owned).unwrap(),expected);assert_eq!(std::fs::read_to_string(borrowed).unwrap(),expected);
}

#[test]
fn default_jsonc_retains_strict_errors_and_evaluates_arguments_once(){
 let source=r#"
import {parseJsonc,borrowRaw} from 'std:json';import {strictEqual} from 'std:testing';
function input(log:int[]):string{log.push(1);return '/*comment*/[1,]';}
function flag(log:int[]):boolean{log.push(2);return true;}
function absent(log:int[]):undefined{log.push(3);return undefined;}
function main(){
 const log:int[]=[];
 match(parseJsonc<int[]>(input(log),flag(log),absent(log))){Ok(value)=>{strictEqual(value.join(','),'1');},Err(error)=>{throw error.message;}}
 strictEqual(log.join(','),'1,2,3');
 for(const text of ['[1,]','{"ignored":"\\ud800"}','1e400']){
  match(parseJsonc(text)){Ok(_)=>{throw 'invalid JSONC accepted';},Err(error)=>{strictEqual(error.code,'invalid-json');}}
  match(parseJsonc<int[]>(text)){Ok(_)=>{throw 'invalid typed JSONC accepted';},Err(error)=>{strictEqual(error.code,'invalid-json');}}
  match(borrowRaw(text)){Ok(_)=>{throw 'invalid raw accepted';},Err(error)=>{strictEqual(error.code,'invalid-json');}}
 }
}
"#;
 let dir=support::TempDirectory::new();let path=dir.source(&support::format_round_trip(source));support::success(&support::run(support::cli().arg("run").arg(path),90));
}
