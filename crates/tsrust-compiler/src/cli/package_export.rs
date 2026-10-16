//! Export a standalone, relocatable source package. No registry access or rustc
//! invocation is needed during export; compilation is the recipient's next step.
use std::{fs,io,path::Path};
pub(crate) fn export(root:&Path,rust:&str)->io::Result<()> {
 let dependencies=tsrust_compiler::native_runtime::separate(rust).dependencies;
 if dependencies {
  crate::native_dependency_bundle::materialize(root)?;
 }else{
  fs::create_dir(root)?;
 }
 // Only this call's newly created tree can be cleaned up on failure. Existing
 // directories and symlinks fail above, before any caller-owned files are touched.
 let result=(|| {
  if dependencies {let packages=crate::rust_binding_packages::packages(rust).map_err(io::Error::other)?;crate::rust_binding_packages::overlay(root,&packages).map_err(io::Error::other)?;}
  if !dependencies {fs::create_dir(root.join("src"))?;fs::create_dir(root.join(".cargo"))?;}
  let (mut manifest,lock)=if dependencies {
   fs::remove_file(root.join("src/lib.rs"))?;
   (fs::read_to_string(root.join("Cargo.toml"))?.replace("name = \"tsrust-native-dependencies\"","name = \"tsrust-export\""),
    fs::read_to_string(root.join("Cargo.lock"))?.replace("name = \"tsrust-native-dependencies\"","name = \"tsrust-export\""))
  }else{
   ("[package]\nname = \"tsrust-export\"\nversion = \"0.1.0\"\nedition = \"2024\"\npublish = false\n\n[workspace]\n".into(),
    "version = 4\n\n[[package]]\nname = \"tsrust-export\"\nversion = \"0.1.0\"\n".into())
  };
  if rust.contains("mod tsrust_native_threads {") || rust.contains("mod tsr_rust") { manifest = manifest.replace("default = []", "default = [\"threads\"]"); }
  if rust.contains("mod tsrust_native_http {"){manifest=manifest.replace("default = []", "default = [\"http\"]").replace("default = [\"threads\"]", "default = [\"threads\", \"http\"]");}
  if rust.contains("mod tsrust_native_futures {") {manifest=manifest.replace("default = []", "default = [\"async\"]").replace("default = [\"threads\"]", "default = [\"threads\", \"async\"]");}
  if rust.contains("mod tsrust_native_date {"){manifest=manifest.replace("default = [", "default = [\"calendar\", ");}
  if rust.contains("mod tsrust_native_crypto {"){manifest=manifest.replace("default = [", "default = [\"crypto\", ");}
  if rust.contains("mod tsrust_native_signals {"){manifest=manifest.replace("default = [", "default = [\"signals\", ");}
  if rust.contains("mod tsrust_native_uuid {"){manifest=manifest.replace("default = [", "default = [\"uuid\", ");}
  if rust.contains("mod tsrust_native_decimal {"){manifest=manifest.replace("default = [", "default = [\"decimal\", ");}
  if rust.contains("mod tsrust_native_sql {"){manifest=manifest.replace("default = [", "default = [\"sql\", ");}
  if rust.contains("mod tsrust_native_sqlite {"){manifest=manifest.replace("default = [", "default = [\"sqlite\", ");}
  manifest.push_str("\n[[bin]]\nname = \"program\"\npath = \"src/main.rs\"\n\n[profile.dev]\noverflow-checks = true\nincremental = false\n\n[profile.release]\noverflow-checks = true\nincremental = false\n");
  fs::write(root.join("Cargo.toml"),manifest)?;fs::write(root.join("Cargo.lock"),lock)?;
  if !dependencies {fs::write(root.join(".cargo/config.toml"),"[net]\noffline = true\n")?;}
  fs::write(root.join("src/main.rs"),rust)?;
  fs::write(root.join("rust-toolchain.toml"),format!("[toolchain]\nchannel = {:?}\nprofile = \"minimal\"\n",tsrust_compiler::RUST_BACKEND_VERSION))?;
  fs::write(root.join(".gitignore"),"/target/\n")?;
  fs::write(root.join("README.md"),format!("# Exported Tsrust program\n\nRequires the installed Rust {} toolchain. From this directory:\n\n```sh\ncargo build --offline --locked\n./target/debug/program\n# Optimized build, retaining checked integer overflow:\ncargo build --offline --locked --release\n./target/release/program\n```\n\nThe package can be moved independently of the Tsrust project and compiler.\nNative dependency sources and their license files, when needed, are included\nin vendor/. No dependency download, external library path or manual --extern\nflag is required. Rust itself must already be installed for offline builds.\nThe executable name is program; change the [[bin]] name in Cargo.toml if needed.\n\nRun Cargo from this directory so it discovers .cargo/config.toml. From another\ndirectory, pass both --manifest-path and --config with paths into this package.\nDebug and release profiles preserve integer overflow checks. The exported executable preserves the compiler-generated entrypoint and behavior.\n",tsrust_compiler::RUST_BACKEND_VERSION))?;
  Ok(())
 })();
 if result.is_err(){let _=fs::remove_dir_all(root);}
 result
}
