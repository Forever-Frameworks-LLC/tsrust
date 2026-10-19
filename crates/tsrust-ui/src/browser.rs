//! The JavaScript host owns each Mount and must free it on removal/page exit.
use crate::{Kind,Node};
use std::{cell::RefCell,rc::Rc};
use wasm_bindgen::{prelude::*,JsCast};
use web_sys::{Element,Event};

struct State { node:Option<Node>, elements:Vec<Element>, error:Option<String> }
struct Listener { element:Element, callback:Closure<dyn FnMut(Event)> }
impl Drop for Listener {
    fn drop(&mut self) { let _=self.element.remove_event_listener_with_callback("click",self.callback.as_ref().unchecked_ref()); }
}
#[wasm_bindgen]
pub struct Mount {
    listeners:Vec<Listener>,
    state:Rc<RefCell<State>>,
    root:Element,
    style:Element,
}
#[wasm_bindgen]
impl Mount {
    pub fn dispose(&mut self) {
        self.listeners.clear();
        self.state.borrow_mut().node.take();
        self.root.remove();
        self.style.remove();
    }
    pub fn error(&self)->Option<String> {self.state.borrow().error.clone()}
}
impl Drop for Mount { fn drop(&mut self){self.dispose();} }

fn elements(node:&Node,document:&web_sys::Document,all:&mut Vec<Element>,handlers:&mut Vec<usize>,theme:&super::theme::Theme,mode:&str)->Result<Element,JsValue> {
    let (theme,mode)=node.theme.as_ref().map(|(t,m)|(t,m.as_str())).unwrap_or((theme,mode));
    let tag=node.tag();
    let element=document.create_element(tag)?;
    for (key,value) in [("role",&node.semantics.role),("aria-label",&node.semantics.label),("id",&node.semantics.id),("data-slot",&node.semantics.slot),("data-size",&node.semantics.size)] {
        if !value.is_empty(){element.set_attribute(key,value)?;}
    }
    if node.kind==Kind::View{element.set_attribute("data-tsrust-view","")?;}
    if node.kind==Kind::Text{element.set_attribute("data-tsrust-text","")?;}
    if node.semantics.role=="separator"{element.set_attribute("aria-orientation",if node.semantics.orientation=="vertical"{"vertical"}else{"horizontal"})?;}
    if node.semantics.hidden{element.set_attribute("aria-hidden","true")?;}
    if node.theme.is_some(){element.set_attribute("data-tsrust-theme","")?;}
    element.set_attribute("style",&super::web::scope(node,theme,mode))?;
    element.set_attribute("class",&format!("ui {}",node.class_name))?;
    element.set_attribute("data-tsrust-node",&(all.len()+1).to_string())?;
    if matches!(node.kind,Kind::Button|Kind::DisabledButton){element.set_attribute("type","button")?;}
    if node.kind==Kind::DisabledButton {element.set_attribute("disabled","")?;}
    if node.kind!=Kind::View {element.set_text_content(Some(&node.text));}
    if node.press.is_some(){handlers.push(all.len());}
    all.push(element.clone());
    for child in &node.children {let child=elements(child,document,all,handlers,theme,mode)?;element.append_child(&child)?;}
    Ok(element)
}
fn update(node:&Node,elements:&[Element],next:&mut usize) {
    let element=&elements[*next];*next+=1;
    if matches!(node.kind,Kind::Button|Kind::DisabledButton) {
        if node.kind==Kind::DisabledButton {let _=element.set_attribute("disabled","");}else{let _=element.remove_attribute("disabled");}
    }

    if !node.parts.is_empty() && element.text_content().as_deref()!=Some(node.text.as_str()) {
        element.set_text_content(Some(&node.text));
    }
    for child in &node.children {update(child,elements,next);}
}
pub fn mount(mut node:Node,title:&str)->Result<Mount,JsValue> {
    node.refresh().map_err(|e|JsValue::from_str(&e))?;
    let document=web_sys::window().and_then(|w|w.document()).ok_or_else(||JsValue::from_str("Browser document unavailable"))?;
    document.set_title(title);
    thread_local! {static SEQUENCE:std::cell::Cell<u64>=const{std::cell::Cell::new(0)};}
    let id=SEQUENCE.with(|id|{let next=id.get().checked_add(1).expect("UI mount identities exhausted");id.set(next);next});
    let scope=format!("[data-tsrust-mount=\"{id}\"]");
    let css=super::web::stylesheet(&node,&scope);
    let style=document.create_element("style")?;style.set_text_content(Some(&css));
    let mut all=Vec::new();let mut handlers=Vec::new();let tree=elements(&node,&document,&mut all,&mut handlers,&super::theme::Theme::default(),"light")?;
    let root=document.create_element("div")?;root.set_attribute("data-tsrust-mount",&id.to_string())?;let default=super::theme::Theme::default();let (theme,mode)=node.theme.as_ref().map(|(t,m)|(t,m.as_str())).unwrap_or((&default,"light"));root.set_attribute("style",&format!("{}color:var(--ui-foreground);background:var(--ui-background);font-size:var(--ui-text-base);min-height:100vh",theme.variables(mode)))?;root.append_child(&tree)?;
    let state=Rc::new(RefCell::new(State{node:Some(node),elements:all,error:None}));
    let mut mount=Mount{listeners:vec![],state:state.clone(),root,style};
    for index in handlers {
        let element=state.borrow().elements[index].clone();
        let weak=Rc::downgrade(&state);
        let callback=Closure::wrap(Box::new(move |_event:Event| {
            let Some(state)=weak.upgrade() else{return;};
            let mut state=state.borrow_mut();
            let Some(node)=&mut state.node else{return;};
            if let Err(error)=node.activate(index as u64+1).and_then(|_|node.refresh()) {
                state.error=Some(error);state.node.take();return;
            }
            update(state.node.as_ref().unwrap(),&state.elements,&mut 0);
        }) as Box<dyn FnMut(Event)>);
        element.add_event_listener_with_callback("click",callback.as_ref().unchecked_ref())?;
        mount.listeners.push(Listener{element:element.clone(),callback});
    }
    mount.root.append_child(&mount.style)?;
    document.body().ok_or_else(||JsValue::from_str("Browser body unavailable"))?.append_child(&mount.root)?;
    Ok(mount)
}

// This slot exists only during entry construction. Returned JS Mount objects own
// every tree; no mounted state remains in the slot after launch returns.
thread_local! {static CONSTRUCTING:RefCell<Option<Vec<Mount>>>=const {RefCell::new(None)};}
pub fn run(node:Node,title:String)->Result<(),String> {
    CONSTRUCTING.with(|slot| {
        let mut slot=slot.borrow_mut();
        let mounts=slot.as_mut().ok_or("Browser run requires the launch entry")?;
        let mounted=mount(node,&title).map_err(|e|e.as_string().unwrap_or_else(||"DOM mounting failed".into()))?;
        mounts.push(mounted);Ok(())
    })
}
pub fn launch(entry:impl FnOnce()->bool)->Result<js_sys::Array,JsValue> {
    CONSTRUCTING.with(|slot| {
        if slot.borrow().is_some(){return Err(JsValue::from_str("Reentrant UI launch"));}
        *slot.borrow_mut()=Some(vec![]);
        let success=entry();
        let mounts=slot.borrow_mut().take().unwrap();
        if !success {return Err(JsValue::from_str("UI entry failed"));}
        Ok(mounts.into_iter().map(JsValue::from).collect())
    })
}
