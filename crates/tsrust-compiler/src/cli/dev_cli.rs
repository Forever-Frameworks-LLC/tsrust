//! Native development supervisor. Builds never replace a running executable inode.
#[cfg(unix)]
#[derive(Clone, Copy)]
pub(crate) enum Status { Info, Success, Warning, Error }
#[cfg(unix)]
pub(crate) fn report(status:Status,message:impl std::fmt::Display) {
    use std::io::IsTerminal;
    let code=match status {Status::Info=>"1;36",Status::Success=>"1;32",Status::Warning=>"1;33",Status::Error=>"1;31"};
    eprintln!("{}",crate::styled(&format!("[dev] {message}"),code,crate::color_enabled(std::io::stderr().is_terminal())));
}
#[cfg(unix)]
mod platform {
    use super::{report,Status};
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
    enum Artifact {Native(TempDirectory,PathBuf),Web}
    fn build(root:&Path,release:bool,verbose:bool,expected:&Fingerprint,watcher:&mut Watcher,worker:&crate::dev_frontend::Worker,web:Option<&mut crate::dev_web::Web>)->Result<Option<Artifact>,String> {
        use std::sync::mpsc::{self,TrySendError,RecvTimeoutError};
        let inputs=expected.as_ref().map_err(Clone::clone)?;
        let config=inputs.get(&root.join(project_config::FILE)).ok_or("Missing project configuration in build snapshot")?;
        let project=project_config::parse(root,std::str::from_utf8(config.bytes()).map_err(|e|e.to_string())?)?;
        let (reply,response)=mpsc::channel();
        let mut request=crate::dev_frontend::Request{entry:project.entry,target:if web.is_some(){"web"}else{"native"},sources:crate::dev_frontend::Snapshot{root:root.to_owned(),files:expected.as_ref().map_err(Clone::clone)?.clone()},reply};
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
            match response.recv_timeout(Duration::from_millis(20)) {
                Ok(Ok(output))=>break output,
                Ok(Err(error))=>{if &watcher.snapshot(root)!=expected{return Ok(None);}eprint!("{error}");return Err("Build failed".into());},
                Err(RecvTimeoutError::Disconnected)=>return Err("Frontend worker stopped".into()),
                Err(RecvTimeoutError::Timeout)=>{},
            }
        };
        if &watcher.snapshot(root)!=expected {return Ok(None);}
        if verbose {eprintln!("[build] syntax reused={} parsed={} headers reused={} parsed={}",frontend.stats.reused,frontend.stats.parsed,frontend.stats.headers_reused,frontend.stats.headers_parsed);}
        if let Some(web)=web {
            let web_started=Instant::now();
            let built=web.build(&frontend.rust,root,release,|command|{
                let mut process=Process::spawn(command)?;
                match wait_for_build(&mut process,root,expected,watcher)? {Some(status) if status.success()=>Ok(true),Some(status)=>Err(format!("Web tool failed ({status})")),None=>Ok(false)}
            })?;
            if verbose {eprintln!("[build] check={:.1}ms emit={:.1}ms web={:.1}ms",frontend.check.as_secs_f64()*1000.0,frontend.emit.as_secs_f64()*1000.0,web_started.elapsed().as_secs_f64()*1000.0);}
            return Ok(built.then_some(Artifact::Web));
        }
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
        let cache_root=std::env::var_os("TSRUST_CACHE_DIR").map(PathBuf::from).unwrap_or_else(||root.join("target/tsrust-native"));
        let cache=tsrust_compiler::native_cache::NativeCache::new(&cache_root,&[
            b"native-cache-abi-4-ui-dev-profile",crate::native_dependency_bundle::IDENTITY.as_bytes(),backend_identity.as_bytes(),env!("CARGO_PKG_VERSION").as_bytes(),std::env::consts::OS.as_bytes(),std::env::consts::ARCH.as_bytes(),
            if release{b"release"}else{b"debug"},b"edition=2024;overflow-checks=on",frontend.rust.as_bytes(),&frontend.identity,features.as_bytes(),
        ]);
        let cache_started=Instant::now();let reused=cache.try_copy_to(&binary);let cache_elapsed=cache_started.elapsed();
        let native_started=Instant::now();
        if !reused {
            let prepare_started=Instant::now();
            let mut prepared=crate::runtime_build::Prepared::for_dev(&frontend.rust,&backend_identity,&cache_root,&files.0,release)?;
            if verbose{eprintln!("[build] runtime-prepare={:.1}ms",prepare_started.elapsed().as_secs_f64()*1000.);}
            if let Some(mut command)=prepared.command.take() {
                let runtime_started=Instant::now();
                if verbose{if let Some(directory)=command.get_current_dir(){eprintln!("[build] dependency-workspace={}",directory.display());}}
                if frontend.features.contains(&"ui") { report(Status::Info,"Preparing native UI renderer (runtime cache miss)."); }
                let errors=files.0.join("runtime.stderr");
                command.stdout(std::process::Stdio::null()).stderr(std::fs::File::create(&errors).map_err(|e|e.to_string())?);
                let mut compiler=Process::spawn(&mut command)?;
                let Some(status)=wait_for_build(&mut compiler,root,expected,watcher)? else{return Ok(None);};
                if !status.success(){return Err(format!("Runtime compilation failed:\n{}",std::fs::read_to_string(errors).unwrap_or_default()));}
                prepared.publish();
                if verbose{eprintln!("[build] runtime-build+publish={:.1}ms",runtime_started.elapsed().as_secs_f64()*1000.);}
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
        Ok(Some(Artifact::Native(files,binary)))
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
        let mut release=false;let mut verbose=false;let mut target=None;let mut port=None;let mut program_args=Vec::new();let mut passthrough=false;
        let mut args=args.peekable();
        while let Some(arg)=args.next() {
            if passthrough {program_args.push(arg);continue;}
            match arg.as_str() {
                "--"=>passthrough=true,
                "--target" if target.is_none()=>{let value=args.next().ok_or("--target requires native or web")?;if !matches!(value.as_str(),"native"|"web"){return Err("--target requires native or web".into());}target=Some(value);},
                "--port" if port.is_none()=>{port=Some(args.next().ok_or("--port requires 0 through 65535")?.parse::<u16>().map_err(|_|"--port requires 0 through 65535")?);},
                "--release" if !release=>release=true,
                "--verbose" if !verbose=>verbose=true,
                "--color"=>{let mode=args.next().ok_or("--color requires auto, always or never")?;if !matches!(mode.as_str(),"auto"|"always"|"never"){return Err("--color requires auto, always or never".into());}},
                _ if arg.starts_with("--color=")=>{if !matches!(&arg[8..],"auto"|"always"|"never"){return Err("--color requires auto, always or never".into());}},
                _=>return Err(format!("Unknown or repeated dev option: {arg}. Usage: tsrust dev [--target native|web] [--port 4317] [--release] [--verbose] [--color auto|always|never] [-- program-arguments]")),
            }
        }
        crate::ui_cli::check_default_setup()?;
        let project=project_config::discover()?.ok_or("dev needs tsrust.config.json; create a project with `tsrust new <name>`")?;
        let target=match target {Some(target)=>target,None=>{
            let targets=tsrust_compiler::ui_project::configured_targets(&project.entry,crate::dev_web::read_optional)?;
            if targets.is_some_and(|targets|targets.web&&!targets.native){"web".into()}else{"native".into()}
        }};
        if target=="native" && port.is_some(){return Err("--port applies only to --target web".into());}
        if target=="web" && !program_args.is_empty(){return Err("Web dev does not accept native program arguments".into());}
        let _signals=Signals::install()?;
        let root=project.root;
        let mut web=if target=="web"{Some(crate::dev_web::Web::new(&root,port.unwrap_or(4317))?)}else{None};
        report(Status::Info,format_args!("Watching {}. Press Ctrl+C to stop.",root.display()));
        let worker=crate::dev_frontend::Worker::new()?;
        let mut watcher=Watcher::default();
        let mut previous:Option<Fingerprint>=None;let mut running:Option<Running>=None;let mut built_once=false;
        while !STOP.load(Ordering::Relaxed) {
            let mut input=watcher.snapshot(&root);
            if previous.as_ref()!=Some(&input) {
                // Debounce edits, but initial startup has no preceding edit burst.
                if previous.is_some() {
                    std::thread::sleep(Duration::from_millis(100));
                    if watcher.snapshot(&root)!=input {continue;}
                }
                let build_started=Instant::now();
                if let Some(web)=&web {web.building();}
                if input.is_ok(){
                    if let Err(error)=crate::docs_cli::prepare_root(&root){previous=Some(input);report(Status::Error,&error);if let Some(web)=&web{web.failure(&error);}continue;}
                    // Generated modules enter the same immutable frontend snapshot.
                    watcher.invalidate(&root);input=watcher.snapshot(&root);
                    if let Ok(inputs)=&input{if !crate::docs_cli::matches_snapshot(&root,inputs)?{continue;}}
                }
                previous=Some(input.clone());
                let result=match &input {Err(error)=>Err(error.clone()),Ok(_)=>build(&root,release,verbose,&input,&mut watcher,&worker,web.as_mut())};
                match result {
                    Ok(Some(artifact)) if !STOP.load(Ordering::Relaxed) && watcher.snapshot(&root)==input=>{
                        report(Status::Success,format_args!("{} in {:.0} ms",if built_once {"Rebuilt"} else {"Built"},build_started.elapsed().as_secs_f64()*1000.0));built_once=true;
                        let Artifact::Native(files,binary)=artifact else{if let Some(web)=&mut web{web.publish()?;}continue;};
                        if let Some(mut old)=running.take(){old.process.stop();}
                        let mut command=Command::new(binary);command.current_dir(&root).args(&program_args);
                        match Process::spawn(&mut command) {
                            Ok(process)=>{
                                if verbose {report(Status::Info,format_args!("Started program (pid {}).",process.child.id()));}
                                running=Some(Running{process,_files:files});
                            },
                            Err(error)=>report(Status::Error,format_args!("Cannot start program: {error}")),
                        }
                    },
                    Ok(_)=>{previous=None;},
                    Err(error)=>{if let Some(web)=&web{web.failure(&error);}report(Status::Error,format_args!("{error}; {}.",if running.is_some()||web.as_ref().is_some_and(|web|web.ready()){"previous application remains running"}else{"waiting for changes"}));},
                }
            }
            if let Some(active)=&mut running {
                if let Some(status)=active.process.child.try_wait().map_err(|e|e.to_string())? {
                    if verbose || !status.success() {report(if status.success(){Status::Info}else{Status::Error},format_args!("Program exited with {status}; waiting for changes."));}
                    running.take();
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        drop(running);report(Status::Info,"Stopped.");Ok(0)
    }
}
#[cfg(unix)] pub use platform::run;
#[cfg(not(unix))]
pub fn run(_:impl Iterator<Item=String>)->Result<u8,String> {Err("dev process supervision currently supports macOS and Linux".into())}
