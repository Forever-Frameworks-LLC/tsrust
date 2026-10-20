//! One application owns its location. Links and platform listeners retain only weak state.
use super::{Child,Node,Signal};
use std::{cell::RefCell,rc::{Rc,Weak}};
struct Navigation { read:Box<dyn Fn()->Result<String,String>>,write:Box<dyn Fn(String)->Result<(),String>>,#[cfg(not(all(target_arch="wasm32",feature="ui-web")))]history:RefCell<(Vec<String>,usize)> }
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
#[cfg(any(test,all(target_arch="wasm32",feature="ui-web")))]
fn decoded_browser_path(path:&str)->Result<String,String>{
    let mut bytes=Vec::with_capacity(path.len());let mut source=path.as_bytes().iter().copied();
    while let Some(byte)=source.next(){if byte==b'%' {let high=source.next().and_then(|byte|(byte as char).to_digit(16)).ok_or("Invalid browser path escape")?;let low=source.next().and_then(|byte|(byte as char).to_digit(16)).ok_or("Invalid browser path escape")?;let value=(high*16+low)as u8;if value.is_ascii(){return Err("Escaped ASCII application paths are unsupported".into());}bytes.push(value);}else{bytes.push(byte);}}
    let path=String::from_utf8(bytes).map_err(|_|"Browser path is not valid UTF-8")?;checked_path(&path)
}
#[cfg(all(target_arch="wasm32",feature="ui-web"))]
fn browser_path()->Result<String,String>{use wasm_bindgen::JsValue;let window=web_sys::window().ok_or("Browser window unavailable")?;let location=js_sys::Reflect::get(window.as_ref(),&JsValue::from_str("location")).map_err(|_|"Browser location unavailable")?;let get=|name|js_sys::Reflect::get(&location,&JsValue::from_str(name)).ok().and_then(|value|value.as_string()).ok_or_else(||String::from("Browser location is invalid"));decoded_browser_path(&(get("pathname")?+&get("hash")?))}
#[cfg(not(all(target_arch="wasm32",feature="ui-web")))]
fn browser_path()->Result<String,String>{Ok("/".into())}
#[cfg(all(target_arch="wasm32",feature="ui-web"))]
fn browser_history(method:&str,path:Option<&str>)->Result<(),String>{use wasm_bindgen::{JsValue,JsCast};let window=web_sys::window().ok_or("Browser window unavailable")?;let history=js_sys::Reflect::get(window.as_ref(),&JsValue::from_str("history")).map_err(|_|"Browser history unavailable")?;let call=js_sys::Reflect::get(&history,&JsValue::from_str(method)).ok().and_then(|value|value.dyn_into::<js_sys::Function>().ok()).ok_or("Browser history method unavailable")?;match path{Some(path)=>call.call3(&history,&JsValue::NULL,&JsValue::from_str(""),&JsValue::from_str(path)),None=>call.call0(&history)}.map_err(|_|"Browser history update failed")?;Ok(())}
#[cfg(not(all(target_arch="wasm32",feature="ui-web")))]
fn browser_history(_: &str,_:Option<&str>)->Result<(),String>{Ok(())}
pub fn location<T:Clone+'static>(decode:fn(String)->T,encode:fn(&T)->String)->Result<Signal<T>,String>{
    if ACTIVE.with(|value|value.borrow().upgrade().is_some()){return Err("Create UI location once in the application root".into());}
    let path=browser_path()?;let state=Signal::new(decode(path.clone()));let read=state.handle();let write=state.handle();
    let owner=Rc::new(Navigation{read:Box::new(move||read.get().map(|value|encode(&value))),write:Box::new(move|path|write.set(decode(path))),#[cfg(not(all(target_arch="wasm32",feature="ui-web")))]history:RefCell::new((vec![path],0))});
    ACTIVE.with(|value|*value.borrow_mut()=Rc::downgrade(&owner));let weak=Rc::downgrade(&owner);
    Ok(state.with_owner(owner,move|next|{let owner=weak.upgrade().ok_or("UI location is disposed")?;let path=checked_path(&encode(next))?;push(&owner,&path)?;Ok(decode(path))}))
}
fn push(owner:&Navigation,path:&str)->Result<(),String>{let path=checked_path(path)?;if (owner.read)()?==path{return Ok(());}browser_history("pushState",Some(&path))?;#[cfg(not(all(target_arch="wasm32",feature="ui-web")))]{let mut history=owner.history.borrow_mut();let end=history.1+1;history.0.truncate(end);history.0.push(path);history.1+=1;}Ok(())}
pub(crate) fn route_predicate(path:&str,exact:bool)->Result<impl FnMut()->Result<bool,String>+'static,String>{
    let path=checked_path(path)?;
    if path.contains('#'){return Err("UI Route paths cannot contain a heading fragment".into());}
    let owner=Rc::downgrade(&active()?);
    Ok(move||{let owner=owner.upgrade().ok_or("UI Route location is disposed")?;let current=(owner.read)()?;let current=current.split('#').next().unwrap_or("/");Ok(current==path || !exact && (path=="/" || current.strip_prefix(&path).is_some_and(|rest|rest.starts_with('/'))))})
}
pub(crate) fn current()->Option<String>{active().ok().and_then(|owner|(owner.read)().ok())}
pub(crate) fn is_current(href:&str)->bool{!href.contains('#')&&current().is_some_and(|path|path.split('#').next()==Some(href))}
pub fn navigate(path:&str)->Result<(),String>{let path=checked_path(path)?;let owner=active()?;push(&owner,&path)?;(owner.write)(path)}
pub fn traverse(forward:bool)->Result<(),String>{
    let owner=active()?;
    #[cfg(all(target_arch="wasm32",feature="ui-web"))]
    {let _=owner;browser_history(if forward{"forward"}else{"back"},None)}
    #[cfg(not(all(target_arch="wasm32",feature="ui-web")))]
    {let mut history=owner.history.borrow_mut();let next=if forward{(history.1+1).min(history.0.len()-1)}else{history.1.saturating_sub(1)};let path=history.0[next].clone();history.1=next;drop(history);(owner.write)(path)}
}
#[cfg(all(target_arch="wasm32",feature="ui-web"))]
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
#[cfg(not(all(target_arch="wasm32",feature="ui-web")))]
fn open_external(href:&str)->Result<(),String>{
    #[cfg(target_os="macos")]let mut command=std::process::Command::new("/usr/bin/open");
    #[cfg(target_os="windows")]let mut command={let mut command=std::process::Command::new("rundll32.exe");command.arg("url.dll,FileProtocolHandler");command};
    #[cfg(not(any(target_os="macos",target_os="windows")))]let mut command=std::process::Command::new("xdg-open");
    let result=command.arg(href).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status().map_err(|error|format!("Cannot open HTTPS link: {error}"))?;
    if result.success(){Ok(())}else{Err(format!("HTTPS link launcher exited with {result}"))}
}
#[cfg(all(target_arch="wasm32",feature="ui-web"))]
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
