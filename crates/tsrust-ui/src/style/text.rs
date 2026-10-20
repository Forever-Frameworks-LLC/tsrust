//! Shared text flow and decoration utilities.
use super::Utility;
#[derive(Debug,Clone,Copy,PartialEq,Eq,Default)]
pub enum WhiteSpace { Normal, NoWrap, Pre, #[default] PreWrap }
impl WhiteSpace {
 pub(crate) fn css(self)->&'static str {match self{Self::Normal=>"normal",Self::NoWrap=>"nowrap",Self::Pre=>"pre",Self::PreWrap=>"pre-wrap"}}
 #[cfg(any(test,feature="native"))]
 pub(crate) fn collapses(self)->bool {matches!(self,Self::Normal|Self::NoWrap)}
 #[cfg(any(test,feature="native"))]
 pub(crate) fn wraps(self)->bool {matches!(self,Self::Normal|Self::PreWrap)}
}
pub(super) fn tokens()->Vec<String> {["whitespace-normal","whitespace-nowrap","whitespace-pre","whitespace-pre-wrap","underline","no-underline","underline-offset-auto","underline-offset-0","underline-offset-1","underline-offset-2","underline-offset-4","underline-offset-8"].map(str::to_owned).to_vec()}
pub(super) fn resolve(token:&str)->Option<Utility> {Some(match token {
 "whitespace-normal"=>Utility::WhiteSpace(WhiteSpace::Normal),"whitespace-nowrap"=>Utility::WhiteSpace(WhiteSpace::NoWrap),"whitespace-pre"=>Utility::WhiteSpace(WhiteSpace::Pre),"whitespace-pre-wrap"=>Utility::WhiteSpace(WhiteSpace::PreWrap),
 "underline"=>Utility::Underline(true),"no-underline"=>Utility::Underline(false),"underline-offset-auto"=>Utility::UnderlineOffset(None),
 "underline-offset-0"=>Utility::UnderlineOffset(Some(0.)),"underline-offset-1"=>Utility::UnderlineOffset(Some(1.)),"underline-offset-2"=>Utility::UnderlineOffset(Some(2.)),"underline-offset-4"=>Utility::UnderlineOffset(Some(4.)),"underline-offset-8"=>Utility::UnderlineOffset(Some(8.)),_=>return None})}
#[cfg(test)]mod tests {
 use super::*;
 #[test]fn text_utilities_and_link_predicates_have_one_shared_contract(){
  for token in tokens(){assert!(super::super::validate(&token).is_ok());assert!(!super::super::documentation(&token).is_empty());}
  let css=super::super::css("whitespace-normal whitespace-nowrap underline-offset-4 hover:underline [a]:hover:bg-primary/80 rounded-4xl",&crate::theme::Theme::default(),"light",7,"");
  assert!(css.contains("white-space:nowrap"));assert!(css.contains("text-underline-offset:4px"));assert!(css.contains(":where(a):where(:hover:not(:disabled))"));assert!(css.contains("text-decoration-line:underline"));
  assert_eq!(super::super::theme_paths("rounded-4xl"),super::super::theme_paths("rounded-xl"));
  assert!(super::super::validate_inline("whitespace-nowrap hover:underline underline-offset-4").is_ok());
  for invalid in ["[button]:underline","whitespace-break-spaces","underline-offset-3"]{assert!(super::super::validate(invalid).is_err());}
 }
}
#[cfg(test)]mod control_tests {
 #[test]fn whitespace_rules_leave_editing_controls_to_their_own_flow(){
  let css=super::super::css("whitespace-nowrap",&crate::theme::Theme::default(),"light",1,"");assert!(css.contains(":where(:not(input):not(textarea)){white-space:nowrap}"));
 }
}
