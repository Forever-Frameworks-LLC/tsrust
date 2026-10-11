//! Native notifications trigger reconciliation; they are never source truth.
use std::{collections::{BTreeMap,BTreeSet},fs,io,path::{Path,PathBuf},os::{fd::{AsRawFd,FromRawFd,OwnedFd},unix::{fs::MetadataExt,ffi::OsStrExt}}};
fn walk(root:&Path,visit:&mut impl FnMut(&Path,&fs::Metadata)->io::Result<()>)->io::Result<()> {
 let metadata=fs::symlink_metadata(root)?;
 if metadata.file_type().is_symlink(){return Ok(());}
 visit(root,&metadata)?;
 if metadata.is_dir() {
  for entry in fs::read_dir(root)? {
   let entry=entry?;let kind=entry.file_type()?;let path=entry.path();
   if kind.is_symlink(){continue;}
   if kind.is_dir() && !matches!(entry.file_name().to_str(),Some(".git"|"node_modules"|"target")) {walk(&path,visit)?;}
   else if kind.is_file() && (path.extension().is_some_and(|ext|ext=="tsr") || entry.file_name()=="tsrust.config.json") {visit(&path,&entry.metadata()?)?;}
  }
 }
 Ok(())
}
fn c_path(path:&Path)->io::Result<std::ffi::CString> {std::ffi::CString::new(path.as_os_str().as_bytes()).map_err(|_|io::Error::new(io::ErrorKind::InvalidInput,"NUL in watch path"))}
#[cfg(target_os="macos")]
pub(crate) struct Events {queue:OwnedFd,entries:BTreeMap<PathBuf,((u64,u64),OwnedFd)>}
#[cfg(target_os="macos")]
impl Events {
 pub(crate) fn new(root:&Path)->io::Result<Self> {
  // SAFETY: kqueue returns a fresh descriptor; OwnedFd closes it exactly once.
  let raw=unsafe{libc::kqueue()};if raw<0{return Err(io::Error::last_os_error());}
  let queue=unsafe{OwnedFd::from_raw_fd(raw)};
  if unsafe{libc::fcntl(raw,libc::F_SETFD,libc::FD_CLOEXEC)}<0{return Err(io::Error::last_os_error());}
  let mut events=Self{queue,entries:BTreeMap::new()};events.refresh(root)?;Ok(events)
 }
 pub(crate) fn refresh(&mut self,root:&Path)->io::Result<()> {
  let mut active=BTreeSet::new();
  walk(root,&mut |path,metadata| {
   active.insert(path.to_owned());let identity=(metadata.dev(),metadata.ino());
   if self.entries.get(path).is_some_and(|(old,_)|*old==identity){return Ok(());}
   let path_c=c_path(path)?;
   // SAFETY: valid C path, no variadic mode needed without O_CREAT. Refuse symlinks.
   let raw=unsafe{libc::open(path_c.as_ptr(),libc::O_EVTONLY|libc::O_CLOEXEC|libc::O_NOFOLLOW)};
   if raw<0{return Err(io::Error::last_os_error());}
   let file=unsafe{OwnedFd::from_raw_fd(raw)};
   let change=libc::kevent{ident:raw as _,filter:libc::EVFILT_VNODE,flags:libc::EV_ADD|libc::EV_CLEAR,fflags:libc::NOTE_WRITE|libc::NOTE_EXTEND|libc::NOTE_ATTRIB|libc::NOTE_RENAME|libc::NOTE_DELETE|libc::NOTE_REVOKE,data:0,udata:std::ptr::null_mut()};
   // SAFETY: one initialized input event, no output buffer, nonblocking registration.
   if unsafe{libc::kevent(self.queue.as_raw_fd(),&change,1,std::ptr::null_mut(),0,std::ptr::null())}<0{return Err(io::Error::last_os_error());}
   self.entries.insert(path.to_owned(),(identity,file));Ok(())
  })?;
  self.entries.retain(|path,_|active.contains(path));Ok(())
 }
 pub(crate) fn changed(&mut self)->io::Result<bool> {
  let mut events=std::mem::MaybeUninit::<[libc::kevent;64]>::uninit();let timeout=libc::timespec{tv_sec:0,tv_nsec:0};let mut changed=false;
  // Drain ordinary bursts together. Bound work if writers never stop; remaining
  // events still request reconciliation on the next supervisor poll.
  for _ in 0..16 {
   // SAFETY: writable storage for 64 events; no event payload is read.
   let count=unsafe{libc::kevent(self.queue.as_raw_fd(),std::ptr::null(),0,events.as_mut_ptr().cast(),64,&timeout)};
   if count<0 {let error=io::Error::last_os_error();if error.kind()==io::ErrorKind::Interrupted{return Ok(true);}return Err(error);}
   if count==0 {break;}changed=true;
  }
  // Every event, including revoke/error, requests a complete reconciliation.
  Ok(changed)
 }
}
#[cfg(target_os="linux")]
pub(crate) struct Events {queue:OwnedFd,entries:BTreeMap<PathBuf,((u64,u64),i32)>}
#[cfg(target_os="linux")]
impl Events {
 pub(crate) fn new(root:&Path)->io::Result<Self> {
  let raw=unsafe{libc::inotify_init1(libc::IN_NONBLOCK|libc::IN_CLOEXEC)};if raw<0{return Err(io::Error::last_os_error());}
  let mut events=Self{queue:unsafe{OwnedFd::from_raw_fd(raw)},entries:BTreeMap::new()};events.refresh(root)?;Ok(events)
 }
 pub(crate) fn refresh(&mut self,root:&Path)->io::Result<()> {
  let mut active=BTreeSet::new();
  walk(root,&mut |path,metadata| {
   if !metadata.is_dir(){return Ok(());}
   active.insert(path.to_owned());let identity=(metadata.dev(),metadata.ino());
   if self.entries.get(path).is_some_and(|(old,_)|*old==identity){return Ok(());}
   let path_c=c_path(path)?;
   let mask=libc::IN_MODIFY|libc::IN_ATTRIB|libc::IN_CREATE|libc::IN_DELETE|libc::IN_MOVED_FROM|libc::IN_MOVED_TO|libc::IN_DELETE_SELF|libc::IN_MOVE_SELF|libc::IN_ONLYDIR|libc::IN_DONT_FOLLOW;
   let watch=unsafe{libc::inotify_add_watch(self.queue.as_raw_fd(),path_c.as_ptr(),mask)};if watch<0{return Err(io::Error::last_os_error());}
   if let Some((_,old))=self.entries.insert(path.to_owned(),(identity,watch)){if old!=watch {unsafe{libc::inotify_rm_watch(self.queue.as_raw_fd(),old)};}}Ok(())
  })?;
  self.entries.retain(|path,(_,watch)| {if active.contains(path){true}else{unsafe{libc::inotify_rm_watch(self.queue.as_raw_fd(),*watch)};false}});Ok(())
 }
 pub(crate) fn changed(&mut self)->io::Result<bool> {
  let mut buffer=[0u8;65536];
  // SAFETY: writable buffer and live descriptor. All records, including overflow,
  // trigger reconciliation, so no unaligned event parsing is needed.
  let count=unsafe{libc::read(self.queue.as_raw_fd(),buffer.as_mut_ptr().cast(),buffer.len())};
  if count<0 {let error=io::Error::last_os_error();if error.kind()==io::ErrorKind::WouldBlock{return Ok(false);}if error.kind()==io::ErrorKind::Interrupted{return Ok(true);}return Err(error);}
  Ok(count>0)
 }
}
#[cfg(not(any(target_os="macos",target_os="linux")))]
pub(crate) struct Events;
#[cfg(not(any(target_os="macos",target_os="linux")))]
impl Events {
 pub(crate) fn new(_: &Path)->io::Result<Self>{Err(io::Error::new(io::ErrorKind::Unsupported,"native watching unavailable"))}
 pub(crate) fn refresh(&mut self,_:&Path)->io::Result<()>{Ok(())}
 pub(crate) fn changed(&mut self)->io::Result<bool>{Ok(true)}
}
#[cfg(test)]
mod tests {
 use super::*;
 use std::{fs,time::{Duration,Instant}};
 fn changed(events:&mut Events) {
  let until=Instant::now()+Duration::from_secs(2);
  while Instant::now()<until {if events.changed().unwrap(){return;}std::thread::sleep(Duration::from_millis(10));}
  panic!("missing filesystem notification");
 }
 #[test]
 fn native_events_cover_atomic_saves_new_directories_and_idle() {
  let temp=crate::TempDirectory::new().unwrap();let root=&temp.0;
  fs::write(root.join("main.tsr"),"first").unwrap();
  let mut events=Events::new(root).unwrap();assert!(!events.changed().unwrap());
  fs::write(root.join("main.tsr"),"other").unwrap();changed(&mut events);events.refresh(root).unwrap();
  fs::write(root.join("replacement"),"third").unwrap();fs::rename(root.join("replacement"),root.join("main.tsr")).unwrap();changed(&mut events);events.refresh(root).unwrap();
  fs::write(root.join("main.tsr"),"fourth").unwrap();changed(&mut events);
  fs::create_dir(root.join("nested")).unwrap();fs::write(root.join("nested/new.tsr"),"new").unwrap();changed(&mut events);events.refresh(root).unwrap();
  fs::write(root.join("nested/new.tsr"),"edit").unwrap();changed(&mut events);
  fs::remove_file(root.join("nested/new.tsr")).unwrap();changed(&mut events);events.refresh(root).unwrap();
  let _=events.changed().unwrap();for _ in 0..20 {assert!(!events.changed().unwrap());}
 }
 #[test]
 fn coalesced_batch_is_drained_before_the_next_idle_poll() {
  let temp=crate::TempDirectory::new().unwrap();let root=&temp.0;
  for i in 0..100 {fs::write(root.join(format!("file{i}.tsr")),"before").unwrap();}
  let mut events=Events::new(root).unwrap();
  for i in 0..100 {fs::write(root.join(format!("file{i}.tsr")),"after").unwrap();}
  changed(&mut events);assert!(!events.changed().unwrap(),"old events must not cause repeated rescans");
 }
 #[test]
 fn excluded_directories_and_symlinks_do_not_subscribe() {
  let temp=crate::TempDirectory::new().unwrap();let root=&temp.0;
  for name in ["target","node_modules",".git"] {fs::create_dir(root.join(name)).unwrap();fs::write(root.join(name).join("hidden.tsr"),"x").unwrap();}
  let outside=crate::TempDirectory::new().unwrap();fs::write(outside.0.join("external.tsr"),"x").unwrap();std::os::unix::fs::symlink(&outside.0,root.join("linked")).unwrap();
  let mut events=Events::new(root).unwrap();
  for name in ["target","node_modules",".git"] {fs::write(root.join(name).join("hidden.tsr"),"changed").unwrap();}
  fs::write(outside.0.join("external.tsr"),"changed").unwrap();assert!(!events.changed().unwrap());
 }
}
