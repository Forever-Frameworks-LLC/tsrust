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
 (matches!(key,"slot"|"size"|"orientation") && name(value)).then_some((key,value))
}
#[derive(Clone,Copy)]
pub(super) enum Predicate<'a> { Data(&'a str,&'a str), HasData(&'a str,&'a str), InData(&'a str,&'a str), GroupData(&'a str,&'a str,&'a str), Class(&'a str), Dark, MinWidth(u16), Invalid, Current }
impl<'a> Predicate<'a> {
 fn parse(text:&'a str)->Option<Self> {
  match text {"dark"=>return Some(Self::Dark),"sm"=>return Some(Self::MinWidth(640)),"md"=>return Some(Self::MinWidth(768)),"lg"=>return Some(Self::MinWidth(1024)),"xl"=>return Some(Self::MinWidth(1280)),"2xl"=>return Some(Self::MinWidth(1536)),"aria-invalid"=>return Some(Self::Invalid),_=>{}}
  if text=="data-horizontal"{return Some(Self::Data("orientation","horizontal"));}
  if text=="aria-current"{return Some(Self::Current);}
  if text=="data-vertical"{return Some(Self::Data("orientation","vertical"));}

  if let Some(value)=text.strip_prefix("data-[").and_then(|v|v.strip_suffix(']')){let (k,v)=attribute(value)?;return Some(Self::Data(k,v));}
  if let Some(value)=text.strip_prefix("has-data-[").and_then(|v|v.strip_suffix(']')){let (k,v)=attribute(value)?;return Some(Self::HasData(k,v));}
  if let Some(value)=text.strip_prefix("in-data-[").and_then(|v|v.strip_suffix(']')){let (k,v)=attribute(value)?;return Some(Self::InData(k,v));}
  if let Some(value)=text.strip_prefix("group-data-[") {let (attr,group)=value.split_once("]/")?;let (k,v)=attribute(attr)?;return name(group).then_some(Self::GroupData(k,v,group));}
  text.strip_prefix("[.").and_then(|v|v.strip_suffix(']')).filter(|v|name(v)).map(Self::Class)
 }
 #[cfg(any(test,feature="native"))]
 fn matches(self,node:&Node,ancestors:&[&Node])->bool {
  fn data(node:&Node,key:&str,value:&str)->bool {match key {"slot"=>node.semantics.slot==value,"size"=>node.semantics.size==value,"orientation"=>node.semantics.role=="separator" && (if node.semantics.orientation.is_empty(){"horizontal"}else{&node.semantics.orientation})==value,_=>false}}
  fn has(node:&Node,key:&str,value:&str)->bool {node.children.iter().any(|child|data(child,key,value)||has(child,key,value))}
  match self {Self::Data(k,v)=>data(node,k,v),Self::HasData(k,v)=>has(node,k,v),Self::InData(k,v)=>ancestors.iter().any(|parent|data(parent,k,v)),Self::GroupData(k,v,group)=>ancestors.iter().any(|parent|parent.class_name.split_whitespace().any(|class|class.strip_prefix("group/")==Some(group)) && data(parent,k,v)),Self::Class(class)=>node.class_name.split_whitespace().any(|token|token==class),Self::Dark|Self::MinWidth(_)=>true,Self::Invalid=>node.semantics.invalid=="true",Self::Current=>super::super::navigation::is_current(&node.semantics.href)}
 }
 fn selector(self,scope:&str)->String {
  match self {
   Self::Dark|Self::MinWidth(_)=>String::new(),Self::Invalid=>":where([aria-invalid=\"true\"])".into(),Self::Current=>":where([aria-current=\"page\"],[aria-current=\"location\"])".into(),
   Self::Data(k,v)=>format!(":where([data-{k}=\"{v}\"])") ,
   Self::HasData(k,v)=>format!(":where(:has([data-{k}=\"{v}\"]))"),
   Self::InData(k,v)=>format!(":where({scope}[data-{k}=\"{v}\"] *)"),
   Self::GroupData(k,v,group)=>format!(":where({scope}.group\\/{group}[data-{k}=\"{v}\"] *)"),
   Self::Class(class)=>format!(":where([class~=\"{class}\"])") ,
  }
 }

}
pub(super) struct Parts<'a> {pub state:State,pub token:&'a str,pub placeholder:bool,predicates:Vec<Predicate<'a>>}
pub(super) fn parts(class:&str)->Result<Parts<'_>,String> {
 let mut start=0;let mut depth=0usize;let mut prefixes=Vec::new();
 for (at,ch) in class.char_indices(){match ch {'['|'('=>depth+=1,']'|')'=>{if depth==0{return Err(format!("Unbalanced UI utility: {class}"));}depth-=1;},':' if depth==0=>{prefixes.push(&class[start..at]);start=at+1;},_=>{}}}
 if depth!=0{return Err(format!("Unbalanced UI utility: {class}"));}
 let mut state=State::Base;let mut predicates=Vec::new();let mut placeholder=false;
 for prefix in prefixes {
  let interaction=match prefix {"hover"=>Some(State::Hover),"focus"|"focus-visible"=>Some(State::Focus),"active"=>Some(State::Active),"disabled"=>Some(State::Disabled),_=>None};
  if prefix=="placeholder" {if placeholder{return Err("Duplicate placeholder variant".into());}placeholder=true;}
  else if let Some(value)=interaction {if state!=State::Base{return Err("Compound UI interaction states are unsupported".into());}state=value;}
  else if let Some(value)=Predicate::parse(prefix){predicates.push(value);}
  else{return Err(format!("Unsupported UI state or selector: {prefix}"));}
 }
 Ok(Parts{state,token:&class[start..],placeholder,predicates})
}
pub(super) fn marker(token:&str)->bool {token.strip_prefix("group/").is_some_and(name)}
#[cfg(test)]
pub(crate) fn resolved(node:&Node,ancestors:&[&Node],theme:&Theme,mode:&str)->Vec<(State,Utility)> {
 scoped(node,ancestors,theme,mode,&std::collections::BTreeMap::new()).0
}
#[cfg(any(test,feature="native"))]
pub(crate) fn scoped(node:&Node,ancestors:&[&Node],theme:&Theme,mode:&str,inherited:&std::collections::BTreeMap<String,f32>)->(Vec<(State,Utility)>,std::collections::BTreeMap<String,f32>) {
 scoped_at_width(node,ancestors,theme,mode,inherited,0.)
}
#[cfg(any(test,feature="native"))]
pub(crate) fn scoped_at_width(node:&Node,ancestors:&[&Node],theme:&Theme,mode:&str,inherited:&std::collections::BTreeMap<String,f32>,width:f32)->(Vec<(State,Utility)>,std::collections::BTreeMap<String,f32>) {
 let mut variables=inherited.clone();let mut filtered=Vec::new();
 for class in node.class_name.split_whitespace(){
  let parsed=parts(class).expect("validated utility");
  if !parsed.predicates.iter().all(|p|match p {Predicate::Dark=>mode=="dark",Predicate::MinWidth(min)=>width>=*min as f32,_=>p.matches(node,ancestors)}){continue;}
  if let Some((name,scale))=super::custom_spacing::declaration(parsed.token){variables.insert(name.into(),super::layout::spacing(scale,theme));}
  let token=if parsed.placeholder{format!("placeholder:{}",parsed.token)}else{parsed.token.to_owned()};
  filtered.push(match parsed.state {State::Base=>token,state=>format!("{}:{token}",state_name(state))});
 }
 (super::resolved_with_variables(&filtered.join(" "),theme,mode,&variables),variables)
}
#[cfg(any(test,feature="native"))]
fn state_name(state:State)->&'static str {match state{State::Base=>"",State::Hover=>"hover",State::Focus=>"focus",State::Active=>"active",State::Disabled=>"disabled"}}
pub(super) fn css(classes:&str,theme:&Theme,mode:&str,id:u64,scope:&str)->String {
 let mut output=String::new();let scope=if scope.is_empty(){String::new()}else{format!("{scope} ")};
 // States override base properties, preserving the existing shared merge contract.
 for state in [State::Base,State::Hover,State::Focus,State::Active,State::Disabled] {
  for class in classes.split_whitespace(){
   let parsed=parts(class).expect("validated utility");if parsed.state!=state || marker(parsed.token){continue;}
   if parsed.predicates.iter().any(|p|matches!(p,Predicate::Dark)) && mode!="dark"{continue;}
   let media=parsed.predicates.iter().filter_map(|p|if let Predicate::MinWidth(width)=p{Some(*width)}else{None}).max();
   let token=if parsed.placeholder{format!("placeholder:{}",parsed.token)}else{parsed.token.to_owned()};
   let declarations=semantic_color(parsed.token,parsed.placeholder).or_else(||super::custom_spacing::css(parsed.token,theme)).unwrap_or_else(||super::resolved(&token,theme,mode).into_iter().map(|(_,v)|v.css()).collect::<Vec<_>>().join(";"));
   if declarations.is_empty(){continue;}
   let mut after=String::new();
   for predicate in parsed.predicates {after.push_str(&predicate.selector(&scope));}
   after.push_str(match state {State::Base=>"",State::Hover=>":where(:hover:not(:disabled))",State::Focus=>":where(:focus-visible)",State::Active=>":where(:active:not(:disabled))",State::Disabled=>":where(:disabled)"});
   if parsed.placeholder{after.push_str("::placeholder");}
   if let Some(width)=media{output.push_str(&format!("@media(min-width:{width}px){{"));}
   output.push_str(&format!("{scope}.ui[data-tsrust-node=\"{id}\"]{after}{{{declarations}}}"));
   if media.is_some(){output.push('}');}
  }
 }
 output
}
// Resolve semantic colors in the enclosing live scope, including late-mounted content.
fn semantic_color(token:&str,placeholder:bool)->Option<String>{
 let (base,opacity)=super::opacity_token(token).unwrap_or((token,1.));
 let (property,key)=base.split_once('-')?;
 if !super::super::theme::COLORS.contains(&key){return None;}
 let property=match property{"text"=>"color","bg"=>"background","border"=>"border-color","ring"=>"--ui-shadow-ring-color",_=>return None};
 if placeholder&&property!="color"{return None;}
 let value=if opacity==1.{format!("var(--ui-{key})")}else{format!("color-mix(in srgb,var(--ui-{key}) {}%,transparent)",opacity*100.)};
 Some(format!("{property}:{value}{}",if placeholder{";opacity:1"}else{""}))
}

#[cfg(test)] mod tests {
 use super::*;
 use crate::{Kind,Child};
 #[test] fn semantic_colors_and_selected_navigation_follow_live_scopes(){
  let state=crate::navigation::location(String::from,Clone::clone).unwrap();state.set("/docs/ui/components/button".into()).unwrap();
  let link=crate::navigation::link(vec![],"text-muted-foreground aria-current:text-primary aria-current:bg-primary/10".into(),"/docs/ui/components").unwrap();
  let theme=Theme::default();let rules=resolved(&link,&[],&theme,"light");assert!(rules.iter().any(|(_,v)|matches!(v,Utility::Foreground(color) if *color==theme.color("light","primary").unwrap())));
  let css=css("text-syntax-keyword placeholder:text-primary/50 aria-current:bg-primary/10",&theme,"light",7,"");assert!(css.contains("color:var(--ui-syntax-keyword)"));assert!(css.contains("::placeholder{color:color-mix(in srgb,var(--ui-primary) 50%,transparent);opacity:1}"));assert!(css.contains("[aria-current=\"location\"]"));assert!(css.contains("background:color-mix(in srgb,var(--ui-primary) 10%,transparent)"));
  state.set("/other".into()).unwrap();let rules=resolved(&link,&[],&theme,"light");assert!(!rules.iter().any(|(_,v)|matches!(v,Utility::Background(_))));
 }
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

#[cfg(test)] mod content_tests {
 use super::*;
 #[test]fn orientation_and_ancestor_metadata_match_the_browser_contract(){
  let horizontal=Node::view(vec![],"data-horizontal:h-px data-vertical:w-px".into()).unwrap().with_semantics(r#"{"role":"separator"}"#).unwrap();
  let rules=resolved(&horizontal,&[],&Theme::default(),"light");assert!(rules.iter().any(|(_,v)|matches!(v,Utility::Dimension(1,super::super::Length::Pixels(1.)))));assert!(!rules.iter().any(|(_,v)|matches!(v,Utility::Dimension(0,_))));
  let parent=Node::view(vec![],String::new()).unwrap().with_semantics(r#"{"slot":"tooltip-content"}"#).unwrap();
  let child=Node::view(vec![],"in-data-[slot=tooltip-content]:bg-background/20 dark:in-data-[slot=tooltip-content]:bg-background/10".into()).unwrap();
  let light=resolved(&child,&[&parent],&Theme::default(),"light");let dark=resolved(&child,&[&parent],&Theme::default(),"dark");
  assert!(light.iter().any(|(_,v)|matches!(v,Utility::Background(v) if (v.alpha()-0.2).abs()<0.00001)));
  assert!(dark.iter().any(|(_,v)|matches!(v,Utility::Background(v) if (v.alpha()-0.1).abs()<0.00001)));assert!(resolved(&child,&[],&Theme::default(),"light").is_empty());
  let css=css("in-data-[slot=tooltip-content]:bg-background/20 data-horizontal:h-px",&Theme::default(),"light",2,"[data-tsrust-mount=\"1\"]");assert!(css.contains(":where([data-tsrust-mount=\"1\"] [data-slot=\"tooltip-content\"] *)"));assert!(css.contains("[data-orientation=\"horizontal\"]"));
  let control=Node::button("Nested".into(),String::new(),false,||Ok(())).unwrap();let key=Node::view(vec![crate::Child::Node(control)],String::new()).unwrap().with_semantics(r#"{"tag":"kbd"}"#).unwrap();assert!(Node::button_children(vec![crate::Child::Node(key)],String::new(),false,None).is_err());
 }
}
