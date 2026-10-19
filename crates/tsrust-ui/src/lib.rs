//! Owned semantic nodes shared by the web and native renderers.
//! Synchronous handlers mutate explicitly owned state through weak handles.

#[cfg(feature = "native")]
mod native;
#[cfg(all(target_arch="wasm32",feature="ui-web"))]
pub mod browser;
mod state;
mod branch;
mod editable;
pub(crate) use editable::{ValueBinding,ChangeHandler};
pub use state::{Signal,TextBinding};
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
    Input,
    Textarea,
}

pub enum TextPart { Literal(String), Signal(TextBinding) }
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
    pub size: String,
    pub orientation: String,
    pub html_for:String,
    pub described_by:String,
    pub invalid:String,
}

pub struct Node {
    depth: usize,
    identity:std::cell::Cell<u64>,
    next_identity:std::cell::Cell<u64>,
    anonymous_text:bool,
    pub(crate) theme: Option<(theme::Theme,String)>,
    pub(crate) semantics: Semantics,
    pub(crate) kind: Kind,
    pub(crate) class_name: String,
    pub(crate) text: String,
    pub(crate) children: Vec<Node>,
    pub(crate) parts: Vec<TextPart>,
    pub(crate) disabled: Option<Box<dyn FnMut()->Result<bool,String>>>,
    pub(crate) press: Option<Box<dyn FnMut() -> Result<(), String>>>,
    pub(crate) editable: Option<editable::Editable>,
    read_only: Option<Box<dyn FnMut()->Result<bool,String>>>,
    branch:Option<branch::Branch>,
    state: Vec<Box<dyn std::any::Any>>,
}

impl Node {
    pub fn with_semantics(mut self, config: &str) -> Result<Self,String> {
        if config.is_empty(){return Ok(self);}
        let value:Semantics=serde_json::from_str(config).map_err(|e|format!("Invalid UI semantics: {e}"))?;
        let tag=if value.tag.is_empty(){self.tag()}else{&value.tag};
        let allowed=match self.kind {
            Kind::View=>matches!(tag,"div"|"section"|"span"|"label"),
            Kind::Text=>matches!(tag,"div"|"span"|"p"|"h1"|"h2"|"h3"|"h4"|"h5"|"h6"|"code"|"kbd"),
            Kind::Button|Kind::DisabledButton=>tag=="button",
            Kind::Input=>tag=="input",
            Kind::Textarea=>tag=="textarea",
        };
        if !allowed{return Err(format!("Unsupported UI element for {:?}: {tag}",self.kind));}
        if !matches!(value.role.as_str(),""|"alert"|"status"|"separator"|"group"|"note"|"region"|"none"|"presentation") {return Err(format!("Unsupported UI role: {}",value.role));}
        if matches!(self.kind,Kind::Button|Kind::DisabledButton|Kind::Input|Kind::Textarea) && !value.role.is_empty(){return Err("Interactive controls keep their implicit role".into());}
        if !value.orientation.is_empty() && (value.role!="separator" || !matches!(value.orientation.as_str(),"horizontal"|"vertical")){return Err("aria-orientation requires a separator and horizontal or vertical".into());}
        let name_allowed=if value.role.is_empty(){matches!(tag,"button"|"input"|"textarea"|"h1"|"h2"|"h3"|"h4"|"h5"|"h6"|"section")}else{!matches!(value.role.as_str(),"none"|"presentation")};
        if !value.label.is_empty() && !name_allowed{return Err("aria-label requires a button, heading, section or a role that supports naming".into());}
        if value.role=="region" && value.label.trim().is_empty(){return Err("A region requires an accessible label".into());}
        if !value.html_for.is_empty() && tag!="label"{return Err("htmlFor requires a label element".into());}
        if value.html_for.chars().any(|ch|ch.is_ascii_whitespace()||ch.is_control()){return Err("htmlFor must name a single document ID".into());}
        if value.described_by.chars().any(|ch|ch.is_control() && !ch.is_ascii_whitespace()){return Err("aria-describedby contains an invalid ID".into());}
        if !matches!(value.invalid.as_str(),""|"true"|"false"){return Err("aria-invalid supports boolean values".into());}
        if value.id.chars().any(|ch|ch.is_ascii_whitespace()||ch.is_control()){return Err("UI id cannot contain whitespace or control characters".into());}
        if value.slot.chars().any(|ch|!(ch.is_ascii_alphanumeric()||ch=='-')){return Err("UI slot must contain ASCII letters, digits or hyphens".into());}
        if value.size.chars().any(|ch|!(ch.is_ascii_alphanumeric()||ch=='-')){return Err("UI data-size must contain ASCII letters, digits or hyphens".into());}
        self.semantics=value;self.validate_structure()?;Ok(self)
    }
    fn has_enabled_controls(&self)->bool {self.enabled_control() || self.children.iter().any(Self::has_enabled_controls)}
    pub(crate) fn enabled_control(&self)->bool {matches!(self.kind,Kind::Button|Kind::Input|Kind::Textarea) && !self.is_disabled()}
    pub(crate) fn is_disabled(&self)->bool {self.kind==Kind::DisabledButton || self.editable.as_ref().is_some_and(|field|field.disabled)}
    fn apply_disabled(&mut self,next:bool)->bool {
        if let Some(field)=&mut self.editable {let changed=field.disabled!=next;field.disabled=next;changed}
        else {let kind=if next{Kind::DisabledButton}else{Kind::Button};let changed=self.kind!=kind;self.kind=kind;changed}
    }
    pub(crate) fn tag(&self)->&str {
        if !self.semantics.tag.is_empty(){&self.semantics.tag}else{match self.kind{Kind::View=>"div",Kind::Text=>"span",Kind::Button|Kind::DisabledButton=>"button",Kind::Input=>"input",Kind::Textarea=>"textarea"}}
    }
    pub fn view(children: Vec<Child>, class_name: String) -> Result<Self,String> {
        Self::new(Kind::View,class_name,String::new(),Self::child_nodes(children)?)
    }
    fn child_nodes(children: Vec<Child>) -> Result<Vec<Node>,String> {
        fn flush(parts: &mut Vec<TextPart>, nodes: &mut Vec<Node>) -> Result<(),String> {
            if parts.iter().any(|part|match part{TextPart::Literal(text)=>!text.is_empty(),TextPart::Signal(_)=>true}) {
                nodes.push(Node::text_run(std::mem::take(parts))?);
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
        let children=Self::child_nodes(children)?;
        if !children.iter().all(Self::phrasing){return Err("Button children must be noninteractive phrasing content: text, span, code or kbd".into());}
        let mut node=Self::new(Kind::View,class_name,String::new(),children)?;
        node.kind=if disabled{Kind::DisabledButton}else{Kind::Button};node.press=press;Ok(node)
    }
    #[cfg(any(test,feature="native"))]
    pub(crate) fn content_text(&self)->String {
        fn append(node:&Node,out:&mut String) {
            if node.semantics.hidden || node.editable.is_some(){return;}
            out.push_str(&node.text);for child in &node.children{append(child,out);}
        }
        let mut output=String::new();append(self,&mut output);output
    }
    pub fn with_disabled(mut self,mut predicate:impl FnMut()->Result<bool,String>+'static)->Result<Self,String> {
        if !matches!(self.kind,Kind::Button|Kind::DisabledButton|Kind::Input|Kind::Textarea) {return Err("A disabled condition requires Button, Input or Textarea".into());}
        self.apply_disabled(predicate()?);self.disabled=Some(Box::new(predicate));Ok(self)
    }

    pub fn with_read_only(mut self,mut predicate:impl FnMut()->Result<bool,String>+'static)->Result<Self,String> {
        let field=self.editable.as_mut().ok_or("A readOnly condition requires Input or Textarea")?;
        field.read_only=predicate()?;self.read_only=Some(Box::new(predicate));Ok(self)
    }
    pub fn input_control(binding:Option<ValueBinding>,default_value:Option<String>,options:&str,class_name:String,changed:Option<ChangeHandler>)->Result<Self,String> {
        let options:editable::Options=serde_json::from_str(options).map_err(|e|format!("Invalid input options: {e}"))?;
        if !matches!(options.input_type.as_str(),""|"text"){return Err("This input backend supports type=text; other input types are not implemented".into());}
        let field=editable::Editable {
            value:editable::normalize(default_value.as_deref().unwrap_or(""),options.multiline),
            placeholder:options.placeholder,disabled:options.disabled,read_only:options.read_only,multiline:options.multiline,
            required:options.required,name:options.name,auto_complete:options.auto_complete,binding,changed,
        };
        Self::editable(field,class_name)
    }

    pub fn with_theme(mut self, json: &str, mode: &str) -> Result<Self,String> {
        let theme=theme::Theme::parse(json)?;theme.color(mode,"background")?;
        self.theme=Some((theme,mode.to_owned()));self.inherit_branch_theme();Ok(self)
    }
    pub fn own_state<T: Clone + 'static>(mut self, state: Signal<T>) -> Result<Self,String> { self.state.push(Box::new(state)); Ok(self) }
    pub fn text(parts:Vec<TextPart>,class_name:String)->Result<Self,String> {
        let text=Self::text_value(&parts)?;
        let mut node=Self::new(Kind::Text,class_name,text,vec![])?;
        if parts.iter().any(|part|matches!(part,TextPart::Signal(_))) {node.parts=parts;}
        Ok(node)
    }
    #[doc(hidden)]
    pub fn text_run(parts:Vec<TextPart>)->Result<Self,String> {
        let mut node=Self::text(parts,String::new())?;node.anonymous_text=true;Ok(node)
    }
    fn text_value(parts:&[TextPart])->Result<String,String> {
        let mut text=String::new();
        for part in parts {match part {
            TextPart::Literal(value)=>text.push_str(value),
            TextPart::Signal(handle)=>text.push_str(&handle.value()?),
        }}
        Ok(text)
    }
    pub fn bound_text(handle:Signal,class_name:String)->Result<Self,String> {Self::text(vec![TextPart::Signal(handle.text_binding(i64::to_string))],class_name)}
    pub fn button(text: String, class_name: String, disabled: bool, press: impl FnMut()->Result<(),String>+'static) -> Result<Self,String> {
        let mut node=Self::new(if disabled {Kind::DisabledButton}else{Kind::Button},class_name,text,vec![])?;
        node.press=Some(Box::new(press));Ok(node)
    }
    #[cfg(any(test,feature="native",all(target_arch="wasm32",feature="ui-web")))]
    pub(crate) fn refresh(&mut self) -> Result<usize,String> {
        let changed=self.refresh_inner()?;self.ensure_identities();Ok(changed)
    }
    #[cfg(any(test,feature="native",all(target_arch="wasm32",feature="ui-web")))]
    fn refresh_inner(&mut self) -> Result<usize,String> {
        let mut changed=self.refresh_branch()?;
        let disabled=if let Some(predicate)=&mut self.disabled {Some(predicate()?)}else{None};
        if let Some(value)=disabled {changed+=usize::from(self.apply_disabled(value));}
        let read_only=if let Some(predicate)=&mut self.read_only {Some(predicate()?)}else{None};
        if let Some(field)=&mut self.editable {
            if let Some(value)=read_only {if value!=field.read_only {field.read_only=value;changed+=1;}}
            let value_changed=field.refresh()?;
            let text_changed=self.text!=field.value;
            if text_changed {self.text=field.value.clone();}
            changed+=usize::from(value_changed||text_changed);
        }

        if !self.parts.is_empty() {
            let text=Self::text_value(&self.parts)?;
            if text!=self.text {self.text=text;changed+=1;}
        }
        for child in &mut self.children {changed+=child.refresh_inner()?;}
        self.depth=usize::from(self.branch.is_none())+self.children.iter().map(|child|child.depth).max().unwrap_or(0);
        if self.depth>128{return Err("UI nesting exceeds 128 nodes".into());}
        self.validate_structure()?;
        Ok(changed)
    }
    /// IDs initially follow preorder; removed IDs are never reused within this tree.
    #[cfg(any(test,feature="native",all(target_arch="wasm32",feature="ui-web")))]
    pub(crate) fn activate(&mut self, target: u64) -> Result<bool,String> {
        self.ensure_identities();
        fn visit(node:&mut Node,target:u64)->Result<bool,String> {
            if node.identity.get()==target {
                if node.kind!=Kind::Button {return Ok(false);}
                if let Some(handler)=&mut node.press {handler()?;return Ok(true);}
                return Ok(false);
            }
            for child in &mut node.children {if visit(child,target)? {return Ok(true);}}
            Ok(false)
        }
        visit(self,target)
    }

    pub(crate) fn editable(mut field:editable::Editable,class_name:String)->Result<Self,String> {
        field.refresh()?;
        let mut node=Self::new(Kind::View,class_name,String::new(),vec![])?;
        node.kind=if field.multiline{Kind::Textarea}else{Kind::Input};node.text=field.value.clone();node.editable=Some(field);Ok(node)
    }
    #[cfg(any(test,feature="native",all(target_arch="wasm32",feature="ui-web")))]
    pub(crate) fn input(&mut self,target:u64,value:&str)->Result<bool,String> {
        self.ensure_identities();
        fn visit(node:&mut Node,target:u64,value:&str)->Result<bool,String>{
            if node.identity.get()==target {
                let Some(field)=&mut node.editable else{return Ok(false);};
                let changed=field.input(value,&node.semantics.id)?;node.text=field.value.clone();return Ok(changed);
            }
            for child in &mut node.children {if visit(child,target,value)?{return Ok(true);}}
            Ok(false)
        }
        visit(self,target,value)
    }

    #[cfg(feature="native")]
    fn control_value(&self,target:u64)->Option<&str> {
        fn find(node:&Node,target:u64)->Option<&str>{if node.identity.get()==target{return node.editable.as_ref().map(|field|field.value.as_str());}node.children.iter().find_map(|child|find(child,target))}
        find(self,target)
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
        if (kind==Kind::Text || (matches!(kind,Kind::Button|Kind::DisabledButton) && !text.is_empty())) && style::requires_container(&class_name) {
            return Err("Container layout utilities require View or composed Button children; Text remains a text leaf".into());
        }
        match kind {
            Kind::Input|Kind::Textarea=>return Err("Construct editable controls with input_control".into()),
            Kind::View if !text.is_empty() => return Err("View text must be inside Text".into()),
            Kind::Text | Kind::DisabledButton | Kind::Button if !children.is_empty() => {
                return Err("Text and Button accept text children only".into());
            }
            _ => {}
        }
        let class_name = class_name.split_whitespace().collect::<Vec<_>>().join(" ");
        for child in &children{child.reset_identities();}
        Ok(Self {
            depth,
            identity:std::cell::Cell::new(0),next_identity:std::cell::Cell::new(0),anonymous_text:false,branch:None,
            theme: None,
            semantics: Semantics::default(),
            kind,
            class_name,
            text,
            children,
            parts: Vec::new(), disabled:None, press: None, state: Vec::new(), editable:None, read_only:None,
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
        let mut node=Node::text(vec![TextPart::Literal("Count: ".into()),TextPart::Signal(first.text_binding(i64::to_string)),TextPart::Literal(" / ".into()),TextPart::Signal(second.text_binding(i64::to_string))],String::new()).unwrap();
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
        let mut root=Node::view(vec![literal(""),literal("Hello "),Child::Text(TextPart::Signal(state.text_binding(i64::to_string))),Child::Node(inline),literal(" end")],String::new()).unwrap().own_state(state).unwrap();
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

#[cfg(test)]
mod scalar_binding_tests {
 use super::*;
 #[test]
 fn typed_text_callbacks_and_owners_share_updates_without_retaining_disposed_state() {
  let label=Signal::new(String::from("a"));let ready=Signal::new(false);let amount=Signal::new(1.25f64);
  let observed=(label.handle(),ready.handle(),amount.handle());
  let parts=vec![TextPart::Signal(label.text_binding(Clone::clone)),TextPart::Literal(" / ".into()),TextPart::Signal(ready.text_binding(bool::to_string)),TextPart::Literal(" / ".into()),TextPart::Signal(amount.text_binding(f64::to_string))];
  let text=Node::text(parts,String::new()).unwrap();
  let action=(label.handle(),ready.handle(),amount.handle());
  let button=Node::button("Change".into(),String::new(),false,move||{action.0.set("b".into())?;action.1.set(true)?;action.2.set(1.75)?;Ok(())}).unwrap();
  let mut root=Node::view(vec![Child::Node(text),Child::Node(button)],String::new()).unwrap().own_state(label).unwrap().own_state(ready).unwrap().own_state(amount).unwrap();
  assert_eq!(root.children[0].text,"a / false / 1.25");assert!(root.activate(3).unwrap());assert_eq!(root.refresh().unwrap(),1);assert_eq!(root.children[0].text,"b / true / 1.75");
  drop(root);assert!(observed.0.get().is_err());assert!(observed.1.get().is_err());assert!(observed.2.get().is_err());
 }
}

#[cfg(test)]
mod editable_semantics_tests {
 use super::*;
 #[test]
 fn controls_escape_values_normalize_lines_and_reject_false_semantics() {
  let input=Node::input_control(None,Some("<&\"'\r\n🙂".into()),"{}",String::new(),None).unwrap().with_semantics(r#"{"id":"name","describedBy":"hint","invalid":"false"}"#).unwrap();let html=document(&input,"Input");assert!(html.contains("value=\"&lt;&amp;&quot;&#39;🙂\""));assert!(!html.contains("</input>"));assert!(html.contains("aria-invalid=\"false\""));
  let mut area=Node::input_control(None,Some("a\r\nb\rc".into()),r#"{"multiline":true}"#,String::new(),None).unwrap();assert_eq!(area.text,"a\nb\nc");area.input(1,"x\r\ny").unwrap();assert_eq!(area.text,"x\ny");
  assert!(Node::input_control(None,None,r#"{"type":"password"}"#,String::new(),None).is_err());
  assert!(Node::new(Kind::Input,String::new(),String::new(),vec![]).is_err());
  assert!(Node::new(Kind::Text,String::new(),"label".into(),vec![]).unwrap().with_semantics(r#"{"htmlFor":"name"}"#).is_err());
  assert!(Node::new(Kind::View,String::new(),String::new(),vec![input]).unwrap().with_semantics(r#"{"hidden":true}"#).is_err());
 }
 #[test]
 fn disabled_read_only_and_unchanged_values_do_not_invoke_callbacks() {
  use std::{cell::Cell,rc::Rc};let calls=Rc::new(Cell::new(0));let observed=calls.clone();let mut node=Node::input_control(None,None,"{}",String::new(),Some(Box::new(move|_,_,_|{calls.set(calls.get()+1);Ok(())}))).unwrap();assert!(node.input(1,"new").unwrap());assert_eq!(observed.get(),1);assert!(!node.input(1,"new").unwrap());node.editable.as_mut().unwrap().read_only=true;assert!(!node.input(1,"other").unwrap());assert_eq!(node.text,"new");node.editable.as_mut().unwrap().read_only=false;node.apply_disabled(true);assert!(!node.input(1,"other").unwrap());assert_eq!(observed.get(),1);
 }
}

#[cfg(test)]
mod editable_cleanup_tests {
 use super::*;
 use std::{cell::RefCell,rc::Rc};
 struct Probe(&'static str,Rc<RefCell<Vec<&'static str>>>);
 impl Probe {fn touch(&self){}}
 impl Drop for Probe {fn drop(&mut self){self.1.borrow_mut().push(self.0);}}
 #[test]
 fn failed_edit_releases_independent_callback_environments_before_state_owner() {
  let released=Rc::new(RefCell::new(Vec::new()));let probe=|name|Probe(name,released.clone());let read=probe("binding");let write=probe("handler");let disabled=probe("disabled");let read_only=probe("read-only");
  let mut node=Node::input_control(Some(Box::new(move||{read.touch();Ok("initial".into())})),None,"{}",String::new(),Some(Box::new(move|_,_,_|{write.touch();Err("application failure".into())}))).unwrap().with_disabled(move||{disabled.touch();Ok(false)}).unwrap().with_read_only(move||{read_only.touch();Ok(false)}).unwrap();node.state.push(Box::new(probe("owner")));
  assert_eq!(node.input(1,"next"),Err("application failure".into()));assert!(released.borrow().is_empty());drop(node);assert_eq!(*released.borrow(),vec!["disabled","binding","handler","read-only","owner"]);
 }
 #[test]
 fn failed_initial_getter_drops_both_environments_once() {
  let released=Rc::new(RefCell::new(Vec::new()));let read=Probe("binding",released.clone());let write=Probe("handler",released.clone());let result=Node::input_control(Some(Box::new(move||{read.touch();Err("getter failure".into())})),None,"{}",String::new(),Some(Box::new(move|_,_,_|{write.touch();Ok(())})));assert!(result.is_err());assert_eq!(*released.borrow(),vec!["binding","handler"]);
 }
}
#[cfg(test)]
mod editable_event_tests {
 use super::*;
 #[test]
 fn committed_changes_receive_managed_name_and_id_metadata() {
  use std::{cell::RefCell,rc::Rc};let events=Rc::new(RefCell::new(Vec::new()));let observed=events.clone();
  let mut node=Node::input_control(None,None,r#"{"name":"profile"}"#,String::new(),Some(Box::new(move|value,name,id|{events.borrow_mut().push((value.to_owned(),name.to_owned(),id.to_owned()));Ok(())}))).unwrap().with_semantics(r#"{"id":"name"}"#).unwrap();node.input(1,"Ada🙂").unwrap();assert_eq!(*observed.borrow(),vec![("Ada🙂".into(),"profile".into(),"name".into())]);assert!(!node.input(1,"Ada🙂").unwrap());assert_eq!(observed.borrow().len(),1);
 }
}
