//! A bounded, persistent frontend worker. Cancelled responses are discarded by callers.
use std::{path::PathBuf,sync::mpsc::{self,Receiver,SyncSender},time::{Duration,Instant}};
use tsrust_compiler::{project::{FileSourceProvider,SourceProvider},project_cli::CheckedInput,project_session::{ProjectSession,SessionStats}};
pub(crate) struct Output {pub rust:String,pub features:Vec<&'static str>,pub identity:Vec<u8>,pub stats:SessionStats,pub check:Duration,pub emit:Duration}
pub(crate) struct Request {pub entry:PathBuf,pub reply:mpsc::Sender<Result<Output,String>>}
pub(crate) struct Worker {pub requests:SyncSender<Request>}
impl Worker {
 pub fn new()->Result<Self,String> {
  let (requests,rx)=mpsc::sync_channel::<Request>(1);
  std::thread::Builder::new().name("dev-frontend".into()).spawn(move||run(rx)).map_err(|e|e.to_string())?;
  Ok(Self{requests})
 }
}
fn run(requests:Receiver<Request>) {
 let mut session=ProjectSession::default();
 for request in requests {let result=compile(&mut session,&request.entry);let _=request.reply.send(result);}
}
fn compile(session:&mut ProjectSession,entry:&std::path::Path)->Result<Output,String> {
 let started=Instant::now();
 let source=std::fs::read_to_string(entry).map_err(|e|e.to_string())?;
 let analysis=tsrust_compiler::project_cli::check_input_cached(entry,&source,session).map_err(|error|error.render(&entry.display().to_string(),&source,false))?;
 let check=started.elapsed();let started=Instant::now();
 let(rust,features)=analysis.emit_rust_with_native_features();let emit=started.elapsed();
 let mut identity=Vec::new();
 match &analysis {
  CheckedInput::Project(project)=>for(id,module)in &project.graph.modules {for value in [id.0.as_bytes(),module.source.text.as_bytes()] {identity.extend_from_slice(&(value.len()as u64).to_le_bytes());identity.extend_from_slice(value);}},
  CheckedInput::Single(_)=>{let id=FileSourceProvider.resolve(None,entry.to_str().ok_or("Source path must be valid UTF-8.")?)?;identity.extend_from_slice(&(id.0.len()as u64).to_le_bytes());identity.extend_from_slice(id.0.as_bytes());identity.extend_from_slice(source.as_bytes());},
 }
 let stats=if matches!(analysis,CheckedInput::Project(_)){session.stats()}else{SessionStats::default()};
 Ok(Output{rust,features,identity,stats,check,emit})
}
