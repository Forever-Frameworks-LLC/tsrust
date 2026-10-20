use super::*;
use std::{cell::Cell,rc::Rc};

struct Probe(Rc<Cell<usize>>);
impl Drop for Probe{fn drop(&mut self){self.0.set(self.0.get()+1);}}

#[test]
fn metadata_binding_preserves_identity_variants_and_releases_its_factory(){
 let value=Signal::new(false);let read=value.handle();let palette=value.handle();
 let drops=Rc::new(Cell::new(0));let probe=Probe(drops.clone());
 let mut node=Node::input_control(None,Some("Kept 🦀".into()),"{}",String::new(),None).unwrap()
 .with_semantics_binding(move||{let _=&probe;let next=read.get()?;Ok(serde_json::json!({"id":"draft","label":if next{"Renamed"}else{"Initial"},"describedBy":if next{"two"}else{"one"},"size":if next{"lg"}else{"sm"},"invalid":if next{"true"}else{"false"}}).to_string())}).unwrap()
 .with_variant_binding(move||Ok(if palette.get()?{"active"}else{"idle"}.into())).unwrap();
 node.ensure_identities();let identity=node.identity.get();value.set(true).unwrap();assert!(node.refresh().unwrap()>0);
 assert_eq!(node.identity.get(),identity);assert_eq!(node.semantics.label,"Renamed");assert_eq!(node.semantics.described_by,"two");assert_eq!(node.semantics.size,"lg");assert_eq!(node.semantics.invalid,"true");assert_eq!(node.semantics.variant,"active");assert_eq!(node.text,"Kept 🦀");
 assert_eq!(node.refresh().unwrap(),0);assert_eq!(drops.get(),0);drop(node);assert_eq!(drops.get(),1);
}

#[test]
fn invalid_metadata_updates_leave_valid_values_and_fail_without_aliasing(){
 let config=Signal::new(String::from(r#"{"label":"Valid","size":"sm"}"#));let read=config.handle();
 let mut node=Node::button("Action".into(),String::new(),false,||Ok(())).unwrap().with_semantics_binding(move||read.get()).unwrap();
 for bad in [r#"{"label":"Invalid","size":"has spaces"}"#,r#"{"role":"made-up"}"#,r#"{"id":"two ids"}"#,"not JSON"]{
  config.set(bad.into()).unwrap();assert!(node.refresh().is_err());assert_eq!(node.semantics.label,"Valid");assert_eq!(node.semantics.size,"sm");
 }
 drop(config);assert!(node.refresh().unwrap_err().contains("disposed"));
}

#[test]
fn hidden_metadata_checks_the_final_refreshed_control_state(){
 let hidden=Signal::new(false);let metadata=hidden.handle();let disabled=hidden.handle();
 let child=Node::button("Action".into(),String::new(),false,||Ok(())).unwrap().with_disabled(move||disabled.get()).unwrap();
 let mut node=Node::view(vec![Child::Node(child)],String::new()).unwrap().with_semantics_binding(move||Ok(serde_json::json!({"hidden":metadata.get()?}).to_string())).unwrap();
 hidden.set(true).unwrap();node.refresh().unwrap();assert!(node.semantics.hidden);assert!(node.children[0].is_disabled());
 hidden.set(false).unwrap();node.refresh().unwrap();assert!(!node.semantics.hidden);assert!(!node.children[0].is_disabled());
 let hidden=Signal::new(false);let metadata=hidden.handle();let child=Node::button("Action".into(),String::new(),false,||Ok(())).unwrap();
 let mut node=Node::view(vec![Child::Node(child)],String::new()).unwrap().with_semantics_binding(move||Ok(serde_json::json!({"hidden":metadata.get()?}).to_string())).unwrap();
 hidden.set(true).unwrap();assert!(node.refresh().is_err());
}

#[test]
fn metadata_callback_errors_do_not_replace_prior_values(){
 let failed=Signal::new(false);let read=failed.handle();let mut node=Node::button("Action".into(),String::new(),false,||Ok(())).unwrap().with_semantics_binding(move||if read.get()?{Err("Metadata failed".into())}else{Ok(String::from(r#"{"label":"Before"}"#))}).unwrap();
 failed.set(true).unwrap();assert_eq!(node.refresh().unwrap_err(),"Metadata failed");assert_eq!(node.semantics.label,"Before");
}
