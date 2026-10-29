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
    wait(&rx,&mut lines,"Build failed; previous application remains running.");
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
 use std::{io::{BufRead,BufReader},process::Stdio,sync::mpsc,time::{Duration,Instant}};
 unsafe extern "C" {fn kill(pid:i32,signal:i32)->i32;}
 let dir=support::TempDirectory::new();support::success(&support::run(support::cli().current_dir(&dir.0).args(["new","cache-demo"]),20));let root=dir.0.join("cache-demo");
 for verbose in [false,true] {
  let mut command=support::cli();command.current_dir(&root).arg("dev");if verbose{command.arg("--verbose");}
  let mut child=command.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
  let out=child.stdout.take().unwrap();let err=child.stderr.take().unwrap();let(tx,rx)=mpsc::channel();let sender=tx.clone();
  std::thread::spawn(move||for line in BufReader::new(out).lines(){if sender.send(line.unwrap()).is_err(){break;}});
  std::thread::spawn(move||for line in BufReader::new(err).lines(){if tx.send(line.unwrap()).is_err(){break;}});
  let mut lines=Vec::new();let mut success=false;
  let deadline=Instant::now()+Duration::from_secs(30);
  while Instant::now()<deadline {
   match rx.recv_timeout(deadline.saturating_duration_since(Instant::now())){Ok(line)=>{success|=line.contains("Hello from cache-demo!");lines.push(line);if success{break;}},Err(_)=>break}
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

#[test]
fn run_forwards_program_options_after_separator_with_explicit_and_configured_inputs() {
    let dir=support::TempDirectory::new();
    let source=support::format_round_trip(r#"function main(){console.log("ran program");}"#);
    fs::write(dir.0.join("main.tsr"),source).unwrap();
    fs::write(dir.0.join("tsrust.config.json"),r#"{"schemaVersion":1,"name":"arguments","build":{"entry":"main.tsr","output":"target/app"}}"#).unwrap();
    fs::create_dir(dir.0.join("nested")).unwrap();
    for explicit in [false,true] {
        let mut command=support::cli();command.current_dir(dir.0.join("nested")).arg("run");
        if explicit {command.arg("../main.tsr");}
        command.args(["--","--help","--release","--color=invalid","","two words","λ","--"]);
        let output=support::run(&mut command,60);support::success(&output);
        assert_eq!(String::from_utf8(output.stdout).unwrap(),"ran program\n");
    }
    let output=support::run(support::cli().current_dir(&dir.0).args(["run","main.tsr","--unknown"]),10);
    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr).unwrap().contains("Unknown or repeated option"));
}

#[cfg(feature = "legacy-ui")]
#[test]
fn new_ui_scaffolds_an_explicit_bootstrap_and_root_page() {
    let dir=support::TempDirectory::new();
    support::success(&support::run(support::cli().current_dir(&dir.0).args(["new","counter","--ui","--target","native"]),20));
    let root=dir.0.join("counter");
    for name in ["src/main.tsr","src/app/page.tsrx","tests/page.test.tsr"] {
        let source=fs::read_to_string(root.join(name)).unwrap();
        assert_eq!(source,support::format_round_trip(&source));
    }
    assert!(!root.join("src/greeting.tsr").exists());
    support::success(&support::run(support::cli().current_dir(&root).args(["check","--json"]),20));
    support::success(&support::run(support::cli().current_dir(&root).args(["format","--check"]),20));
    support::success(&support::run(support::cli().current_dir(&root).arg("test"),60));
    let original=fs::read(root.join("src/app/page.tsrx")).unwrap();
    assert!(!support::run(support::cli().current_dir(&dir.0).args(["new","counter","--ui","--target","native"]),20).status.success());
    assert_eq!(fs::read(root.join("src/app/page.tsrx")).unwrap(),original);
}

#[cfg(feature = "legacy-ui")]
#[test]
fn ui_add_bootstraps_inline_config_and_source_layout_without_init() {
 let dir=support::TempDirectory::new();let root=dir.0.join("first-ui");fs::create_dir(&root).unwrap();
 let run=|args:&[&str]|support::run(support::cli().current_dir(&root).args(args),30);
 support::success(&run(&["ui","add","button@3.0.0","--target","native","--dry-run","--json"]));assert_eq!(fs::read_dir(&root).unwrap().count(),0);
 assert!(!run(&["ui","add","button@missing"]).status.success());assert_eq!(fs::read_dir(&root).unwrap().count(),0);
 let output=run(&["ui","add","button@3.0.0","--target","native","--offline"]);support::success(&output);assert!(String::from_utf8(output.stdout).unwrap().contains("withTheme"));
 let config:serde_json::Value=serde_json::from_slice(&fs::read(root.join("tsrust.config.json")).unwrap()).unwrap();assert!(config["ui"]["theme"].is_object());
 for path in ["src/app/page.tsrx","src/components/ui/button.tsrx","src/main.tsr","ui-components.lock.json"]{assert!(root.join(path).is_file(),"{path}");}
 assert!(!root.join("ui.theme.json").exists());assert!(!root.join("app").exists());
 support::success(&run(&["format"]));support::success(&run(&["format","--check"]));support::success(&run(&["check"]));
 let before=fs::read(root.join("tsrust.config.json")).unwrap();let lock=fs::read(root.join("ui-components.lock.json")).unwrap();
 fs::create_dir(root.join("nested")).unwrap();support::success(&support::run(support::cli().current_dir(root.join("nested")).args(["ui","add","button@3.0.0"]),30));
 assert_eq!(fs::read(root.join("tsrust.config.json")).unwrap(),before);assert_eq!(fs::read(root.join("ui-components.lock.json")).unwrap(),lock);
}

#[cfg(feature = "legacy-ui")]
#[test]
fn ui_imports_report_explicit_setup_without_writing() {
 let dir=support::TempDirectory::new();let root=dir.0.join("import-ui");fs::create_dir_all(root.join("src")).unwrap();
 let source=support::format_round_trip("import {Button} from './components/ui/button';import {document} from 'std:ui';function main(){console.log(document(<Button disabled>Imported</Button>,'UI'));}");
 fs::write(root.join("src/main.tsrx"),&source).unwrap();
 let check=support::run(support::cli().current_dir(&root).args(["check"]),15);assert!(!check.status.success());assert!(!root.join("tsrust.config.json").exists());
 let bad=support::run(support::cli().current_dir(&root).args(["run","--unknown"]),15);assert!(!bad.status.success());assert!(!root.join("tsrust.config.json").exists());
 let output=support::run(support::cli().current_dir(&root).args(["run"]),20);assert!(!output.status.success());assert!(String::from_utf8(output.stderr).unwrap().contains("tsrust ui add button@3.0.0"));assert!(!root.join("tsrust.config.json").exists());assert!(!root.join("src/components").exists());
 support::success(&support::run(support::cli().current_dir(&root).args(["ui","add","button@3.0.0","--target","native"]),30));
 assert_eq!(fs::read_to_string(root.join("src/main.tsrx")).unwrap(),source);
 support::success(&support::run(support::cli().current_dir(&root).args(["check"]),20));
}

#[cfg(feature = "legacy-ui")]
#[test]
fn ui_web_only_target_is_explicit_and_cannot_launch_native(){
 let dir=support::TempDirectory::new();let root=dir.0.join("web-ui");fs::create_dir(&root).unwrap();
 let no_target=support::run(support::cli().current_dir(&root).args(["ui","add","button@3.0.0","--yes"]),20);assert!(!no_target.status.success());assert_eq!(fs::read_dir(&root).unwrap().count(),0);
 let added=support::run(support::cli().current_dir(&root).args(["ui","add","button@3.0.0","--target","web"]),30);support::success(&added);assert!(!String::from_utf8(added.stdout).unwrap().contains("run tsrust dev"));
 let config:serde_json::Value=serde_json::from_slice(&fs::read(root.join("tsrust.config.json")).unwrap()).unwrap();assert_eq!(config["ui"]["target"],serde_json::json!({"native":false,"web":true,"embedded":false}));
 let output=support::run(support::cli().current_dir(&root).args(["build"]),20);assert!(!output.status.success());assert!(String::from_utf8(output.stderr).unwrap().contains("native target is disabled"));
 let package=root.join("export");support::success(&support::run(support::cli().current_dir(&root).args(["emit","--package","-o"]).arg(&package),30));let manifest=fs::read_to_string(package.join("Cargo.toml")).unwrap();let defaults=manifest.lines().find(|line|line.starts_with("default =")).unwrap();assert!(defaults.contains("ui-web") && !defaults.contains("ui-native"),"{defaults}");
}

#[cfg(feature = "legacy-ui")]
#[test]
fn ui_batch_upgrades_preserve_owned_files_and_check_versioned_imports() {
 let dir=support::TempDirectory::new();let root=&dir.0;let components=root.join("src/components/ui");fs::create_dir_all(&components).unwrap();
 for name in ["button","card"]{fs::write(components.join(format!("{name}.tsrx")),"// Owner component\nexport const owner = 1;\n").unwrap();}
 let source="import {Button} from './components/ui/button-v3';import {Card} from './components/ui/card-v3';import {document} from 'std:ui';function main(){console.log(document(<Card><Button>Current components</Button></Card>,'UI'));}";
 fs::write(root.join("src/main.tsrx"),source).unwrap();let original=fs::read(root.join("src/main.tsrx")).unwrap();
 let preview=support::run(support::cli().args(["ui","add","button","card","--target","native","--dry-run","--json"]).current_dir(root),20);support::success(&preview);let preview:serde_json::Value=serde_json::from_slice(&preview.stdout).unwrap();assert!(preview.to_string().contains("button-v3.tsrx")&&preview.to_string().contains("card-v3.tsrx"));assert!(!root.join("tsrust.config.json").exists());
 let obsolete=support::run(support::cli().args(["ui","add","card@2.0.0","--target","native"]).current_dir(root),20);assert!(!obsolete.status.success());assert!(String::from_utf8_lossy(&obsolete.stderr).contains("Only card@3.0.0"));assert!(!root.join("tsrust.config.json").exists());
 support::success(&support::run(support::cli().args(["ui","add","button","card","--target","native"]).current_dir(root),30));
 for name in ["button","card"]{assert_eq!(fs::read_to_string(components.join(format!("{name}.tsrx"))).unwrap(),"// Owner component\nexport const owner = 1;\n");}assert_eq!(fs::read(root.join("src/main.tsrx")).unwrap(),original);
 let lock=fs::read(root.join("ui-components.lock.json")).unwrap();support::success(&support::run(support::cli().args(["ui","add","card","button"]).current_dir(root),30));assert_eq!(fs::read(root.join("ui-components.lock.json")).unwrap(),lock);
 support::success(&support::run(support::cli().args(["check","src/main.tsrx"]).current_dir(root),30));
}
