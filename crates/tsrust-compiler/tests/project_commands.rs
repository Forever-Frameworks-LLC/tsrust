mod support;
use std::fs;

#[test]
fn new_scaffolds_a_formatted_checkable_project_without_overwriting() {
    let dir=support::TempDirectory::new();
    let output=support::run(support::cli().current_dir(&dir.0).args(["new","hello"]),20);
    support::success(&output);
    let root=dir.0.join("hello");
    for name in ["tsrust.config.json","src/main.tsr","src/greeting.tsr","tests/greeting.test.tsr","README.md",".gitignore"] {
        assert!(root.join(name).is_file(),"{name}");
    }
    for name in ["src/main.tsr","src/greeting.tsr","tests/greeting.test.tsr"] {
        let source=fs::read_to_string(root.join(name)).unwrap();
        assert_eq!(source,support::format_round_trip(&source));
    }
    support::success(&support::run(support::cli().current_dir(root.join("src")).arg("check"),20));
    support::success(&support::run(support::cli().current_dir(&root).args(["format","--check"]),20));
    let original=fs::read(root.join("src/main.tsr")).unwrap();
    let output=support::run(support::cli().current_dir(&dir.0).args(["new","hello"]),20);
    assert!(!output.status.success());assert_eq!(fs::read(root.join("src/main.tsr")).unwrap(),original);
}

#[test]
fn scaffold_uses_config_entry_output_and_root_from_nested_directories() {
    let dir=support::TempDirectory::new();
    support::success(&support::run(support::cli().current_dir(&dir.0).args(["new","hello"]),20));
    let root=dir.0.join("hello");
    let output=support::run(support::cli().current_dir(root.join("src")).arg("run"),60);
    support::success(&output);assert_eq!(String::from_utf8(output.stdout).unwrap(),"Hello from hello!\n");
    support::success(&support::run(support::cli().current_dir(root.join("src")).arg("build"),60));
    assert!(root.join(format!("target/hello{}",std::env::consts::EXE_SUFFIX)).is_file());
    support::success(&support::run(support::cli().current_dir(root.join("src")).arg("test"),60));
    fs::write(root.join("src/main.tsr"),"import {open,close} from 'std:fs';function main(){match(open(\"README.md\")){Ok(file)=>{match(close(file)){Ok(_)=>{console.log(\"project-root\");},Err(error)=>{throw error.message;}}},Err(error)=>{throw error.message;}}}").unwrap();
    let output=support::run(support::cli().current_dir(root.join("src")).arg("run"),60);
    support::success(&output);assert_eq!(String::from_utf8(output.stdout).unwrap(),"project-root\n");
    fs::write(root.join("tests/bad.tsr"),"function bad(){const value:int=\"bad\";}").unwrap();
    assert!(!support::run(support::cli().current_dir(root.join("src")).arg("check"),20).status.success());
}

#[test]
fn invalid_config_and_names_are_actionable_and_do_not_write_files() {
    let dir=support::TempDirectory::new();
    for args in [vec!["new"],vec!["new","bad name"],vec!["new","hello","extra"],vec!["new","--unknown"]] {
        assert!(!support::run(support::cli().current_dir(&dir.0).args(args),20).status.success());
    }
    assert!(fs::read_dir(&dir.0).unwrap().next().is_none());
    fs::write(dir.0.join("tsrust.config.json"),"{\"schemaVersion\":1,\"name\":\"x\",\"build\":{\"entry\":\"../escape.tsr\",\"output\":\"target/x\"}}").unwrap();
    let output=support::run(support::cli().current_dir(&dir.0).arg("run"),20);
    assert!(!output.status.success());assert!(String::from_utf8_lossy(&output.stderr).contains("build.entry"));
    fs::write(dir.0.join("tsrust.config.json"),"{\"schemaVersion\":1,\"name\":\"x\",\"buid\":{}}").unwrap();
    let output=support::run(support::cli().current_dir(&dir.0).arg("check"),20);
    assert!(!output.status.success());assert!(String::from_utf8_lossy(&output.stderr).contains("buid"));
}

#[cfg(unix)]
#[test]
fn dev_restarts_dependencies_keeps_last_good_on_error_and_cleans_up_on_interrupt() {
    use std::{io::{BufRead,BufReader},process::{Child,Stdio},sync::mpsc,time::{Duration,Instant}};
    unsafe extern "C" {fn kill(pid:i32,signal:i32)->i32;}
    struct Dev(Child);
    impl Drop for Dev {
        fn drop(&mut self) {
            unsafe {kill(self.0.id() as i32,15);}
            let until=Instant::now()+Duration::from_secs(5);
            while self.0.try_wait().unwrap().is_none() && Instant::now()<until {std::thread::sleep(Duration::from_millis(20));}
            let _=self.0.kill();let _=self.0.wait();
        }
    }
    fn wait(rx:&mpsc::Receiver<String>,lines:&mut Vec<String>,needle:&str) {
        let until=Instant::now()+Duration::from_secs(30);
        while !lines.iter().any(|line|line.contains(needle)) {
            let line=rx.recv_timeout(until.saturating_duration_since(Instant::now())).unwrap_or_else(|_|panic!("missing {needle}: {lines:?}"));
            lines.push(line);
        }
    }
    fn pid(lines:&[String])->i32 {
        lines.iter().rev().find_map(|line|line.strip_prefix("[dev] Started program (pid ").and_then(|value|value.strip_suffix(").")).map(|value|value.parse().unwrap())).unwrap()
    }
    let dir=support::TempDirectory::new();
    support::success(&support::run(support::cli().current_dir(&dir.0).args(["new","hello"]),20));
    let root=dir.0.join("hello");
    fs::write(root.join("src/main.tsr"),support::format_round_trip("import {greeting} from './greeting.tsr';import {stable} from './stable.tsr';function main(){console.log(greeting());const value=stable();while(true){}}")).unwrap();
    fs::write(root.join("src/stable.tsr"),support::format_round_trip("export function stable():int{return 1;}")).unwrap();
    fs::write(root.join("src/greeting.tsr"),"export function greeting():string{return 42;}").unwrap();
    let mut dev=Dev(support::cli().current_dir(root.join("src")).args(["dev","--verbose","--","--help"]).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap());
    let (tx,rx)=mpsc::channel();
    let out=dev.0.stdout.take().unwrap();let err=dev.0.stderr.take().unwrap();
    let sender=tx.clone();std::thread::spawn(move||{for line in BufReader::new(out).lines(){if sender.send(line.unwrap()).is_err(){break;}}});
    std::thread::spawn(move||{for line in BufReader::new(err).lines(){if tx.send(line.unwrap()).is_err(){break;}}});
    let mut lines=Vec::new();wait(&rx,&mut lines,"Build failed; waiting for changes.");
    assert!(!lines.iter().any(|line|line.contains("Started program")));
    fs::write(root.join("src/greeting.tsr"),"export function greeting():string{return \"first-version\";}").unwrap();
    wait(&rx,&mut lines,"first-version");wait(&rx,&mut lines,"Started program");let old=pid(&lines);
    let config=fs::read(root.join("tsrust.config.json")).unwrap();
    fs::write(root.join("tsrust.config.json"),"{invalid-json").unwrap();
    wait(&rx,&mut lines,"tsrust.config.json:");
    assert_eq!(unsafe {kill(old,0)},0,"config errors must retain last good process");
    fs::write(root.join("tsrust.config.json"),config).unwrap();
    fs::write(root.join("src/greeting.tsr"),"export function greeting():string{return 42;}").unwrap();
    wait(&rx,&mut lines,"Build failed; previous program remains running.");
    assert_eq!(unsafe {kill(old,0)},0,"last good program must still be alive");
    fs::write(root.join("src/greeting.tsr"),"export function greeting():string{return \"second-version\";}").unwrap();
    wait(&rx,&mut lines,"second-version");
    // stderr/stdout reader threads may deliver the start line in either order.
    while lines.iter().filter(|line|line.contains("Started program")).count()<2 {lines.push(rx.recv_timeout(Duration::from_secs(5)).unwrap());}
    assert!(lines.iter().any(|line|line.contains("syntax reused=1") && line.contains("headers reused=2")),"{lines:?}");
    let new=pid(&lines);assert_ne!(old,new);assert_ne!(unsafe {kill(old,0)},0,"old program must be reaped");
    unsafe {kill(dev.0.id() as i32,2);}
    wait(&rx,&mut lines,"[dev] Stopped.");
    assert!(dev.0.wait().unwrap().success());assert_ne!(unsafe {kill(new,0)},0,"interrupted dev must reap its program");
}

#[cfg(unix)]
#[test]
fn dev_is_quiet_by_default_and_reuses_native_builds_across_sessions() {
 use std::{io::{BufRead,BufReader},process::Stdio,sync::mpsc,time::Duration};
 unsafe extern "C" {fn kill(pid:i32,signal:i32)->i32;}
 let dir=support::TempDirectory::new();support::success(&support::run(support::cli().current_dir(&dir.0).args(["new","cache-demo"]),20));let root=dir.0.join("cache-demo");
 for verbose in [false,true] {
  let mut command=support::cli();command.current_dir(&root).arg("dev");if verbose{command.arg("--verbose");}
  let mut child=command.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
  let out=child.stdout.take().unwrap();let err=child.stderr.take().unwrap();let(tx,rx)=mpsc::channel();let sender=tx.clone();
  std::thread::spawn(move||for line in BufReader::new(out).lines(){if sender.send(line.unwrap()).is_err(){break;}});
  std::thread::spawn(move||for line in BufReader::new(err).lines(){if tx.send(line.unwrap()).is_err(){break;}});
  let mut lines=Vec::new();let mut success=false;
  for _ in 0..30 {
   match rx.recv_timeout(Duration::from_secs(2)){Ok(line)=>{success|=line.contains("Hello from cache-demo!");lines.push(line);if success{break;}},Err(_)=>break}
  }
  unsafe {kill(child.id() as i32,2);}
  let status=child.wait().unwrap();lines.extend(rx.try_iter());
  assert!(success&&status.success(),"{lines:?}");
  if verbose {assert!(lines.iter().any(|line|line.contains("cache=hit")),"{lines:?}");}
  else {
   assert!(lines.iter().any(|line|line.contains("Built in")&&line.contains("ms")),"{lines:?}");
   assert!(!lines.iter().any(|line|line.starts_with("Built /")||line.contains("pid ")||line.contains("exit status: 0")),"{lines:?}");
  }
 }
}
