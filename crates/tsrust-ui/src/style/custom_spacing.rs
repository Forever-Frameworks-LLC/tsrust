//! Numeric spacing variables, with no arbitrary CSS, expressions or variable cycles.
use super::{Theme,Utility};
use std::collections::BTreeMap;
pub(super) fn declaration(token:&str)->Option<(&str,&str)> {
 let (name,value)=token.strip_prefix("[--")?.strip_suffix(']')?.split_once(":--spacing(")?;
 let scale=value.strip_suffix(')')?;
 (super::structural::name(name) && super::layout::SPACING.contains(&scale)).then_some((name,scale))
}
pub(super) fn reference(token:&str)->Option<(&str,&str)> {
 let (prefix,name)=token.strip_suffix(')')?.split_once("-(--")?;
 (matches!(prefix,"p"|"px"|"py"|"pt"|"pr"|"pb"|"pl"|"gap"|"gap-x"|"gap-y") && super::structural::name(name)).then_some((prefix,name))
}
pub(super) fn resolve(token:&str,theme:&Theme,variables:&BTreeMap<String,f32>)->Option<Vec<Utility>> {
 let (prefix,name)=reference(token)?;let value=variables.get(name).copied().unwrap_or(0.);
 let mut values=super::layout::resolve(&format!("{prefix}-0"),theme)?;
 for utility in &mut values {match utility {Utility::Padding(_,v)|Utility::GapAxis(_,v)=>*v=value,_=>unreachable!()}}
 Some(values)
}
pub(super) fn css(token:&str,theme:&Theme)->Option<String> {
 if let Some((name,scale))=declaration(token){return Some(format!("--{name}:{}px",super::layout::spacing(scale,theme)));}
 let (_,name)=reference(token)?;
 Some(resolve(token,theme,&BTreeMap::new())?.into_iter().map(|utility|match utility {
  Utility::Padding(side,_)=>format!("padding-{}:var(--{name},0px)",["top","right","bottom","left"][side as usize]),
  Utility::GapAxis(axis,_)=>format!("{}-gap:var(--{name},0px)",if axis==0{"row"}else{"column"}),_=>unreachable!(),
 }).collect::<Vec<_>>().join(";"))
}

#[cfg(all(test, feature = "runtime"))] mod tests {
 use super::*;
 use crate::{Node,Kind};
 #[test] fn card_scoped_spacing_inherits_values_and_defaults_missing_names_to_zero() {
  let root=Node::new(Kind::View,"[--panel:--spacing(4)] data-[size=sm]:[--panel:--spacing(3)] p-(--panel)".into(),String::new(),vec![]).unwrap().with_semantics(r#"{"size":"sm"}"#).unwrap();
  let mut theme=Theme::default();theme.spacing.md=20.;
  let (rules,vars)=super::super::structural::scoped(&root,&[],&theme,"light",&BTreeMap::new());assert_eq!(vars["panel"],14.);assert!(rules.iter().any(|(_,v)|matches!(v,Utility::Padding(0,14.))));
  let child=Node::new(Kind::View,"px-(--panel) p-4 py-(--absent)".into(),String::new(),vec![]).unwrap();
  theme.spacing.md=8.;let (rules,vars)=super::super::structural::scoped(&child,&[&root],&theme,"light",&vars);
  assert_eq!(vars["panel"],14.);assert!(rules.iter().any(|(_,v)|matches!(v,Utility::Padding(0,0.))));assert!(rules.iter().any(|(_,v)|matches!(v,Utility::Padding(1,8.))));
  assert_eq!(css("p-(--panel)",&theme).unwrap(),"padding-top:var(--panel,0px);padding-right:var(--panel,0px);padding-bottom:var(--panel,0px);padding-left:var(--panel,0px)");
  for invalid in ["hover:[--panel:--spacing(4)]","[--bad_name:--spacing(4)]","[--panel:--spacing(7)]","[--panel:var(--other)]","p-(--)","w-(--panel)"]{assert!(super::super::validate(invalid).is_err(),"{invalid}");}
 }
}
