mod support;
#[path="support/crypto_suite_reports.rs"] mod crypto_suite_reports;
#[test]
fn crypto_registered_suite_preserves_primitive_password_and_catalog_contracts(){
 crypto_suite_reports::assert_sources();
 support::format_round_trip(include_str!("fixtures/crypto/runtime.test.tsr"));
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
 // Both required profiles compile the same coherent suite once, never each example separately.
 for release in [false,true]{let mut command=support::cli();command.current_dir(&root).args(["test","crates/tsrust-compiler/tests/fixtures/crypto/runtime.test.tsr","--timeout=30000","--reporter=json","--profile"]);if release{command.arg("--release");}
 crypto_suite_reports::assert_report(support::run(&mut command,900));}
}
#[test]
fn crypto_catalog_references_preserve_verbatim_registered_sources(){crypto_suite_reports::assert_sources();}

#[test]
fn hasher_lifecycle_and_exact_crypto_argument_types_are_checked(){
 for source in ["import {createHasher} from 'std:crypto';function main(){createHasher('sha1');}","import {createHasher} from 'std:crypto';function main(){const h=createHasher('sha256');const out=h.finalize();h.finalize();}","import {createHasher} from 'std:crypto';function main(){const h=createHasher('sha256');console.log(clone(h));}","import {hashPassword} from 'std:crypto';import {ThreadPool} from 'std:threads';import {alloc,set} from 'std:bytes';async function main():Task<void>{const pool=new ThreadPool({workers:1,capacity:1});const bytes=unwrap(alloc(1));const task=hashPassword(pool,bytes);unwrap(set(bytes,0,1));console.log(await task);pool.shutdown();}","import {Hasher} from 'std:crypto';function bad(state:ReadOnly<Hasher>):void{state.finalize();}function main(){}","import {Hasher} from 'std:crypto';import {alloc} from 'std:bytes';function bad(state:ReadOnly<Hasher>):void{state.update(unwrap(alloc(0)));}function main(){}","import {hmacSha256} from 'std:crypto';function main(){hmacSha256('key','message');}","import {hkdfSha256} from 'std:crypto';import {alloc} from 'std:bytes';function main(){const input=unwrap(alloc(0));hkdfSha256(input,input,input,1.5);}"]{let source=support::format_round_trip(source);let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("check").arg(directory.source(&source)),30);assert!(!output.status.success(),"accepted {source}");}
}
#[test]
fn opaque_hasher_signatures_emit_their_runtime_without_a_constructor_call(){let source=support::format_round_trip("import {Hasher} from 'std:crypto';function consume(state:Owned<Hasher>):void{}function main(){}");let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("emit").arg(directory.source(&source)),30);support::success(&output);let emitted=String::from_utf8(output.stdout).unwrap();assert!(emitted.contains("mod tsrust_native_crypto {"));assert!(emitted.contains("tsrust_native_crypto::Hasher"));}

#[test]
fn password_exports_select_the_backend_feature_and_async_pool(){
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(include_str!("../../../examples/crypto/passwords.tsr")));let exported=directory.0.join("package");support::success(&support::run(support::cli().arg("emit").arg(path).args(["--package","-o"]).arg(&exported),60));
 let manifest=std::fs::read_to_string(exported.join("Cargo.toml")).unwrap();let manifest=manifest.parse::<toml_edit::DocumentMut>().unwrap();assert!(manifest["features"]["default"].as_array().unwrap().iter().any(|feature|feature.as_str()==Some("crypto-password")));
 let application=std::fs::read_to_string(exported.join("src/main.rs")).unwrap();assert!(application.contains("mod tsrust_native_crypto_password {"));assert!(application.contains("pub struct PoolSubmission"));
}
