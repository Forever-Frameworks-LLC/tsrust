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
    #[cfg(test)] reads:usize,
}
impl Watcher {
    pub(crate) fn snapshot(&mut self,root:&Path)->Fingerprint {
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
