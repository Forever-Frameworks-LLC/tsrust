//! One property merge policy: later classes override earlier classes in each state.
use crate::theme::{Theme,COLORS};
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum State { Base, Hover, Focus, Active, Disabled }
#[derive(Debug, Clone, Copy)]
pub enum Utility {
 Row, Column, Gap(f32), PaddingX(f32), PaddingY(f32), FontSize(f32), Bold(bool),
 Background([u8;3]), Foreground([u8;3]), Rounded(f32), BorderWidth(f32), BorderColor([u8;3]),
 Align(u8), Items(u8), Justify(u8), Ring([u8;3]),
}
impl Utility {
 pub fn category(self)->u8 {match self {Self::Row|Self::Column=>0,Self::Gap(_)=>1,Self::PaddingX(_)=>2,Self::PaddingY(_)=>3,Self::FontSize(_)=>4,Self::Bold(_)=>5,Self::Background(_)=>6,Self::Foreground(_)=>7,Self::Rounded(_)=>8,Self::BorderWidth(_)=>9,Self::BorderColor(_)=>10,Self::Align(_)=>11,Self::Items(_)=>12,Self::Justify(_)=>13,Self::Ring(_)=>14}}
 pub(crate) fn css(self)->String {match self {
  Self::Row=>"flex-direction:row".into(),Self::Column=>"flex-direction:column".into(),Self::Gap(v)=>format!("gap:{v}px"),
  Self::PaddingX(v)=>format!("padding-left:{v}px;padding-right:{v}px"),Self::PaddingY(v)=>format!("padding-top:{v}px;padding-bottom:{v}px"),
  Self::FontSize(v)=>format!("font-size:{v}px"),Self::Bold(v)=>format!("font-weight:{}",if v {700}else{400}),Self::Rounded(v)=>format!("border-radius:{v}px"),
  Self::Background([r,g,b])=>format!("background:#{r:02x}{g:02x}{b:02x}"),Self::Foreground([r,g,b])=>format!("color:#{r:02x}{g:02x}{b:02x}"),
  Self::BorderWidth(v)=>format!("border-width:{v}px;border-style:solid"),Self::BorderColor([r,g,b])=>format!("border-color:#{r:02x}{g:02x}{b:02x}"),
  Self::Align(v)=>format!("text-align:{}",["left","center","right"][v as usize]),Self::Items(v)=>format!("align-items:{}",["flex-start","center","stretch"][v as usize]),Self::Justify(v)=>format!("justify-content:{}",["flex-start","center","space-between"][v as usize]),
  Self::Ring([r,g,b])=>format!("outline:2px solid #{r:02x}{g:02x}{b:02x};outline-offset:2px"),
 }}
}
pub fn tokens()->Vec<String> {
 let mut tokens=["flex-row","flex-col","font-bold","font-normal","rounded","rounded-none","rounded-sm","rounded-lg","border","border-0","text-left","text-center","text-right","items-start","items-center","items-stretch","justify-start","justify-center","justify-between","text-base","text-xl","ring"].into_iter().map(str::to_owned).collect::<Vec<_>>();
 for prefix in ["p","px","py","gap"] {for size in ["0","2","4","6"] {tokens.push(format!("{prefix}-{size}"));}}
 for color in COLORS {for prefix in ["bg","text","border"] {tokens.push(format!("{prefix}-{color}"));}}
 tokens
}
fn split(token:&str)->Result<(State,&str),String> {
 match token.split_once(':') {None=>Ok((State::Base,token)),Some((prefix,token))=>Ok((match prefix {"hover"=>State::Hover,"focus"=>State::Focus,"active"=>State::Active,"disabled"=>State::Disabled,_=>return Err(format!("Unsupported UI state: {prefix}"))},token))}
}
pub fn validate(classes:&str)->Result<(),String> {
 let known=tokens(); for class in classes.split_whitespace() {let (_,token)=split(class)?;if !known.iter().any(|v|v==token) {return Err(format!("Unsupported UI utility: {class}"));}}Ok(())
}
pub(crate) fn resolved(classes:&str,theme:&Theme,mode:&str)->Vec<(State,Utility)> {
 let mut rules=std::collections::BTreeMap::new();
 for class in classes.split_whitespace() {
  let (state,token)=split(class).expect("validated utility");
  let mut put=|value:Utility|{rules.insert((state,value.category()),value);};
  let dimension=|size:&str|match size {"0"=>0.,"2"=>theme.spacing.sm as f32,"4"=>theme.spacing.md as f32,"6"=>theme.spacing.lg as f32,_=>unreachable!()};
  if let Some(size)=token.strip_prefix("p-") {let value=dimension(size);put(Utility::PaddingX(value));put(Utility::PaddingY(value));continue;}
  if let Some(size)=token.strip_prefix("px-") {put(Utility::PaddingX(dimension(size)));continue;}
  if let Some(size)=token.strip_prefix("py-") {put(Utility::PaddingY(dimension(size)));continue;}
  if let Some(size)=token.strip_prefix("gap-") {put(Utility::Gap(dimension(size)));continue;}
  let value=match token {
   "flex-row"=>Utility::Row,"flex-col"=>Utility::Column,"font-bold"=>Utility::Bold(true),"font-normal"=>Utility::Bold(false),
   "text-base"=>Utility::FontSize(theme.typography.base as f32),"text-xl"=>Utility::FontSize(theme.typography.xl as f32),
   "rounded"=>Utility::Rounded(theme.radius.md as f32),"rounded-sm"=>Utility::Rounded(theme.radius.sm as f32),"rounded-lg"=>Utility::Rounded(theme.radius.lg as f32),"rounded-none"=>Utility::Rounded(0.),
   "border"=>Utility::BorderWidth(1.),"border-0"=>Utility::BorderWidth(0.),"ring"=>Utility::Ring(theme.color(mode,"ring").unwrap()),
   "text-left"=>Utility::Align(0),"text-center"=>Utility::Align(1),"text-right"=>Utility::Align(2),
   "items-start"=>Utility::Items(0),"items-center"=>Utility::Items(1),"items-stretch"=>Utility::Items(2),
   "justify-start"=>Utility::Justify(0),"justify-center"=>Utility::Justify(1),"justify-between"=>Utility::Justify(2),
   _=>{let (kind,color)=token.split_once('-').unwrap();let color=theme.color(mode,color).expect("validated theme token");match kind {"bg"=>Utility::Background(color),"text"=>Utility::Foreground(color),"border"=>Utility::BorderColor(color),_=>unreachable!()}},
  };put(value);
 }
 rules.into_iter().map(|((state,_),value)|(state,value)).collect()
}
// Keep semantic references in browser CSS so nested theme scopes inherit correctly.
pub(crate) fn css(classes:&str,theme:&Theme,mode:&str,id:usize)->String {
 let mut output=String::new();
 for state in [State::Base,State::Hover,State::Focus,State::Active,State::Disabled] {
  let declarations=resolved(classes,theme,mode).into_iter().filter(|(s,_)|*s==state).map(|(_,v)|v.css()).collect::<Vec<_>>().join(";");
  if declarations.is_empty(){continue;}
  let selector=match state {State::Base=>"",State::Hover=>":where(:hover:not(:disabled))",State::Focus=>":where(:focus-visible)",State::Active=>":where(:active:not(:disabled))",State::Disabled=>":where(:disabled)"};
  output.push_str(&format!(".ui[data-tsrust-node=\"{id}\"]{selector}{{{declarations}}}"));
 }
 output
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn caller_overrides_merge_by_property_and_state() {
  let theme=Theme::default();let values=resolved("p-4 px-2 bg-primary bg-accent hover:bg-primary",&theme,"light");
  assert!(values.iter().any(|(s,v)|*s==State::Base && matches!(v,Utility::PaddingX(v) if *v==8.)));
  assert!(values.iter().any(|(s,v)|*s==State::Base && matches!(v,Utility::PaddingY(v) if *v==16.)));
  assert_eq!(values.iter().filter(|(s,v)|*s==State::Base && matches!(v,Utility::Background(_))).count(),1);
  assert!(validate("hover:unknown").is_err());assert!(validate("sm:p-2").is_err());
 }
}
