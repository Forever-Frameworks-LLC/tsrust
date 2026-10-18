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

fn elements(node:&Node,document:&web_sys::Document,all:&mut Vec<Element>)->Result<Element,JsValue> {
    let tag=match node.kind {Kind::View=>"div",Kind::Text=>"span",Kind::Button|Kind::DisabledButton=>"button"};
    let element=document.create_element(tag)?;
    element.set_attribute("class",&format!("ui {}",node.class_name))?;
    element.set_attribute("data-tsrust-node",&(all.len()+1).to_string())?;
    if matches!(node.kind,Kind::Button|Kind::DisabledButton){element.set_attribute("type","button")?;}
    if node.kind==Kind::DisabledButton {element.set_attribute("disabled","")?;}
    if node.kind!=Kind::View {element.set_text_content(Some(&node.text));}
    all.push(element.clone());
    for child in &node.children {let child=elements(child,document,all)?;element.append_child(&child)?;}
    Ok(element)
}
fn update(node:&Node,elements:&[Element],next:&mut usize) {
    let element=&elements[*next];*next+=1;
    if !node.parts.is_empty() && element.text_content().as_deref()!=Some(node.text.as_str()) {
        element.set_text_content(Some(&node.text));
    }
    for child in &node.children {update(child,elements,next);}
}
pub fn mount(mut node:Node,title:&str)->Result<Mount,JsValue> {
    node.refresh().map_err(|e|JsValue::from_str(&e))?;
    let document=web_sys::window().and_then(|w|w.document()).ok_or_else(||JsValue::from_str("Browser document unavailable"))?;
    document.set_title(title);
    let html=super::document(&node,title);
    let css=html.split_once("<style>").unwrap().1.split_once("</style>").unwrap().0;
    let style=document.create_element("style")?;style.set_text_content(Some(css));
    let mut all=Vec::new();let root=elements(&node,&document,&mut all)?;
    let state=Rc::new(RefCell::new(State{node:Some(node),elements:all,error:None}));
    let mut mount=Mount{listeners:vec![],state:state.clone(),root,style};
    for (index,element) in state.borrow().elements.iter().enumerate() {
        if element.tag_name()!="BUTTON" || element.has_attribute("disabled") {continue;}
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
