//! Native development supervisor. Builds never replace a running executable inode.
#[cfg(unix)]
mod platform {
    use std::{path::{Path,PathBuf},process::{Child,Command},sync::atomic::{AtomicBool,Ordering},time::{Duration,Instant}};
    use std::os::unix::process::CommandExt;
    use crate::{dev_watch::{Fingerprint,Watcher},project_config,TempDirectory};
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
    fn wait_for_build(process:&mut Process,root:&Path,expected:&Fingerprint,watcher:&mut Watcher)->Result<Option<std::process::ExitStatus>,String> {
        let mut last_input_check=Instant::now();
        loop {
            if STOP.load(Ordering::Relaxed){return Ok(None);}
            if last_input_check.elapsed()>=Duration::from_millis(100) {
                if &watcher.snapshot(root)!=expected{return Ok(None);}
                last_input_check=Instant::now();
            }
            if let Some(status)=process.child.try_wait().map_err(|e|e.to_string())? {process.stop();return Ok(Some(status));}
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    fn build(root:&Path,release:bool,verbose:bool,expected:&Fingerprint,watcher:&mut Watcher,worker:&crate::dev_frontend::Worker)->Result<Option<(TempDirectory,PathBuf)>,String> {
        use std::sync::mpsc::{self,TrySendError,TryRecvError};
        let inputs=expected.as_ref().map_err(Clone::clone)?;
        let config=inputs.get(&root.join(project_config::FILE)).ok_or("Missing project configuration in build snapshot")?;
        let project=project_config::parse(root,std::str::from_utf8(config.bytes()).map_err(|e|e.to_string())?)?;
        let (reply,response)=mpsc::channel();
        let mut request=crate::dev_frontend::Request{entry:project.entry,sources:crate::dev_frontend::Snapshot{root:root.to_owned(),files:expected.as_ref().map_err(Clone::clone)?.clone()},reply};
        loop {
            if STOP.load(Ordering::Relaxed) || &watcher.snapshot(root)!=expected {return Ok(None);}
            match worker.requests.try_send(request) {
                Ok(())=>break,
                Err(TrySendError::Full(pending))=>request=pending,
                Err(TrySendError::Disconnected(_))=>return Err("Frontend worker stopped".into()),
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let mut last_input_check=Instant::now();
        let frontend=loop {
            if STOP.load(Ordering::Relaxed){return Ok(None);}
            if last_input_check.elapsed()>=Duration::from_millis(100) {
                if &watcher.snapshot(root)!=expected {return Ok(None);}
                last_input_check=Instant::now();
            }
            match response.try_recv() {
                Ok(Ok(output))=>break output,
                Ok(Err(error))=>{if &watcher.snapshot(root)!=expected{return Ok(None);}eprint!("{error}");return Err("Build failed".into());},
                Err(TryRecvError::Disconnected)=>return Err("Frontend worker stopped".into()),
                Err(TryRecvError::Empty)=>std::thread::sleep(Duration::from_millis(20)),
            }
        };
        if &watcher.snapshot(root)!=expected {return Ok(None);}
        if verbose {eprintln!("[build] syntax reused={} parsed={} headers reused={} parsed={}",frontend.stats.reused,frontend.stats.parsed,frontend.stats.headers_reused,frontend.stats.headers_parsed);}
        let files=TempDirectory::new().map_err(|e|e.to_string())?;
        let backend_started=Instant::now();
        let probe_out=files.0.join("backend-version.stdout");let probe_err=files.0.join("backend-version.stderr");
        let mut command=crate::backend_command();command.arg("--version")
            .stdout(std::fs::File::create(&probe_out).map_err(|e|e.to_string())?)
            .stderr(std::fs::File::create(&probe_err).map_err(|e|e.to_string())?);
        let mut probe=Process::spawn(&mut command)?;
        let Some(status)=wait_for_build(&mut probe,root,expected,watcher)? else{return Ok(None);};
        let backend_identity=crate::validate_backend_output(std::process::Output{status,stdout:std::fs::read(probe_out).map_err(|e|e.to_string())?,stderr:std::fs::read(probe_err).map_err(|e|e.to_string())?})?;
        let backend_elapsed=backend_started.elapsed();
        let binary=files.0.join("program");
        let features=frontend.features.join(",");
        let cache=tsrust_compiler::native_cache::NativeCache::new(&root.join("target/tsrust-native"),&[
            b"native-cache-abi-2",backend_identity.as_bytes(),env!("CARGO_PKG_VERSION").as_bytes(),std::env::consts::OS.as_bytes(),std::env::consts::ARCH.as_bytes(),
            if release{b"release"}else{b"debug"},b"edition=2024;overflow-checks=on",frontend.rust.as_bytes(),&frontend.identity,features.as_bytes(),
        ]);
        let cache_started=Instant::now();let reused=cache.try_copy_to(&binary);let cache_elapsed=cache_started.elapsed();
        let native_started=Instant::now();
        if !reused {
            let mut prepared=crate::runtime_build::Prepared::for_dev(&frontend.rust,&backend_identity,&root.join("target/tsrust-native"),&files.0,release)?;
            if let Some(mut command)=prepared.command.take() {
                let errors=files.0.join("runtime.stderr");
                command.stdout(std::process::Stdio::null()).stderr(std::fs::File::create(&errors).map_err(|e|e.to_string())?);
                let mut compiler=Process::spawn(&mut command)?;
                let Some(status)=wait_for_build(&mut compiler,root,expected,watcher)? else{return Ok(None);};
                if !status.success(){return Err(format!("Runtime compilation failed:\n{}",std::fs::read_to_string(errors).unwrap_or_default()));}
                prepared.publish();
            }
            if verbose{eprintln!("[build] runtime={}",prepared.state);}
            // A file avoids pipe backpressure while the supervisor watches edits.
            let errors=files.0.join("backend.stderr");
            let mut command=crate::backend_command();prepared.link(&mut command);
            command.current_dir(root).arg("--edition=2024").args(["-C",if release{"opt-level=3"}else{"opt-level=0"}]).args(["-C","overflow-checks=on"]).arg(&prepared.application).arg("-o").arg(&binary)
                .stdout(std::process::Stdio::null()).stderr(std::fs::File::create(&errors).map_err(|e|e.to_string())?);
            let mut compiler=Process::spawn(&mut command)?;
            let Some(status)=wait_for_build(&mut compiler,root,expected,watcher)? else{return Ok(None);};
            if !status.success(){return Err(format!("Native build failed:\n{}",std::fs::read_to_string(errors).unwrap_or_default()));}
            let _=cache.try_publish(&binary);
        }
        if verbose {eprintln!("[build] check={:.1}ms emit={:.1}ms backend={:.1}ms cache={} ({:.1}ms) native+link={:.1}ms",frontend.check.as_secs_f64()*1000.0,frontend.emit.as_secs_f64()*1000.0,backend_elapsed.as_secs_f64()*1000.0,if reused{"hit"}else{"miss"},cache_elapsed.as_secs_f64()*1000.0,if reused{0.0}else{native_started.elapsed().as_secs_f64()*1000.0});}
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
        let mut release=false;let mut verbose=false;let mut program_args=Vec::new();let mut passthrough=false;
        for arg in args {
            if passthrough {program_args.push(arg);continue;}
            match arg.as_str() {
                "--"=>passthrough=true,
                "--release" if !release=>release=true,
                "--verbose" if !verbose=>verbose=true,
                _=>return Err(format!("Unknown or repeated dev option: {arg}. Usage: tsrust dev [--release] [--verbose] [-- program-arguments]")),
            }
        }
        let project=project_config::discover()?.ok_or("dev needs tsrust.config.json; create a project with `tsrust new <name>`")?;
        let _signals=Signals::install()?;
        let root=project.root;
        eprintln!("[dev] Watching {}. Press Ctrl+C to stop.",root.display());
        let worker=crate::dev_frontend::Worker::new()?;
        let mut watcher=Watcher::default();
        let mut previous:Option<Fingerprint>=None;let mut running:Option<Running>=None;let mut built_once=false;
        while !STOP.load(Ordering::Relaxed) {
            let input=watcher.snapshot(&root);
            if previous.as_ref()!=Some(&input) {
                // Debounce edits, but initial startup has no preceding edit burst.
                if previous.is_some() {
                    std::thread::sleep(Duration::from_millis(100));
                    if watcher.snapshot(&root)!=input {continue;}
                }
                previous=Some(input.clone());
                let build_started=Instant::now();
                let result=match &input {Err(error)=>Err(error.clone()),Ok(_)=>build(&root,release,verbose,&input,&mut watcher,&worker)};
                match result {
                    Ok(Some((files,binary))) if !STOP.load(Ordering::Relaxed) && watcher.snapshot(&root)==input=>{
                        eprintln!("[dev] {} in {:.0} ms",if built_once {"Rebuilt"} else {"Built"},build_started.elapsed().as_secs_f64()*1000.0);built_once=true;
                        if let Some(mut old)=running.take(){old.process.stop();}
                        let mut command=Command::new(binary);command.current_dir(&root).args(&program_args);
                        match Process::spawn(&mut command) {
                            Ok(process)=>{
                                if verbose {eprintln!("[dev] Started program (pid {}).",process.child.id());}
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
                    if verbose || !status.success() {eprintln!("[dev] Program exited with {status}; waiting for changes.");}
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
