//! Backend owner/cancellation witnesses. Generated APIs have a separate target.
#[path="../src/stdlib/fs/runtime/resource.rs"]pub(crate) mod resource;
#[path="../src/stdlib/fs/runtime/fs.rs"]pub(crate) mod fs;
#[path="../src/stdlib/fs/runtime/buffered_fs.rs"]pub(crate) mod buffered_fs;
mod tsrust_native_files{pub(crate) use crate::{fs,buffered_fs};}
include!("../src/stdlib/futures/runtime.rs");
include!("../src/stdlib/io/runtime.rs");
include!("../src/stdlib/process/runtime.rs");
mod support;
fn helper(directory:&support::TempDirectory)->std::path::PathBuf{let path=directory.0.join("helper.rs");std::fs::write(&path,include_str!("../../../benchmarks/runtime/process-helper.rs")).unwrap();let binary=directory.0.join(format!("helper{}",std::env::consts::EXE_SUFFIX));support::success(&support::run(support::rustc().arg("--edition=2024").arg(path).arg("-o").arg(&binary),60));binary}
#[test]
fn process_owners_and_running_future_are_send(){fn send<T:Send>(){}send::<tsrust_native_process::Child>();send::<tsrust_native_process::Options>();let future=tsrust_native_process::run("unused",&[],Default::default());fn send_value<T:Send>(_:T){}send_value(future);}
#[test]
fn cancelling_borrowed_wait_retains_owner_and_kill_reaps(){let directory=support::TempDirectory::new();let helper=helper(&directory);tsrust_native_futures::block_on(async{let mut child=tsrust_native_process::spawn(helper.to_str().unwrap(),&["sleep"],Default::default()).unwrap();let id=child.id().unwrap();assert!(tokio::time::timeout(std::time::Duration::from_millis(10),child.wait()).await.is_err());assert_eq!(child.id(),Some(id));assert!(child.try_wait().unwrap().is_none());child.kill().await.unwrap();assert!(child.id().is_none());assert!(!child.wait().await.unwrap().success);});}
#[cfg(unix)]
#[test]
fn cancelling_started_child_scope_awaits_actual_os_reap_before_join(){let directory=support::TempDirectory::new();let helper=helper(&directory);tsrust_native_futures::block_on(async{let (send,receive)=tokio::sync::oneshot::channel();let job=tsrust_native_futures::start(async move{let child=tsrust_native_process::spawn(helper.to_str().unwrap(),&["sleep"],Default::default()).unwrap();send.send(child.id().unwrap()).unwrap();let _keep_owner=child;std::future::pending::<()>().await;Ok::<_,tsrust_native_futures::Failure<()>>(())});let id=receive.await.unwrap();assert!(job.cancel());assert!(matches!(job.join().await,Err(tsrust_native_futures::Failure::Cancelled)));assert_eq!(nix::sys::wait::waitpid(nix::unistd::Pid::from_raw(id.try_into().unwrap()),Some(nix::sys::wait::WaitPidFlag::WNOHANG)),Err(nix::errno::Errno::ECHILD));});}
#[test]
fn dropping_unpolled_run_closes_owned_input_without_spawning(){use std::sync::{Arc,atomic::{AtomicUsize,Ordering}};struct Input(Arc<AtomicUsize>);impl Drop for Input{fn drop(&mut self){self.0.fetch_add(1,Ordering::SeqCst);}}impl tokio::io::AsyncRead for Input{fn poll_read(self:std::pin::Pin<&mut Self>,_:&mut std::task::Context<'_>,_:&mut tokio::io::ReadBuf<'_>)->std::task::Poll<std::io::Result<()>>{panic!("unpolled run read input")}}let drops=Arc::new(AtomicUsize::new(0));let reader=tsrust_native_io::Reader::asynchronous(Box::new(Input(drops.clone())));drop(tsrust_native_process::run("unused",&[],tsrust_native_process::Options{input:Some(tsrust_native_process::Input::Reader(reader)),..Default::default()}));assert_eq!(drops.load(Ordering::SeqCst),1);}
