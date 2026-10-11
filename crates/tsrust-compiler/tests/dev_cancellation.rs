#![cfg(unix)]
mod support;
use std::{fs,os::unix::{fs::PermissionsExt,process::CommandExt},process::{Child,Stdio},time::{Duration,Instant}};
unsafe extern "C" {fn kill(pid:i32,signal:i32)->i32;}
struct Group(Child);
impl Drop for Group {fn drop(&mut self){
 unsafe{kill(self.0.id()as i32,15);}
 let until=Instant::now()+Duration::from_secs(2);
 while self.0.try_wait().ok().flatten().is_none() && Instant::now()<until {std::thread::sleep(Duration::from_millis(20));}
 unsafe{kill(-(self.0.id()as i32),9);}let _=self.0.wait();
}}
#[test]
fn interrupt_cancels_backend_version_probe_and_its_descendants() {
 let dir=support::TempDirectory::new();
 support::success(&support::run(support::cli().current_dir(&dir.0).args(["new","app"]),20));
 let bin=dir.0.join("bin");fs::create_dir(&bin).unwrap();
 let wrapper=bin.join("rustup");
 fs::write(&wrapper,"#!/bin/sh\nsleep 30 &\nchild=$!\necho \"$$ $child\" > \"$PROBE_MARKER\"\ntrap 'kill $child 2>/dev/null; wait $child; exit 0' TERM\nwait $child\n").unwrap();
 fs::set_permissions(&wrapper,fs::Permissions::from_mode(0o755)).unwrap();
 let marker=dir.0.join("probe");let mut paths=vec![bin];paths.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap()));
 let mut command=support::cli();command.current_dir(dir.0.join("app")).arg("dev").env("PATH",std::env::join_paths(paths).unwrap()).env("PROBE_MARKER",&marker).stdout(Stdio::null()).stderr(Stdio::null()).process_group(0);
 let mut dev=Group(command.spawn().unwrap());let until=Instant::now()+Duration::from_secs(10);
 while !marker.exists() && Instant::now()<until {std::thread::sleep(Duration::from_millis(20));}
 let pids=fs::read_to_string(&marker).expect("backend probe did not start");
 unsafe{kill(dev.0.id()as i32,2);}
 let until=Instant::now()+Duration::from_secs(3);
 let status=loop {if let Some(status)=dev.0.try_wait().unwrap(){break status;}assert!(Instant::now()<until,"dev remained blocked in backend version probe");std::thread::sleep(Duration::from_millis(20));};
 assert!(status.success());
 for pid in pids.split_whitespace().map(|s|s.parse::<i32>().unwrap()) {assert_ne!(unsafe{kill(pid,0)},0,"backend descendant {pid} survived interrupt");}
}

fn wait_text(path:&std::path::Path,needle:&str)->String {
 let until=Instant::now()+Duration::from_secs(20);
 loop {let text=fs::read_to_string(path).unwrap_or_default();if text.contains(needle){return text;}assert!(Instant::now()<until,"missing {needle}: {text}");std::thread::sleep(Duration::from_millis(20));}
}
#[test]
fn edit_cancels_native_build_and_keeps_last_good_until_replacement() {cancel_build(false);}
#[test]
fn edit_cancels_runtime_build_and_keeps_last_good_until_replacement() {cancel_build(true);}
fn cancel_build(runtime:bool) {
 let dir=support::TempDirectory::new();
 support::success(&support::run(support::cli().current_dir(&dir.0).args(["new","app"]),20));
 let root=dir.0.join("app");let entry=root.join("src/main.tsr");
 let write=|text:&str| {
  let (import,call)=if runtime && text=="stale-candidate" {("import {fromArray} from 'std:bytes';","match(fromArray([1])){Ok(bytes)=>{},Err(error)=>{throw error.message;}}") } else {("","")};
  fs::write(&entry,support::format_round_trip(&format!("{import}import {{basename}} from 'std:path';function main(){{{call}console.log(basename('/tmp/{text}'));while(true){{}}}}"))).unwrap();
 };
 write("last-good");
 let bin=dir.0.join("bin");fs::create_dir(&bin).unwrap();let wrapper=bin.join("rustup");
 fs::write(&wrapper,r#"#!/bin/sh
case "$*" in *--version*) exec "$REAL_RUSTUP" "$@";; esac
if [ -f "$HOLD_BUILD" ]; then
  sleep 30 &
  child=$!
  echo "$*" > "$PROBE_KIND"
  echo "$$ $child" > "$PROBE_MARKER"
  trap 'kill $child 2>/dev/null; wait $child; exit 0' TERM
  wait $child
fi
exec "$REAL_RUSTUP" "$@"
"#).unwrap();fs::set_permissions(&wrapper,fs::Permissions::from_mode(0o755)).unwrap();
 let original=std::env::var_os("PATH").unwrap();
 let real=std::env::split_paths(&original).map(|p|p.join("rustup")).find(|p|p.is_file()).unwrap();
 let mut paths=vec![bin];paths.extend(std::env::split_paths(&original));
 let marker=dir.0.join("probe");let kind=dir.0.join("kind");let hold=dir.0.join("hold");let log=dir.0.join("output");let file=fs::File::create(&log).unwrap();
 let mut command=support::cli();command.current_dir(&root).args(["dev","--verbose"]).env("PATH",std::env::join_paths(paths).unwrap()).env("REAL_RUSTUP",real).env("PROBE_MARKER",&marker).env("HOLD_BUILD",&hold).env("PROBE_KIND",&kind)
  .stdout(file.try_clone().unwrap()).stderr(file).process_group(0);
 let mut dev=Group(command.spawn().unwrap());
 let text=wait_text(&log,"last-good");let old:i32=text.lines().find_map(|line|line.strip_prefix("[dev] Started program (pid ").and_then(|s|s.strip_suffix(")."))).unwrap().parse().unwrap();
 fs::write(&hold,"").unwrap();write("stale-candidate");let pids=wait_text(&marker," ");
 assert_eq!(fs::read_to_string(&kind).unwrap().contains("--crate-type=rlib"),runtime);
 assert_eq!(unsafe{kill(old,0)},0,"last good program stopped during build");
 fs::remove_file(&hold).unwrap();write("newest-version");let text=wait_text(&log,"newest-version");
 assert!(text.contains("runtime=miss") && text.contains("runtime=hit"),"{text}");
 assert!(!text.contains("stale-candidate"),"stale executable ran: {text}");
 assert_ne!(unsafe{kill(old,0)},0,"old program not reaped");
 for pid in pids.split_whitespace().map(|s|s.parse::<i32>().unwrap()) {assert_ne!(unsafe{kill(pid,0)},0,"cancelled compiler descendant survived");}
 unsafe{kill(dev.0.id()as i32,2);}wait_text(&log,"[dev] Stopped.");assert!(dev.0.wait().unwrap().success());
}
