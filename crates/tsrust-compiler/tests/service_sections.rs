mod support;
fn root()->std::path::PathBuf{std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")}
#[test]
fn service_examples_check_and_signal_loans_reject(){
 for example in ["examples/service/main.tsr","examples/service/contracts.tsr","examples/service-observability/main.tsr"]{support::success(&support::run(support::cli().arg("check").arg(root().join(example)),30));}
 let directory=support::TempDirectory::new();
 for source in ["import {listen,receive} from 'std:signals';async function main():Task<void>{const signal=unwrap(listen());const a=receive(signal);const b=receive(signal);await a;await b;}","import {listen} from 'std:signals';async function main():Task<void>{const signal=unwrap(listen());const copy=clone(signal);}","import {App} from 'std:http';function main(){const app=new App();app.configure({bodyLimit:1});}"]{let output=support::run(support::cli().arg("check").arg(directory.source(&support::format_round_trip(source))),30);assert!(!output.status.success(),"accepted invalid service contract");}
}
// Native contracts, observability and deployment run together in tools/check-service.mjs.


#[cfg(any(unix,windows))]
mod delivered_signals {
    use super::support;
    use std::{io::{BufRead,BufReader,Read},process::{Child,Command,Stdio},sync::mpsc,time::{Duration,Instant}};
    #[cfg(windows)]const HELPER:&str="delivered_signals::internal_owned_console_signal_sender";
    struct Running {
        child:Child,
        lines:mpsc::Receiver<std::io::Result<String>>,
        output:Option<std::thread::JoinHandle<()>>,
        errors:Option<std::thread::JoinHandle<Vec<u8>>>,
    }
    impl Running {
        fn spawn(binary:&std::path::Path)->Self {
            let mut command=Command::new(binary);command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
            #[cfg(windows)]{
                use std::os::windows::process::CommandExt;
                // A separate console is essential: Ctrl+C cannot target one group.
                // Do not use CREATE_NEW_PROCESS_GROUP, which disables Ctrl+C.
                command.creation_flags(windows_sys::Win32::System::Threading::CREATE_NEW_CONSOLE);
            }
            let mut child=command.spawn().unwrap();let stdout=child.stdout.take().unwrap();let mut stderr=child.stderr.take().unwrap();let(tx,lines)=mpsc::channel();
            let output=std::thread::spawn(move||{for line in BufReader::new(stdout).lines(){if tx.send(line).is_err(){break;}}});
            let errors=std::thread::spawn(move||{let mut bytes=Vec::new();stderr.read_to_end(&mut bytes).unwrap();bytes});
            Self{child,lines,output:Some(output),errors:Some(errors)}
        }
        fn line(&self)->String {self.lines.recv_timeout(Duration::from_secs(5)).expect("generated listener did not acknowledge readiness/delivery").unwrap()}
        fn deliver(&mut self,name:&str){
            assert!(self.child.try_wait().unwrap().is_none(),"signal recipient exited");
            #[cfg(unix)]{
                // Exact positive owned PID only; never a process group or owner PID.
                assert!(self.child.id()>0);let signal=if name=="SIGINT"{"INT"}else{assert_eq!(name,"SIGTERM");"TERM"};
                support::success(&support::run(Command::new("/bin/kill").args(["-s",signal,&self.child.id().to_string()]),5));
            }
            #[cfg(windows)]{
                use std::os::windows::process::CommandExt;
                let event=if name=="SIGINT"{"0"}else{assert_eq!(name,"SIGBREAK");"1"};
                let mut helper=Command::new(std::env::current_exe().unwrap());helper.args(["--exact",HELPER,"--ignored","--nocapture","--test-threads=1"])
                    .env("TSRUST_SIGNAL_DELIVERY_HELPER_PID",self.child.id().to_string()).env("TSRUST_SIGNAL_DELIVERY_HELPER_EVENT",event)
                    .creation_flags(windows_sys::Win32::System::Threading::CREATE_NO_WINDOW);
                support::success(&support::run(&mut helper,10));
            }
        }
        fn finish(&mut self){
            let deadline=Instant::now()+Duration::from_secs(5);let status=loop{
                if let Some(status)=self.child.try_wait().unwrap(){break status;}
                assert!(Instant::now()<deadline,"generated listener did not exit after both receives");std::thread::sleep(Duration::from_millis(5));
            };
            assert!(status.success(),"listener exited abnormally: {status}");self.output.take().unwrap().join().unwrap();let errors=self.errors.take().unwrap().join().unwrap();assert!(errors.is_empty(),"{}",String::from_utf8_lossy(&errors));assert!(self.lines.try_recv().is_err(),"unexpected extra listener output");
        }
    }
    impl Drop for Running {fn drop(&mut self){if self.child.try_wait().ok().flatten().is_none(){let _=self.child.kill();}let _=self.child.wait();if let Some(thread)=self.output.take(){let _=thread.join();}if let Some(thread)=self.errors.take(){let _=thread.join();}}}

    #[cfg(windows)]
    fn console_sender(target:u32,event:u32){
        use windows_sys::Win32::System::Console::{AttachConsole,FreeConsole,GenerateConsoleCtrlEvent,GetConsoleProcessList,SetConsoleCtrlHandler};
        unsafe extern "system" fn ignore(_:u32)->i32{1}
        struct Detach;impl Drop for Detach{fn drop(&mut self){unsafe{FreeConsole();}}}
        assert!(target>0&&target!=std::process::id());assert!(event<=1);
        // These calls run only in the disposable no-console helper, never in the
        // test owner. Attach resets handlers, so install the helper handler after it.
        unsafe{FreeConsole();}
        assert_ne!(unsafe{AttachConsole(target)},0,"cannot attach owned recipient console");let _detach=Detach;
        let mut attached=[0u32;3];let count=unsafe{GetConsoleProcessList(attached.as_mut_ptr(),attached.len() as u32)};
        assert_eq!(count,2,"refuse console-wide delivery unless only recipient and helper are attached");assert!(attached[..2].contains(&target)&&attached[..2].contains(&std::process::id()));
        assert_ne!(unsafe{SetConsoleCtrlHandler(Some(ignore),1)},0);
        assert_ne!(unsafe{GenerateConsoleCtrlEvent(event,0)},0);
    }

    #[cfg(windows)]
    #[test]
    #[ignore="internal owned-console sender; invoked only by the delivery test with --exact --ignored"]
    fn internal_owned_console_signal_sender(){
        let target=std::env::var("TSRUST_SIGNAL_DELIVERY_HELPER_PID").expect("internal sender requires explicit owned target PID").parse().expect("invalid owned target PID");
        let event=std::env::var("TSRUST_SIGNAL_DELIVERY_HELPER_EVENT").expect("internal sender requires explicit event").parse().expect("invalid console event");
        console_sender(target,event);
    }

    #[test]
    fn generated_listener_receives_real_platform_signals(){
        let directory=support::TempDirectory::new();let source=support::format_round_trip(r#"
            import {listen,receive} from 'std:signals';
            async function main():Task<void>{
                const listener=unwrap(listen());
                console.log('ready');console.log(await receive(listener));
                console.log('ready');console.log(await receive(listener));
            }
        "#);let path=directory.source(&source);let binary=directory.0.join(format!("signal-listener{}",std::env::consts::EXE_SUFFIX));
        support::success(&support::run(support::cli().arg("build").arg(&path).arg("-o").arg(&binary),180));
        let mut running=Running::spawn(&binary);
        #[cfg(unix)]let expected=["SIGINT","SIGTERM"];
        #[cfg(windows)]let expected=["SIGINT","SIGBREAK"];
        for name in expected{assert_eq!(running.line(),"ready");running.deliver(name);assert_eq!(running.line(),name);}
        running.finish();
    }
}
