#![cfg(unix)]
mod support;
use std::{fs,process::{Child,Stdio},time::{Duration,Instant}};
unsafe extern "C" {fn kill(pid:i32,signal:i32)->i32;}
struct Dev(Child);
impl Drop for Dev {fn drop(&mut self){if self.0.try_wait().unwrap().is_none(){unsafe{kill(self.0.id() as i32,2);}let _=self.0.wait();}}}
fn wait(path:&std::path::Path,needle:&str)->String {
 let start=Instant::now();loop{let text=fs::read_to_string(path).unwrap_or_default();if text.contains(needle){return text;}assert!(start.elapsed()<Duration::from_secs(40),"missing {needle}: {text}");std::thread::sleep(Duration::from_millis(25));}
}
#[test]
fn dev_colors_statuses_without_coloring_program_output_and_respects_plain_modes(){
 let dir=support::TempDirectory::new();support::success(&support::run(support::cli().current_dir(&dir.0).args(["new","colors"]),20));let root=dir.0.join("colors");
 for (index,args,env,color) in [(0,vec!["--color","always"],None,true),(1,vec!["--color=never"],None,false),(2,vec![],Some(("NO_COLOR","1")),false),(3,vec!["--","--color=always"],None,false)] {
  fs::write(root.join("src/main.tsr"),"function main(){const bad:int=true;}").unwrap();
  let log=dir.0.join(format!("{index}.log"));let out=dir.0.join(format!("{index}.out"));let mut command=support::cli();command.current_dir(&root).arg("dev").args(args).env_remove("NO_COLOR").env("TERM","xterm").stdout(Stdio::from(fs::File::create(&out).unwrap())).stderr(Stdio::from(fs::File::create(&log).unwrap()));if let Some((k,v))=env{command.env(k,v);}
  let mut dev=Dev(command.spawn().unwrap());let text=wait(&log,"Build failed;");
  if color {assert!(text.contains("\x1b[1;36m[dev] Watching"),"{text}");assert!(text.contains("\x1b[1;31m[dev] Build failed"),"{text}");
   fs::write(root.join("src/main.tsr"),support::format_round_trip("function main(){console.log(\"plain-program-output\");}")).unwrap();
   let text=wait(&log,"Built in");assert!(text.contains("\x1b[1;32m[dev] Built in"),"{text}");assert_eq!(wait(&out,"plain-program-output"),"plain-program-output\n");
  } else {assert!(!text.contains('\x1b'),"{text}");}
  unsafe{kill(dev.0.id()as i32,2);}assert!(dev.0.wait().unwrap().success());
 }
}
