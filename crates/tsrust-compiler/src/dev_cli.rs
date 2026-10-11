//! Native development supervisor. Builds never replace a running executable inode.
#[cfg(unix)]
mod platform {
    use std::{collections::BTreeMap,fs,path::{Path,PathBuf},process::{Child,Command},sync::atomic::{AtomicBool,Ordering},time::{Duration,Instant}};
    use std::os::unix::process::CommandExt;
    use crate::{project_config,TempDirectory};
    static STOP:AtomicBool=AtomicBool::new(false);
    unsafe extern "C" {fn signal(number:i32,handler:usize)->usize;fn kill(pid:i32,signal:i32)->i32;}
    extern "C" fn stop(_:i32) {STOP.store(true,Ordering::Relaxed);}
    struct Signals {interrupt:usize,terminate:usize}
    impl Signals {
        fn install()->Result<Self,String> {
            STOP.store(false,Ordering::Relaxed);
            // SAFETY: the handler only stores to a lock-free atomic flag. Restore
            // both previous dispositions when this supervisor returns.
            unsafe {
                let interrupt=signal(2,stop as *const () as usize);
                if interrupt==usize::MAX {return Err("Cannot install dev interrupt handler".into());}
                let terminate=signal(15,stop as *const () as usize);
                if terminate==usize::MAX {signal(2,interrupt);return Err("Cannot install dev termination handler".into());}
                Ok(Self{interrupt,terminate})
            }
        }
    }
    impl Drop for Signals {fn drop(&mut self){unsafe {signal(2,self.interrupt);signal(15,self.terminate);}}}
    struct Process {child:Child,stopped:bool}
    impl Process {
        fn spawn(command:&mut Command)->Result<Self,String> {
            command.process_group(0);
            Ok(Self{child:command.spawn().map_err(|e|e.to_string())?,stopped:false})
        }
        fn stop(&mut self) {
            if self.stopped{return;}
            self.stopped=true;
            let group=-(self.child.id() as i32);
            // SAFETY: each child starts its own process group. The negative PID
            // targets only that group, including compiler/program descendants.
            unsafe {kill(group,15);}
            let until=Instant::now()+Duration::from_millis(500);
            while Instant::now()<until {
                if self.child.try_wait().ok().flatten().is_some(){break;}
                std::thread::sleep(Duration::from_millis(20));
            }
            unsafe {kill(group,9);}
            let _=self.child.wait();
        }
    }
    impl Drop for Process {fn drop(&mut self){self.stop();}}
    struct Running {process:Process,_files:TempDirectory}
    type Fingerprint=Result<BTreeMap<PathBuf,Vec<u8>>,String>;
    fn fingerprint(root:&Path)->Fingerprint {
        let mut files=tsrust_compiler::source_discovery::discover(&[root.to_owned()])?;
        files.push(root.join(project_config::FILE));
        let mut values=BTreeMap::new();
        for path in files {values.insert(path.clone(),fs::read(&path).map_err(|e|format!("{}: {e}",path.display()))?);}
        Ok(values)
    }
    fn build(root:&Path,release:bool,expected:&Fingerprint)->Result<Option<(TempDirectory,PathBuf)>,String> {
        let project=project_config::load(root)?;
        let files=TempDirectory::new().map_err(|e|e.to_string())?;
        let binary=files.0.join("program");
        let mut command=Command::new(std::env::current_exe().map_err(|e|e.to_string())?);
        command.current_dir(root).arg("build").arg(&project.entry).arg("--output").arg(&binary)
            .env("TSRUST_CACHE_DIR",files.0.join("cache"));
        if release {command.arg("--release");}
        let mut compiler=Process::spawn(&mut command)?;
        let status=loop {
            if STOP.load(Ordering::Relaxed) || &fingerprint(root)!=expected {return Ok(None);}
            if let Some(status)=compiler.child.try_wait().map_err(|e|e.to_string())? {break status;}
            std::thread::sleep(Duration::from_millis(100));
        };
        compiler.stop();
        if !status.success(){return Err("Build failed".into());}
        // A private per-build cache must not accumulate across edit cycles.
        let _=fs::remove_dir_all(files.0.join("cache"));
        Ok(Some((files,binary)))
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn stopping_a_process_group_reaps_its_shell_and_descendant() {
            use std::io::{BufRead,BufReader};
            use std::process::Stdio;
            let mut command=Command::new("/bin/sh");
            command.args(["-c","sleep 30 & child=$!; echo $child; trap 'kill $child 2>/dev/null; wait $child; exit 0' TERM; wait $child"])
                .stdout(Stdio::piped()).stderr(Stdio::null());
            let mut process=Process::spawn(&mut command).unwrap();
            let mut line=String::new();BufReader::new(process.child.stdout.take().unwrap()).read_line(&mut line).unwrap();
            let descendant:i32=line.trim().parse().unwrap();
            process.stop();
            assert!(process.child.try_wait().unwrap().is_some());
            assert_ne!(unsafe {kill(descendant,0)},0,"descendant survived process-group cleanup");
        }
    }
    pub fn run(args:impl Iterator<Item=String>)->Result<u8,String> {
        let mut release=false;let mut program_args=Vec::new();let mut passthrough=false;
        for arg in args {
            if passthrough {program_args.push(arg);continue;}
            match arg.as_str() {
                "--"=>passthrough=true,
                "--release" if !release=>release=true,
                _=>return Err(format!("Unknown or repeated dev option: {arg}. Usage: tsrust dev [--release] [-- program-arguments]")),
            }
        }
        let project=project_config::discover()?.ok_or("dev needs tsrust.config.json; create a project with `tsrust new <name>`")?;
        let _signals=Signals::install()?;
        let root=project.root;
        eprintln!("[dev] Watching {}. Press Ctrl+C to stop.",root.display());
        let mut previous:Option<Fingerprint>=None;let mut running:Option<Running>=None;
        while !STOP.load(Ordering::Relaxed) {
            let input=fingerprint(&root);
            if previous.as_ref()!=Some(&input) {
                // Require one quiet interval; a new edit replaces pending work.
                std::thread::sleep(Duration::from_millis(100));
                if fingerprint(&root)!=input {continue;}
                previous=Some(input.clone());
                let result=match &input {Err(error)=>Err(error.clone()),Ok(_)=>build(&root,release,&input)};
                match result {
                    Ok(Some((files,binary))) if !STOP.load(Ordering::Relaxed) && fingerprint(&root)==input=>{
                        if let Some(mut old)=running.take(){old.process.stop();}
                        let mut command=Command::new(binary);command.current_dir(&root).args(&program_args);
                        match Process::spawn(&mut command) {
                            Ok(process)=>{
                                eprintln!("[dev] Started program (pid {}).",process.child.id());
                                running=Some(Running{process,_files:files});
                            },
                            Err(error)=>eprintln!("[dev] Cannot start program: {error}"),
                        }
                    },
                    Ok(_)=>{},
                    Err(error) if error=="Build failed"=>eprintln!("[dev] Build failed; {}.",if running.is_some(){"previous program remains running"}else{"waiting for changes"}),
                    Err(error)=>eprintln!("[dev] {error}"),
                }
            }
            if let Some(active)=&mut running {
                if let Some(status)=active.process.child.try_wait().map_err(|e|e.to_string())? {
                    eprintln!("[dev] Program exited with {status}; waiting for changes.");
                    running.take();
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        drop(running);eprintln!("[dev] Stopped.");Ok(0)
    }
}
#[cfg(unix)] pub use platform::run;
#[cfg(not(unix))]
pub fn run(_:impl Iterator<Item=String>)->Result<u8,String> {Err("dev process supervision currently supports macOS and Linux".into())}
