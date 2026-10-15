mod support;
#[test]
fn emitted_dependency_packages_build_offline_after_relocation_without_source_or_repo() {
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
 let home=dir.0.join("empty-cargo-home");std::fs::create_dir(&home).unwrap();let target=dir.0.join("native-target");
 for release in [false,true] {
  let mut command=std::process::Command::new("rustup");command.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&relocated).env("CARGO_HOME",&home).env("CARGO_INCREMENTAL","0").env("CARGO_TARGET_DIR",&target);
  if release{command.arg("--release");}
  support::success(&support::run(&mut command,120));
  let output=support::run(&mut std::process::Command::new(target.join(if release{"release/program"}else{"debug/program"})),20);support::success(&output);assert_eq!(output.stdout,expected.as_bytes());
 }
 assert!(relocated.join("vendor/serde_json-1.0.151/LICENSE-MIT").exists());
 assert!(relocated.join("vendor/rayon-1.11.0/LICENSE-MIT").exists());
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
 let dir=support::TempDirectory::new();let input=dir.0.join("library.tsr");let package=dir.0.join("standalone");
 std::fs::write(&input,support::format_round_trip("export function answer():int{return 42;}" )).unwrap();
 support::success(&support::run(support::cli().arg("emit").arg(&input).arg("--package").arg("-o").arg(&package),30));
 let target=dir.0.join("native-target");
 support::success(&support::run(std::process::Command::new("rustup").args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&package).env("CARGO_HOME",dir.0.join("empty-cargo-home")).env("CARGO_TARGET_DIR",&target).env("CARGO_INCREMENTAL","0"),60));
 support::success(&support::run(&mut std::process::Command::new(target.join("debug/program")),20));
 assert!(!package.join("vendor").exists());
}
#[test]
fn async_package_builds_offline_after_relocation(){
 let dir=support::TempDirectory::new();let input=dir.source(&support::format_round_trip(include_str!("../../../examples/tokio-tasks/channels.tsr")));
 let package=dir.0.join("package");support::success(&support::run(support::cli().arg("emit").arg(&input).arg("--package").arg("-o").arg(&package),30));
 std::fs::remove_file(input).unwrap();let relocated=dir.0.join("relocated");std::fs::rename(package,&relocated).unwrap();
 let target=dir.0.join("native-target");
 for release in [false,true]{
  let mut command=std::process::Command::new("rustup");command.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&relocated).env("CARGO_HOME",dir.0.join("empty-cargo-home")).env("CARGO_INCREMENTAL","0").env("CARGO_TARGET_DIR",&target);
  if release{command.arg("--release");}support::success(&support::run(&mut command,120));
  let output=support::run(&mut std::process::Command::new(target.join(if release{"release/program"}else{"debug/program"})),30);support::success(&output);assert_eq!(output.stdout,b"42\n");
 }
}
#[test]
fn http_package_builds_offline_after_relocation(){
 let dir=support::TempDirectory::new();let input=dir.source(&support::format_round_trip(include_str!("../../../examples/http/loopback.tsr")));
 let package=dir.0.join("package");support::success(&support::run(support::cli().arg("emit").arg(&input).arg("--package").arg("-o").arg(&package),60));
 std::fs::remove_file(input).unwrap();let relocated=dir.0.join("relocated");std::fs::rename(package,&relocated).unwrap();
 let target=dir.0.join("native-target");
 for release in [false,true]{
  let mut command=std::process::Command::new("rustup");command.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&relocated).env("CARGO_HOME",dir.0.join("empty-cargo-home")).env("CARGO_INCREMENTAL","0").env("CARGO_TARGET_DIR",&target);
  if release{command.arg("--release");}support::success(&support::run(&mut command,300));
  let output=support::run(&mut std::process::Command::new(target.join(if release{"release/program"}else{"debug/program"})),30);support::success(&output);assert_eq!(output.stdout,b"200\n{\"id\":\"42\",\"message\":\"Hello\"}\nhello\n");
 }
}
#[test]
fn https_package_builds_offline_after_relocation(){
 let cert=serde_json::to_string(include_str!("fixtures/tls/server.pem")).unwrap();let key=serde_json::to_string(include_str!("fixtures/tls/server.key")).unwrap();let ca=serde_json::to_string(include_str!("fixtures/tls/ca.pem")).unwrap();
 let source=format!("import {{App,Client,request}} from 'std:http';import {{strictEqual}} from 'std:testing';async function main():Task<void>{{const app=new App();const server=await app.listen('127.0.0.1:0',{{cert:{cert},key:{key}}});const client=new Client({{ca:{ca}}});const response=await client.send(request('GET','https://'+server.address()+'/', ''));strictEqual(response.status(),404);await server.shutdown();}}");
 let dir=support::TempDirectory::new();let input=dir.source(&support::format_round_trip(&source));let package=dir.0.join("package");
 support::success(&support::run(support::cli().arg("emit").arg(&input).arg("--package").arg("-o").arg(&package),60));
 std::fs::remove_file(input).unwrap();let relocated=dir.0.join("relocated");std::fs::rename(package,&relocated).unwrap();let target=dir.0.join("native-target");
 for release in [false,true]{
  let mut command=std::process::Command::new("rustup");command.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&relocated).env("CARGO_HOME",dir.0.join("empty-cargo-home")).env("CARGO_INCREMENTAL","0").env("CARGO_BUILD_JOBS","1").env("CARGO_TARGET_DIR",&target);
  if release{command.arg("--release");}support::success(&support::run(&mut command,300));
  support::success(&support::run(&mut std::process::Command::new(target.join(if release{"release/program"}else{"debug/program"})),30));
 }
}
