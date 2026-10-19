//! The JavaScript host owns each Mount and must free it on removal/page exit.
use crate::{Kind,Node};
use std::{cell::RefCell,rc::Rc};
use wasm_bindgen::{prelude::*,JsCast};
use web_sys::{Element,Event};

struct State { node:Option<Node>, elements:std::collections::BTreeMap<u64,Element>, error:Option<String>, composing:std::collections::BTreeMap<u64,String>, root:Element, style:Element, scope:String }
struct Listener { element:Element, event:&'static str, callback:Closure<dyn FnMut(Event)> }
impl Drop for Listener {
    fn drop(&mut self) { let _=self.element.remove_event_listener_with_callback_and_bool(self.event,self.callback.as_ref().unchecked_ref(),true); }
}
#[wasm_bindgen]
pub struct Mount {
    listeners:Vec<Listener>,
    state:Rc<RefCell<State>>,
}
#[wasm_bindgen]
impl Mount {
    pub fn dispose(&mut self) {
        self.listeners.clear();
        let mut state=self.state.borrow_mut();state.node.take();state.composing.clear();state.elements.clear();state.root.remove();state.style.remove();
    }
    pub fn error(&self)->Option<String> {self.state.borrow().error.clone()}
}
impl Drop for Mount { fn drop(&mut self){self.dispose();} }

fn order_children(parent:&Element,children:&[Element])->Result<(),JsValue> {
    let mut next=parent.first_child();
    for child in children {
        let node:&web_sys::Node=child.as_ref();
        if next.as_ref()!=Some(node){parent.insert_before(node,next.as_ref())?;}
        next=child.next_sibling();
    }
    while let Some(node)=next{next=node.next_sibling();parent.remove_child(&node)?;}
    Ok(())
}
fn sync_element(node:&Node,document:&web_sys::Document,all:&mut std::collections::BTreeMap<u64,Element>,composing:&mut std::collections::BTreeMap<u64,String>,theme:&super::theme::Theme,mode:&str)->Result<Element,JsValue> {
    let (theme,mode)=node.theme.as_ref().map(|(t,m)|(t,m.as_str())).unwrap_or((theme,mode));
    let tag=node.tag();
    let id=node.identity.get();let element=if let Some(element)=all.get(&id){element.clone()}else{
    let element=document.create_element(tag)?;
    for (key,value) in [("role",&node.semantics.role),("aria-label",&node.semantics.label),("id",&node.semantics.id),("data-slot",&node.semantics.slot),("data-size",&node.semantics.size),("for",&node.semantics.html_for),("aria-describedby",&node.semantics.described_by),("aria-invalid",&node.semantics.invalid)] {
        if !value.is_empty(){element.set_attribute(key,value)?;}
    }
    if node.kind==Kind::View{element.set_attribute("data-tsrust-view","")?;}
    if node.kind==Kind::Text{element.set_attribute("data-tsrust-text","")?;}
    if node.semantics.role=="separator"{let orientation=if node.semantics.orientation=="vertical"{"vertical"}else{"horizontal"};element.set_attribute("aria-orientation",orientation)?;element.set_attribute("data-orientation",orientation)?;element.set_attribute(if orientation=="vertical"{"data-vertical"}else{"data-horizontal"},"")?;}
    if node.semantics.hidden{element.set_attribute("aria-hidden","true")?;}
    if node.theme.is_some(){element.set_attribute("data-tsrust-theme","")?;}
    element.set_attribute("style",&super::web::scope(node,theme,mode))?;
    element.set_attribute("class",&format!("ui {}",node.class_name))?;
    element.set_attribute("data-tsrust-node",&id.to_string())?;
    if matches!(node.kind,Kind::Button|Kind::DisabledButton){element.set_attribute("type","button")?;}
    if node.is_disabled() {element.set_attribute("disabled","")?;}
    if let Some(field)=&node.editable {
        element.set_attribute("placeholder",&field.placeholder)?;
        if field.read_only{element.set_attribute("readonly","")?;}
        if field.required{element.set_attribute("required","")?;}
        if !field.name.is_empty(){element.set_attribute("name",&field.name)?;}
        if !field.auto_complete.is_empty(){element.set_attribute("autocomplete",&field.auto_complete)?;}
        if !field.multiline{element.set_attribute("type","text")?;}
        value_property(&element,&field.value)?;
    }
    if node.kind!=Kind::View && node.editable.is_none() {element.set_text_content(Some(&node.text));}
    if node.press.is_some(){element.set_attribute("data-tsrust-press","")?;}
    all.insert(id,element.clone());element};

    if matches!(node.kind,Kind::Button|Kind::DisabledButton|Kind::Input|Kind::Textarea) {
        if node.is_disabled(){element.set_attribute("disabled","")?;}else{element.remove_attribute("disabled")?;}
    }
    if let Some(field)=&node.editable {
        let preserve=composing.get(&id).is_some_and(|original|original==&field.value)&&!field.disabled&&!field.read_only;
        if !preserve{composing.remove(&id);value_property(&element,&field.value)?;}
        if field.read_only{element.set_attribute("readonly","")?;}else{element.remove_attribute("readonly")?;}
    }
    if !node.parts.is_empty()&&element.text_content().as_deref()!=Some(node.text.as_str()){element.set_text_content(Some(&node.text));}
    if node.kind==Kind::View||!node.children.is_empty(){
        let mut children=Vec::new();for child in node.visible_children(){children.push(sync_element(child,document,all,composing,theme,mode)?);}
        order_children(&element,&children)?;
    }
    Ok(element)
}

fn control_value(element:&Element)->Result<String,String> {
    let value=js_sys::Reflect::get(element.as_ref(),&JsValue::from_str("value")).map_err(|_|String::from("DOM control value unavailable"))?;
    let value=value.dyn_ref::<js_sys::JsString>().ok_or("DOM control value must be a string")?;
    if !value.is_valid_utf16(){return Err("DOM control value contains an unpaired UTF-16 surrogate".into());}
    Ok(value.as_string().expect("checked JavaScript string"))
}
fn value_property(element:&Element,value:&str)->Result<(),JsValue> {
    let key=JsValue::from_str("value");
    if control_value(element).map_err(|error|JsValue::from_str(&error))?!=value && !js_sys::Reflect::set(element.as_ref(),&key,&JsValue::from_str(value))? {
        return Err(JsValue::from_str("DOM control value is not writable"));
    }
    Ok(())
}
fn update(state:&mut State,document:&web_sys::Document)->Result<(),JsValue> {
    let node=state.node.as_ref().expect("live mount");
    fn collect(node:&Node,live:&mut std::collections::BTreeSet<u64>){if let Some(node)=node.visible_root(){live.insert(node.identity.get());for child in node.visible_children(){collect(child,live);}}}
    let mut live=std::collections::BTreeSet::new();collect(node,&mut live);
    // Remove retired siblings first: moving a retained input can reset focus and IME.
    state.elements.retain(|id,element|{if live.contains(id){true}else{element.remove();false}});
    state.composing.retain(|id,_|live.contains(id));
    let default=super::theme::Theme::default();let (theme,mode)=node.theme.as_ref().map(|(t,m)|(t,m.as_str())).unwrap_or((&default,"light"));
    let mut roots=Vec::new();if let Some(root)=node.visible_root(){roots.push(sync_element(root,document,&mut state.elements,&mut state.composing,theme,mode)?);}
    order_children(&state.root,&roots)?;
    let css=super::web::stylesheet(node,&state.scope);if state.style.text_content().as_deref()!=Some(&css){state.style.set_text_content(Some(&css));}
    Ok(())
}
pub fn mount(mut node:Node,title:&str)->Result<Mount,JsValue> {
    node.refresh().map_err(|e|JsValue::from_str(&e))?;
    let document=web_sys::window().and_then(|w|w.document()).ok_or_else(||JsValue::from_str("Browser document unavailable"))?;
    document.set_title(title);
    thread_local! {static SEQUENCE:std::cell::Cell<u64>=const{std::cell::Cell::new(0)};}
    let id=SEQUENCE.with(|id|{let next=id.get().checked_add(1).expect("UI mount identities exhausted");id.set(next);next});
    let scope=format!("[data-tsrust-mount=\"{id}\"]");
    let style=document.create_element("style")?;
    let root=document.create_element("div")?;root.set_attribute("data-tsrust-mount",&id.to_string())?;let default=super::theme::Theme::default();let (theme,mode)=node.theme.as_ref().map(|(t,m)|(t,m.as_str())).unwrap_or((&default,"light"));root.set_attribute("style",&format!("{}color:var(--ui-foreground);background:var(--ui-background);font-size:var(--ui-text-base);min-height:100vh",theme.variables(mode)))?;
    let mut state=State{node:Some(node),elements:Default::default(),error:None,composing:Default::default(),root:root.clone(),style:style.clone(),scope};
    update(&mut state,&document)?;
    let state=Rc::new(RefCell::new(state));let mut mount=Mount{listeners:vec![],state:state.clone()};
    for event in ["click","input","compositionstart","compositionend"] {
        let weak=Rc::downgrade(&state);let document=document.clone();
        let callback=Closure::wrap(Box::new(move |message:Event| {
            let Some(state)=weak.upgrade() else{return;};let Ok(mut state)=state.try_borrow_mut() else{return;};
            if state.node.is_none(){return;}
            let Some(target)=message.target().and_then(|target|target.dyn_into::<Element>().ok()) else{return;};
            let target=if event=="click"{match target.closest("[data-tsrust-press]"){Ok(Some(element))=>element,_=>return}}else{target};
            if event!="click"&&!matches!(target.tag_name().as_str(),"INPUT"|"TEXTAREA"){return;}
            let Some(id)=target.get_attribute("data-tsrust-node").and_then(|value|value.parse::<u64>().ok()) else{return;};
            if state.elements.get(&id)!=Some(&target){return;}
            if event=="compositionstart" {
                match control_value(&target){Ok(value)=>{state.composing.insert(id,value);},Err(error)=>{state.error=Some(error);state.node.take();state.composing.clear();}}
                return;
            }
            if event=="input"&&state.composing.contains_key(&id){return;}
            if event=="compositionend"&&state.composing.remove(&id).is_none(){return;}
            let input=event!="click";let result=if input{control_value(&target)}else{Ok(String::new())};
            let node=state.node.as_mut().unwrap();
            if let Err(error)=result.and_then(|value|if input{node.input(id,&value)}else{node.activate(id)}).and_then(|_|node.refresh()) {
                state.error=Some(error);state.node.take();state.composing.clear();return;
            }
            if let Err(error)=update(&mut state,&document){state.error=Some(error.as_string().unwrap_or_else(||"DOM update failed".into()));state.node.take();state.composing.clear();}
        }) as Box<dyn FnMut(Event)>);
        // Capture keeps the existing control behavior for nonbubbling synthetic events too.
        root.add_event_listener_with_callback_and_bool(event,callback.as_ref().unchecked_ref(),true)?;
        mount.listeners.push(Listener{element:root.clone(),event,callback});
    }
    let body=document.body().ok_or_else(||JsValue::from_str("Browser body unavailable"))?;
    body.append_child(&style)?;body.append_child(&root)?;Ok(mount)
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
