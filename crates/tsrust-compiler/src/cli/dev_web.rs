//! Web target for the existing dev supervisor: one frontend, warm workspace and loopback server.
use std::{collections::BTreeMap,fs,io::{Read,Write},net::{TcpListener,TcpStream},path::{Path,PathBuf},process::{Command,Stdio},sync::{Arc,Mutex,atomic::{AtomicBool,Ordering}},thread,time::Duration};
use std::hash::{Hash,Hasher};
use crate::{TempDirectory,dev_cli::{report,Status}};
#[derive(Default)]
struct Published {revision:u64,assets:BTreeMap<String,Arc<Vec<u8>>>,gzip:BTreeMap<String,Arc<Vec<u8>>>,building:bool,error:String}
pub(crate) struct Web {root:PathBuf,workspace:PathBuf,_lock:fs::File,state:Arc<Mutex<Published>>,stop:Arc<AtomicBool>,server:Option<thread::JoinHandle<()>>,pending:Option<TempDirectory>}
fn write_changed(path:&Path,value:impl AsRef<[u8]>)->Result<(),String>{let value=value.as_ref();if fs::read(path).ok().as_deref()!=Some(value){fs::write(path,value).map_err(|e|e.to_string())?;}Ok(())}
pub(crate) fn read_optional(path:&Path)->Result<Option<String>,String>{match fs::read_to_string(path){Ok(text)=>Ok(Some(text)),Err(error) if error.kind()==std::io::ErrorKind::NotFound=>Ok(None),Err(error)=>Err(error.to_string())}}
impl Web {
 pub fn new(root:&Path,port:u16)->Result<Self,String>{
  let config=fs::read_to_string(root.join("tsrust.config.json")).map_err(|e|e.to_string())?;let project=crate::project_config::parse(root,&config)?;
  tsrust_compiler::ui_project::require_target(&project.entry,"web",read_optional)?;
  let cache=root.join("target/tsrust-web");fs::create_dir_all(&cache).map_err(|e|e.to_string())?;
  let lock=fs::OpenOptions::new().create(true).truncate(false).read(true).write(true).open(cache.join("dev.lock")).map_err(|e|e.to_string())?;lock.try_lock().map_err(|_|"Another web dev server owns this project")?;
  let state=Arc::new(Mutex::new(Published::default()));
  if let Some(seed)=std::env::var_os("TSRUST_DEV_SEED"){state.lock().unwrap().assets=load_assets(Path::new(&seed))?;{let mut published=state.lock().unwrap();published.gzip=gzip_assets(&published.assets)?;}state.lock().unwrap().revision=1;}
  let listener=TcpListener::bind(("127.0.0.1",port)).map_err(|e|format!("Cannot listen for web development: {e}"))?;let address=listener.local_addr().map_err(|e|e.to_string())?;listener.set_nonblocking(true).map_err(|e|e.to_string())?;
  let stop=Arc::new(AtomicBool::new(false));let shared=state.clone();let stopping=stop.clone();
  let server=thread::Builder::new().name("dev-web-http".into()).spawn(move||{while !stopping.load(Ordering::Relaxed){match listener.accept(){Ok((stream,_))=>{let _=serve(stream,&shared);},Err(error) if error.kind()==std::io::ErrorKind::WouldBlock=>thread::sleep(Duration::from_millis(20)),Err(_)=>break}}}).map_err(|e|e.to_string())?;
  report(Status::Info,format_args!("Web: http://{address}/"));
  let mut key=std::collections::hash_map::DefaultHasher::new();crate::native_dependency_bundle::IDENTITY.hash(&mut key);
  Ok(Self{root:root.to_owned(),workspace:cache.join(format!("workspace-{:016x}",key.finish())),_lock:lock,state,stop,server:Some(server),pending:None})
 }
 pub fn ready(&self)->bool{!self.state.lock().unwrap().assets.is_empty()}
 pub fn building(&self){let mut state=self.state.lock().unwrap();state.building=true;state.error.clear();}
 pub fn failure(&self,error:&str){let mut state=self.state.lock().unwrap();state.building=false;state.error=error.into();}
 pub fn build(&mut self,rust:&str,root:&Path,release:bool,mut run:impl FnMut(&mut Command)->Result<bool,String>)->Result<bool,String>{
  self.pending.take();
  if !rust.contains("mod tsrust_native_ui {")||rust.split("mod tsrust_native_").skip(1).filter_map(|value|value.split_whitespace().next()).any(|module|!matches!(module,"ui"|"json")){return Err("Web UI currently supports std:ui and JSON; native services, files and task runtimes are not supported".into());}
  let identity=self.workspace.join("tsrust-dependencies.identity");
  if self.workspace.exists(){
   if fs::read(&identity).ok().as_deref()!=Some(crate::native_dependency_bundle::IDENTITY.as_bytes()){return Err("Web dependency cache identity mismatch; refusing to overwrite an unrelated workspace".into());}
  }else{
   crate::package_export::export(&self.workspace,rust,true,&crate::ui_cli::attribution_files(&crate::project_config::parse(root,&fs::read_to_string(root.join("tsrust.config.json")).map_err(|e|e.to_string())?)?.entry)?).map_err(|e|e.to_string())?;
   fs::write(&identity,crate::native_dependency_bundle::IDENTITY).map_err(|e|e.to_string())?;
  }
  let source=format!("{rust}\n#[wasm_bindgen::prelude::wasm_bindgen]\npub fn launch()->Result<js_sys::Array,wasm_bindgen::JsValue>{{tsrust_native_ui::browser::launch(||main()==std::process::ExitCode::SUCCESS)}}\n");write_changed(&self.workspace.join("src/main.rs"),source)?;
  let manifest=self.workspace.join("Cargo.toml");let base=fs::read_to_string(&manifest).map_err(|e|e.to_string())?;let base=base.split("\n# tsrust web dev\n").next().unwrap();let profile=if release{""}else{"\n[profile.release.package.tsrust-export]\nopt-level = 0\ncodegen-units = 16\ndebug = false\n"};write_changed(&manifest,format!("{base}\n# tsrust web dev\n[lib]\npath = \"src/main.rs\"\ncrate-type = [\"cdylib\"]\n{profile}"))?;
  let files=TempDirectory::new().map_err(|e|e.to_string())?;let errors=files.0.join("web.stderr");
  let target=std::env::var_os("CARGO_TARGET_DIR").map(PathBuf::from).unwrap_or_else(||self.root.join("target/tsrust-web/cargo-target"));
  let mut cargo=crate::backend_cargo_command();cargo.current_dir(&self.workspace).args(["build","--offline","--locked","--release","--target","wasm32-unknown-unknown","--lib"]).env("CARGO_TARGET_DIR",&target).env("CARGO_INCREMENTAL","0").env("CARGO_BUILD_JOBS","1").stdout(Stdio::null()).stderr(fs::File::create(&errors).map_err(|e|e.to_string())?);
  match run(&mut cargo){Ok(false)=>return Ok(false),Ok(true)=>{},Err(error)=>return Err(format!("{error}\n{}",fs::read_to_string(&errors).unwrap_or_default()))}
  let executable=std::env::var_os("WASM_BINDGEN").unwrap_or_else(||"wasm-bindgen".into());let mut bindgen=Command::new(executable);bindgen.arg(target.join("wasm32-unknown-unknown/release/tsrust_export.wasm")).args(["--target","web","--out-name","app","--no-typescript","--out-dir"]).arg(&files.0).stdout(Stdio::null()).stderr(fs::File::create(&errors).map_err(|e|e.to_string())?);
  match run(&mut bindgen){Ok(false)=>return Ok(false),Ok(true)=>{},Err(error)=>return Err(format!("{error}\n{}\nWeb dev requires wasm32-unknown-unknown and wasm-bindgen-cli 0.2.129 installed locally.",fs::read_to_string(&errors).unwrap_or_default()))}
  self.pending=Some(files);Ok(true)
 }
 pub fn publish(&mut self)->Result<(),String>{
  let files=self.pending.take().ok_or("No completed web build")?;let mut assets=load_assets(&files.0)?;
  let notices=self.workspace.join("attributions");if notices.is_dir(){for file in fs::read_dir(notices).map_err(|e|e.to_string())?{let file=file.map_err(|e|e.to_string())?;if file.file_type().map_err(|e|e.to_string())?.is_file(){assets.insert(format!("attributions/{}",file.file_name().to_string_lossy()),Arc::new(fs::read(file.path()).map_err(|e|e.to_string())?));}}}
  let gzip=gzip_assets(&assets)?;
  let mut state=self.state.lock().unwrap();state.assets=assets;state.gzip=gzip;state.revision=state.revision.checked_add(1).ok_or("Web revision exhausted")?;state.building=false;state.error.clear();Ok(())
 }
}
impl Drop for Web {fn drop(&mut self){self.stop.store(true,Ordering::Relaxed);if let Some(server)=self.server.take(){let _=server.join();}}}
fn gzip_assets(assets:&BTreeMap<String,Arc<Vec<u8>>>)->Result<BTreeMap<String,Arc<Vec<u8>>>,String>{
 assets.iter().map(|(name,bytes)|{let mut encoder=flate2::write::GzEncoder::new(Vec::new(),flate2::Compression::fast());encoder.write_all(bytes).map_err(|e|e.to_string())?;let bytes=encoder.finish().map_err(|e|e.to_string())?;Ok((name.clone(),Arc::new(bytes)))}).collect()
}
fn encodings(request:&str)->(f32,f32){
 let (mut gzip,mut identity,mut wildcard):(Option<f32>,Option<f32>,Option<f32>)=(None,None,None);
 for line in request.lines().skip(1){let Some((name,value))=line.split_once(':')else{continue;};if !name.trim().eq_ignore_ascii_case("accept-encoding"){continue;}
  for item in value.split(','){let mut parts=item.split(';');let name=parts.next().unwrap_or("").trim();let mut quality=1.;for parameter in parts{if let Some((key,value))=parameter.trim().split_once('='){if key.eq_ignore_ascii_case("q"){quality=value.trim().parse::<f32>().ok().filter(|q|q.is_finite()&&(0. ..=1.).contains(q)).unwrap_or(0.);}}}
   let slot=if name.eq_ignore_ascii_case("gzip"){&mut gzip}else if name.eq_ignore_ascii_case("identity"){&mut identity}else if name=="*"{&mut wildcard}else{continue;};*slot=Some(slot.map_or(quality,|previous|previous.min(quality)));
  }
 }
 (gzip.or(wildcard).unwrap_or(0.),identity.unwrap_or(if wildcard==Some(0.){0.}else{1.}))
}
fn load_assets(root:&Path)->Result<BTreeMap<String,Arc<Vec<u8>>>,String>{let mut assets=BTreeMap::new();for name in ["app.js","app_bg.wasm"]{assets.insert(name.into(),Arc::new(fs::read(root.join(name)).map_err(|e|format!("Missing web asset {name}: {e}"))?));}Ok(assets)}
fn html(revision:u64,ready:bool)->String{
 let application=if ready{format!(r#"import init,{{launch}} from '/app.js?v={revision}';
await init({{module_or_path:'/app_bg.wasm?v={revision}'}});
const mounts=launch();window.ui={{mounts,dispose(){{for(const mount of mounts)mount.free();mounts.length=0;}}}};
addEventListener('pagehide',()=>window.ui.dispose());
addEventListener('pageshow',event=>{{if(event.persisted&&mounts.length===0)mounts.push(...launch());}});"#)}else{String::new()};
 format!(r#"<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>Tsrust</title></head><body>{}<script type="module">
let timer,error='';function watch(){{timer=setInterval(async()=>{{try{{const status=await(await fetch('/__tsrust_dev',{{cache:'no-store'}})).json();if(status.revision!=={revision})location.reload();if(status.error&&error!==status.error){{console.error(status.error);error=status.error;}}}}catch{{}}}},750);}}watch();addEventListener('pagehide',()=>clearInterval(timer));addEventListener('pageshow',event=>{{if(event.persisted)watch();}});
{application}
</script></body></html>"#,if ready{""}else{"<p>Starting application…</p>"})
}
fn serve(mut stream:TcpStream,shared:&Mutex<Published>)->std::io::Result<()>{
 stream.set_nonblocking(false)?;stream.set_read_timeout(Some(Duration::from_secs(1)))?;stream.set_write_timeout(Some(Duration::from_secs(5)))?;
 let mut input=Vec::new();let mut block=[0u8;2048];while !input.windows(4).any(|part|part==b"\r\n\r\n") {let count=stream.read(&mut block)?;if count==0{return Ok(());}input.extend_from_slice(&block[..count]);if input.len()>16384{return Ok(());}}
 let request=String::from_utf8_lossy(&input);let mut first=request.lines().next().unwrap_or("").split_whitespace();let method=first.next().unwrap_or("");let url=first.next().unwrap_or("/");let (path,query)=url.split_once('?').unwrap_or((url,""));
 let (gzip_quality,identity_quality)=encodings(&request);let mut compressed=false;
 let state=shared.lock().unwrap();let mut status="200 OK";let mut mime="text/html; charset=utf-8";
 let body=if !matches!(method,"GET"|"HEAD"){status="405 Method Not Allowed";Arc::new(Vec::new())}
 else if path=="/__tsrust_dev"{mime="application/json";Arc::new(serde_json::to_vec(&serde_json::json!({"revision":state.revision,"building":state.building,"error":state.error})).unwrap())}
 else if let Some(asset)=state.assets.get(path.trim_start_matches('/')){
  if !query.is_empty()&&query!=format!("v={}",state.revision){status="409 Conflict";Arc::new(Vec::new())}else{mime=if path.ends_with(".wasm"){"application/wasm"}else if path.ends_with(".js"){"text/javascript; charset=utf-8"}else{"text/plain; charset=utf-8"};if gzip_quality>0. && gzip_quality>=identity_quality {if let Some(gzip)=state.gzip.get(path.trim_start_matches('/')){compressed=true;gzip.clone()}else if identity_quality>0.{asset.clone()}else{status="406 Not Acceptable";Arc::new(Vec::new())}}else if identity_quality>0.{asset.clone()}else{status="406 Not Acceptable";Arc::new(Vec::new())}}
 }else if path=="/"||request.lines().skip(1).any(|line|line.to_ascii_lowercase().starts_with("accept:")&&line.contains("text/html")){Arc::new(html(state.revision,!state.assets.is_empty()).into_bytes())}
 else{status="404 Not Found";Arc::new(Vec::new())};drop(state);
 write!(stream,"HTTP/1.1 {status}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\n{}Vary: Accept-Encoding\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",body.len(),if compressed{"Content-Encoding: gzip\r\n"}else{""})?;if method!="HEAD"{stream.write_all(&body)?;}Ok(())
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn nested_entry_discovery_and_server_drop_release_the_project_lock(){
  let files=TempDirectory::new().unwrap();let root=fs::canonicalize(&files.0).unwrap();fs::create_dir(root.join("src")).unwrap();fs::write(root.join("src/main.tsrx"),"function main(){}").unwrap();let mut ui=tsrust_compiler::ui_project::Config::default();ui.target=Some(tsrust_compiler::ui_project::Targets{web:true,native:false,embedded:false});let config=serde_json::json!({"schemaVersion":1,"name":"web-dev","build":{"entry":"src/main.tsrx","output":"target/app"},"ui":ui});fs::write(root.join("tsrust.config.json"),config.to_string()).unwrap();let server=Web::new(&root,0).unwrap();assert!(!server.ready());assert!(Web::new(&root,0).is_err());drop(server);assert!(Web::new(&root,0).is_ok());
 }
 #[test]fn development_html_owns_reload_and_preserves_mount_cleanup(){let source=html(7,true);assert!(source.contains("/app.js?v=7"));assert!(source.contains("/app_bg.wasm?v=7"));assert!(source.contains("pagehide"));assert!(source.contains("mount.free()"));assert!(!source.contains("Starting application"));}
 #[test]fn local_server_keeps_failed_build_assets_and_rejects_mixed_revisions(){
  fn request(state:Arc<Mutex<Published>>,url:&str,accept:&str)->String{let listener=TcpListener::bind(("127.0.0.1",0)).unwrap();let port=listener.local_addr().unwrap().port();listener.set_nonblocking(true).unwrap();let worker=thread::spawn(move||{let stream=loop{match listener.accept(){Ok((stream,_))=>break stream,Err(error) if error.kind()==std::io::ErrorKind::WouldBlock=>thread::sleep(Duration::from_millis(1)),Err(error)=>panic!("{error}")}};serve(stream,&state).unwrap()});let mut stream=TcpStream::connect(("127.0.0.1",port)).unwrap();write!(stream,"GET {url} HTTP/1.1\r\nHost: localhost\r\nAccept: {accept}\r\n\r\n").unwrap();let mut result=String::new();stream.read_to_string(&mut result).unwrap();worker.join().unwrap();result}
  let mut state=Published{revision:2,error:"Build failed".into(),..Default::default()};state.assets.insert("app.js".into(),Arc::new(b"working source".to_vec()));state.assets.insert("app_bg.wasm".into(),Arc::new(vec![b'x';2_000_000]));let state=Arc::new(Mutex::new(state));let large=request(state.clone(),"/app_bg.wasm?v=2","*/*");assert!(large.contains("application/wasm"));assert_eq!(large.split_once("\r\n\r\n").unwrap().1.len(),2_000_000);assert!(request(state.clone(),"/app.js?v=2","*/*").contains("working source"));assert!(request(state.clone(),"/app.js?v=1","*/*").contains("409 Conflict"));assert!(request(state.clone(),"/docs/deep","text/html").contains("/app.js?v=2"));assert!(request(state.clone(),"/missing.js","*/*").contains("404 Not Found"));assert!(request(state,"/__tsrust_dev","*/*").contains("Build failed"));
 }
}
