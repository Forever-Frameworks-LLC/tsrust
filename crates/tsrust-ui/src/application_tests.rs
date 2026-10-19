use super::*;
use std::{cell::Cell,rc::Rc};

const THEME:&str=include_str!("theme.json");
fn text(value:&str)->Node {Node::text(vec![TextPart::Literal(value.into())],String::new()).unwrap()}

#[test]
fn live_theme_preserves_nodes_fields_and_nested_scopes_and_does_not_retain_state() {
    let mode=Signal::new(String::from("light"));let change=mode.handle();
    let field=Node::input_control(None,Some("original".into()),"{}",String::new(),None).unwrap();
    let nested=text("nested").with_theme(THEME,"light").unwrap();
    let mut root=Node::view(vec![Child::Node(field),Child::Node(nested)],String::new()).unwrap()
        .with_theme_binding(THEME,mode.text_binding(Clone::clone)).unwrap().own_state(mode).unwrap();
    root.refresh().unwrap();let id=root.children[0].identity.get();root.input(id,"edited 🦀").unwrap();
    change.set("dark".into()).unwrap();assert_eq!(root.refresh().unwrap(),1);
    assert_eq!(root.children[0].identity.get(),id);assert_eq!(root.children[0].text,"edited 🦀");
    assert_eq!(root.theme.as_ref().unwrap().1,"dark");assert_eq!(root.children[1].theme.as_ref().unwrap().1,"light");
    let html=document(&root,"Theme");assert!(html.contains("edited 🦀"));
    change.set("unknown".into()).unwrap();assert!(root.refresh().is_err());assert_eq!(root.theme.as_ref().unwrap().1,"dark");
    drop(root);assert!(change.get().is_err());
}

#[test]
fn live_theme_follows_transparent_branches_and_preserves_explicit_child_scopes() {
    let mode=Signal::new(String::from("light"));let select=Signal::new(true);let test=select.handle();
    let branch=Node::branch(move||test.get(),||Ok(Some(text("first"))),||Ok(Some(text("second").with_theme(THEME,"light")?))).unwrap();
    let mut root=branch.with_theme_binding(THEME,mode.text_binding(Clone::clone)).unwrap();
    root.refresh().unwrap();mode.set("dark".into()).unwrap();root.refresh().unwrap();
    assert_eq!(root.visible_root().unwrap().theme.as_ref().unwrap().1,"dark");
    select.set(false).unwrap();root.refresh().unwrap();assert_eq!(root.visible_root().unwrap().theme.as_ref().unwrap().1,"light");
    select.set(true).unwrap();root.refresh().unwrap();assert_eq!(root.visible_root().unwrap().theme.as_ref().unwrap().1,"dark");
    drop(mode);assert!(root.refresh().is_err());
}

#[test]
fn hidden_subtrees_preserve_state_but_reject_input_and_release_once_with_the_tree() {
    struct Probe(Rc<Cell<usize>>);impl Drop for Probe{fn drop(&mut self){self.0.set(self.0.get()+1);}}
    let drops=Rc::new(Cell::new(0));let probe=Probe(drops.clone());
    let count=Signal::new(0);let press=count.handle();let read=count.handle();
    let button=Node::button_children(vec![Child::Text(TextPart::Signal(read.text_binding(i64::to_string)))],String::new(),false,Some(Box::new(move||{let _=&probe;press.set(press.get()?+1)}))).unwrap();
    let hidden=Signal::new(false);let condition=hidden.handle();
    let field=Node::input_control(None,Some("draft".into()),"{}",String::new(),None).unwrap();
    let mut root=Node::view(vec![Child::Node(button),Child::Node(field)],String::new()).unwrap().with_hidden(move||condition.get()).unwrap().own_state(count).unwrap();
    root.refresh().unwrap();let button=root.children[0].identity.get();let field=root.children[1].identity.get();
    assert!(root.activate(button).unwrap());root.input(field,"kept").unwrap();root.refresh().unwrap();
    hidden.set(true).unwrap();root.refresh().unwrap();assert!(root.visible_root().is_none());assert_eq!(root.content_text(),"");assert!(!document(&root,"Hidden").contains("<input"));
    assert!(!root.activate(button).unwrap());assert!(!root.input(field,"ignored").unwrap());assert_eq!(drops.get(),0);
    hidden.set(false).unwrap();root.refresh().unwrap();assert_eq!(root.children[0].identity.get(),button);assert_eq!(root.children[0].content_text(),"1");assert_eq!(root.children[1].text,"kept");
    drop(root);assert_eq!(drops.get(),1);
}

#[test]
fn header_drag_regions_exclude_disabled_controls_labels_and_retained_hidden_nodes() {
    let disabled=Node::button_children(vec![Child::Node(text("disabled"))],String::new(),true,None).unwrap();
    let input=Node::input_control(None,None,"{}",String::new(),None).unwrap();
    let label=Node::view(vec![Child::Node(text("label"))],String::new()).unwrap().with_semantics(r#"{"tag":"label"}"#).unwrap();
    let hidden=Node::view(vec![Child::Node(text("hidden"))],String::new()).unwrap().with_hidden(||Ok(true)).unwrap();
    let root=Node::view(vec![Child::Node(text("drag")),Child::Node(disabled),Child::Node(input),Child::Node(label),Child::Node(hidden)],String::new()).unwrap().with_semantics(r#"{"windowDragRegion":true}"#).unwrap();
    root.ensure_identities();assert!(root.window_drag_at(|id|id==root.identity.get()));assert!(root.window_drag_at(|id|id==root.children[0].identity.get()));
    for child in &root.children[1..] {assert!(!root.window_drag_at(|id|id==child.identity.get() || child.children.iter().any(|child|child.identity.get()==id)));}
    assert!(text("invalid").with_semantics(r#"{"windowDragRegion":true}"#).is_err());
}
