mod support;
#[test]
fn source_policies_preserve_aliases_path_order_request_bodies_and_error_headers(){
 let source=support::format_round_trip(include_str!("../../../examples/http-middleware/main.tsr"));
 let directory=support::TempDirectory::new();let path=directory.source(&source);
 support::success(&support::run(support::cli().current_dir(&directory.0).args(["setup","--portable","--offline"]),180));
 let package=directory.0.join("package");
 support::success(&support::run(support::cli().arg("emit").arg(&path).args(["--package","-o"]).arg(&package),60));
 std::fs::remove_file(path).unwrap();
 let relocated=directory.0.join("relocated");std::fs::rename(package,&relocated).unwrap();
 let manifest=std::fs::read_to_string(relocated.join("Cargo.toml")).unwrap();
 let document=manifest.parse::<toml_edit::DocumentMut>().unwrap();
 assert!(document["features"]["default"].as_array().unwrap().iter().any(|feature|feature.as_str()==Some("http-middleware")));
 assert!(relocated.join("vendor/async-compression-0.4.51/LICENSE-MIT").is_file());
 assert!(relocated.join("vendor/compression-codecs-0.4.47/LICENSE-MIT").is_file());
 assert!(relocated.join("vendor/compression-core-0.4.34/LICENSE-MIT").is_file());
 let generated=relocated.join("src/main.rs");let emitted=std::fs::read_to_string(&generated).unwrap();
 std::fs::write(generated,format!("#![forbid(unsafe_code)]\n{emitted}")).unwrap();
 let target=std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
 assert!(target.is_absolute());
 let home=directory.0.join("empty-cargo-home");std::fs::create_dir(&home).unwrap();
 for release in [false,true]{
  let mut command=std::process::Command::new("rustup");
  command.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&relocated)
   .env("CARGO_HOME",&home).env("CARGO_TARGET_DIR",&target).env("CARGO_INCREMENTAL","0").env("CARGO_BUILD_JOBS","1");
  if release{command.arg("--release");}
  support::success(&support::run(&mut command,180));
  let executable=target.join(if release{"release"}else{"debug"}).join(format!("program{}",std::env::consts::EXE_SUFFIX));
  let output=support::run(std::process::Command::new(executable).current_dir(&relocated),30);support::success(&output);
  assert_eq!(output.stdout,b"source middleware exact\n");assert!(output.stderr.is_empty(),"{}",String::from_utf8_lossy(&output.stderr));
 }
}
#[test]
fn middleware_factories_enforce_typed_options_arity_and_single_installation(){let prefix="import {App,Router,Middleware,CorsOptions,cors,secureHeaders,compress} from 'std:http';";for body in ["const policy:Middleware={};","new Middleware();","cors({origins:[1]});","cors({methods:true});","cors({maxAgeSeconds:1.5});","cors({credentials:'true'});","secureHeaders({});","compress(1);","const app=new App();const policy=secureHeaders();app.use(policy);app.use(policy);","const router=new Router();const policy=compress();router.use('/a',policy);router.use('/b',policy);","const policy=secureHeaders();clone(policy);"]{let source=support::format_round_trip(&format!("{prefix}function main(){{{body}}}"));let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("check").arg(directory.source(&source)),30);assert!(!output.status.success(),"accepted {body}");}}
#[test]
fn middleware_offline_exports_select_the_optional_backend_only_when_used(){
 for(name,source,selected)in[("plain","import {App} from 'std:http';function main(){const app=new App();console.log('plain');}",false),("policies","import {App,cors,secureHeaders,compress} from 'std:http';function main(){const app=new App();app.use(cors());app.use(secureHeaders());app.use(compress());console.log('policies');}",true)]{let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));let exported=directory.0.join(name);let output=support::run(support::cli().arg("emit").arg(&path).arg("--package").arg("-o").arg(&exported),60);support::success(&output);let manifest=std::fs::read_to_string(exported.join("Cargo.toml")).unwrap();let default=manifest.lines().find(|line|line.starts_with("default = ")).unwrap();assert_eq!(default.contains("http-middleware"),selected,"{default}");}
}
