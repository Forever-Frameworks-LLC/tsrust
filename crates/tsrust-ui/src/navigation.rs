//! Each window owns its location. Links and platform listeners retain only weak state.
use super::{Child,Node,Signal};
use std::{cell::RefCell,rc::{Rc,Weak}};
struct Navigation { read:Box<dyn Fn()->Result<String,String>>,write:Box<dyn Fn(String)->Result<(),String>>,#[cfg(not(all(target_arch="wasm32",feature="gpui-web")))]history:RefCell<(Vec<String>,usize)> }
#[derive(Clone,Default)]
pub(crate) struct Context(Weak<Navigation>);
pub(crate) struct Scope(Weak<Navigation>);
impl Drop for Scope {fn drop(&mut self){ACTIVE.with(|active|*active.borrow_mut()=self.0.clone());}}
impl Context {
 pub(crate) fn current()->Self {Self(ACTIVE.with(|active|active.borrow().clone()))}
 pub(crate) fn enter(&self)->Scope {Scope(ACTIVE.with(|active|active.replace(self.0.clone())))}
}
thread_local! {static ACTIVE:RefCell<Weak<Navigation>>=const{RefCell::new(Weak::new())};}
fn active()->Result<Rc<Navigation>,String>{ACTIVE.with(|value|value.borrow().upgrade()).ok_or_else(||"No live UI location owner".into())}
/// Absolute application paths only. External URLs are deliberately a separate contract.
pub fn checked_path(path:&str)->Result<String,String>{
    if !path.starts_with('/')||path.starts_with("//")||path.contains(['\\','?','%'])||path.chars().any(|ch|ch.is_control()||ch.is_whitespace())||path.split('/').any(|part|matches!(part,"."|"..")){return Err("UI links require an absolute application path without queries, escapes or traversal".into());}
    let (path,fragment)=path.split_once('#').map_or((path,None),|(path,fragment)|(path,Some(fragment)));
    if fragment.is_some_and(|value|value.is_empty()||value.contains('#')){return Err("UI link fragment is invalid".into());}
    let base=if path.len()>1{path.trim_end_matches('/')}else{path};Ok(match fragment{Some(fragment)=>format!("{base}#{fragment}"),None=>base.into()})
}
// Browsers encode Unicode path/fragment bytes even when the authored link has no escapes.
#[cfg(any(test,all(target_arch="wasm32",feature="gpui-web")))]
fn decoded_browser_path(path:&str)->Result<String,String>{
    let mut bytes=Vec::with_capacity(path.len());let mut source=path.as_bytes().iter().copied();
    while let Some(byte)=source.next(){if byte==b'%' {let high=source.next().and_then(|byte|(byte as char).to_digit(16)).ok_or("Invalid browser path escape")?;let low=source.next().and_then(|byte|(byte as char).to_digit(16)).ok_or("Invalid browser path escape")?;let value=(high*16+low)as u8;if value.is_ascii(){return Err("Escaped ASCII application paths are unsupported".into());}bytes.push(value);}else{bytes.push(byte);}}
    let path=String::from_utf8(bytes).map_err(|_|"Browser path is not valid UTF-8")?;checked_path(&path)
}
#[cfg(all(target_arch="wasm32",feature="gpui-web"))]
fn browser_path()->Result<String,String>{use wasm_bindgen::JsValue;let window=web_sys::window().ok_or("Browser window unavailable")?;let location=js_sys::Reflect::get(window.as_ref(),&JsValue::from_str("location")).map_err(|_|"Browser location unavailable")?;let get=|name|js_sys::Reflect::get(&location,&JsValue::from_str(name)).ok().and_then(|value|value.as_string()).ok_or_else(||String::from("Browser location is invalid"));decoded_browser_path(&(get("pathname")?+&get("hash")?))}
#[cfg(not(all(target_arch="wasm32",feature="gpui-web")))]
fn browser_path()->Result<String,String>{Ok("/".into())}
#[cfg(all(target_arch="wasm32",feature="gpui-web"))]
fn browser_history(method:&str,path:Option<&str>)->Result<(),String>{use wasm_bindgen::{JsValue,JsCast};let window=web_sys::window().ok_or("Browser window unavailable")?;let history=js_sys::Reflect::get(window.as_ref(),&JsValue::from_str("history")).map_err(|_|"Browser history unavailable")?;let call=js_sys::Reflect::get(&history,&JsValue::from_str(method)).ok().and_then(|value|value.dyn_into::<js_sys::Function>().ok()).ok_or("Browser history method unavailable")?;match path{Some(path)=>call.call3(&history,&JsValue::NULL,&JsValue::from_str(""),&JsValue::from_str(path)),None=>call.call0(&history)}.map_err(|_|"Browser history update failed")?;Ok(())}
#[cfg(not(all(target_arch="wasm32",feature="gpui-web")))]
fn browser_history(_: &str,_:Option<&str>)->Result<(),String>{Ok(())}
pub fn location<T:Clone+'static>(decode:fn(String)->T,encode:fn(&T)->String)->Result<Signal<T>,String>{
    if ACTIVE.with(|value|value.borrow().upgrade().is_some()){return Err("Create UI location once in the application root".into());}
    create_location(&browser_path()?,decode,encode)
}
pub fn create_location<T:Clone+'static>(initial:&str,decode:fn(String)->T,encode:fn(&T)->String)->Result<Signal<T>,String>{
    let path=checked_path(initial)?;let state=Signal::new(decode(path.clone()));let read=state.handle();let write=state.handle();
    let owner=Rc::new(Navigation{read:Box::new(move||read.get().map(|value|encode(&value))),write:Box::new(move|path|write.set(decode(path))),#[cfg(not(all(target_arch="wasm32",feature="gpui-web")))]history:RefCell::new((vec![path],0))});
    ACTIVE.with(|value|*value.borrow_mut()=Rc::downgrade(&owner));let weak=Rc::downgrade(&owner);
    Ok(state.with_owner(owner,move|next|{let owner=weak.upgrade().ok_or("UI location is disposed")?;let path=checked_path(&encode(next))?;push(&owner,&path)?;Ok(decode(path))}))
}
fn push(owner:&Navigation,path:&str)->Result<(),String>{let path=checked_path(path)?;if (owner.read)()?==path{return Ok(());}browser_history("pushState",Some(&path))?;#[cfg(not(all(target_arch="wasm32",feature="gpui-web")))]{let mut history=owner.history.borrow_mut();let end=history.1+1;history.0.truncate(end);history.0.push(path);history.1+=1;}Ok(())}
pub(crate) fn route_predicate(path:&str,exact:bool)->Result<impl FnMut()->Result<bool,String>+'static,String>{
    let path=checked_pattern(path)?;
    let owner=Rc::downgrade(&active()?);
    Ok(move||{let owner=owner.upgrade().ok_or("UI Route location is disposed")?;let current=(owner.read)()?;Ok(matches_path(&path,&current,exact))})
}
fn checked_pattern(pattern:&str)->Result<String,String> {
 let pattern=checked_path(pattern)?;
 if pattern.contains('#'){return Err("UI Route paths cannot contain a heading fragment".into());}
 let mut names=std::collections::BTreeSet::new();
 for segment in pattern.split('/') {if let Some(name)=segment.strip_prefix(':') {
  if name.is_empty() || !name.bytes().enumerate().all(|(index,byte)|byte.is_ascii_alphabetic() || byte==b'_' || index>0 && byte.is_ascii_digit()) || !names.insert(name.to_owned()) {return Err("UI route parameters require unique identifier names after ':'".into());}
 }}
 Ok(pattern)
}
fn matches_path(pattern:&str,path:&str,exact:bool)->bool {
 let path=path.split('#').next().unwrap_or("/");
 if pattern=="/"{return !exact || path=="/";}
 let mut current=path.split('/').skip(1);
 for segment in pattern.split('/').skip(1) {
  let Some(value)=current.next() else{return false};
  if segment.starts_with(':') {if value.is_empty(){return false;}} else if segment!=value{return false;}
 }
 !exact || current.next().is_none()
}
pub fn route_parameter(path:&str,pattern:&str,name:&str)->Result<String,String> {
 let pattern=checked_pattern(pattern)?;let current=checked_path(path)?;
 if !matches_path(&pattern,&current,true){return Ok(String::new());}
 Ok(pattern.split('/').zip(current.split('#').next().unwrap_or("/").split('/')).find_map(|(segment,value)|(segment.strip_prefix(':')==Some(name)).then(||value.to_owned())).unwrap_or_default())
}
pub(crate) fn current()->Option<String>{active().ok().and_then(|owner|(owner.read)().ok())}
pub(crate) fn current_kind(href:&str)->Option<&'static str>{
    if href.is_empty()||external(href){return None;}
    let current=current()?;
    if href.contains('#'){return (current==href).then_some("location");}
    let path=current.split('#').next()?;
    if path==href{Some("page")}else if href!="/"&&path.strip_prefix(href).is_some_and(|rest|rest.starts_with('/')){Some("location")}else{None}
}
#[cfg(any(test,feature="gpui"))]
pub(crate) fn is_current(href:&str)->bool{current_kind(href).is_some()}
pub fn navigate(path:&str)->Result<(),String>{let path=checked_path(path)?;let owner=active()?;push(&owner,&path)?;(owner.write)(path)}
pub fn traverse(forward:bool)->Result<(),String>{
    let owner=active()?;
    #[cfg(all(target_arch="wasm32",feature="gpui-web"))]
    {let _=owner;browser_history(if forward{"forward"}else{"back"},None)}
    #[cfg(not(all(target_arch="wasm32",feature="gpui-web")))]
    {let mut history=owner.history.borrow_mut();let next=if forward{(history.1+1).min(history.0.len()-1)}else{history.1.saturating_sub(1)};let path=history.0[next].clone();history.1=next;drop(history);(owner.write)(path)}
}
#[cfg(all(target_arch="wasm32",feature="gpui-web"))]
pub(crate) fn restore_browser()->Result<(),String>{let Ok(owner)=active()else{return Ok(())};(owner.write)(browser_path()?)}
/// HTTPS links use the platform launcher; application paths retain local routing.
pub(crate) fn external(href:&str)->bool{href.starts_with("https://")}
pub fn checked_href(href:&str)->Result<String,String>{
    if !external(href){return checked_path(href);}
    if href.contains('\\') || href.chars().any(|ch|ch.is_control()||ch.is_whitespace()){return Err("HTTPS links cannot contain whitespace, controls or backslashes".into());}
    let url=url::Url::parse(href).map_err(|error|format!("Invalid HTTPS link: {error}"))?;
    if url.scheme()!="https" || url.host_str().is_none() || !url.username().is_empty() || url.password().is_some(){return Err("HTTPS links require a host and cannot contain credentials".into());}
    Ok(url.into())
}
#[cfg(not(all(target_arch="wasm32",feature="gpui-web")))]
fn open_external(href:&str)->Result<(),String>{
    #[cfg(target_os="macos")]let mut command=std::process::Command::new("/usr/bin/open");
    #[cfg(target_os="windows")]let mut command={let mut command=std::process::Command::new("rundll32.exe");command.arg("url.dll,FileProtocolHandler");command};
    #[cfg(not(any(target_os="macos",target_os="windows")))]let mut command=std::process::Command::new("xdg-open");
    let result=command.arg(href).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status().map_err(|error|format!("Cannot open HTTPS link: {error}"))?;
    if result.success(){Ok(())}else{Err(format!("HTTPS link launcher exited with {result}"))}
}
#[cfg(all(target_arch="wasm32",feature="gpui-web"))]
fn open_external(_:&str)->Result<(),String>{Err("Browser HTTPS links use anchor activation".into())}
pub fn link(children:Vec<Child>,class_name:String,path:&str)->Result<Node,String>{
    let path=checked_href(path)?;let href=path.clone();let owner=ACTIVE.with(|value|value.borrow().clone());
    let mut node=Node::button_children(children,class_name,false,Some(Box::new(move||{
        if external(&path){return open_external(&path);}
        let owner=owner.upgrade().ok_or("UI link location is disposed")?;push(&owner,&path)?;(owner.write)(path.clone())
    })))?;node.semantics.href=href;Ok(node)
}

#[cfg(test)]
mod tests{
 #[test]fn window_roots_keep_independent_navigation_history_and_disposal(){
  let first=create_location("/first",String::from,Clone::clone).unwrap();let read_first=first.handle();
  let mut first=Node::button("first".into(),String::new(),false,||navigate("/first/next")).unwrap().own_state(first).unwrap();
  first.ensure_identities();let first_id=first.identity.get();let first_context=first.navigation.clone();
  let second=create_location("/second",String::from,Clone::clone).unwrap();let read_second=second.handle();
  let mut second=Node::button("second".into(),String::new(),false,||navigate("/second/next")).unwrap().own_state(second).unwrap();
  second.ensure_identities();let second_id=second.identity.get();
  first.activate(first_id).unwrap();assert_eq!(read_first.get().unwrap(),"/first/next");assert_eq!(read_second.get().unwrap(),"/second");
  assert_eq!(current().unwrap(),"/second");
  second.activate(second_id).unwrap();assert_eq!(read_second.get().unwrap(),"/second/next");
  {let _scope=first_context.enter();traverse(false).unwrap();assert_eq!(current().unwrap(),"/first");traverse(true).unwrap();assert_eq!(current().unwrap(),"/first/next");}
  assert_eq!(current().unwrap(),"/second/next");
  drop(first);assert!(read_first.get().is_err());assert_eq!(read_second.get().unwrap(),"/second/next");
  {let _scope=first_context.enter();assert!(navigate("/stale").is_err());}
  assert_eq!(current().unwrap(),"/second/next");
 }
 #[test]fn named_route_parameters_match_segments_without_decoding_or_rebinding(){
  for (pattern,path,exact,expected) in [("/users/:id","/users/42",true,true),("/users/:id","/users/42#bio",true,true),("/users/:id","/users/42/edit",true,false),("/users/:id","/users/42/edit",false,true),("/users/:id","/users",false,false),("/users/:id","/users//edit",false,false),("/users/:id","/users-other/42",false,false),("/","/users",false,true),("/","/users",true,false),("/a//b","/a/b",true,false)] {assert_eq!(matches_path(pattern,path,exact),expected,"{pattern} {path}");}
  assert_eq!(route_parameter("/users/42#bio","/users/:id","id").unwrap(),"42");
  assert_eq!(route_parameter("/teams/rust/users/ada/","/teams/:team/users/:user","user").unwrap(),"ada");
  assert_eq!(route_parameter("/users/42/edit","/users/:id","id").unwrap(),"");
  assert_eq!(route_parameter("/users/42","/users/:id","missing").unwrap(),"");
  for pattern in ["/users/:","/users/:1id","/:id/:id","/users#bio"] {assert!(route_parameter("/users/42",pattern,"id").is_err(),"{pattern}");}
  let state=create_location("/users/1",String::from,Clone::clone).unwrap();
  let mut matches=route_predicate("/users/:id",true).unwrap();assert!(matches().unwrap());state.set("/users/2".into()).unwrap();assert!(matches().unwrap());
  state.set("/elsewhere".into()).unwrap();assert!(!matches().unwrap());drop(state);assert!(matches().is_err());
 }
 #[test]fn selected_links_distinguish_page_ancestors_fragments_and_unrelated_segments(){
  let state=location(String::from,Clone::clone).unwrap();state.set("/docs/ui/components/button#variants".into()).unwrap();
  for (href,expected) in [("/docs/ui/components/button",Some("page")),("/docs",Some("location")),("/docs/ui/components",Some("location")),("/docs/ui/components/button#variants",Some("location")),("/docs/u",None),("/docs/ui/components/buttons",None),("/",None),("/docs/ui/components/button#props",None),("https://example.com/docs",None)]{assert_eq!(current_kind(href),expected,"{href}");}
  state.set("/".into()).unwrap();assert_eq!(current_kind("/"),Some("page"));drop(state);assert_eq!(current_kind("/"),None);
 }
 #[test]fn browser_unicode_paths_decode_without_accepting_escaped_separators(){assert_eq!(super::decoded_browser_path("/docs/caf%C3%A9#%E6%96%87").unwrap(),"/docs/café#文");for path in ["/docs%2Fhidden","/docs%23fragment","/docs%20space","/%FF","/%C3","/%ZZ"]{assert!(super::decoded_browser_path(path).is_err(),"{path}");}}
 #[test]fn https_links_validate_without_a_location_or_launcher(){
  for href in ["javascript:alert(1)","http://host/","file:///tmp/app","//host/","https://user:secret@host/","https://host/has space","https://host/\\path","https://host/\n"]{assert!(super::checked_href(href).is_err(),"{href:?}");}
  let node=super::link(vec![],String::new(),"https://github.com/Forever-Frameworks-LLC/tsrust/releases?tag=a&mode=b").unwrap();
  let html=super::super::document(&node,"External");assert!(html.contains("href=\"https://github.com/Forever-Frameworks-LLC/tsrust/releases?tag=a&amp;mode=b\""));assert!(!html.contains(" aria-current="));
 }
 use super::*;
 #[test]fn location_owns_history_and_releases_weak_callbacks(){let state=location(String::from,Clone::clone).unwrap();let weak=state.handle();let mut node=link(vec![],String::new(),"/docs/ui/button").unwrap();(node.press.as_mut().unwrap())().unwrap();assert_eq!(state.get().unwrap(),"/docs/ui/button");traverse(false).unwrap();assert_eq!(state.get().unwrap(),"/");traverse(true).unwrap();assert_eq!(state.get().unwrap(),"/docs/ui/button");state.set("/docs/".into()).unwrap();assert_eq!(state.get().unwrap(),"/docs");assert!(location(String::from,Clone::clone).is_err());assert!(navigate("//unsafe").is_err());drop(state);assert!(weak.get().is_err());assert!((node.press.as_mut().unwrap())().is_err());let next=location(String::from,Clone::clone).unwrap();assert!((node.press.as_mut().unwrap())().is_err());assert_eq!(next.get().unwrap(),"/");}
 #[test]fn paths_are_checked_and_document_links_keep_semantics(){for path in ["https://host/","//host","/a/../b","/a?x","/a%2fb","/a\\b","/a b","/a#"]{assert!(checked_path(path).is_err(),"{path}");}assert_eq!(checked_path("/docs/#start").unwrap(),"/docs#start");let link=link(vec![Child::Text(super::super::TextPart::Literal("Read docs".into()))],String::new(),"/docs").unwrap();let html=super::super::document(&link,"Links");assert!(html.contains("<a ")&&html.contains("href=\"/docs\""));assert!(!html.contains("type=\"button\""));}
}
