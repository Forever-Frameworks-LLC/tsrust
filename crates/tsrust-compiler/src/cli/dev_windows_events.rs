//! Recursive Windows notifications request exact source reconciliation.
use std::{fs,io,os::windows::{ffi::OsStrExt,fs::MetadataExt},path::{Path,PathBuf}};
use windows_sys::Win32::{Foundation::*,Storage::FileSystem::*,System::Threading::WaitForSingleObject};
pub(crate) struct Events {handle:HANDLE,root:PathBuf,created:u64,#[cfg(test)] pub(crate) refreshes:usize}
impl Events {
    pub(crate) fn new(root:&Path)->io::Result<Self>{
        let root=fs::canonicalize(root)?;let created=fs::metadata(&root)?.creation_time();
        let path=root.as_os_str().encode_wide().chain(Some(0)).collect::<Vec<_>>();
        // SAFETY: NUL-terminated canonical path. This handle is closed only by Drop.
        let handle=unsafe{FindFirstChangeNotificationW(path.as_ptr(),1,FILE_NOTIFY_CHANGE_FILE_NAME|FILE_NOTIFY_CHANGE_DIR_NAME|FILE_NOTIFY_CHANGE_LAST_WRITE|FILE_NOTIFY_CHANGE_SIZE|FILE_NOTIFY_CHANGE_ATTRIBUTES)};
        if handle==INVALID_HANDLE_VALUE{return Err(io::Error::last_os_error());}
        Ok(Self{handle,root,created,#[cfg(test)] refreshes:1})
    }
    pub(crate) fn refresh(&mut self,_:&Path)->io::Result<()>{
        #[cfg(test)] {self.refreshes+=1;}
        self.validate()
    }
    fn validate(&self)->io::Result<()>{
        if fs::metadata(&self.root)?.creation_time()!=self.created{return Err(io::Error::new(io::ErrorKind::NotFound,"Watched root was replaced"));}Ok(())
    }
    pub(crate) fn changed(&mut self)->io::Result<bool>{
        // The notification does not cover removal/replacement of the root itself.
        self.validate()?;
        match unsafe{WaitForSingleObject(self.handle,0)} {
            WAIT_TIMEOUT=>Ok(false),
            WAIT_OBJECT_0=>{
                // Rearm before reading so edits during reconciliation stay pending.
                if unsafe{FindNextChangeNotification(self.handle)}==0{return Err(io::Error::last_os_error());}Ok(true)
            },
            _=>Err(io::Error::last_os_error()),
        }
    }
}
impl Drop for Events {fn drop(&mut self){unsafe{FindCloseChangeNotification(self.handle);}}}
