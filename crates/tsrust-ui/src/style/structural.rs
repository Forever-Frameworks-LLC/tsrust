//! Checked metadata predicates, shared between native matching and browser selectors.
use super::State;
#[cfg(any(test,feature="native"))]
use super::Utility;
use super::super::theme::Theme;
#[cfg(any(test,feature="native"))]
use super::super::Node;

pub(super) fn name(value:&str)->bool {!value.is_empty() && value.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-')}
fn attribute(value:&str)->Option<(&str,&str)> {
 let (key,value)=value.split_once('=')?;
 (matches!(key,"slot"|"size") && name(value)).then_some((key,value))
}
#[derive(Clone,Copy)]
pub(super) enum Predicate<'a> { Data(&'a str,&'a str), HasData(&'a str,&'a str), GroupData(&'a str,&'a str,&'a str), Class(&'a str) }
impl<'a> Predicate<'a> {
 fn parse(text:&'a str)->Option<Self> {
  if let Some(value)=text.strip_prefix("data-[").and_then(|v|v.strip_suffix(']')){let (k,v)=attribute(value)?;return Some(Self::Data(k,v));}
  if let Some(value)=text.strip_prefix("has-data-[").and_then(|v|v.strip_suffix(']')){let (k,v)=attribute(value)?;return Some(Self::HasData(k,v));}
  if let Some(value)=text.strip_prefix("group-data-[") {let (attr,group)=value.split_once("]/")?;let (k,v)=attribute(attr)?;return name(group).then_some(Self::GroupData(k,v,group));}
  text.strip_prefix("[.").and_then(|v|v.strip_suffix(']')).filter(|v|name(v)).map(Self::Class)
 }
 #[cfg(any(test,feature="native"))]
 fn matches(self,node:&Node,ancestors:&[&Node])->bool {
  fn data(node:&Node,key:&str,value:&str)->bool {(if key=="slot"{&node.semantics.slot}else{&node.semantics.size})==value}
  fn has(node:&Node,key:&str,value:&str)->bool {node.children.iter().any(|child|data(child,key,value)||has(child,key,value))}
  match self {Self::Data(k,v)=>data(node,k,v),Self::HasData(k,v)=>has(node,k,v),Self::GroupData(k,v,group)=>ancestors.iter().any(|parent|parent.class_name.split_whitespace().any(|class|class.strip_prefix("group/")==Some(group)) && data(parent,k,v)),Self::Class(class)=>node.class_name.split_whitespace().any(|token|token==class)}
 }
 fn selector(self,scope:&str)->String {
  match self {
   Self::Data(k,v)=>format!(":where([data-{k}=\"{v}\"])") ,
   Self::HasData(k,v)=>format!(":where(:has([data-{k}=\"{v}\"]))"),
   Self::GroupData(k,v,group)=>format!(":where({scope}.group\\/{group}[data-{k}=\"{v}\"] *)"),
   Self::Class(class)=>format!(":where([class~=\"{class}\"])") ,
  }
 }

}
pub(super) struct Parts<'a> {pub state:State,pub token:&'a str,predicates:Vec<Predicate<'a>>}
pub(super) fn parts(class:&str)->Result<Parts<'_>,String> {
 let mut start=0;let mut depth=0usize;let mut prefixes=Vec::new();
 for (at,ch) in class.char_indices(){match ch {'['|'('=>depth+=1,']'|')'=>{if depth==0{return Err(format!("Unbalanced UI utility: {class}"));}depth-=1;},':' if depth==0=>{prefixes.push(&class[start..at]);start=at+1;},_=>{}}}
 if depth!=0{return Err(format!("Unbalanced UI utility: {class}"));}
 let mut state=State::Base;let mut predicates=Vec::new();
 for prefix in prefixes {
  let interaction=match prefix {"hover"=>Some(State::Hover),"focus"=>Some(State::Focus),"active"=>Some(State::Active),"disabled"=>Some(State::Disabled),_=>None};
  if let Some(value)=interaction {if state!=State::Base{return Err("Compound UI interaction states are unsupported".into());}state=value;}
  else if let Some(value)=Predicate::parse(prefix){predicates.push(value);}
  else{return Err(format!("Unsupported UI state or selector: {prefix}"));}
 }
 Ok(Parts{state,token:&class[start..],predicates})
}
pub(super) fn marker(token:&str)->bool {token.strip_prefix("group/").is_some_and(name)}
#[cfg(any(test,feature="native"))]
pub(crate) fn resolved(node:&Node,ancestors:&[&Node],theme:&Theme,mode:&str)->Vec<(State,Utility)> {
 let filtered=node.class_name.split_whitespace().filter_map(|class|{
  let parsed=parts(class).expect("validated utility");
  if !parsed.predicates.iter().all(|p|p.matches(node,ancestors)){return None;}
  Some(match parsed.state {State::Base=>parsed.token.to_owned(),state=>format!("{}:{}",state_name(state),parsed.token)})
 }).collect::<Vec<_>>().join(" ");
 super::resolved(&filtered,theme,mode)
}
#[cfg(any(test,feature="native"))]
fn state_name(state:State)->&'static str {match state{State::Base=>"",State::Hover=>"hover",State::Focus=>"focus",State::Active=>"active",State::Disabled=>"disabled"}}
pub(super) fn css(classes:&str,theme:&Theme,mode:&str,id:usize,scope:&str)->String {
 let mut output=String::new();let scope=if scope.is_empty(){String::new()}else{format!("{scope} ")};
 // States override base properties, preserving the existing shared merge contract.
 for state in [State::Base,State::Hover,State::Focus,State::Active,State::Disabled] {
  for class in classes.split_whitespace(){
   let parsed=parts(class).expect("validated utility");if parsed.state!=state || marker(parsed.token){continue;}
   let declarations=super::resolved(parsed.token,theme,mode).into_iter().map(|(_,v)|v.css()).collect::<Vec<_>>().join(";");
   if declarations.is_empty(){continue;}
   let mut after=String::new();
   for predicate in parsed.predicates {after.push_str(&predicate.selector(&scope));}
   after.push_str(match state {State::Base=>"",State::Hover=>":where(:hover:not(:disabled))",State::Focus=>":where(:focus-visible)",State::Active=>":where(:active:not(:disabled))",State::Disabled=>":where(:disabled)"});
   output.push_str(&format!("{scope}.ui[data-tsrust-node=\"{id}\"]{after}{{{declarations}}}"));
  }
 }
 output
}

#[cfg(test)] mod tests {
 use super::*;
 use crate::{Kind,Child};
 fn view(classes:&str,metadata:&str,children:Vec<Node>)->Node {Node::view(children.into_iter().map(Child::Node).collect(),classes.into()).unwrap().with_semantics(metadata).unwrap()}
 #[test] fn checked_predicates_match_only_the_intended_tree_context() {
  let leaf=view("p-4 group-data-[size=sm]/panel:px-3 data-[slot=body]:pt-2 group-data-[size=sm]/tile:group-data-[size=sm]/panel:min-h-8",r#"{"slot":"body"}"#,vec![]);
  let root=view("group/panel group/tile p-4 has-data-[slot=body]:pb-0 data-[size=sm]:pt-3",r#"{"size":"sm"}"#,vec![leaf]);
  let theme=Theme::default();let rules=resolved(&root,&[],&theme,"light");
  assert!(rules.iter().any(|(_,v)|matches!(v,Utility::Padding(0,12.))));
  assert!(rules.iter().any(|(_,v)|matches!(v,Utility::Padding(2,0.))));
  let leaf=&root.children[0];let rules=resolved(leaf,&[&root],&theme,"light");
  assert!(rules.iter().any(|(_,v)|matches!(v,Utility::Dimension(3,super::super::Length::Pixels(32.)))));
  for (side,value) in [(0,8.),(1,12.),(2,16.),(3,12.)] {assert!(rules.iter().any(|(_,v)|matches!(v,Utility::Padding(s,n) if *s==side && *n==value)));}
  assert!(!Predicate::HasData("slot","body").matches(leaf,&[]));
  assert!(!Predicate::GroupData("size","sm","panel").matches(&root,&[]));
  let other=view("group/other",r#"{"size":"sm"}"#,vec![]);
  assert!(!Predicate::GroupData("size","sm","panel").matches(leaf,&[&other]));
  let rules=super::super::resolved_node(&root.children[0],&[],&theme,"light");assert!(rules.iter().any(|(_,v)|matches!(v,Utility::Padding(1,16.))));
 }
 #[test] fn selectors_keep_source_order_and_zero_predicate_specificity() {
  let style=css("p-4 data-[size=sm]:p-3 p-2 has-data-[slot=body]:pb-0 group-data-[size=sm]/panel:hover:bg-primary/50 [.border]:pt-6",&Theme::default(),"light",7,"");
  assert!(style.find("padding-left:16px").unwrap()<style.find("padding-left:12px").unwrap());
  assert!(style.find("padding-left:12px").unwrap()<style.find("padding-left:8px").unwrap());
  assert!(style.contains(":where(:has([data-slot=\"body\"]))"));
  assert!(style.contains(".ui[data-tsrust-node=\"7\"]:where(.group\\/panel[data-size=\"sm\"] *):where(:hover:not(:disabled))"));
  let scoped=css("group-data-[size=sm]/panel:p-3",&Theme::default(),"light",7,"[data-tsrust-mount=\"2\"]");
  assert!(scoped.starts_with("[data-tsrust-mount=\"2\"] .ui"));
  assert!(scoped.contains(":where([data-tsrust-mount=\"2\"] .group\\/panel[data-size=\"sm\"] *)"));
  assert!(style.contains(":where([class~=\"border\"]){padding-top:24px}"));
  for invalid in ["data-[unknown=sm]:p-3","data-[size=]:p-3","group-data-[size=sm]/:p-3","group/bad_name","has-[p]:p-3","hover:focus:p-3","[.x:hover]:p-3","data-[size=sm:p-3"]{assert!(super::super::validate(invalid).is_err(),"{invalid}");}
  assert_eq!(super::super::theme_paths("group-data-[size=sm]/panel:hover:bg-primary/50"),vec![vec!["light","primary"],vec!["dark","primary"]]);
  assert_eq!(super::super::documentation("data-[size=sm]:p-3").is_empty(),false);
 }
 #[test] fn size_metadata_is_checked_and_does_not_create_a_role() {
  assert!(view("",r#"{"size":"sm"}"#,vec![]).semantics.role.is_empty());
  assert!(Node::new(Kind::View,String::new(),String::new(),vec![]).unwrap().with_semantics(r#"{"size":"bad size"}"#).is_err());
 }
}
