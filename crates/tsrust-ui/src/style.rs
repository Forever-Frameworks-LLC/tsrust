//! One property merge policy: later classes override earlier classes in each state.
use crate::theme::{Theme,COLORS};
mod layout;
pub use layout::Length;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum State { Base, Hover, Focus, Active, Disabled }
#[derive(Debug, Clone, Copy)]
pub enum Utility {
 Row, Column, GapAxis(u8,f32), Padding(u8,f32), FontSize(f32), Bold(bool),
 Display(bool), Dimension(u8,Length), SelfAlign(u8,u8), Shrink(f32), Grow(f32), Wrap(bool), Tracks(u8,u8), Placement(u8,u16), AutoRowsMin,
 Background([u8;3]), Foreground([u8;3]), Rounded(f32), BorderWidth(f32), BorderColor([u8;3]),
 Align(u8), Items(u8), Justify(u8), Ring([u8;3]),
}
impl Utility {
 pub fn category(self)->u8 {match self {Self::Row|Self::Column=>0,Self::GapAxis(axis,_)=>16+axis,Self::Padding(side,_)=>20+side,Self::Display(_)=>30,Self::Dimension(property,_)=>24+property,Self::SelfAlign(axis,_)=>31+axis,Self::Shrink(_)=>33,Self::Grow(_)=>34,Self::Wrap(_)=>35,Self::Tracks(axis,_)=>36+axis,Self::Placement(property,_)=>38+property,Self::AutoRowsMin=>42,Self::FontSize(_)=>4,Self::Bold(_)=>5,Self::Background(_)=>6,Self::Foreground(_)=>7,Self::Rounded(_)=>8,Self::BorderWidth(_)=>9,Self::BorderColor(_)=>10,Self::Align(_)=>11,Self::Items(_)=>12,Self::Justify(_)=>13,Self::Ring(_)=>14}}
 pub(crate) fn css(self)->String {match self {
  Self::Row=>"flex-direction:row".into(),Self::Column=>"flex-direction:column".into(),Self::GapAxis(axis,v)=>format!("{}-gap:{v}px",if axis==0 {"row"}else{"column"}),
  Self::Display(grid)=>format!("display:{}",if grid{"grid"}else{"flex"}),
  Self::Dimension(property,value)=>format!("{}:{}",["width","height","min-width","min-height","max-width","max-height"][property as usize],value.css()),
  Self::SelfAlign(axis,value)=>format!("{}-self:{}",if axis==0{"align"}else{"justify"},["start","center","end","stretch","auto"][value as usize]),
  Self::Shrink(value)=>format!("flex-shrink:{value}"),Self::Grow(value)=>format!("flex-grow:{value}"),Self::Wrap(value)=>format!("flex-wrap:{}",if value{"wrap"}else{"nowrap"}),
  Self::Tracks(axis,count)=>format!("grid-template-{}:{}",if axis==0{"columns"}else{"rows"},match count {0=>"none".into(),255=>"1fr auto".into(),254=>"auto auto".into(),_=>format!("repeat({count},minmax(0,1fr))")}),
  Self::Placement(property,count)=>format!("{}:{}{count}",["grid-column-start","grid-column-end","grid-row-start","grid-row-end"][property as usize],if property%2==1{"span "}else{""}),
  Self::AutoRowsMin=>"grid-auto-rows:min-content".into(),
  Self::Padding(side,v)=>format!("padding-{}:{v}px",["top","right","bottom","left"][side as usize]),
  Self::FontSize(v)=>format!("font-size:{v}px"),Self::Bold(v)=>format!("font-weight:{}",if v {700}else{400}),Self::Rounded(v)=>format!("border-radius:{v}px"),
  Self::Background([r,g,b])=>format!("background:#{r:02x}{g:02x}{b:02x}"),Self::Foreground([r,g,b])=>format!("color:#{r:02x}{g:02x}{b:02x}"),
  Self::BorderWidth(v)=>format!("border-width:{v}px;border-style:solid"),Self::BorderColor([r,g,b])=>format!("border-color:#{r:02x}{g:02x}{b:02x}"),
  Self::Align(v)=>format!("text-align:{}",["left","center","right"][v as usize]),Self::Items(v)=>format!("align-items:{}",["flex-start","center","stretch"][v as usize]),Self::Justify(v)=>format!("justify-content:{}",["flex-start","center","space-between"][v as usize]),
  Self::Ring([r,g,b])=>format!("outline:2px solid #{r:02x}{g:02x}{b:02x};outline-offset:2px"),
 }}
}
fn build_tokens()->Vec<String> {
 let mut tokens=["flex-row","flex-col","font-bold","font-normal","rounded","rounded-none","rounded-sm","rounded-lg","border","border-0","text-left","text-center","text-right","items-start","items-center","items-stretch","justify-start","justify-center","justify-between","text-base","text-xl","ring"].into_iter().map(str::to_owned).collect::<Vec<_>>();
 tokens.extend(layout::tokens());
 for color in COLORS {for prefix in ["bg","text","border"] {tokens.push(format!("{prefix}-{color}"));}}
 tokens
}
fn split(token:&str)->Result<(State,&str),String> {
 match token.split_once(':') {None=>Ok((State::Base,token)),Some((prefix,token))=>Ok((match prefix {"hover"=>State::Hover,"focus"=>State::Focus,"active"=>State::Active,"disabled"=>State::Disabled,_=>return Err(format!("Unsupported UI state: {prefix}"))},token))}
}
fn known_tokens()->&'static std::collections::BTreeSet<String> {
 static TOKENS:std::sync::OnceLock<std::collections::BTreeSet<String>>=std::sync::OnceLock::new();
 TOKENS.get_or_init(||build_tokens().into_iter().collect())
}
pub fn tokens()->Vec<String> {known_tokens().iter().cloned().collect()}
pub fn validate(classes:&str)->Result<(),String> {
 let known=known_tokens(); for class in classes.split_whitespace() {let (_,token)=split(class)?;if !known.contains(token) {return Err(format!("Unsupported UI utility: {class}"));}}Ok(())
}
pub(crate) fn resolved(classes:&str,theme:&Theme,mode:&str)->Vec<(State,Utility)> {
 let mut rules=std::collections::BTreeMap::new();
 for class in classes.split_whitespace() {
  let (state,token)=split(class).expect("validated utility");
  let mut put=|value:Utility|{rules.insert((state,value.category()),value);};
  if let Some(values)=layout::resolve(token,theme){for value in values {put(value);}continue;}
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
  assert!(values.iter().any(|(s,v)|*s==State::Base && matches!(v,Utility::Padding(3,v) if *v==8.)));
  assert!(values.iter().any(|(s,v)|*s==State::Base && matches!(v,Utility::Padding(0,v) if *v==16.)));
  assert_eq!(values.iter().filter(|(s,v)|*s==State::Base && matches!(v,Utility::Background(_))).count(),1);
  assert!(validate("hover:unknown").is_err());assert!(validate("sm:p-2").is_err());
 }
}

/// Theme references used by authoring navigation and both renderers.
pub fn theme_paths(class:&str)->Vec<Vec<String>> {
 if validate(class).is_err(){return vec![];}
 let token=class.rsplit(':').next().unwrap();
 let color=if token=="ring" {Some("ring")}else{token.strip_prefix("bg-").or_else(||token.strip_prefix("text-")).or_else(||token.strip_prefix("border-"))};
 if let Some(color)=color.filter(|color|COLORS.contains(color)) {return ["light","dark"].into_iter().map(|mode|vec![mode.into(),crate::theme::key_name(color)]).collect();}
 let field=match token {"rounded"=>Some(("radius","md")),"rounded-sm"=>Some(("radius","sm")),"rounded-lg"=>Some(("radius","lg")),"text-base"=>Some(("typography","base")),"text-xl"=>Some(("typography","xl")),_=>None};
 if let Some((group,key))=field{return vec![vec![group.into(),key.into()]];}
 layout::theme_keys(token).into_iter().map(|key|vec!["spacing".into(),key.into()]).collect()
}

/// Observable utility behavior, shared with completion and hover.
pub fn documentation(class:&str)->String {
 if validate(class).is_err(){return String::new();}
 let (_,token)=split(class).unwrap();
 let (purpose,example)=if token.starts_with("grid-cols-") || token.starts_with("grid-rows-") {
  ("Sets explicit grid tracks. Numeric counts 1–12 divide available space equally with zero minimum; none removes explicit tracks. The two written track lists preserve their auto/fraction sizing. Requires a grid container.",format!("<View className=\"grid {class}\" />"))
 }else if token.starts_with("col-") || token.starts_with("row-") {
  ("Places a child in its parent grid. Starts are one-based lines 1–13; spans cover 1–12 tracks. Placement changes visual layout without changing source, keyboard or accessibility order.",format!("<View className=\"grid grid-cols-2\"><View className=\"{class}\" /></View>"))
 }else if token=="grid" || token=="flex" || token.starts_with("flex-") {
  ("Selects grid/flex display, flex direction or flex wrapping. Containers default to flex column. Wrapping needs a finite width and children that cannot shrink to one line; grid uses source-order auto placement.",format!("<View className=\"{class}\" />"))
 }else if token.starts_with("gap-") || token.starts_with('p') && token.contains('-') {
  ("Sets row/column gaps or inner padding. p/px/py expand to independent sides; later classes win per side and state. 2/4/6 use spacing.sm/md/lg, 1 is sm/2, 3 averages sm/md; other numeric sizes use spacing.md/4 per unit. px is exactly one logical pixel and 0 is zero.",format!("<View className=\"{class}\" />"))
 }else if token.starts_with("w-") || token.starts_with("h-") || token.starts_with("min-") || token.starts_with("max-") || token.starts_with("size-") {
  ("Sets preferred, minimum or maximum width/height; size sets both preferred dimensions. Numeric sizes use the project spacing scale; px is one logical pixel. full and fractions use the parent content box. Percentage height needs a definite parent height; otherwise it resolves as auto. auto uses content sizing; max-*-none removes that bound. Min/max constraints can override preferred size.",format!("<View className=\"{class}\" />"))
 }else if token.starts_with("self-") || token.starts_with("justify-self-") {
  ("Aligns this item inside its parent. self applies to the flex cross axis or grid block axis; justify-self applies to the grid inline axis. auto restores the parent's alignment; stretch fills the available track when no explicit size prevents it.",format!("<View className=\"grid\"><View className=\"{class}\" /></View>"))
 }else if token.starts_with("shrink") || token.starts_with("grow") {
  ("Controls distribution of flex space. grow uses factor 1 and grow-0 uses 0; shrink uses 1 and shrink-0 uses 0. These factors affect flex items only and respect their min/max constraints.",format!("<View className=\"flex-row\"><View className=\"{class}\" /></View>"))
 }else if token=="auto-rows-min" {
  ("Sizes implicit grid rows to their children's minimum content height; explicit row tracks keep their own sizing.",format!("<View className=\"grid {class}\" />"))
 }else {
  ("Uses the enclosing theme for colors, radius or typography and applies the checked property in the selected interaction state. Later classes override earlier classes by property within each state.",format!("<View className=\"{class}\" />"))
 };
 let declarations=resolved(class,&Theme::default(),"light").into_iter().map(|(_,value)|value.css()).collect::<Vec<_>>().join("; ");
 format!("`{class}`\n\nShared web/native utility. {purpose} Container display/track/wrap utilities require View or a composed Button; text leaves reject them. No function arguments. Unknown utilities and unsupported state prefixes fail at check/construction time. Example: `{example}`. With the default light theme: `{declarations}`. The enclosing theme supplies custom values; layout uses logical pixels. Arbitrary CSS, breakpoints, named grid lines and subgrid remain unsupported. Ownership: changes layout/style without changing child ownership.")
}

#[cfg(test)] mod layout_tests {
 use super::*;
 #[test] fn every_layout_token_resolves_and_keeps_property_order() {
  let theme=Theme::default();for token in layout::tokens(){assert!(validate(&token).is_ok(),"{token}");assert!(!resolved(&token,&theme,"light").is_empty(),"{token}");assert!(!documentation(&token).is_empty());}
  let rules=resolved("pl-6 p-4 pr-2 gap-x-6 gap-2 gap-y-4 size-8 w-12",&theme,"light");
  assert!(rules.iter().any(|(_,value)|matches!(value,Utility::Padding(3,16.))));
  assert!(rules.iter().any(|(_,value)|matches!(value,Utility::Padding(1,8.))));
  assert!(rules.iter().any(|(_,value)|matches!(value,Utility::GapAxis(0,16.))));
  assert!(rules.iter().any(|(_,value)|matches!(value,Utility::GapAxis(1,8.))));
  assert!(rules.iter().any(|(_,value)|matches!(value,Utility::Dimension(0,Length::Pixels(48.)))));
  assert!(rules.iter().any(|(_,value)|matches!(value,Utility::Dimension(1,Length::Pixels(32.)))));
  for token in ["grid-cols-0","grid-cols-13","col-start-0","col-span-13","max-w-auto","w-none","w-[-2px]","grid-cols-[2fr_1fr]","overflow-auto"]{assert!(validate(token).is_err(),"{token}");}
 }
 #[test] fn container_layout_rejects_text_leaves_and_preserves_item_dimensions() {
  use crate::{Node,Kind,Child,TextPart};
  assert!(Node::new(Kind::Text,"grid grid-cols-2".into(),"text".into(),vec![]).is_err());
  assert!(Node::new(Kind::Text,"w-64 pl-3".into(),"text".into(),vec![]).is_ok());
  assert!(Node::button("text".into(),"grid".into(),false,||Ok(())).is_err());
  assert!(Node::button_children(vec![Child::Text(TextPart::Literal("text".into()))],"grid grid-cols-2".into(),false,None).is_ok());
 }
 #[test] fn custom_spacing_and_reference_navigation_share_one_model() {
  let mut theme=Theme::default();theme.spacing.sm=10.;theme.spacing.md=20.;theme.spacing.lg=30.;
  for (token,expected) in [("p-1",5.),("p-2",10.),("p-3",15.),("p-4",20.),("p-6",30.),("p-8",40.)] {
   assert!(resolved(token,&theme,"light").iter().any(|(_,value)|matches!(value,Utility::Padding(_,v) if *v==expected)),"{token}");
  }
  assert_eq!(theme_paths("pl-3"),vec![vec!["spacing","sm"],vec!["spacing","md"]]);
  assert_eq!(theme_paths("w-64"),vec![vec!["spacing","md"]]);assert!(theme_paths("w-full").is_empty());
  assert!(css("grid grid-cols-2 gap-4",&theme,"light",1).contains("grid-template-columns:repeat(2,minmax(0,1fr))"));
 }
}

/// Text leaves have no child boxes to lay out as a flex/grid container.
pub fn requires_container(classes:&str)->bool {
 classes.split_whitespace().any(|class|split(class).is_ok_and(|(_,token)|matches!(token,"flex"|"grid"|"flex-wrap"|"flex-nowrap"|"auto-rows-min") || token.starts_with("grid-cols-") || token.starts_with("grid-rows-")))
}
