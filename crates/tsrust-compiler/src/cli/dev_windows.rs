//! Windows development process ownership and exact file identities.
use std::{fs::File,io,os::windows::io::{AsRawHandle,FromRawHandle,OwnedHandle},path::Path,process::{Child,Command},sync::atomic::{AtomicBool,Ordering},time::{Duration,Instant}};
use windows_sys::Win32::{Foundation::*,Storage::FileSystem::*,System::{Console::*,Diagnostics::ToolHelp::*,JobObjects::*,Threading::*}};

pub(crate) static STOP:AtomicBool=AtomicBool::new(false);
unsafe extern "system" fn stop(event:u32)->i32 {
    if matches!(event,CTRL_C_EVENT|CTRL_BREAK_EVENT){STOP.store(true,Ordering::Relaxed);1}else{0}
}
pub(crate) struct Signals;
impl Signals {
    pub(crate) fn install()->Result<Self,String>{
        STOP.store(false,Ordering::Relaxed);
        // SAFETY: the callback only stores to an atomic and lives for the process.
        if unsafe{SetConsoleCtrlHandler(Some(stop),1)}==0{return Err(io::Error::last_os_error().to_string());}
        Ok(Self)
    }
}
impl Drop for Signals {fn drop(&mut self){unsafe{SetConsoleCtrlHandler(Some(stop),0);}}}

pub(crate) struct Process {pub(crate) child:Child,job:OwnedHandle,stopped:bool}
impl Process {
    pub(crate) fn spawn(command:&mut Command)->Result<Self,String>{
        use std::os::windows::process::CommandExt;
        // A suspended child cannot create descendants before joining its job.
        // No breakaway is permitted, and the job handle is never inherited.
        let raw=unsafe{CreateJobObjectW(std::ptr::null(),std::ptr::null())};
        if raw.is_null(){return Err(io::Error::last_os_error().to_string());}
        let job=unsafe{OwnedHandle::from_raw_handle(raw)};
        let mut limits:JOBOBJECT_EXTENDED_LIMIT_INFORMATION=unsafe{std::mem::zeroed()};
        limits.BasicLimitInformation.LimitFlags=JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        if unsafe{SetInformationJobObject(raw,JobObjectExtendedLimitInformation,(&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),std::mem::size_of_val(&limits) as u32)}==0{return Err(io::Error::last_os_error().to_string());}
        command.creation_flags(CREATE_SUSPENDED|CREATE_NEW_PROCESS_GROUP);
        let child=command.spawn().map_err(|e|e.to_string())?;
        let mut process=Self{child,job,stopped:false};
        if unsafe{AssignProcessToJobObject(raw,process.child.as_raw_handle())}==0 {
            let error=io::Error::last_os_error();let _=process.child.kill();let _=process.child.wait();return Err(error.to_string());
        }
        // Stable std::process does not expose the primary thread handle. The
        // suspended process has exactly one thread, found by its owning PID.
        let resume=||->io::Result<()> {
            let raw=unsafe{CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD,0)};
            if raw==INVALID_HANDLE_VALUE{return Err(io::Error::last_os_error());}
            let snapshot=unsafe{OwnedHandle::from_raw_handle(raw)};
            let mut entry:THREADENTRY32=unsafe{std::mem::zeroed()};entry.dwSize=std::mem::size_of_val(&entry) as u32;
            let mut found=unsafe{Thread32First(snapshot.as_raw_handle(),&mut entry)};
            while found!=0 {
                if entry.th32OwnerProcessID==process.child.id(){
                    let raw=unsafe{OpenThread(THREAD_SUSPEND_RESUME,0,entry.th32ThreadID)};
                    if raw.is_null(){return Err(io::Error::last_os_error());}
                    let thread=unsafe{OwnedHandle::from_raw_handle(raw)};
                    if unsafe{ResumeThread(thread.as_raw_handle())}==u32::MAX{return Err(io::Error::last_os_error());}
                    return Ok(());
                }
                found=unsafe{Thread32Next(snapshot.as_raw_handle(),&mut entry)};
            }
            Err(io::Error::new(io::ErrorKind::NotFound,"Suspended child thread was not found"))
        };
        if let Err(error)=resume(){process.stop();return Err(error.to_string());}
        Ok(process)
    }
    pub(crate) fn stop(&mut self){
        if self.stopped{return;}self.stopped=true;
        // Ctrl+Break targets only this child's console group. A bounded grace
        // period lets cooperative programs exit; the job also contains orphans.
        unsafe{GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT,self.child.id());}
        let until=Instant::now()+Duration::from_millis(500);
        while Instant::now()<until {
            if self.child.try_wait().ok().flatten().is_some(){break;}
            std::thread::sleep(Duration::from_millis(20));
        }
        unsafe{TerminateJobObject(self.job.as_raw_handle(),1);}
        let _=self.child.wait();
    }
}
impl Drop for Process {fn drop(&mut self){self.stop();}}

#[derive(Clone,PartialEq,Eq)]
pub(crate) struct Stamp {volume:u32,index:(u32,u32),length:(u32,u32),created:i64,modified:i64,changed:i64}
impl Stamp {
    pub(crate) fn read(path:&Path)->io::Result<Self>{Self::file(&File::open(path)?)}
    pub(crate) fn file(file:&File)->io::Result<Self>{
        let mut basic:FILE_BASIC_INFO=unsafe{std::mem::zeroed()};
        let mut identity:BY_HANDLE_FILE_INFORMATION=unsafe{std::mem::zeroed()};
        // SAFETY: borrowed live file handle and correctly sized writable records.
        if unsafe{GetFileInformationByHandleEx(file.as_raw_handle(),FileBasicInfo,(&mut basic as *mut FILE_BASIC_INFO).cast(),std::mem::size_of_val(&basic) as u32)}==0 || unsafe{GetFileInformationByHandle(file.as_raw_handle(),&mut identity)}==0{return Err(io::Error::last_os_error());}
        Ok(Self{volume:identity.dwVolumeSerialNumber,index:(identity.nFileIndexHigh,identity.nFileIndexLow),length:(identity.nFileSizeHigh,identity.nFileSizeLow),created:basic.CreationTime,modified:basic.LastWriteTime,changed:basic.ChangeTime})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn powershell()->Command {
        Command::new(std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap()).join("System32/WindowsPowerShell/v1.0/powershell.exe"))
    }
    #[test]
    fn stopping_job_reaps_descendants_even_after_parent_exit_and_preserves_unrelated_process(){
        use std::process::Stdio;
        let directory=crate::TempDirectory::new().unwrap();let marker=directory.0.join("child");
        let unrelated=powershell().args(["-NoProfile","-Command","Start-Sleep 30"]).stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap();
        struct Cleanup(Child);impl Drop for Cleanup{fn drop(&mut self){let _=self.0.kill();let _=self.0.wait();}}
        let mut unrelated=Cleanup(unrelated);
        let mut command=powershell();
        command.args(["-NoProfile","-Command","$p=Start-Process powershell.exe -ArgumentList '-NoProfile','-Command','Start-Sleep 30' -PassThru -WindowStyle Hidden; [System.IO.File]::WriteAllText($env:TSRUST_CHILD_MARKER,[string]$p.Id)"]).env("TSRUST_CHILD_MARKER",&marker).stdout(Stdio::null()).stderr(Stdio::null());
        let mut process=Process::spawn(&mut command).unwrap();
        let until=Instant::now()+Duration::from_secs(15);
        while !marker.exists(){assert!(Instant::now()<until,"child marker not published");std::thread::sleep(Duration::from_millis(20));}
        let pid=std::fs::read_to_string(marker).unwrap().parse::<u32>().unwrap();
        let raw=unsafe{OpenProcess(PROCESS_SYNCHRONIZE,0,pid)};assert!(!raw.is_null());
        let descendant=unsafe{OwnedHandle::from_raw_handle(raw)};
        while process.child.try_wait().unwrap().is_none(){assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(20));}
        assert_eq!(unsafe{WaitForSingleObject(descendant.as_raw_handle(),0)},WAIT_TIMEOUT);
        process.stop();
        assert_eq!(unsafe{WaitForSingleObject(descendant.as_raw_handle(),3000)},WAIT_OBJECT_0,"orphan survived job cleanup");
        assert!(unrelated.0.try_wait().unwrap().is_none(),"cleanup touched an unrelated process");
    }
}
