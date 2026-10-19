//! Owned semantic nodes shared by the web and native renderers.
//! Synchronous handlers mutate explicitly owned state through weak handles.

#[cfg(feature = "native")]
mod native;
#[cfg(all(target_arch="wasm32",feature="ui-web"))]
pub mod browser;
mod state;
pub use state::Signal;
pub mod style;
pub mod theme;
mod web;
#[cfg(feature = "native")]
pub use native::run;
#[cfg(feature = "acceptance")]
#[doc(hidden)]
pub use native::verify_window;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    View,
    Text,
    DisabledButton,
    Button,
}

pub enum TextPart { Literal(String), Signal(Signal) }
/// Owned child content. Text parts are grouped without cloning their state references.
pub enum Child { Node(Node), Text(TextPart) }

#[derive(Clone, Debug, Default, serde::Deserialize)]
#[serde(default, deny_unknown_fields, rename_all="camelCase")]
pub(crate) struct Semantics {
    pub tag: String,
    pub role: String,
    pub label: String,
    pub id: String,
    pub hidden: bool,
    pub slot: String,
    pub orientation: String,
}

pub struct Node {
    depth: usize,
    pub(crate) theme: Option<(theme::Theme,String)>,
    pub(crate) semantics: Semantics,
    pub(crate) kind: Kind,
    pub(crate) class_name: String,
    pub(crate) text: String,
    pub(crate) children: Vec<Node>,
    pub(crate) parts: Vec<TextPart>,
    pub(crate) disabled: Option<Box<dyn FnMut()->Result<bool,String>>>,
    pub(crate) press: Option<Box<dyn FnMut() -> Result<(), String>>>,
    state: Vec<Signal>,
}

impl Node {
    pub fn with_semantics(mut self, config: &str) -> Result<Self,String> {
        if config.is_empty(){return Ok(self);}
        let value:Semantics=serde_json::from_str(config).map_err(|e|format!("Invalid UI semantics: {e}"))?;
        let tag=if value.tag.is_empty(){self.tag()}else{&value.tag};
        let allowed=match self.kind {
            Kind::View=>matches!(tag,"div"|"section"|"span"),
            Kind::Text=>matches!(tag,"div"|"span"|"p"|"h1"|"h2"|"h3"|"h4"|"h5"|"h6"|"code"|"kbd"),
            Kind::Button|Kind::DisabledButton=>tag=="button",
        };
        if !allowed{return Err(format!("Unsupported UI element for {:?}: {tag}",self.kind));}
        if !matches!(value.role.as_str(),""|"alert"|"status"|"separator"|"group"|"note"|"region"|"none"|"presentation") {return Err(format!("Unsupported UI role: {}",value.role));}
        if matches!(self.kind,Kind::Button|Kind::DisabledButton) && !value.role.is_empty(){return Err("Button keeps its implicit button role".into());}
        if !value.orientation.is_empty() && (value.role!="separator" || !matches!(value.orientation.as_str(),"horizontal"|"vertical")){return Err("aria-orientation requires a separator and horizontal or vertical".into());}
        let name_allowed=if value.role.is_empty(){matches!(tag,"button"|"h1"|"h2"|"h3"|"h4"|"h5"|"h6"|"section")}else{!matches!(value.role.as_str(),"none"|"presentation")};
        if !value.label.is_empty() && !name_allowed{return Err("aria-label requires a button, heading, section or a role that supports naming".into());}
        if value.role=="region" && value.label.trim().is_empty(){return Err("A region requires an accessible label".into());}
        if value.role=="separator" && (!self.text.is_empty() || !self.children.is_empty()){return Err("A separator cannot contain text or children".into());}
        if value.id.chars().any(|ch|ch.is_ascii_whitespace()||ch.is_control()){return Err("UI id cannot contain whitespace or control characters".into());}
        if value.slot.chars().any(|ch|!(ch.is_ascii_alphanumeric()||ch=='-')){return Err("UI slot must contain ASCII letters, digits or hyphens".into());}
        if value.hidden && self.has_enabled_controls(){return Err("An aria-hidden subtree cannot contain enabled interactive controls".into());}
        self.semantics=value;Ok(self)
    }
    fn has_enabled_controls(&self)->bool {self.kind==Kind::Button || self.children.iter().any(Self::has_enabled_controls)}
    pub(crate) fn tag(&self)->&str {
        if !self.semantics.tag.is_empty(){&self.semantics.tag}else{match self.kind{Kind::View=>"div",Kind::Text=>"span",Kind::Button|Kind::DisabledButton=>"button"}}
    }
    pub fn view(children: Vec<Child>, class_name: String) -> Result<Self,String> {
        Self::new(Kind::View,class_name,String::new(),Self::child_nodes(children)?)
    }
    fn child_nodes(children: Vec<Child>) -> Result<Vec<Node>,String> {
        fn flush(parts: &mut Vec<TextPart>, nodes: &mut Vec<Node>) -> Result<(),String> {
            if parts.iter().any(|part|match part{TextPart::Literal(text)=>!text.is_empty(),TextPart::Signal(_)=>true}) {
                nodes.push(Node::text(std::mem::take(parts),String::new())?);
            } else { parts.clear(); }
            Ok(())
        }
        let mut nodes=Vec::new();let mut parts=Vec::new();
        for child in children {
            match child {
                Child::Text(part)=>parts.push(part),
                Child::Node(node)=>{flush(&mut parts,&mut nodes)?;nodes.push(node);}
            }
        }
        flush(&mut parts,&mut nodes)?;Ok(nodes)
    }
    pub fn button_children(children:Vec<Child>,class_name:String,disabled:bool,press:Option<Box<dyn FnMut()->Result<(),String>>>) -> Result<Self,String> {
        fn phrasing(node:&Node)->bool {
            matches!(node.tag(),"span"|"code"|"kbd")
                && matches!(node.semantics.role.as_str(),""|"none"|"presentation")
                && !matches!(node.kind,Kind::Button|Kind::DisabledButton)
                && node.children.iter().all(phrasing)
        }
        let children=Self::child_nodes(children)?;
        if !children.iter().all(phrasing){return Err("Button children must be noninteractive phrasing content: text, span, code or kbd".into());}
        let mut node=Self::new(Kind::View,class_name,String::new(),children)?;
        node.kind=if disabled{Kind::DisabledButton}else{Kind::Button};node.press=press;Ok(node)
    }
    pub(crate) fn content_text(&self)->String {
        fn append(node:&Node,out:&mut String) {
            if node.semantics.hidden{return;}
            out.push_str(&node.text);for child in &node.children{append(child,out);}
        }
        let mut output=String::new();append(self,&mut output);output
    }
    pub fn with_disabled(mut self,mut predicate:impl FnMut()->Result<bool,String>+'static)->Result<Self,String> {
        if !matches!(self.kind,Kind::Button|Kind::DisabledButton) {return Err("A disabled condition requires a Button".into());}
        self.kind=if predicate()? {Kind::DisabledButton}else{Kind::Button};self.disabled=Some(Box::new(predicate));Ok(self)
    }

    pub fn with_theme(mut self, json: &str, mode: &str) -> Result<Self,String> {
        let theme=theme::Theme::parse(json)?;theme.color(mode,"background")?;
        self.theme=Some((theme,mode.to_owned()));Ok(self)
    }
    pub fn own_state(mut self, state: Signal) -> Result<Self,String> { self.state.push(state); Ok(self) }
    pub fn text(parts:Vec<TextPart>,class_name:String)->Result<Self,String> {
        let text=Self::text_value(&parts)?;
        let mut node=Self::new(Kind::Text,class_name,text,vec![])?;
        if parts.iter().any(|part|matches!(part,TextPart::Signal(_))) {node.parts=parts;}
        Ok(node)
    }
    fn text_value(parts:&[TextPart])->Result<String,String> {
        let mut text=String::new();
        for part in parts {match part {
            TextPart::Literal(value)=>text.push_str(value),
            TextPart::Signal(handle)=>text.push_str(&handle.value().ok_or("Text binding refers to disposed state")?.to_string()),
        }}
        Ok(text)
    }
    pub fn bound_text(handle:Signal,class_name:String)->Result<Self,String> {Self::text(vec![TextPart::Signal(handle)],class_name)}
    pub fn button(text: String, class_name: String, disabled: bool, press: impl FnMut()->Result<(),String>+'static) -> Result<Self,String> {
        let mut node=Self::new(if disabled {Kind::DisabledButton}else{Kind::Button},class_name,text,vec![])?;
        node.press=Some(Box::new(press));Ok(node)
    }
    pub(crate) fn refresh(&mut self) -> Result<usize,String> {
        let mut changed=0;
        if let Some(predicate)=&mut self.disabled {
            let kind=if predicate()? {Kind::DisabledButton}else{Kind::Button};if kind!=self.kind {self.kind=kind;changed+=1;}
        }

        if !self.parts.is_empty() {
            let text=Self::text_value(&self.parts)?;
            if text!=self.text {self.text=text;changed+=1;}
        }
        for child in &mut self.children {changed+=child.refresh()?;}
        if self.semantics.hidden && self.has_enabled_controls(){return Err("An aria-hidden subtree cannot contain enabled interactive controls".into());}
        Ok(changed)
    }
    /// Stable preorder IDs belong to this mounted tree, with zero reserved for its window.
    pub(crate) fn activate(&mut self, target: u64) -> Result<bool,String> {
        fn visit(node:&mut Node,target:u64,next:&mut u64)->Result<bool,String> {
            let id=*next;*next+=1;
            if id==target {
                if node.kind!=Kind::Button {return Ok(false);}
                if let Some(handler)=&mut node.press {handler()?;return Ok(true);}
                return Ok(false);
            }
            for child in &mut node.children {if visit(child,target,next)? {return Ok(true);}}
            Ok(false)
        }
        visit(self,target,&mut 1)
    }

    pub fn new(
        kind: Kind,
        class_name: String,
        text: String,
        children: Vec<Node>,
    ) -> Result<Self, String> {
        let depth = 1 + children.iter().map(|child| child.depth).max().unwrap_or(0);
        if depth > 128 {
            return Err("UI nesting exceeds 128 nodes".into());
        }
        style::validate(&class_name)?;
        match kind {
            Kind::View if !text.is_empty() => return Err("View text must be inside Text".into()),
            Kind::Text | Kind::DisabledButton | Kind::Button if !children.is_empty() => {
                return Err("Text and Button accept text children only".into());
            }
            _ => {}
        }
        let class_name = class_name.split_whitespace().collect::<Vec<_>>().join(" ");
        Ok(Self {
            depth,
            theme: None,
            semantics: Semantics::default(),
            kind,
            class_name,
            text,
            children,
            parts: Vec::new(), disabled:None, press: None, state: Vec::new(),
        })
    }
}

pub fn document(root: &Node, title: &str) -> String {
    web::document(root, title)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_shape_unknown_utilities_and_depth_reject() {
        assert!(
            Node::new(
                Kind::View,
                "unknown".into(),
                String::new(),
                vec![]
            )
            .is_err()
        );
        assert!(Node::new(Kind::View, String::new(), "unwrapped text".into(), vec![]).is_err());
        let leaf = Node::new(Kind::Text, String::new(), "text".into(), vec![]).unwrap();
        assert!(
            Node::new(
                Kind::DisabledButton,
                String::new(),
                String::new(),
                vec![Node::new(Kind::Text, String::new(), "text".into(), vec![]).unwrap()]
            )
            .is_err()
        );
        let mut node = leaf;
        for _ in 1..128 {
            node = Node::new(Kind::View, String::new(), String::new(), vec![node]).unwrap();
        }
        assert!(Node::new(Kind::View, String::new(), String::new(), vec![node]).is_err());
    }
}

#[cfg(test)]
mod interaction_tests {
    use super::*;
    #[test]
    fn handler_capture_is_destroyed_once_after_failure_and_owner_drop() {
        use std::{rc::Rc,cell::Cell};
        struct Probe(Rc<Cell<usize>>);
        impl Drop for Probe {fn drop(&mut self){self.0.set(self.0.get()+1);}}
        let releases=Rc::new(Cell::new(0));let probe=Probe(releases.clone());
        let mut node=Node::button("Fail".into(),String::new(),false,move||{assert_eq!(probe.0.get(),0);Err("handler failed".into())}).unwrap();
        assert_eq!(node.activate(1).unwrap_err(),"handler failed");
        assert_eq!(releases.get(),0);drop(node);assert_eq!(releases.get(),1);
    }
    #[test]
    fn text_mixes_literals_and_multiple_reactive_children() {
        let first=Signal::new(2);let second=Signal::new(5);let action=first.handle();
        let mut node=Node::text(vec![TextPart::Literal("Count: ".into()),TextPart::Signal(first.handle()),TextPart::Literal(" / ".into()),TextPart::Signal(second.handle())],String::new()).unwrap();
        assert_eq!(node.text,"Count: 2 / 5");action.set(action.get().unwrap() - 3).unwrap();assert_eq!(node.refresh().unwrap(),1);assert_eq!(node.text,"Count: -1 / 5");
        assert_eq!(node.refresh().unwrap(),0);drop(first);assert!(node.refresh().is_err());
    }
    #[test]
    fn counter_dispatch_updates_only_affected_binding_and_releases_state() {
        fn counter()->(Node,Signal) {
            let state=Signal::new(0);let observer=state.handle();let action=state.handle();let disabled=state.handle();
            let text=Node::bound_text(state.handle(),String::new()).unwrap();
            let button=Node::button("Increment".into(),String::new(),false,move ||{action.set(action.get()? + 1)?;Ok(())}).unwrap();
            let disabled=Node::button("Disabled".into(),String::new(),true,move ||{disabled.set(disabled.get()? + 100)?;Ok(())}).unwrap();
            (Node::new(Kind::View,String::new(),String::new(),vec![text,button,disabled]).unwrap().own_state(state).unwrap(),observer)
        }
        let (first,a)=counter();let (second,b)=counter();
        let mut root=Node::new(Kind::View,String::new(),String::new(),vec![first,second]).unwrap();
        assert_eq!(root.refresh().unwrap(),0);
        assert!(root.activate(4).unwrap());assert_eq!(root.refresh().unwrap(),1);
        assert_eq!(a.value(),Some(1));assert_eq!(b.value(),Some(0));
        assert!(!root.activate(5).unwrap());assert_eq!(root.refresh().unwrap(),0);
        assert!(root.activate(8).unwrap());assert_eq!(root.refresh().unwrap(),1);
        assert_eq!(b.value(),Some(1));
        assert!(!root.activate(999).unwrap());
        drop(root);assert_eq!(a.value(),None);assert_eq!(b.value(),None);assert!(a.set(1).is_err());
    }
}

#[cfg(test)]mod condition_tests {
 use super::*;
 #[test]fn disabled_expression_refreshes_after_actions_and_prevents_dispatch() {
  let count=Signal::new(0);let increment=count.handle();let condition=count.handle();let action=count.handle();let observe=count.handle();
  let increment=Node::button("Increment".into(),String::new(),false,move||{increment.set(increment.get()?+1)}).unwrap();
  let conditional=Node::button("Disabled".into(),String::new(),false,move||{action.set(action.get()?+100)}).unwrap().with_disabled(move||Ok(condition.get()?<5)).unwrap();
  let mut root=Node::new(Kind::View,String::new(),String::new(),vec![increment,conditional]).unwrap().own_state(count).unwrap();
  assert!(!root.activate(3).unwrap());assert_eq!(observe.get().unwrap(),0);
  for _ in 0..5 {assert!(root.activate(2).unwrap());root.refresh().unwrap();}
  assert_eq!(root.children[1].kind,Kind::Button);assert!(root.activate(3).unwrap());assert_eq!(observe.get().unwrap(),105);
  drop(root);assert!(observe.get().is_err());
 }
 #[test]fn failed_condition_releases_callback_once_before_state_owner() {
  use std::{rc::Rc,cell::Cell};struct Probe(Rc<Cell<usize>>);impl Drop for Probe{fn drop(&mut self){self.0.set(self.0.get()+1);}}
  let releases=Rc::new(Cell::new(0));let probe=Probe(releases.clone());
  let node=Node::button("Fails".into(),String::new(),false,||Ok(())).unwrap();
  assert!(node.with_disabled(move||{assert_eq!(probe.0.get(),0);Err("condition failed".into())}).is_err());assert_eq!(releases.get(),1);
 }
}

#[cfg(test)]
mod semantics_tests {
    use super::*;
    fn text(value: &str) -> Node { Node::new(Kind::Text, String::new(), value.into(), vec![]).unwrap() }
    #[test]
    fn semantics_validate_tag_role_identity_and_hidden_controls() {
        for config in [r#"{"tag":"script"}"#, r#"{"role":"unknown"}"#, r#"{"role":"region"}"#,
            r#"{"role":"separator"}"#, r#"{"id":"two words"}"#, r#"{"slot":"bad slot"}"#,
            r#"{"orientation":"vertical"}"#, r#"{"role":"separator","orientation":"diagonal"}"#,
            r#"{"unknown":true}"#, r#"{"label":"Name"}"#, r#"{"tag":"p","label":"Name"}"#,
            r#"{"tag":"code","label":"Name"}"#, r#"{"role":"presentation","label":"Name"}"#] {
            assert!(text("hello").with_semantics(config).is_err(), "{config}");
        }
        assert!(text("hello").with_semantics(r#"{"tag":"h6","id":"heading","label":"Override"}"#).is_ok());
        assert!(text("").with_semantics(r#"{"role":"separator","orientation":"vertical"}"#).is_ok());
        assert!(Node::button("action".into(),String::new(),false,||Ok(())).unwrap().with_semantics(r#"{"role":"none"}"#).is_err());
        let child=Node::button("action".into(),String::new(),false,||Ok(())).unwrap();
        assert!(Node::new(Kind::View,String::new(),String::new(),vec![child]).unwrap().with_semantics(r#"{"hidden":true}"#).is_err());
    }
    #[test]
    fn newly_enabled_hidden_control_stops_refresh_and_releases_capture() {
        use std::{rc::Rc,cell::Cell};
        struct Probe(Rc<Cell<usize>>);
        impl Drop for Probe { fn drop(&mut self) { self.0.set(self.0.get()+1); } }
        let released=Rc::new(Cell::new(0)); let probe=Probe(released.clone());
        let disabled=Rc::new(Cell::new(true)); let condition=disabled.clone();
        let child=Node::button("action".into(),String::new(),true,move||{assert_eq!(probe.0.get(),0);Ok(())}).unwrap()
            .with_disabled(move||Ok(condition.get())).unwrap();
        let mut root=Node::new(Kind::View,String::new(),String::new(),vec![child]).unwrap().with_semantics(r#"{"hidden":true}"#).unwrap();
        assert_eq!(root.refresh().unwrap(),0); disabled.set(false);
        assert!(root.refresh().unwrap_err().contains("aria-hidden"));
        drop(root); assert_eq!(released.get(),1);
    }
}

#[cfg(test)]
mod composition_tests {
    use super::*;
    fn literal(value:&str)->Child {Child::Text(TextPart::Literal(value.into()))}
    #[test]
    fn mixed_children_group_adjacent_text_without_retaining_state() {
        let state=Signal::new(1);let observe=state.handle();
        let inline=Node::new(Kind::Text,String::new(),"Badge".into(),vec![]).unwrap();
        let mut root=Node::view(vec![literal(""),literal("Hello "),Child::Text(TextPart::Signal(state.handle())),Child::Node(inline),literal(" end")],String::new()).unwrap().own_state(state).unwrap();
        assert_eq!(root.children.len(),3);assert_eq!(root.content_text(),"Hello 1Badge end");
        observe.set(2).unwrap();assert_eq!(root.refresh().unwrap(),1);assert_eq!(root.content_text(),"Hello 2Badge end");
        drop(root);assert!(observe.get().is_err());
        assert!(Node::view(vec![literal("")],String::new()).unwrap().children.is_empty());
    }
    #[test]
    fn no_handler_buttons_and_inline_spans_preserve_content_and_hidden_names() {
        let decoration=Node::new(Kind::Text,String::new(),"hidden".into(),vec![]).unwrap().with_semantics(r#"{"hidden":true}"#).unwrap();
        let span=Node::view(vec![literal("Ready")],String::new()).unwrap().with_semantics(r#"{"tag":"span"}"#).unwrap();
        let mut button=Node::button_children(vec![Child::Node(span),literal(" now"),Child::Node(decoration)],String::new(),false,None).unwrap();
        assert_eq!(button.content_text(),"Ready now");assert!(!button.activate(1).unwrap());
        let html=document(&button,"Composed");assert!(html.contains("<span data-tsrust-view"));assert!(html.contains("aria-hidden=\"true\""));
        let disable=std::rc::Rc::new(std::cell::Cell::new(true));let check=disable.clone();
        let mut button=button.with_disabled(move||Ok(check.get())).unwrap();assert_eq!(button.kind,Kind::DisabledButton);
        disable.set(false);assert_eq!(button.refresh().unwrap(),1);assert_eq!(button.kind,Kind::Button);
    }
    #[test]
    fn invalid_button_children_release_both_handler_environments_once() {
        use std::{rc::Rc,cell::Cell};struct Probe(Rc<Cell<usize>>);impl Drop for Probe{fn drop(&mut self){self.0.set(self.0.get()+1);}}
        let releases=Rc::new(Cell::new(0));let child_probe=Probe(releases.clone());let parent_probe=Probe(releases.clone());
        let child=Node::button("Nested".into(),String::new(),false,move||{assert_eq!(child_probe.0.get(),0);Ok(())}).unwrap();
        assert!(Node::button_children(vec![Child::Node(child)],String::new(),false,Some(Box::new(move||{assert_eq!(parent_probe.0.get(),0);Ok(())}))).is_err());
        assert_eq!(releases.get(),2);
        for tag in ["div","p","h2"] {
            let child=Node::new(Kind::Text,String::new(),"Flow".into(),vec![]).unwrap().with_semantics(&format!(r#"{{"tag":"{tag}"}}"#)).unwrap();
            assert!(Node::button_children(vec![Child::Node(child)],String::new(),false,None).is_err());
        }
    }
}
