mod support;
use support::session::Session;
#[test]fn compression_exports_hover_and_lazy_signatures_agree(){let source="import {gzip} from 'std:compression';function main(){}";let mut session=Session::start(&[]);session.open("main.tsr",source);session.settle();let hover=session.hover("main.tsr",source,source.find("gzip").unwrap()+1).unwrap();assert!(hover.contains("Task<Result<Bytes")&&hover.contains("Ownership:")&&hover.contains("SpawnError")&&hover.contains("0 through 9"),"{hover}");let imports="import { } from 'std:compression';function main(){}";session.open("imports.tsr",imports);session.settle();let names=session.completion("imports.tsr",imports,"import { ".len()).unwrap();for name in["gzip","gunzip","deflate","inflate","deflateRaw","inflateRaw","CompressionOptions","DecompressionOptions","CompressionError"]{assert!(names.iter().any(|value|value==name),"missing {name}");}assert!(!names.iter().any(|name|name.starts_with("__")));}

#[test]fn stream_constructor_is_synchronous_and_methods_keep_typed_failures(){let source="import {compressReader,CompressionReader as Decoder} from 'std:compression';import {readInto as fill,Writer,copy} from 'std:io';import {Bytes} from 'std:bytes';async function inspect(input:Mutable<Decoder>,destination:Mutable<Bytes>,output:Mutable<Writer>):Task<void>{const pending=fill(input,destination);const result=await pending;console.log(result);const copied=await copy(input,output);console.log(copied);}function main(){}";let mut session=Session::start(&[]);session.open("main.tsr",source);session.settle();let constructor=session.hover("main.tsr",source,source.find("compressReader").unwrap()+1).unwrap();assert!(constructor.contains("Result<CompressionReader")&&!constructor.contains("Task<Result<CompressionReader"),"{constructor}");let hover=session.hover("main.tsr",source,source.find("fill(input").unwrap()+1).unwrap();assert!(hover.contains("CompressionError")&&hover.contains("IoError")&&hover.contains("SpawnError")&&hover.contains("ThreadError<undefined>"),"{hover}");let methods="import {CompressionReader as Decoder} from 'std:compression';async function inspect(input:Mutable<Decoder>):Task<void>{input.}function main(){}";session.open("methods.tsr",methods);session.settle();let names=session.completion("methods.tsr",methods,methods.find("input.}").unwrap()+6).unwrap();for name in["read","readInto","readExact","readAll","finish","close"]{assert!(names.iter().any(|value|value==name),"missing {name}: {names:?}");}}

#[test]
fn copy_signatures_include_full_tcp_and_preserve_compression_failures() {
 let source="import {TcpStream} from 'std:net';import {CompressionReader,CompressionWriter} from 'std:compression';import {copy} from 'std:io';async function inspect(input:Mutable<CompressionReader>,output:Mutable<TcpStream>):Task<void>{console.log(await copy(input,output));}function main(){}";
 let mut session=Session::start(&[]);session.open("mixed.tsr",source);session.settle();
 let hover=session.hover("mixed.tsr",source,source.find("copy(input").unwrap()+1).unwrap();
 for expected in ["TcpStream","CompressionReader","CompressionWriter","CompressionError","IoError","SpawnError","ThreadError<undefined>"] {assert!(hover.contains(expected),"{expected}: {hover}");}
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
 let copy=catalog["std:io"].as_array().unwrap().iter().find(|entry|entry["name"]=="copy").unwrap();
 let signatures=copy["inferredSignature"].as_str().unwrap();assert_eq!(signatures.lines().count(),3);
 assert!(signatures.lines().next().unwrap().ends_with("IoError>>"));
 let close=catalog["std:io"].as_array().unwrap().iter().find(|entry|entry["name"]=="close").unwrap();
 assert!(!close["inferredSignature"].as_str().unwrap().contains("TcpStream"));
}

#[test]
fn declared_compression_fields_have_behavior_examples_and_ownership_docs() {
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
 let mut count=0;
 for name in ["CompressionOptions","DecompressionOptions","CompressionError"] {
  let declaration=tsrust_compiler::native_types::lookup("std:compression",name).unwrap();
  let entry=catalog["std:compression"].as_array().unwrap().iter().find(|entry|entry["name"]==name).unwrap();
  for field in declaration.fields {
   let metadata=&entry["fields"][field.name];let docs=metadata["documentation"].as_str().unwrap_or("");
   for required in ["Ownership:","```tsrust","Output:"] {assert!(docs.contains(required),"{name}.{} missing {required}",field.name);}
   let examples=metadata["examples"].as_array().unwrap();assert!(!examples.is_empty(),"{name}.{}",field.name);
   for example in examples {assert!(!example["source"].as_str().unwrap().is_empty());assert!(example["output"].as_str().is_some());}
   count+=1;
  }
 }
 assert_eq!(count,5);
}
