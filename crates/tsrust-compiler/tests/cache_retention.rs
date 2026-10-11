mod support;
use tsrust_compiler::native_cache::NativeCache;
#[test]
fn repeated_builds_bound_artifacts_and_locks_without_removing_private_copies() {
 let dir=support::TempDirectory::new();let root=dir.0.join("cache");let artifact=dir.0.join("artifact");std::fs::write(&artifact,b"native artifact").unwrap();
 let first=NativeCache::new(&root,&[b"first"]);first.publish(&artifact).unwrap();let private=dir.0.join("running");assert!(first.copy_to(&private));
 for i in 0u64..48 {NativeCache::new(&root,&[&i.to_le_bytes()]).publish(&artifact).unwrap();}
 assert!(first.lookup().is_none());assert_eq!(std::fs::read(private).unwrap(),b"native artifact");
 let entries=std::fs::read_dir(root.join("v2")).unwrap().map(Result::unwrap).collect::<Vec<_>>();
 assert_eq!(entries.iter().filter(|e|e.path().is_dir()).count(),32);assert_eq!(entries.iter().filter(|e|e.path().is_file()).count(),1);
}
#[test]
fn oversized_artifacts_run_without_being_retained() {
 let dir=support::TempDirectory::new();let artifact=dir.0.join("large");std::fs::File::create(&artifact).unwrap().set_len(256*1024*1024+1).unwrap();
 let cache=NativeCache::new(&dir.0.join("cache"),&[b"large"]);cache.publish(&artifact).unwrap();assert!(cache.lookup().is_none());assert!(artifact.exists());
}
#[test]
fn concurrent_readers_and_eviction_preserve_verified_private_artifacts() {
 let dir=support::TempDirectory::new();let root=dir.0.join("cache");
 std::thread::scope(|scope| {
  for worker in 0u64..4 {let root=&root;let temporary=&dir.0;scope.spawn(move||{
   let artifact=temporary.join(format!("input-{worker}"));let private=temporary.join(format!("copy-{worker}"));let content=vec![worker as u8;1024];std::fs::write(&artifact,&content).unwrap();
   for i in 0u64..24 {let cache=NativeCache::new(root,&[&worker.to_le_bytes(),&i.to_le_bytes()]);cache.publish(&artifact).unwrap();if cache.copy_to(&private){assert_eq!(std::fs::read(&private).unwrap(),content);}}
  });}
 });
 assert!(std::fs::read_dir(root.join("v2")).unwrap().filter(|e|e.as_ref().unwrap().path().is_dir()).count()<=32);
}
#[test]
fn byte_limits_and_abandoned_staging_cleanup_preserve_legacy_namespace() {
 let dir=support::TempDirectory::new();let root=dir.0.join("cache");let namespace=root.join("v2");std::fs::create_dir_all(&namespace).unwrap();
 std::fs::write(root.join("legacy.lock"),b"older process lock").unwrap();
 std::fs::create_dir(namespace.join(".staging-abandoned")).unwrap();
 for i in 0..2 {let path=namespace.join(format!("{i:016x}"));std::fs::create_dir(&path).unwrap();std::fs::File::create(path.join("program")).unwrap().set_len(140*1024*1024).unwrap();}
 let input=dir.0.join("input");std::fs::write(&input,b"current").unwrap();let cache=NativeCache::new(&root,&[b"current"]);cache.publish(&input).unwrap();assert!(cache.lookup().is_some());
 let size:u64=std::fs::read_dir(&namespace).unwrap().map(Result::unwrap).filter(|e|e.path().is_dir()).map(|e|std::fs::read_dir(e.path()).unwrap().map(Result::unwrap).map(|f|f.metadata().unwrap().len()).sum::<u64>()).sum();
 assert!(size<=256*1024*1024);assert!(!namespace.join(".staging-abandoned").exists());assert_eq!(std::fs::read(root.join("legacy.lock")).unwrap(),b"older process lock");
}
