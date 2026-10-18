//! Owned semantic nodes shared by the web and native renderers.
//! Synchronous handlers mutate explicitly owned state through weak handles.

#[cfg(feature = "native")]
mod native;
#[cfg(all(target_arch="wasm32",feature="ui-web"))]
pub mod browser;
mod state;
pub use state::{Signal, SignalHandle};
mod style;
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

pub enum TextPart { Literal(String), Signal(SignalHandle) }

pub struct Node {
    depth: usize,
    pub(crate) kind: Kind,
    pub(crate) class_name: String,
    pub(crate) text: String,
    pub(crate) children: Vec<Node>,
    pub(crate) parts: Vec<TextPart>,
    pub(crate) press: Option<Box<dyn FnMut() -> Result<(), String>>>,
    state: Vec<Signal>,
}

impl Node {
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
    pub fn bound_text(handle:SignalHandle,class_name:String)->Result<Self,String> {Self::text(vec![TextPart::Signal(handle)],class_name)}
    pub fn button(text: String, class_name: String, disabled: bool, press: impl FnMut()->Result<(),String>+'static) -> Result<Self,String> {
        let mut node=Self::new(if disabled {Kind::DisabledButton}else{Kind::Button},class_name,text,vec![])?;
        node.press=Some(Box::new(press));Ok(node)
    }
    pub(crate) fn refresh(&mut self) -> Result<usize,String> {
        let mut changed=0;
        if !self.parts.is_empty() {
            let text=Self::text_value(&self.parts)?;
            if text!=self.text {self.text=text;changed+=1;}
        }
        for child in &mut self.children {changed+=child.refresh()?;}
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
        let mut categories = std::collections::BTreeSet::new();
        for token in class_name.split_whitespace() {
            let utility = style::Utility::parse(token)
                .ok_or_else(|| format!("Unsupported UI utility: {token}"))?;
            if !categories.insert(utility.category()) {
                return Err(format!("Conflicting UI utility: {token}"));
            }
        }
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
            kind,
            class_name,
            text,
            children,
            parts: Vec::new(), press: None, state: Vec::new(),
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
    fn invalid_shape_conflicting_utilities_and_depth_reject() {
        assert!(
            Node::new(
                Kind::View,
                "flex-row flex-col".into(),
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
        assert_eq!(node.text,"Count: 2 / 5");assert!(action.add(-3));assert_eq!(node.refresh().unwrap(),1);assert_eq!(node.text,"Count: -1 / 5");
        assert_eq!(node.refresh().unwrap(),0);drop(first);assert!(node.refresh().is_err());
    }
    #[test]
    fn counter_dispatch_updates_only_affected_binding_and_releases_state() {
        fn counter()->(Node,SignalHandle) {
            let state=Signal::new(0);let observer=state.handle();let action=state.handle();let disabled=state.handle();
            let text=Node::bound_text(state.handle(),String::new()).unwrap();
            let button=Node::button("Increment".into(),String::new(),false,move ||{assert!(action.add(1));Ok(())}).unwrap();
            let disabled=Node::button("Disabled".into(),String::new(),true,move ||{assert!(disabled.add(100));Ok(())}).unwrap();
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
        drop(root);assert_eq!(a.value(),None);assert_eq!(b.value(),None);assert!(!a.add(1));
    }
}
