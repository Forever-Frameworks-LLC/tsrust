//! Public buffer ownership, typed effects and generated native behavior.
mod support;
#[path="support/compression_suite_reports.rs"] mod compression_suite_reports;
use tsrust_compiler::project::{FileSourceProvider,load_project,check_graph};
fn check(source:&str)->Result<String,String>{let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));let graph=load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap();check_graph(graph).map(|checked|{assert!(checked.warnings().is_empty());let rust=checked.emit_rust();fn safe(tokens:proc_macro2::TokenStream)->bool{tokens.into_iter().all(|token|match token{proc_macro2::TokenTree::Ident(name)=>name!="unsafe",proc_macro2::TokenTree::Group(group)=>safe(group.stream()),_=>true})}assert!(safe(rust.parse().unwrap()));rust}).map_err(|failure|failure.code.into())}
#[test]fn six_buffer_operations_preserve_results_and_execute(){let source=include_str!("../../../examples/compression/buffers.tsr");let rust=check(source).unwrap();assert!(rust.contains("tsrust_native_compression::run"));compression_suite_reports::assert_registered_source("six_buffer_operations_preserve_results_and_execute",source);}
#[test]fn lazy_buffers_move_input_but_do_not_borrow_or_own_pool(){let prefix="import {gzip} from 'std:compression';import {ThreadPool} from 'std:threads';import {fromUtf8,length} from 'std:bytes';";assert!(check(&format!("{prefix}async function main():Task<void>{{const pool=new ThreadPool({{workers:1,capacity:1}});const bytes=unwrap(fromUtf8('owned'));const pending=gzip(pool,bytes);const shutdown=pool.shutdown();const result=await pending;unwrap(shutdown);match(result){{Ok(bytes)=>{{console.log(length(bytes));}},Err(error)=>{{console.log(error.code);}}}}}}")).is_ok());assert!(check(&format!("{prefix}async function main():Task<void>{{const pool=new ThreadPool({{workers:1,capacity:1}});const bytes=unwrap(fromUtf8('owned'));const pending=gzip(pool,bytes);console.log(length(bytes));await pending;}}")).is_err());}
#[test]fn owned_buffers_required_and_operation_values_remain_checked(){assert!(check("import {gzip} from 'std:compression';function main(){const operation=gzip;}").is_ok());assert!(check("import {gzip} from 'std:compression';import {ThreadPool} from 'std:threads';import {fromUtf8,view} from 'std:bytes';async function main():Task<void>{const pool=new ThreadPool({workers:1,capacity:1});const bytes=unwrap(fromUtf8('borrowed'));const part=unwrap(view(bytes,0,2));await gzip(pool,part);}").is_err());}
#[test]fn unused_compression_import_has_no_runtime_activation(){let rust=check("import {gzip,CompressionOptions} from 'std:compression';function main(){console.log(1);}").unwrap();assert!(!rust.contains("mod tsrust_native_compression"));}
#[test]fn buffer_catalog_examples_execute(){compression_suite_reports::assert_sources();let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();let mut seen=std::collections::HashSet::new();for entry in catalog["std:compression"].as_array().unwrap(){for example in entry["examples"].as_array().into_iter().flatten().chain(entry["fields"].as_object().into_iter().flat_map(|fields|fields.values()).flat_map(|field|field["examples"].as_array().into_iter().flatten())){let source=example["source"].as_str().unwrap();if seen.insert(source.to_owned()){check(source).unwrap();}}}}
#[test]fn typed_stream_examples_execute_with_one_pool_and_explicit_finish(){for(source,expected)in[(include_str!("../../../examples/compression/readers.tsr"),"0\nhello\n0 0\n"),(include_str!("../../../examples/compression/writers.tsr"),"2\nhello\n"),(include_str!("../../../examples/compression/pipe.tsr"),"17\none pool pipeline\n")]{check(source).unwrap();assert_eq!(compression_suite_reports::assert_registered_source("typed_stream_examples_execute_with_one_pool_and_explicit_finish",source),expected);}}
#[test]fn stream_tasks_retain_owner_and_destination_loans(){let prefix="import {decompressReader,gzip} from 'std:compression';import {ThreadPool} from 'std:threads';import {reader,readInto,close} from 'std:io';import {fromUtf8,alloc,set} from 'std:bytes';";for action in["unwrap(set(destination,0,1));","unwrap(await close(input));","const second=readInto(input,destination);await second;"]{let source=format!("{prefix}async function main():Task<void>{{const pool=new ThreadPool({{workers:1,capacity:1}});const bytes=unwrap(await gzip(pool,unwrap(fromUtf8('owned'))));const input=unwrap(decompressReader(pool,reader(bytes),'gzip'));const destination=unwrap(alloc(5));const pending=readInto(input,destination);{action}await pending;}}");assert!(check(&source).is_err(),"premature loan release: {action}");}}
#[test]fn consumed_stream_endpoint_and_invalid_format_are_rejected(){for source in["import {compressReader} from 'std:compression';import {ThreadPool} from 'std:threads';import {reader,close} from 'std:io';import {fromUtf8} from 'std:bytes';async function main():Task<void>{const pool=new ThreadPool({workers:1,capacity:1});const source=reader(unwrap(fromUtf8('owned')));const encoded=unwrap(compressReader(pool,source,'gzip'));unwrap(await close(source));unwrap(await encoded.close());}","import {compressReader} from 'std:compression';import {ThreadPool} from 'std:threads';import {reader} from 'std:io';import {fromUtf8} from 'std:bytes';function main(){const pool=new ThreadPool({workers:1,capacity:1});compressReader(pool,reader(unwrap(fromUtf8('x'))),'brotli');}"]{assert!(check(source).is_err(),"accepted invalid endpoint/format");}}

#[test]fn all_b05_copy_endpoint_pairs_keep_explicit_finish_and_error_types(){let source=r#"import {gzip,compressWriter,decompressReader,CompressionReader,CompressionError} from 'std:compression';import {ThreadPool,SpawnError,ThreadError} from 'std:threads';import {Reader,Writer,IoError,reader,writer,copy,close} from 'std:io';import {create,readFile,removeFile} from 'std:fs';import {fromUtf8,toUtf8} from 'std:bytes';async function plain(input:Mutable<Reader>,output:Mutable<Writer>):Task<Result<int,IoError>>{return await copy(input,output);}async function transformed(input:Mutable<CompressionReader>,output:Mutable<Writer>):Task<Result<int,CompressionError|IoError|SpawnError|ThreadError<undefined>>>{return await copy(input,output);}async function main():Task<void>{const pool=new ThreadPool({workers:1,capacity:1});const source=reader(unwrap(fromUtf8('copy')));const encoded=unwrap(compressWriter(pool,writer(unwrap(create('copy.gz'))),'gzip'));console.log(unwrap(await copy(source,encoded,2)));unwrap(await close(source));unwrap(await close(encoded));const decoded=unwrap(decompressReader(pool,reader(unwrap(readFile('copy.gz'))),'gzip'));const destination=writer(unwrap(create('copy.txt')));console.log(unwrap(await transformed(decoded,destination)));unwrap(await close(decoded));unwrap(await close(destination));console.log(unwrap(toUtf8(unwrap(readFile('copy.txt')))));unwrap(removeFile('copy.gz'));unwrap(removeFile('copy.txt'));unwrap(pool.shutdown());}"#;check(source).unwrap();compression_suite_reports::assert_registered_source("all_b05_copy_endpoint_pairs_keep_explicit_finish_and_error_types",source);}
#[test]
fn tcp_and_compression_copy_preserve_typed_errors_and_execute() {
 let source=include_str!("../../../examples/compression/tcp-pipeline.tsr");
 let rust=check(source).unwrap();
 assert!(rust.contains("tsrust_native_io::copy_as"));
 compression_suite_reports::assert_registered_source("tcp_and_compression_copy_preserve_typed_errors_and_execute",source);
 let prefix="import {TcpStream} from 'std:net';import {CompressionReader,CompressionWriter,CompressionError} from 'std:compression';import {IoError,copy} from 'std:io';import {SpawnError,ThreadError} from 'std:threads';";
 for endpoints in ["input:Mutable<TcpStream>,output:Mutable<CompressionWriter>","input:Mutable<CompressionReader>,output:Mutable<TcpStream>"] {
  for (error,accepted) in [("CompressionError|IoError|SpawnError|ThreadError<undefined>",true),("IoError",false)] {
   let source=format!("{prefix}async function forward({endpoints}):Task<Result<int,{error}>>{{return await copy(input,output);}}function main(){{}}");
   assert_eq!(check(&source).is_ok(),accepted,"{endpoints}: {error}");
  }
 }
 let plain=format!("{prefix}async function forward(input:Mutable<TcpStream>,output:Mutable<TcpStream>):Task<Result<int,IoError>>{{return await copy(input,output);}}function main(){{}}");
 assert!(check(&plain).is_ok());
}

#[test]
fn compression_and_network_activate_only_their_required_runtimes() {
 let buffer=check(include_str!("../../../examples/compression/buffers.tsr")).unwrap();
 assert!(!buffer.contains("mod tsrust_native_net")&&!buffer.contains("mod tsrust_native_io")&&!buffer.contains("mod tsrust_native_fs"));
 let network=check(include_str!("../../../examples/network/tcp.tsr")).unwrap();
 assert!(!network.contains("mod tsrust_native_compression"));
}

#[test]
fn compression_packages_relocate_and_build_offline(){for (source,expected) in [(include_str!("../../../examples/compression/buffers.tsr"),b"hello compression\nzlib\nraw\n".as_slice()),(include_str!("../../../examples/compression/pipe.tsr"),b"17\none pool pipeline\n".as_slice())]{let directory=support::TempDirectory::new();let input=directory.source(&support::format_round_trip(source));let package=directory.0.join("package");support::success(&support::run(support::cli().arg("emit").arg(&input).arg("--package").arg("-o").arg(&package),60));let rust=std::fs::read_to_string(package.join("src/main.rs")).unwrap();std::fs::write(package.join("src/main.rs"),format!("#![forbid(unsafe_code)]\n{rust}")).unwrap();std::fs::remove_file(input).unwrap();let relocated=directory.0.join("relocated");std::fs::rename(package,&relocated).unwrap();let home=directory.0.join("empty-cargo-home");std::fs::create_dir(&home).unwrap();let target=std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));let mut build=std::process::Command::new("rustup");build.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&relocated).env("CARGO_HOME",home).env("CARGO_INCREMENTAL","0").env("CARGO_BUILD_JOBS","1").env("CARGO_TARGET_DIR",&target);support::success(&support::run(&mut build,300));let output=support::run(std::process::Command::new(target.join(format!("debug/program{}",std::env::consts::EXE_SUFFIX))).current_dir(&relocated),30);support::success(&output);assert_eq!(output.stdout,expected);}}

#[test]
fn nested_awaited_owned_bytes_select_the_reader_alternative(){
 let source="import {gzip,gunzip} from 'std:compression';import {ThreadPool} from 'std:threads';import {reader,readAll,close} from 'std:io';import {fromUtf8,toUtf8} from 'std:bytes';async function main():Task<void>{const pool=new ThreadPool({workers:1,capacity:1});const input=reader(unwrap(await gunzip(pool,unwrap(await gzip(pool,unwrap(fromUtf8('nested')))))));console.log(unwrap(toUtf8(unwrap(await readAll(input)))));unwrap(await close(input));unwrap(pool.shutdown());}";
 check(source).unwrap();
}

#[test]
fn compression_signature_metadata_does_not_authorize_pool_storage() {
 let prefix="import {compressReader} from 'std:compression';import {ThreadPool} from 'std:threads';import {reader} from 'std:io';import {fromUtf8} from 'std:bytes';";
 assert!(check(&format!("{prefix}function main(){{const pool=new ThreadPool({{workers:1,capacity:1}});match(compressReader(pool,reader(unwrap(fromUtf8('x'))),'gzip',{{level:10}})){{Ok(value)=>{{throw 'unexpected';}},Err(error)=>{{console.log(error.code);}}}}unwrap(pool.shutdown());}}")).is_ok());
 let declared=check(&format!("{prefix}type PoolAlias=ThreadPool;type Stored={{pool:PoolAlias;}};function main(){{}}"));
 assert_eq!(declared.unwrap_err(),"thread-storage");
 for body in ["type PoolAlias=ThreadPool;type Stored={pool:PoolAlias;};function main(){const pool=new ThreadPool({workers:1,capacity:1});const stored:Stored={pool:pool};}","function main(){const pool=new ThreadPool({workers:1,capacity:1});const stored=[pool];}","type Stored={pool:Result<ThreadPool,string>;};function main(){const pool=new ThreadPool({workers:1,capacity:1});const stored:Stored={pool:Ok(pool)};}"] {
  let error=check(&format!("{prefix}{body}")).unwrap_err();assert!(matches!(error.as_str(),"thread-storage"|"thread-lifecycle"),"{body}: {error}");
 }
}

#[test]
fn stream_constructor_reads_format_descriptor_without_consuming_it() {
 let source="import {compressReader,decompressReader,CompressionFormat} from 'std:compression';import {ThreadPool} from 'std:threads';import {reader,close} from 'std:io';import {fromUtf8,toUtf8} from 'std:bytes';async function main():Task<void>{const pool=new ThreadPool({workers:1,capacity:1});const format:CompressionFormat='gzip';const encoded=unwrap(compressReader(pool,reader(unwrap(fromUtf8('descriptor'))),format));const bytes=unwrap(await encoded.readAll());unwrap(await close(unwrap(await encoded.finish())));const decoded=unwrap(decompressReader(pool,reader(bytes),format));console.log(unwrap(toUtf8(unwrap(await decoded.readAll()))));unwrap(await close(unwrap(await decoded.finish())));unwrap(pool.shutdown());}";
 let rust=check(source).unwrap();assert!(rust.contains("match compression_format{ViewUnion"));
 compression_suite_reports::assert_registered_source("stream_constructor_reads_format_descriptor_without_consuming_it",source);
}

#[test]
fn registered_compression_runtime_family_executes_at_both_required_profiles(){
 compression_suite_reports::assert_sources();let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();let suite=root.join("crates/tsrust-compiler/tests/fixtures/compression/runtime.test.tsr");
 // Temporary data only; stable suite paths and an explicit shared cache avoid
 // rebuilding an independent runtime for each catalog/main program.
 let data=support::TempDirectory::new();let cache=root.join("target/tsrust-native");
 for optimized in [false,true]{let mut command=support::cli();command.current_dir(&data.0).env("TSRUST_CACHE_DIR",&cache).args(["test",suite.to_str().unwrap(),"--reporter=json","--profile","--timeout=10000"]);if optimized{command.arg("--release");}compression_suite_reports::assert_report(support::run(&mut command,900));assert!(std::fs::read_dir(&data.0).unwrap().next().is_none(),"compression fixture left runtime files");}
}
