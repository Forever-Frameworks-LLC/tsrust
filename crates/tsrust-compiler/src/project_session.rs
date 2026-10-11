//! Retained pre-link syntax. Linked IDs, source offsets and ownership checks stay fresh.
use std::collections::BTreeMap;
use crate::{Diagnostic,syntax::Program,project::{SourceId,SourceProvider,ProjectGraph,ProjectAnalysis,ProjectDiagnostic,load_project}};

#[derive(Clone,Copy,Debug,Default)]
pub struct SessionStats {pub parsed:usize,pub reused:usize,pub retained:usize}
#[derive(PartialEq,Eq)]
struct Key {source:String,index:usize,test_entry:bool,dependencies:Vec<(SourceId,u64)>}
struct Entry {key:Key,generation:u64,program:Program}
#[derive(Default)]
pub struct ProjectSession {entries:BTreeMap<SourceId,Entry>,generation:u64,stats:SessionStats}
impl ProjectSession {
    pub fn stats(&self)->SessionStats {self.stats}
    pub fn check(&mut self,provider:&impl SourceProvider,entry:&str)->Result<ProjectAnalysis,ProjectDiagnostic> {
        self.stats=SessionStats::default();
        let graph=match load_project(provider,entry) {
            Ok(graph)=>graph,
            Err(error)=>{self.entries.clear();return Err(error);},
        };
        self.entries.retain(|id,_|graph.modules.contains_key(id));
        let result=crate::project_link::check_graph_cached(graph,self);
        self.stats.retained=self.entries.len();result
    }
    pub(crate) fn parse(&mut self,graph:&ProjectGraph,id:&SourceId,index:usize,test_entry:bool,imported:crate::type_aliases::Aliases,records:crate::type_aliases::ImportedRecords)->Result<Program,Diagnostic> {
        let module=&graph.modules[id];
        // Dependencies precede importers. Their generation changes for any source
        // edit or linked module-index change, conservatively invalidating importers.
        let dependencies=module.imports.iter().filter_map(|import|import.target.as_ref())
            .map(|target|(target.clone(),self.entries[target].generation)).collect();
        let key=Key{source:module.source.text.clone(),index,test_entry,dependencies};
        if let Some(entry)=self.entries.get(id).filter(|entry|entry.key==key) {
            self.stats.reused+=1;return Ok(entry.program.clone());
        }
        self.stats.parsed+=1;
        self.entries.remove(id);
        let program=crate::syntax::parse_module(&module.declaration_source,test_entry,imported,records)?;
        self.generation=self.generation.checked_add(1).expect("project session generation exhausted");
        self.entries.insert(id.clone(),Entry{key,generation:self.generation,program:program.clone()});
        Ok(program)
    }
}
