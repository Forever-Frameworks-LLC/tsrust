use std::{env,fs,path::PathBuf,process::Command};
fn main(){
 let here=PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());let node=env::var_os("TSRUST_REALTIME_NODE").expect("explicit pinned Node24.11.1 required");
 println!("cargo:rerun-if-env-changed=TSRUST_REALTIME_NODE");
 for input in [&here, &here.join("../../../http"), &here.join("../../quiet-machine.mjs")]{println!("cargo:rerun-if-changed={}",input.display());}
 let output=Command::new(node).arg(here.join("identity.mjs")).output().unwrap();assert!(output.status.success(),"source identity failed: {}",String::from_utf8_lossy(&output.stderr));
 fs::write(PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("authored-identity.json"),output.stdout).unwrap();
}
