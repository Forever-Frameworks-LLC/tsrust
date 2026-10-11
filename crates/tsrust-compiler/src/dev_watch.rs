//! Reuse unchanged source contents while retaining exact byte comparisons.
use std::{collections::BTreeMap, fs, io::Read, os::unix::fs::MetadataExt, path::{Path,PathBuf}, sync::Arc};

#[derive(Clone,Debug)]
pub(crate) struct Contents(Arc<[u8]>);
impl PartialEq for Contents {
    fn eq(&self,other:&Self)->bool { Arc::ptr_eq(&self.0,&other.0) || self.0==other.0 }
}
impl Eq for Contents {}
pub(crate) type Fingerprint=Result<BTreeMap<PathBuf,Contents>,String>;
#[derive(PartialEq,Eq)]
struct Stamp { device:u64,inode:u64,length:u64,modified:(i64,i64),changed:(i64,i64) }
impl From<fs::Metadata> for Stamp {
    fn from(m:fs::Metadata)->Self {
        Self{device:m.dev(),inode:m.ino(),length:m.len(),modified:(m.mtime(),m.mtime_nsec()),changed:(m.ctime(),m.ctime_nsec())}
    }
}
#[derive(Default)]
pub(crate) struct Watcher {
    entries:BTreeMap<PathBuf,(Stamp,Contents)>,
    initialized:bool,
    events:Option<crate::dev_events::Events>,
    previous:Option<Fingerprint>,
    #[cfg(test)] scans:usize,
    #[cfg(test)] reads:usize,
}
impl Watcher {
    pub(crate) fn snapshot(&mut self,root:&Path)->Fingerprint {
        if !self.initialized {
            self.initialized=true;
            if std::env::var("TSRUST_DEV_WATCH").as_deref()==Ok("poll") {
                eprintln!("[dev] Polling requested by TSRUST_DEV_WATCH=poll.");
            } else {
                match crate::dev_events::Events::new(root) {
                    Ok(events)=>self.events=Some(events),
                    Err(error)=>eprintln!("[dev] Native watching unavailable ({error}); using polling."),
                }
            }
        }
        if let Some(events)=&mut self.events {
            let reconcile=events.changed().and_then(|changed| {
                // new() already subscribed before the first content scan. Preserve
                // events delivered during setup and retry failed reconciliations.
                if changed || self.previous.as_ref().is_some_and(|value|value.is_err()) {events.refresh(root)?;}
                Ok(changed || self.previous.as_ref().is_none_or(|value|value.is_err()))
            });
            match reconcile {
                Ok(false)=>return self.previous.as_ref().unwrap().clone(),
                Ok(true)=>{},
                Err(error)=>{eprintln!("[dev] Native watching failed ({error}); using polling.");self.events=None;},
            }
        }
        let result=self.read_snapshot(root);
        self.previous=Some(result.clone());result
    }
    fn read_snapshot(&mut self,root:&Path)->Fingerprint {
        #[cfg(test)] {self.scans+=1;}
        let mut files=tsrust_compiler::source_discovery::discover(&[root.to_owned()])?;
        files.push(root.join(crate::project_config::FILE));
        let mut values=BTreeMap::new();
        for path in files {
            let error=|e:std::io::Error|format!("{}: {e}",path.display());
            let stamp=Stamp::from(fs::metadata(&path).map_err(error)?);
            let unchanged=self.entries.get(&path).is_some_and(|(old,_)|old==&stamp);
            if !unchanged {
                let mut file=fs::File::open(&path).map_err(error)?;
                let before=Stamp::from(file.metadata().map_err(error)?);
                let mut bytes=Vec::new();file.read_to_end(&mut bytes).map_err(error)?;
                let after=Stamp::from(file.metadata().map_err(error)?);
                let current=Stamp::from(fs::metadata(&path).map_err(error)?);
                if before!=stamp || after!=stamp || current!=stamp {
                    return Err(format!("{} changed while reading; retrying",path.display()));
                }
                #[cfg(test)] {self.reads+=1;}
                self.entries.insert(path.clone(),(stamp,Contents(bytes.into())));
            }
            values.insert(path.clone(),self.entries[&path].1.clone());
        }
        self.entries.retain(|path,_|values.contains_key(path));
        Ok(values)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unchanged_polls_reuse_bytes_and_same_size_edits_are_detected() {
        let directory=crate::TempDirectory::new().unwrap();let canonical=directory.0.canonicalize().unwrap();let root=&canonical;
        std::fs::write(root.join("tsrust.config.json"),"{}").unwrap();
        let source=root.join("main.tsr");std::fs::write(&source,"first").unwrap();
        let mut watcher=Watcher::default();let first=watcher.snapshot(root).unwrap();let reads=watcher.reads;
        for _ in 0..20 {assert_eq!(watcher.snapshot(root).unwrap(),first);}
        assert_eq!(watcher.reads,reads);
        assert_eq!(watcher.scans,1,"idle native watcher must not scan directories");
        assert_eq!(watcher.events.as_ref().unwrap().refreshes,1,"startup must walk subscriptions once");
        let modified=std::fs::metadata(&source).unwrap().modified().unwrap();
        std::fs::write(&source,"other").unwrap();
        std::fs::File::options().write(true).open(&source).unwrap().set_times(std::fs::FileTimes::new().set_modified(modified)).unwrap();
        assert_ne!(watcher.snapshot(root).unwrap(),first);
        assert_eq!(watcher.reads,reads+1);
        std::fs::write(root.join("replacement"),"third").unwrap();std::fs::rename(root.join("replacement"),&source).unwrap();
        let replaced=watcher.snapshot(root).unwrap();assert_eq!(replaced[&source].0.as_ref(),b"third");
        std::fs::remove_file(&source).unwrap();assert!(!watcher.snapshot(root).unwrap().contains_key(&source));
        assert!(!watcher.entries.contains_key(&source));
    }
    #[test]
    fn concurrent_atomic_saves_reconcile_to_clean_contents() {
        let directory=crate::TempDirectory::new().unwrap();let root=directory.0.canonicalize().unwrap();
        std::fs::write(root.join("tsrust.config.json"),"{}").unwrap();
        for i in 0..100 {std::fs::write(root.join(format!("file{i}.tsr")),"initial").unwrap();}
        let mut watcher=Watcher::default();watcher.snapshot(&root).unwrap();
        let writer_root=root.clone();
        let writer=std::thread::spawn(move||{
            for round in 0..20 {for i in 0..100 {
                let staged=writer_root.join("save.tmp");std::fs::write(&staged,format!("{round}:{i}")).unwrap();
                std::fs::rename(staged,writer_root.join(format!("file{i}.tsr"))).unwrap();
            }}
        });
        while !writer.is_finished(){let _=watcher.snapshot(&root);std::thread::yield_now();}
        writer.join().unwrap();
        let final_snapshot=watcher.snapshot(&root).unwrap();
        assert!(watcher.events.is_some(),"ordinary save races must not permanently disable native events");
        let mut clean=Watcher{initialized:true,..Watcher::default()};
        assert_eq!(final_snapshot,clean.snapshot(&root).unwrap());
        for i in 0..100 {assert_eq!(final_snapshot[&root.join(format!("file{i}.tsr"))].0.as_ref(),format!("19:{i}").as_bytes());}
    }
    #[test]
    fn native_failure_falls_back_and_recovers_after_root_recreation() {
        let directory=crate::TempDirectory::new().unwrap();let root=&directory.0;
        std::fs::write(root.join("tsrust.config.json"),"{}").unwrap();
        let mut watcher=Watcher::default();watcher.snapshot(root).unwrap();
        assert!(watcher.events.is_some());
        std::fs::remove_file(root.join("tsrust.config.json")).unwrap();std::fs::remove_dir(root).unwrap();
        assert!(watcher.snapshot(root).is_err());assert!(watcher.events.is_none());
        std::fs::create_dir(root).unwrap();std::fs::write(root.join("tsrust.config.json"),"{}").unwrap();
        assert_eq!(watcher.snapshot(root).unwrap().len(),1);
        let scans=watcher.scans;watcher.snapshot(root).unwrap();assert_eq!(watcher.scans,scans+1);
    }
    #[test]
    fn discovers_new_files_and_config_changes_without_reading_unchanged_sources() {
        let directory=crate::TempDirectory::new().unwrap();let canonical=directory.0.canonicalize().unwrap();let root=&canonical;
        std::fs::write(root.join("tsrust.config.json"),"{}").unwrap();
        let mut watcher=Watcher::default();watcher.snapshot(root).unwrap();
        std::fs::create_dir(root.join("src")).unwrap();std::fs::write(root.join("src/new.tsr"),"function main(){}").unwrap();
        assert_eq!(watcher.snapshot(root).unwrap().len(),2);assert_eq!(watcher.reads,2);
        std::fs::write(root.join("tsrust.config.json"),"{invalid}").unwrap();watcher.snapshot(root).unwrap();assert_eq!(watcher.reads,3);
        std::fs::remove_file(root.join("tsrust.config.json")).unwrap();assert!(watcher.snapshot(root).is_err());
        std::fs::write(root.join("tsrust.config.json"),"{}").unwrap();assert_eq!(watcher.snapshot(root).unwrap().len(),2);
    }
}
