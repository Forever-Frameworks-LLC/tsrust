//! The JavaScript host owns each Mount and must free it on removal/page exit.
use crate::{Kind,Node};
use std::{cell::RefCell,rc::Rc};
use wasm_bindgen::{prelude::*,JsCast};
use web_sys::{Element,Event};

struct State { weak:std::rc::Weak<RefCell<State>>, visibility_queued:bool, observer:Option<web_sys::IntersectionObserver>, resize_observer:Option<web_sys::ResizeObserver>, observed:std::collections::BTreeSet<u64>, disposed:bool, location:String, node:Option<Node>, elements:std::collections::BTreeMap<u64,Element>, error:Option<String>, composing:std::collections::BTreeMap<u64,String>, scrolls:std::collections::BTreeMap<u64,(i32,i32)>, reading_positions:std::collections::BTreeMap<String,std::collections::BTreeMap<u64,(i32,i32)>>, root:Element, style:Element, scope:String }
struct Listener { element:web_sys::EventTarget, event:&'static str, callback:Closure<dyn FnMut(Event)> }
impl Drop for Listener {
    fn drop(&mut self) { let _=self.element.remove_event_listener_with_callback_and_bool(self.event,self.callback.as_ref().unchecked_ref(),true); }
}
#[wasm_bindgen]
pub struct Mount {
    listeners:Vec<Listener>,
    visibility_callback:Option<Closure<dyn FnMut(js_sys::Array,web_sys::IntersectionObserver)>>,
    resize_callback:Option<Closure<dyn FnMut(js_sys::Array,web_sys::ResizeObserver)>>,
    state:Rc<RefCell<State>>,
}
#[wasm_bindgen]
impl Mount {
    pub fn dispose(&mut self) {
        self.listeners.clear();
        let mut state=self.state.borrow_mut();state.disposed=true;if let Some(observer)=state.observer.take(){observer.disconnect();observer.take_records();}if let Some(observer)=state.resize_observer.take(){observer.disconnect();}state.observed.clear();self.visibility_callback.take();self.resize_callback.take();state.node.take();state.composing.clear();state.elements.clear();state.scrolls.clear();state.reading_positions.clear();state.root.remove();state.style.remove();
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
    for (key,value) in [("href",&node.semantics.href),("role",&node.semantics.role),("aria-label",&node.semantics.label),("id",&node.semantics.id),("data-slot",&node.semantics.slot),("data-size",&node.semantics.size),("data-variant",&node.semantics.variant),("for",&node.semantics.html_for),("aria-describedby",&node.semantics.described_by),("aria-invalid",&node.semantics.invalid)] {
        if !value.is_empty(){element.set_attribute(key,value)?;}
    }
    if let Some(index)=node.semantics.tab_index{element.set_attribute("tabindex",&index.to_string())?;}
    if node.kind==Kind::View{element.set_attribute("data-tsrust-view","")?;}
    if node.kind==Kind::Text{element.set_attribute("data-tsrust-text","")?;}
    if node.semantics.role=="separator"{let orientation=if node.semantics.orientation=="vertical"{"vertical"}else{"horizontal"};element.set_attribute("aria-orientation",orientation)?;element.set_attribute("data-orientation",orientation)?;element.set_attribute(if orientation=="vertical"{"data-vertical"}else{"data-horizontal"},"")?;}
    if node.semantics.hidden{element.set_attribute("aria-hidden","true")?;}
    if node.theme.is_some(){element.set_attribute("data-tsrust-theme","")?;}
    element.set_attribute("style",&super::web::scope(node,theme,mode))?;
    element.set_attribute("class",&format!("ui {}",node.class_name))?;
    element.set_attribute("data-tsrust-node",&id.to_string())?;
    if matches!(node.kind,Kind::Button|Kind::DisabledButton)&&node.semantics.href.is_empty(){element.set_attribute("type","button")?;}
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
    if node.press.is_some()||node.copy.is_some(){element.set_attribute("data-tsrust-press","")?;}
    all.insert(id,element.clone());element};

    if node.class_binding.is_some(){
        let classes=format!("ui {}",node.class_name);
        if element.get_attribute("class").as_deref()!=Some(classes.as_str()){element.set_attribute("class",&classes)?;}
    }
    if node.variant_binding.is_some(){
        let variant=&node.semantics.variant;
        if variant.is_empty(){element.remove_attribute("data-variant")?;}
        else if element.get_attribute("data-variant").as_deref()!=Some(variant.as_str()){element.set_attribute("data-variant",variant)?;}
    }
    if !node.semantics.href.is_empty(){
        if let Some(current)=super::navigation::current_kind(&node.semantics.href){element.set_attribute("aria-current",current)?;}else{element.remove_attribute("aria-current")?;}
    }
    if node.theme.is_some(){element.set_attribute("style",&super::web::scope(node,theme,mode))?;}
    if matches!(node.kind,Kind::Button|Kind::DisabledButton|Kind::Input|Kind::Textarea) {
        if node.is_disabled(){element.set_attribute("disabled","")?;}else{element.remove_attribute("disabled")?;}
    }
    if let Some(field)=&node.editable {
        let preserve=composing.get(&id).is_some_and(|original|original==&field.value)&&!field.disabled&&!field.read_only;
        if !preserve{composing.remove(&id);value_property(&element,&field.value)?;}
        if field.read_only{element.set_attribute("readonly","")?;}else{element.remove_attribute("readonly")?;}
    }
    if let Some(copy)=&node.copy {
        element.set_text_content(Some(&node.text));element.set_attribute("aria-live","polite")?;
        element.set_attribute("data-copy-state",if copy.pending{"pending"}else if !copy.error.is_empty(){"error"}else if node.text=="Copied"{"copied"}else{"idle"})?;
        if copy.error.is_empty(){element.remove_attribute("title")?;}else{element.set_attribute("title",&copy.error)?;}
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
    fn collect_owned(node:&Node,owned:&mut std::collections::BTreeSet<u64>){owned.insert(node.identity.get());for child in &node.children{collect_owned(child,owned);}}
    let mut live=std::collections::BTreeSet::new();collect(node,&mut live);
    let mut owned=std::collections::BTreeSet::new();collect_owned(node,&mut owned);
    let reading_nodes=node.reading_nodes();
    for (id,element) in &state.elements {if element.is_connected(){state.scrolls.insert(*id,(element.scroll_left(),element.scroll_top()));}}
    let location=super::navigation::current().unwrap_or_default();
    let navigating=location!=state.location;
    if navigating&&!state.location.is_empty(){
        state.reading_positions.insert(state.location.clone(),state.elements.iter().filter(|(id,element)|reading_nodes.contains(id)&&element.is_connected()).filter_map(|(id,_)|state.scrolls.get(id).map(|position|(*id,*position))).collect());
    }
    // Detach hidden panes but retain their elements/offsets until their model is removed.
    for (id,element) in &state.elements {if !live.contains(id) && state.observed.remove(id) {
        if let Some(observer)=&state.observer{observer.unobserve(element);}
        if let Some(observer)=&state.resize_observer{observer.unobserve(element);}
    }}
    state.elements.retain(|id,element|{if !live.contains(id){element.remove();}owned.contains(id)});
    state.scrolls.retain(|id,_|owned.contains(id));
    for positions in state.reading_positions.values_mut(){positions.retain(|id,_|owned.contains(id));}
    state.composing.retain(|id,_|live.contains(id));
    let default=super::theme::Theme::default();let (theme,mode)=node.theme.as_ref().map(|(t,m)|(t,m.as_str())).unwrap_or((&default,"light"));
    state.root.set_attribute("style",&format!("{}color:var(--ui-foreground);background:var(--ui-background);font-size:var(--ui-text-base);white-space:pre-wrap;min-height:100vh",theme.variables(mode)))?;
    // Restored panes need their sizing/overflow rules before offsets can be applied.
    let css=super::web::stylesheet(node,&state.scope);if state.style.text_content().as_deref()!=Some(&css){state.style.set_text_content(Some(&css));}
    let mut roots=Vec::new();if let Some(root)=node.visible_root(){roots.push(sync_element(root,document,&mut state.elements,&mut state.composing,theme,mode)?);}
    order_children(&state.root,&roots)?;
    if navigating {if let Some(positions)=state.reading_positions.get(&location){for (id,position) in positions{state.scrolls.insert(*id,*position);}}}
    for (id,(x,y)) in &state.scrolls {if let Some(element)=state.elements.get(id).filter(|element|element.is_connected()){element.set_scroll_left(*x);element.set_scroll_top(*y);}}
    observe(state);queue_hidden(state,document);focus_location(state,document)
}
fn observe(state:&mut State) {
    let Some(observer)=state.observer.as_ref() else{return;};
    let mut ids=Vec::new();if let Some(node)=&state.node{node.observed_nodes(true,&mut ids);}
    let live=ids.into_iter().filter(|id|state.elements.get(id).is_some_and(|element|element.is_connected())).collect::<std::collections::BTreeSet<_>>();
    for id in state.observed.difference(&live){if let Some(element)=state.elements.get(id){observer.unobserve(element);if let Some(resize)=&state.resize_observer{resize.unobserve(element);}}}
    for id in live.difference(&state.observed){if let Some(element)=state.elements.get(id){observer.observe(element);if let Some(resize)=&state.resize_observer{resize.observe(element);}}}
    state.observed=live;
}
fn queue_hidden(state:&mut State,document:&web_sys::Document) {
    if state.visibility_queued||state.weak.strong_count()==0{return;}
    state.visibility_queued=true;let weak=state.weak.clone();let document=document.clone();
    wasm_bindgen_futures::spawn_local(async move {
        let Some(state_ref)=weak.upgrade() else{return;};
        let changes={let mut state=state_ref.borrow_mut();state.visibility_queued=false;if state.disposed{return;}
            let mut ids=Vec::new();if let Some(node)=&state.node{node.observed_nodes(false,&mut ids);}
            ids.into_iter().filter(|id|!state.observed.contains(id)).map(|id|(id,false)).collect::<Vec<_>>()};
        if !changes.is_empty(){visibility_update(&state_ref,&document,&changes);}
    });
}
fn visibility_update(state_ref:&Rc<RefCell<State>>,document:&web_sys::Document,changes:&[(u64,bool)]) {
    // Application callbacks can call a host binding which disposes this Mount.
    let Some(mut node)=state_ref.borrow_mut().node.take() else{return;};
    let result=(||->Result<bool,String>{let mut changed=false;for &(id,visible) in changes {changed|=node.visibility_changed(id,visible)?;}if changed{node.refresh()?;}Ok(changed)})();
    let mut state=state_ref.borrow_mut();if state.disposed{return;}
    let changed=match result{Ok(changed)=>changed,Err(error)=>{state.error=Some(error);return;}};
    state.node=Some(node);if !changed{return;}if let Err(error)=update(&mut state,document){state.error=Some(format!("{error:?}"));state.node.take();}
}
fn focus_location(state:&mut State,document:&web_sys::Document)->Result<(),JsValue>{
    if !state.root.is_connected(){return Ok(());}
    if let Some(path)=super::navigation::current(){
        if path!=state.location {
            let target=path.split_once('#').and_then(|(_,fragment)|document.get_element_by_id(fragment)).filter(|target|state.root.contains(Some(target.as_ref()))).or_else(||state.root.query_selector("h1").ok().flatten());
            if let Some(target)=target {
                target.set_attribute("tabindex","-1")?;
                if let Ok(focus)=js_sys::Reflect::get(target.as_ref(),&JsValue::from_str("focus")){if let Some(focus)=focus.dyn_ref::<js_sys::Function>(){let options=js_sys::Object::new();let _=js_sys::Reflect::set(options.as_ref(),&JsValue::from_str("preventScroll"),&JsValue::TRUE);let _=focus.call1(target.as_ref(),options.as_ref());}}
                if path.contains('#')&&!state.reading_positions.contains_key(&path){target.scroll_into_view();}
            }
            state.location=path;
        }
    }
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
    let root=document.create_element("div")?;root.set_attribute("data-tsrust-mount",&id.to_string())?;let default=super::theme::Theme::default();let (theme,mode)=node.theme.as_ref().map(|(t,m)|(t,m.as_str())).unwrap_or((&default,"light"));root.set_attribute("style",&format!("{}color:var(--ui-foreground);background:var(--ui-background);font-size:var(--ui-text-base);white-space:pre-wrap;min-height:100vh",theme.variables(mode)))?;
    let mut state=State{weak:Default::default(),visibility_queued:false,observer:None,resize_observer:None,observed:Default::default(),disposed:false,location:String::new(),node:Some(node),elements:Default::default(),error:None,composing:Default::default(),scrolls:Default::default(),reading_positions:Default::default(),root:root.clone(),style:style.clone(),scope};
    update(&mut state,&document)?;
    let state=Rc::new(RefCell::new(state));state.borrow_mut().weak=Rc::downgrade(&state);let mut mount=Mount{listeners:vec![],visibility_callback:None,resize_callback:None,state:state.clone()};
    let weak=Rc::downgrade(&state);let visibility_document=document.clone();
    let visibility_callback=Closure::wrap(Box::new(move |entries:js_sys::Array,_:web_sys::IntersectionObserver| {
        let Some(state)=weak.upgrade() else{return;};
        let mut changes=Vec::new();
        {let state=state.borrow();if state.disposed{return;}
        for entry in entries.iter().filter_map(|entry|entry.dyn_into::<web_sys::IntersectionObserverEntry>().ok()) {
            let element=entry.target();let Some(id)=element.get_attribute("data-tsrust-node").and_then(|value|value.parse::<u64>().ok()) else{continue;};
            if !state.observed.contains(&id)||state.elements.get(&id)!=Some(&element){continue;}
            let rectangle=entry.intersection_rect();changes.push((id,element.is_connected()&&rectangle.width()>0.&&rectangle.height()>0.));
        }}
        if !changes.is_empty(){visibility_update(&state,&visibility_document,&changes);}
    }) as Box<dyn FnMut(js_sys::Array,web_sys::IntersectionObserver)>);
    let options=web_sys::IntersectionObserverInit::new();let thresholds=js_sys::Array::new();thresholds.push(&JsValue::from_f64(0.));thresholds.push(&JsValue::from_f64(f64::EPSILON));options.set_threshold(&thresholds);
    state.borrow_mut().observer=Some(web_sys::IntersectionObserver::new_with_options(visibility_callback.as_ref().unchecked_ref(),&options)?);mount.visibility_callback=Some(visibility_callback);
    let weak=Rc::downgrade(&state);
    let resize_callback=Closure::wrap(Box::new(move |entries:js_sys::Array,_:web_sys::ResizeObserver| {
        let Some(state_ref)=weak.upgrade() else{return;};
        let (observer,targets)={let state=state_ref.borrow();if state.disposed{return;}
            let targets=entries.iter().filter_map(|entry|entry.dyn_into::<web_sys::ResizeObserverEntry>().ok()).map(|entry|entry.target()).filter(|element|element.get_attribute("data-tsrust-node").and_then(|id|id.parse::<u64>().ok()).is_some_and(|id|state.observed.contains(&id)&&state.elements.get(&id)==Some(element))).collect::<Vec<_>>();
            (state.observer.clone(),targets)};
        // Zero-area targets can keep intersectionRatio=1; force a fresh rectangle report.
        if let Some(observer)=observer{for target in targets{observer.unobserve(&target);observer.observe(&target);}}
    }) as Box<dyn FnMut(js_sys::Array,web_sys::ResizeObserver)>);
    state.borrow_mut().resize_observer=Some(web_sys::ResizeObserver::new(resize_callback.as_ref().unchecked_ref())?);mount.resize_callback=Some(resize_callback);

    for event in ["click","input","compositionstart","compositionend"] {
        let weak=Rc::downgrade(&state);let document=document.clone();
        let callback=Closure::wrap(Box::new(move |message:Event| {
            let Some(state_ref)=weak.upgrade() else{return;};let Ok(mut state)=state_ref.try_borrow_mut() else{return;};
            if state.node.is_none(){return;}
            let Some(target)=message.target().and_then(|target|target.dyn_into::<Element>().ok()) else{return;};
            let target=if event=="click"{match target.closest("[data-tsrust-press]"){Ok(Some(element))=>element,_=>return}}else{target};
            if event=="click" && target.tag_name()=="A" {
                if target.get_attribute("href").is_some_and(|href|super::navigation::external(&href)){return;}
                let property=|name|js_sys::Reflect::get(message.as_ref(),&JsValue::from_str(name)).unwrap_or(JsValue::UNDEFINED);
                if ["ctrlKey","metaKey","shiftKey","altKey"].iter().any(|name|property(name).as_bool()==Some(true))||property("button").as_f64().is_some_and(|button|button!=0.){return;}
                message.prevent_default();
            }
            if event!="click"&&!matches!(target.tag_name().as_str(),"INPUT"|"TEXTAREA"){return;}
            let Some(id)=target.get_attribute("data-tsrust-node").and_then(|value|value.parse::<u64>().ok()) else{return;};
            if state.elements.get(&id)!=Some(&target){return;}
            if event=="compositionstart" {
                match control_value(&target){Ok(value)=>{state.composing.insert(id,value);},Err(error)=>{state.error=Some(error);state.node.take();state.composing.clear();}}
                return;
            }
            if event=="input"&&state.composing.contains_key(&id){return;}
            if event=="compositionend"&&state.composing.remove(&id).is_none(){return;}
            if event=="click" {
                let request=state.node.as_mut().and_then(|node|node.copy_node_mut(id)).and_then(Node::begin_copy);
                if let Some((serial,value))=request {
                    if let Err(error)=update(&mut state,&document){state.error=Some(format!("{error:?}"));state.node.take();return;}
                    // A host clipboard implementation may synchronously dispose the mount.
                    drop(state);
                    let pending=clipboard_write(&value);let weak=Rc::downgrade(&state_ref);let document=document.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let result=wasm_bindgen_futures::JsFuture::from(pending).await.map(|_|()).map_err(|error|error.as_string().unwrap_or_else(||format!("{error:?}")));
                        let Some(state)=weak.upgrade() else{return;};let Ok(mut state)=state.try_borrow_mut() else{return;};
                        let Some(node)=state.node.as_mut().and_then(|node|node.copy_node_mut(id)) else{return;};node.finish_copy(serial,result);
                        if let Err(error)=update(&mut state,&document){state.error=Some(format!("{error:?}"));state.node.take();}
                    });
                    return;
                }
            }
            let input=event!="click";let result=if input{control_value(&target)}else{Ok(String::new())};
            let node=state.node.as_mut().unwrap();
            if let Err(error)=result.and_then(|value|if input{node.input(id,&value)}else{node.activate(id)}).and_then(|_|node.refresh()) {
                state.error=Some(error);state.node.take();state.composing.clear();return;
            }
            if let Err(error)=update(&mut state,&document){state.error=Some(error.as_string().unwrap_or_else(||"DOM update failed".into()));state.node.take();state.composing.clear();}

        }) as Box<dyn FnMut(Event)>);
        // Capture keeps the existing control behavior for nonbubbling synthetic events too.
        root.add_event_listener_with_callback_and_bool(event,callback.as_ref().unchecked_ref(),true)?;
        mount.listeners.push(Listener{element:root.clone().into(),event,callback});
    }
    if let Some(window)=web_sys::window() {
        for event in ["popstate","hashchange"] {
            let weak=Rc::downgrade(&state);let document=document.clone();
            let callback=Closure::wrap(Box::new(move |_:Event| {
                let Some(state)=weak.upgrade() else{return;};let Ok(mut state)=state.try_borrow_mut() else{return;};if state.node.is_none(){return;}
                let result=super::navigation::restore_browser().and_then(|_|state.node.as_mut().unwrap().refresh().map(|_|()));
                if let Err(error)=result{state.error=Some(error);state.node.take();return;}
                if let Err(error)=update(&mut state,&document){state.error=Some(format!("{error:?}"));state.node.take();}
            }) as Box<dyn FnMut(Event)>);
            window.add_event_listener_with_callback_and_bool(event,callback.as_ref().unchecked_ref(),true)?;
            mount.listeners.push(Listener{element:window.clone().into(),event,callback});
        }
    }
    let body=document.body().ok_or_else(||JsValue::from_str("Browser body unavailable"))?;
    body.append_child(&style)?;body.append_child(&root)?;observe(&mut state.borrow_mut());queue_hidden(&mut state.borrow_mut(),&document);focus_location(&mut state.borrow_mut(),&document)?;Ok(mount)
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

pub fn run_configured(node:Node,title:String,options:&str)->Result<(),String> {
    super::window::Options::parse(options)?;
    run(node,title)
}

fn clipboard_write(value:&str)->js_sys::Promise {
    let start=(||->Result<js_sys::Promise,JsValue>{
        let window=web_sys::window().ok_or_else(||JsValue::from_str("Browser window unavailable"))?;
        let navigator=js_sys::Reflect::get(window.as_ref(),&JsValue::from_str("navigator"))?;
        let clipboard=js_sys::Reflect::get(&navigator,&JsValue::from_str("clipboard"))?;
        if clipboard.is_undefined()||clipboard.is_null(){return Err(JsValue::from_str("Clipboard unavailable. Use HTTPS or localhost."));}
        let write=js_sys::Reflect::get(&clipboard,&JsValue::from_str("writeText"))?.dyn_into::<js_sys::Function>()?;
        write.call1(&clipboard,&JsValue::from_str(value))?.dyn_into::<js_sys::Promise>()
    })();start.unwrap_or_else(|error|js_sys::Promise::reject(&error))
}
