use super::*;

#[test]
fn table_metadata_is_checked_and_serialized_without_dom_coercion() {
    let cell=Node::view(vec![],String::new()).unwrap().with_semantics(r#"{"tag":"th","colSpan":3,"scope":"colgroup","state":"selected"}"#).unwrap();
    let html=document(&cell,"Header");
    assert!(html.contains("colspan=\"3\"") && html.contains("scope=\"colgroup\"") && html.contains("data-state=\"selected\""));
    for config in [r#"{"tag":"td","colSpan":0}"#,r#"{"tag":"td","colSpan":1001}"#,r#"{"tag":"td","colSpan":1.5}"#,r#"{"tag":"div","colSpan":1}"#,r#"{"tag":"td","scope":"row"}"#,r#"{"tag":"th","scope":"wrong"}"#,r#"{"state":"selected item"}"#] {
        assert!(Node::view(vec![],String::new()).unwrap().with_semantics(config).is_err(),"{config}");
    }
    for tag in ["tfoot","caption"] {
        let node=Node::view(vec![],String::new()).unwrap().with_semantics(&format!(r#"{{"tag":"{tag}"}}"#)).unwrap();
        assert!(document(&node,"Parts").contains(&format!("<{tag}")));
    }
}

#[test]
fn failed_live_table_metadata_preserves_the_prior_valid_snapshot() {
    let mut node=Node::view(vec![],String::new()).unwrap().with_semantics(r#"{"tag":"td","colSpan":2}"#).unwrap();
    assert!(node.apply_semantics(r#"{"tag":"td","colSpan":1001}"#,true).is_err());
    assert_eq!(node.semantics.col_span,Some(2));
    assert!(node.apply_semantics(r#"{"tag":"td","state":"selected"}"#,true).unwrap());
    assert_eq!(node.semantics.col_span,None);
    assert_eq!(node.semantics.state,"selected");
}

#[test]
fn table_utilities_and_metadata_predicates_keep_paired_css_and_native_rules(){
    use style::{Utility,State};
    let row=Node::view(vec![],"table-row data-[state=selected]:bg-muted border-b".into()).unwrap().with_semantics(r#"{"tag":"tr","state":"selected"}"#).unwrap();
    let body=Node::view(vec![Child::Node(row)],"table-row-group [&_tr:last-child]:border-0".into()).unwrap().with_semantics(r#"{"tag":"tbody"}"#).unwrap();
    let rules=style::resolved_node(&body.children[0],&[&body],&theme::Theme::default(),"light");
    assert!(rules.iter().any(|(state,rule)|*state==State::Base&&matches!(rule,Utility::Background(_))));
    assert!(rules.iter().any(|(_,rule)|matches!(rule,Utility::EdgeBorder(2,0.))));
    let html=document(&body,"Rows");assert!(html.contains("tr:last-child"));assert!(html.contains("data-state=\"selected\""));
    for class in ["table","table-header-group","table-row-group","table-footer-group","table-row","table-cell","table-caption","caption-bottom","align-middle","w-[100px]","w-[0]","max-w-[100.5px]"]{assert!(style::validate(class).is_ok(),"{class}");assert!(!style::documentation(class).is_empty(),"{class}");}
    for class in ["w-[-1px]","w-[NaNpx]","w-[1/2px]","table-cell w-full"]{assert!(style::validate(class).is_err(),"{class}");}
}

#[test]
fn nested_portable_tables_fail_before_either_adapter_can_disagree(){
    let inner=Node::view(vec![],"table".into()).unwrap().with_semantics(r#"{"tag":"table"}"#).unwrap();
    let cell=Node::view(vec![Child::Node(inner)],"table-cell".into()).unwrap().with_semantics(r#"{"tag":"td"}"#).unwrap();
    let row=Node::view(vec![Child::Node(cell)],"table-row".into()).unwrap().with_semantics(r#"{"tag":"tr"}"#).unwrap();
    let result=Node::view(vec![Child::Node(row)],"table".into()).unwrap().with_semantics(r#"{"tag":"table"}"#);
    assert!(result.err().unwrap().contains("Nested portable tables"));
}

#[test]
fn bound_table_classes_validate_structure_and_release_rejected_captures(){
    use std::{cell::Cell,rc::Rc};
    struct Probe(Rc<Cell<usize>>);
    impl Drop for Probe{fn drop(&mut self){self.0.set(self.0.get()+1);}}
    let drops=Rc::new(Cell::new(0));let capture=Probe(drops.clone());
    let inner=Node::view(vec![],"table".into()).unwrap().with_semantics(r#"{"tag":"table"}"#).unwrap();
    let outer=Node::view(vec![Child::Node(inner)],String::new()).unwrap().with_semantics(r#"{"tag":"table"}"#).unwrap();
    let result=outer.with_class_binding(move||{let _=&capture;Ok("table".into())});
    assert!(result.err().unwrap().contains("Nested portable tables"));assert_eq!(drops.get(),1);
    let child=Node::view(vec![],String::new()).unwrap();
    let row=Node::view(vec![Child::Node(child)],String::new()).unwrap().with_semantics(r#"{"tag":"tr"}"#).unwrap();
    assert!(row.with_class_binding(||Ok("table-row".into())).err().unwrap().contains("table-cell children"));
}

#[test]
fn malformed_portable_rows_reject_at_shared_construction(){
    let child=Node::view(vec![],String::new()).unwrap();
    let row=Node::view(vec![Child::Node(child)],"table-row".into()).unwrap().with_semantics(r#"{"tag":"tr"}"#);
    assert!(row.err().unwrap().contains("table-cell children"));
    let child=Node::view(vec![],String::new()).unwrap();
    let group=Node::view(vec![Child::Node(child)],"table-row-group".into()).unwrap().with_semantics(r#"{"tag":"tbody"}"#);
    assert!(group.err().unwrap().contains("table-row children"));
}
