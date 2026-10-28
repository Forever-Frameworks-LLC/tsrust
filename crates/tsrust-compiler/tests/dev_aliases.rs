#![cfg(unix)]
mod support;
use std::{fs,process::{Child,Stdio},time::{Duration,Instant}};
unsafe extern "C" {fn kill(pid:i32,signal:i32)->i32;}
struct Dev(Child);
impl Drop for Dev {fn drop(&mut self){
 if self.0.try_wait().ok().flatten().is_none(){unsafe{kill(self.0.id()as i32,2);}
  let deadline=Instant::now()+Duration::from_secs(5);
  while self.0.try_wait().ok().flatten().is_none()&&Instant::now()<deadline{std::thread::sleep(Duration::from_millis(20));}
  if self.0.try_wait().ok().flatten().is_none(){let _=self.0.kill();}let _=self.0.wait();
 }
}}
fn wait(path:&std::path::Path,needle:&str)->String {
 let deadline=Instant::now()+Duration::from_secs(40);
 loop {let text=fs::read_to_string(path).unwrap_or_default();if text.contains(needle){return text;}assert!(Instant::now()<deadline,"Missing {needle}: {text}");std::thread::sleep(Duration::from_millis(25));}
}
fn config(root:&std::path::Path,target:&str){fs::write(root.join("tsrust.config.json"),format!(r#"{{"schemaVersion":1,"name":"aliases","build":{{"entry":"src/main.tsr","output":"dist/aliases"}},"paths":{{"@/*":"{target}/*"}}}}"#)).unwrap();}
fn pid(log:&str)->i32 {log.lines().find_map(|line|line.split_once("Started program (pid ").map(|(_,tail)|tail.split(')').next().unwrap().parse().unwrap())).unwrap()}
#[test]
fn dev_explicit_cache_directory_is_shared_with_cache_commands_and_reused_on_restart() {
 let dir=support::TempDirectory::new();support::success(&support::run(support::cli().current_dir(&dir.0).args(["new","app"]),20));let root=dir.0.join("app");let cache=dir.0.join("shared cache");
 fs::write(root.join("src/main.tsr"),support::format_round_trip("import {basename} from 'std:path';function main(){console.log(basename('/tmp/shared-ready'));}")).unwrap();
 for expected in ["cache=miss","cache=hit"] {
  let out=dir.0.join("stdout");let err=dir.0.join("stderr");let mut command=support::cli();command.current_dir(&root).args(["dev","--verbose","--color=never"]).env("TSRUST_CACHE_DIR",&cache).stdout(fs::File::create(&out).unwrap()).stderr(fs::File::create(&err).unwrap());
  let mut dev=Dev(command.spawn().unwrap());wait(&out,"shared-ready");let text=wait(&err,"Started program (pid ");assert!(text.contains(expected),"{text}");
  unsafe{kill(dev.0.id()as i32,2);}let deadline=Instant::now()+Duration::from_secs(5);
  loop {if let Some(status)=dev.0.try_wait().unwrap(){assert!(status.success());break;}assert!(Instant::now()<deadline,"dev failed to stop");std::thread::sleep(Duration::from_millis(20));}
 }
 assert!(cache.join("v2").is_dir());assert!(cache.join("runtime/v2").is_dir());assert!(!root.join("target/tsrust-native").exists());
 let output=support::run(support::cli().current_dir(&root).args(["cache","info"]).env("TSRUST_CACHE_DIR",&cache),10);support::success(&output);assert!(String::from_utf8_lossy(&output.stdout).contains("shared cache"));
}
#[test]
fn alias_config_edits_rebuild_and_keep_the_last_program_on_failure(){
 let dir=support::TempDirectory::new();let root=dir.0.join("project");fs::create_dir_all(root.join("src/one")).unwrap();fs::create_dir(root.join("src/two")).unwrap();
 fs::write(root.join("src/main.tsr"),support::format_round_trip("import {greeting} from '@/greeting';function main(){console.log(greeting());while(true){}}" )).unwrap();
 for(name,text)in [("one","first-alias"),("two","second-alias")] {fs::write(root.join(format!("src/{name}/greeting.tsr")),support::format_round_trip(&format!("export function greeting():string{{return '{text}';}}"))).unwrap();}
 let external=dir.0.join("external");fs::rename(root.join("src/one"),&external).unwrap();
 config(&root,"../external");let out=dir.0.join("stdout");let err=dir.0.join("stderr");
 let mut command=support::cli();command.current_dir(&root).args(["dev","--verbose","--color=never"]).stdout(Stdio::from(fs::File::create(&out).unwrap())).stderr(Stdio::from(fs::File::create(&err).unwrap()));
 let mut dev=Dev(command.spawn().unwrap());wait(&out,"first-alias");let initial=pid(&wait(&err,"Started program (pid "));
 fs::write(external.join("greeting.tsr"),support::format_round_trip("export function greeting():string{return 'external-rebuilt';}")).unwrap();
 wait(&out,"external-rebuilt");let text=fs::read_to_string(&err).unwrap();let first:i32=text.rsplit_once("Started program (pid ").unwrap().1.split(')').next().unwrap().parse().unwrap();
 assert_ne!(unsafe{kill(initial,0)},0,"outside-root edit did not replace the original program");
 config(&root,"./missing");wait(&err,"absent from this build snapshot");wait(&err,"previous application remains running");assert_eq!(unsafe{kill(first,0)},0,"last working program was stopped after bad mapping");assert!(!fs::read_to_string(&out).unwrap().contains("second-alias"));
 config(&root,"./src/two");wait(&out,"second-alias");let log=wait(&err,"Rebuilt in");assert!(log.matches("Started program (pid ").count()>=2,"{log}");assert_ne!(unsafe{kill(first,0)},0,"old program survived successful replacement");
 let second:i32=log.rsplit_once("Started program (pid ").unwrap().1.split(')').next().unwrap().parse().unwrap();
 unsafe{kill(dev.0.id()as i32,2);}let deadline=Instant::now()+Duration::from_secs(5);
 loop {if let Some(status)=dev.0.try_wait().unwrap(){assert!(status.success());break;}assert!(Instant::now()<deadline,"dev did not stop");std::thread::sleep(Duration::from_millis(20));}
 assert_ne!(unsafe{kill(second,0)},0,"replacement program survived dev shutdown");
}

#[test]
fn json_data_edits_rebuild_and_preserve_running_program_on_invalid_input() {
 let dir=support::TempDirectory::new();let root=dir.0.join("project");fs::create_dir_all(root.join("src")).unwrap();
 fs::write(root.join("src/main.tsr"),support::format_round_trip("import settings from './settings.jsonc';function main(){console.log(settings.message);while(true){}}" )).unwrap();
 let data=root.join("src/settings.jsonc");fs::write(&data,r#"{"message":"data-first",}"#).unwrap();config(&root,"./src");
 let out=dir.0.join("stdout");let err=dir.0.join("stderr");
 let mut command=support::cli();command.current_dir(&root).args(["dev","--verbose","--color=never"]).stdout(Stdio::from(fs::File::create(&out).unwrap())).stderr(Stdio::from(fs::File::create(&err).unwrap()));
 let mut dev=Dev(command.spawn().unwrap());wait(&out,"data-first");let first=pid(&wait(&err,"Started program (pid "));
 fs::write(&data,"{broken").unwrap();wait(&err,"previous application remains running");assert_eq!(unsafe{kill(first,0)},0,"invalid JSON stopped last working program");
 fs::write(&data,r#"{"message":"data-recovered"}"#).unwrap();wait(&out,"data-recovered");
 let log=fs::read_to_string(&err).unwrap();let second:i32=log.rsplit_once("Started program (pid ").unwrap().1.split(')').next().unwrap().parse().unwrap();
 assert_ne!(first,second);assert_ne!(unsafe{kill(first,0)},0,"old program survived successful data rebuild");
 unsafe{kill(dev.0.id()as i32,2);}let deadline=Instant::now()+Duration::from_secs(5);
 loop{if let Some(status)=dev.0.try_wait().unwrap(){assert!(status.success());break;}assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(20));}
 assert_ne!(unsafe{kill(second,0)},0,"data program survived dev shutdown");
}
