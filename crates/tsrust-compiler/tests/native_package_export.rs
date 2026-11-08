mod support;
// Relocated packages share target/{debug,release}/program during build and execution.
static EXPORT_TARGET: std::sync::Mutex<()> = std::sync::Mutex::new(());
#[test]
fn emitted_dependency_packages_build_offline_after_relocation_without_source_or_repo() {
 let _target = EXPORT_TARGET.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
 for (source, expected) in [
  ("import {parse,kind} from 'std:json';function main(){match(parse('null')){Ok(value)=>{console.log(kind(value));},Err(error)=>{throw error.message;}}}", "null\n"),
  (include_str!("../../../examples/worker-pool/main.tsr"), "42\ncancelled\n0\n"),
  (include_str!("../../../examples/scoped-pools/main.tsr"), "42\nindependent\n0\n7\n"),
  (include_str!("../../../examples/channel-worker/main.tsr"), "hello\ngoodbye\n"),
  (include_str!("../../../examples/channel-batch/main.tsr"), "204\n"),
  (include_str!("../../../examples/rust-rayon/main.tsr"), "55\ncancelled\n"),
 ] {
 let dir=support::TempDirectory::new();let input=dir.0.join("main.tsr");let package=dir.0.join("package");
 std::fs::write(&input,support::format_round_trip(source)).unwrap();
 if source.contains("./compute.rs") {std::fs::write(dir.0.join("compute.rs"),include_str!("../../../examples/rust-rayon/compute.rs")).unwrap();}
 support::success(&support::run(support::cli().arg("emit").arg(&input).arg("--package").arg("-o").arg(&package),30));
 std::fs::remove_file(&input).unwrap();
 if dir.0.join("compute.rs").exists(){std::fs::remove_file(dir.0.join("compute.rs")).unwrap();}let relocated=dir.0.join("relocated");std::fs::rename(&package,&relocated).unwrap();
 let home=dir.0.join("empty-cargo-home");std::fs::create_dir(&home).unwrap();let target=std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
 for release in [false,true] {
  let mut command=std::process::Command::new("rustup");command.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&relocated).env("CARGO_HOME",&home).env("CARGO_INCREMENTAL","0").env("CARGO_BUILD_JOBS","1").env("CARGO_TARGET_DIR",&target);
  if release{command.arg("--release");}
  support::success(&support::run(&mut command,120));
  let output=support::run(&mut std::process::Command::new(target.join(if release{"release/program"}else{"debug/program"})),20);support::success(&output);assert_eq!(output.stdout,expected.as_bytes());
 }
 assert!(relocated.join("vendor/serde_json-1.0.151/LICENSE-MIT").exists());
 assert!(relocated.join("vendor/rayon-1.11.0/LICENSE-MIT").exists());
 assert_eq!(std::fs::read(relocated.join("vendor/r-efi-6.0.0/AUTHORS")).unwrap().as_slice(),include_bytes!("../../../tools/tests/fixtures/r-efi-6.0.0/AUTHORS"));
 assert_eq!(std::fs::read(relocated.join("vendor/r-efi-5.3.0/AUTHORS")).unwrap().as_slice(),include_bytes!("../../../tools/tests/fixtures/r-efi-5.3.0/AUTHORS"));
 }
}
#[test]
fn package_export_refuses_existing_destinations_and_invalid_option_combinations() {
 let dir=support::TempDirectory::new();let input=dir.0.join("main.tsr");std::fs::write(&input,support::format_round_trip("function main(){console.log('hello');}" )).unwrap();
 let existing=dir.0.join("existing");std::fs::create_dir(&existing).unwrap();std::fs::write(existing.join("keep"),"owner data").unwrap();
 let output=support::run(support::cli().arg("emit").arg(&input).arg("--package").arg("-o").arg(&existing),20);assert!(!output.status.success());assert_eq!(std::fs::read_to_string(existing.join("keep")).unwrap(),"owner data");assert_eq!(std::fs::read_dir(&existing).unwrap().count(),1);
 for command in ["emit","run"] {
  let output=support::run(support::cli().arg(command).arg(&input).arg("--package"),20);assert!(!output.status.success());
 }
 #[cfg(unix)] {
  let link=dir.0.join("linked");std::os::unix::fs::symlink(&existing,&link).unwrap();
  let output=support::run(support::cli().arg("emit").arg(&input).arg("--package").arg("-o").arg(&link),20);assert!(!output.status.success());assert_eq!(std::fs::read_to_string(existing.join("keep")).unwrap(),"owner data");
 }
 let package=dir.0.join("plain");support::success(&support::run(support::cli().arg("emit").arg(&input).arg("--package").arg("-o").arg(&package),20));
 assert!(!package.join("vendor").exists());assert!(package.join("src/main.rs").exists());
}

#[test]
fn package_export_preserves_optional_main_and_builds_plain_programs_without_vendor() {
 let _target = EXPORT_TARGET.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
 let dir=support::TempDirectory::new();let input=dir.0.join("library.tsr");let package=dir.0.join("standalone");
 std::fs::write(&input,support::format_round_trip("export function answer():int{return 42;}" )).unwrap();
 support::success(&support::run(support::cli().arg("emit").arg(&input).arg("--package").arg("-o").arg(&package),30));
 let target=std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
 support::success(&support::run(std::process::Command::new("rustup").args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&package).env("CARGO_HOME",dir.0.join("empty-cargo-home")).env("CARGO_TARGET_DIR",&target).env("CARGO_INCREMENTAL","0").env("CARGO_BUILD_JOBS","1"),60));
 support::success(&support::run(&mut std::process::Command::new(target.join("debug/program")),20));
 assert!(!package.join("vendor").exists());
}
#[test]
fn async_package_builds_offline_after_relocation(){
 let _target = EXPORT_TARGET.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
 let dir=support::TempDirectory::new();let input=dir.source(&support::format_round_trip(include_str!("../../../examples/tokio-tasks/channels.tsr")));
 let package=dir.0.join("package");support::success(&support::run(support::cli().arg("emit").arg(&input).arg("--package").arg("-o").arg(&package),30));
 std::fs::remove_file(input).unwrap();let relocated=dir.0.join("relocated");std::fs::rename(package,&relocated).unwrap();
 let target=std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
 for release in [false,true]{
  let mut command=std::process::Command::new("rustup");command.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&relocated).env("CARGO_HOME",dir.0.join("empty-cargo-home")).env("CARGO_INCREMENTAL","0").env("CARGO_BUILD_JOBS","1").env("CARGO_TARGET_DIR",&target);
  if release{command.arg("--release");}support::success(&support::run(&mut command,120));
  let output=support::run(&mut std::process::Command::new(target.join(if release{"release/program"}else{"debug/program"})),30);support::success(&output);assert_eq!(output.stdout,b"42\n");
 }
}
#[test]
fn http_package_builds_offline_after_relocation(){
 let _target = EXPORT_TARGET.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
 let dir=support::TempDirectory::new();let input=dir.source(&support::format_round_trip(include_str!("../../../examples/http/loopback.tsr")));
 let package=dir.0.join("package");support::success(&support::run(support::cli().arg("emit").arg(&input).arg("--package").arg("-o").arg(&package),60));
 std::fs::remove_file(input).unwrap();let relocated=dir.0.join("relocated");std::fs::rename(package,&relocated).unwrap();
 let target=std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
 for release in [false,true]{
  let mut command=std::process::Command::new("rustup");command.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&relocated).env("CARGO_HOME",dir.0.join("empty-cargo-home")).env("CARGO_INCREMENTAL","0").env("CARGO_BUILD_JOBS","1").env("CARGO_TARGET_DIR",&target);
  if release{command.arg("--release");}support::success(&support::run(&mut command,300));
  let output=support::run(&mut std::process::Command::new(target.join(if release{"release/program"}else{"debug/program"})),30);support::success(&output);assert_eq!(output.stdout,b"200\n{\"id\":\"42\",\"message\":\"Hello\"}\nhello\n");
 }
}
#[test]
fn https_package_builds_offline_after_relocation(){
 let _target = EXPORT_TARGET.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
 let cert=serde_json::to_string(include_str!("fixtures/tls/server.pem")).unwrap();let key=serde_json::to_string(include_str!("fixtures/tls/server.key")).unwrap();let ca=serde_json::to_string(include_str!("fixtures/tls/ca.pem")).unwrap();
 let source=format!("import {{App,Client,request}} from 'std:http';import {{strictEqual}} from 'std:testing';async function main():Task<void>{{const app=new App();const server=await app.listen('127.0.0.1:0',{{cert:{cert},key:{key}}});const client=new Client({{ca:{ca}}});const response=await client.send(request('GET','https://'+server.address()+'/', ''));strictEqual(response.status(),404);await server.shutdown();}}");
 let dir=support::TempDirectory::new();let input=dir.source(&support::format_round_trip(&source));let package=dir.0.join("package");
 support::success(&support::run(support::cli().arg("emit").arg(&input).arg("--package").arg("-o").arg(&package),60));
 std::fs::remove_file(input).unwrap();let relocated=dir.0.join("relocated");std::fs::rename(package,&relocated).unwrap();let target=std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
 for release in [false,true]{
  let mut command=std::process::Command::new("rustup");command.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&relocated).env("CARGO_HOME",dir.0.join("empty-cargo-home")).env("CARGO_INCREMENTAL","0").env("CARGO_BUILD_JOBS","1").env("CARGO_TARGET_DIR",&target);
  if release{command.arg("--release");}support::success(&support::run(&mut command,300));
  support::success(&support::run(&mut std::process::Command::new(target.join(if release{"release/program"}else{"debug/program"})),30));
 }
}

#[test]
fn sqlite_package_builds_offline_after_relocation(){
 let _target = EXPORT_TARGET.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
 let dir=support::TempDirectory::new();let input=dir.source(&support::format_round_trip(include_str!("../../../examples/sqlite/people.tsr")));
 let package=dir.0.join("package");support::success(&support::run(support::cli().arg("emit").arg(&input).arg("--package").arg("-o").arg(&package),30));
 std::fs::remove_file(input).unwrap();let relocated=dir.0.join("relocated");std::fs::rename(package,&relocated).unwrap();
 let target=std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
 for release in [false,true]{
  let mut command=std::process::Command::new("rustup");command.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&relocated).env("CARGO_HOME",dir.0.join("empty-cargo-home")).env("CARGO_INCREMENTAL","0").env("CARGO_BUILD_JOBS","1").env("CARGO_TARGET_DIR",&target);
  if release{command.arg("--release");}support::success(&support::run(&mut command,300));
  let output=support::run(&mut std::process::Command::new(target.join(if release{"release/program"}else{"debug/program"})),30);support::success(&output);assert_eq!(output.stdout,b"1: Ada\n2: Grace\n");
 }
}

#[test]
fn file_package_preserves_safe_close_and_builds_offline_after_relocation() {
 let _target = EXPORT_TARGET.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
 let dir=support::TempDirectory::new();
 let input=dir.source(&support::format_round_trip(include_str!("fixtures/native-files/read-close.tsr")));
 let package=dir.0.join("package");
 support::success(&support::run(support::cli().arg("emit").arg(&input).arg("--package").arg("-o").arg(&package),60));
 std::fs::remove_file(input).unwrap();let relocated=dir.0.join("relocated");std::fs::rename(package,&relocated).unwrap();
 let source=relocated.join("src/main.rs");let emitted=std::fs::read_to_string(&source).unwrap();assert!(!emitted.contains("unsafe"));
 std::fs::write(source,format!("#![forbid(unsafe_code)]\n{emitted}")).unwrap();
 assert!(std::fs::read_to_string(relocated.join("Cargo.toml")).unwrap().contains("\"files\""));
 assert!(relocated.join("vendor/nix-0.31.2/LICENSE").exists());
 std::fs::write(relocated.join("input.bin"),[0,65,255,66]).unwrap();
 let target=std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
 for release in [false,true] {
  let mut command=std::process::Command::new("rustup");command.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&relocated).env("CARGO_HOME",dir.0.join("empty-cargo-home")).env("CARGO_INCREMENTAL","0").env("CARGO_BUILD_JOBS","1").env("CARGO_TARGET_DIR",&target);
  if release {command.arg("--release");}support::success(&support::run(&mut command,120));
  let output=support::run(std::process::Command::new(target.join(if release{"release/program"}else{"debug/program"})).current_dir(&relocated),30);support::success(&output);assert!(output.stdout.is_empty());
 }
}

#[test]
fn collection_iterator_testing_completion_builds_offline_after_relocation() {
 let _target=EXPORT_TARGET.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
 let source=r#"
import {Iterator} from 'std:iterator';
import {test,runTests,strictEqual,deepStrictEqual,notDeepStrictEqual,rejects} from 'std:testing';
import {parseDate,formatDate,getDay,startOfYear,endOfYear,parseTime,getHours,getMinutes,getSeconds,getNanoseconds} from 'std:date';
import {parseInt as exact,parseNumber,checkedAdd,checkedSubtract,checkedMultiply,checkedDivide,checkedRemainder} from 'std:numeric';
function main(){
 test('exported',async()=>{
  deepStrictEqual(Iterator.from([1,2,3]).filter(value=>value>1).map(value=>value*2).toArray(),[4,6]);
  notDeepStrictEqual([[1]],[[2]]);
  await rejects(async()=>{throw {code:'exported'};});
  const date=unwrap(parseDate('2024-02-29'));
  strictEqual(getDay(date),4);
  strictEqual(formatDate(startOfYear(date)),'2024-01-01');
  strictEqual(formatDate(endOfYear(date)),'2024-12-31');
  const time=unwrap(parseTime('23:59:58.123456789'));
  strictEqual(getHours(time),23);strictEqual(getMinutes(time),59);
  strictEqual(getSeconds(time),58);strictEqual(getNanoseconds(time),123456789);
  strictEqual(unwrap(exact('ff',16)),255);strictEqual(unwrap(parseNumber('1e2')),100.0);
  strictEqual(unwrap(checkedAdd(7,3)),10);strictEqual(unwrap(checkedSubtract(7,3)),4);
  strictEqual(unwrap(checkedMultiply(7,3)),21);strictEqual(unwrap(checkedDivide(7,3)),2);
  strictEqual(unwrap(checkedRemainder(7,3)),1);
 });
 strictEqual(runTests(),0);
}
"#;
 let dir=support::TempDirectory::new();let input=dir.source(&support::format_round_trip(source));let package=dir.0.join("package");
 support::success(&support::run(support::cli().arg("emit").arg(&input).arg("--package").arg("-o").arg(&package),30));
 std::fs::remove_file(&input).unwrap();let relocated=dir.0.join("relocated");std::fs::rename(package,&relocated).unwrap();
 let home=dir.0.join("empty-cargo-home");std::fs::create_dir(&home).unwrap();
 let target=std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
 for release in [false,true]{
  let mut command=std::process::Command::new("rustup");command.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&relocated).env("CARGO_HOME",&home).env("CARGO_INCREMENTAL","0").env("CARGO_BUILD_JOBS","1").env("CARGO_TARGET_DIR",&target);
  if release{command.arg("--release");}
  support::success(&support::run(&mut command,180));
  let binary=target.join(if release{"release/program"}else{"debug/program"}).with_extension(std::env::consts::EXE_EXTENSION);
  let output=support::run(&mut std::process::Command::new(binary),30);support::success(&output);
  assert_eq!(output.stdout,b"PASS exported\n1 passed, 0 failed, 0 skipped\n");
 }
}


#[test]
fn new_optional_backends_follow_unused_imports_and_actual_value_or_type_uses() {
    use tsrust_compiler::project::{FileSourceProvider,check_graph,load_project};
    let emit=|name:&str,source:&str| {
        let directory=support::TempDirectory::new();
        let path=directory.source(&support::format_round_trip(source));
        let graph=load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap_or_else(|error|panic!("{name}: {error:?}"));
        let checked=check_graph(graph).unwrap_or_else(|error|panic!("{name}: {error:?}"));
        checked.emit_rust_with_native_features()
    };
    // An import is not a value use. None of these cases evaluates an owner,
    // declares a used opaque signature or binds a factory value.
    for(name,source)in [
        ("plain","function main(){console.log('plain');}"),
        ("io","import {readAll,Reader} from 'std:io';function main(){console.log('plain');}"),
        ("iterator","import {Iterator} from 'std:iterator';function main(){console.log('plain');}"),
        ("cancellation","import {CancellationToken} from 'std:futures';function main(){console.log('plain');}"),
        ("password","import {hashPassword,Hasher} from 'std:crypto';function main(){console.log('plain');}"),
        ("websocket","import {connectWebSocket,WebSocketRead} from 'std:http';function main(){console.log('plain');}"),
    ] {
        let(rust,features)=emit(name,source);
        assert!(features.is_empty(),"{name}: {features:?}");
        assert!(!tsrust_compiler::native_runtime::separate(&rust).dependencies,"{name}");
        for marker in ["mod tsrust_native_io {","enum TsrIteratorCursor<","mod tsrust_native_cancellation {","mod tsrust_native_crypto {","mod tsrust_native_crypto_password {","// tsrust-http-websocket-selected","mod tsrust_native_http {"] {
            assert!(!rust.contains(marker),"{name}: unexpectedly selected {marker}");
        }
    }
    // A used value or an opaque owner in an actual signature still needs its
    // backend, even if no constructor executes. Ordinary crypto/HTTP must not
    // silently select the independent password/WebSocket dependency families.
    for(name,source,required,forbidden,marker)in [
        ("ordinary crypto","import {sha256} from 'std:crypto';import {fromUtf8,length} from 'std:bytes';function main(){const digest=sha256(unwrap(fromUtf8('x')));console.log(length(digest));}",&["crypto"][..],&["crypto-password","http-websocket"][..],"mod tsrust_native_crypto {"),
        ("used Hasher signature","import {Hasher} from 'std:crypto';function consume(state:Owned<Hasher>):void{}function main(){}",&["crypto"][..],&["crypto-password"][..],"mod tsrust_native_crypto {"),
        ("password","import {hashPassword} from 'std:crypto';import {ThreadPool} from 'std:threads';import {fromUtf8} from 'std:bytes';async function main():Task<void>{const pool=new ThreadPool({workers:1,capacity:1});const encoded=unwrap(await hashPassword(pool,unwrap(fromUtf8('x')),{memoryKiB:32,iterations:1,parallelism:1}));console.log(encoded.length);unwrap(pool.shutdown());}",&["crypto-password","threads","async"][..],&["http-websocket"][..],"mod tsrust_native_crypto_password {"),
        ("ordinary HTTP","import {App} from 'std:http';function main(){const app=new App();console.log('plain');}",&["http"][..],&["http-websocket","http-middleware","crypto-password"][..],"mod tsrust_native_http {"),
        ("SSE","import {sse,SseEvent} from 'std:http';import {Channel} from 'std:futures';function main(){const events=new Channel<SseEvent>(1);events.close();const response=sse(events);console.log(response.status());}",&["http"][..],&["http-websocket","crypto-password"][..],"mod tsrust_native_http {"),
        ("WebSocket","import {connectWebSocket} from 'std:http';async function main():Task<void>{const socket=await connectWebSocket('ws://127.0.0.1:1/');await socket.close();}",&["http-websocket"][..],&["crypto-password"][..],"// tsrust-http-websocket-selected"),
        ("used socket signature","import {WebSocketRead} from 'std:http';async function inspect(socket:Mutable<WebSocketRead>):Task<void>{const message=await socket.receive();if(message!==undefined&&message.kind==='text'){console.log(message.data);}}function main(){}",&["http-websocket"][..],&["crypto-password"][..],"// tsrust-http-websocket-selected"),
        ("cancellation value","import {CancellationToken} from 'std:futures';function main(){const token=new CancellationToken();console.log(token.isCancelled());}",&["cancellation"][..],&["crypto-password","http-websocket"][..],"mod tsrust_native_cancellation {"),
    ] {
        let(rust,features)=emit(name,source);
        for feature in required {assert!(features.contains(feature),"{name}: missing {feature}: {features:?}");}
        for feature in forbidden {assert!(!features.contains(feature),"{name}: unwanted {feature}: {features:?}");}
        assert!(rust.contains(marker),"{name}: missing {marker}");
        if forbidden.contains(&"crypto-password"){assert!(!rust.contains("mod tsrust_native_crypto_password {"),"{name}");}
        if forbidden.contains(&"http-websocket"){assert!(!rust.contains("// tsrust-http-websocket-selected"),"{name}");}
        assert!(tsrust_compiler::native_runtime::separate(&rust).dependencies,"{name}");
    }
}
