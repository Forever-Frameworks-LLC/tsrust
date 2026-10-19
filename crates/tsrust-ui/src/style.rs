//! One property merge policy: later classes override earlier classes in each state.
use crate::theme::{Theme,COLORS,ResolvedColor};
mod layout;
mod structural;
mod custom_spacing;
mod edges;
#[cfg(any(test,feature="native"))]
pub(crate) use structural::scoped as resolved_scoped_node;
#[cfg(test)]
pub(crate) use structural::resolved as resolved_node;
pub use layout::Length;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum State { Base, Hover, Focus, Active, Disabled }
#[derive(Debug, Clone, Copy)]
pub enum Utility {
 Row, Column, GapAxis(u8,f32), Padding(u8,f32), FontSize(f32), Weight(f32), DefaultLeading(f32), Leading(f32), FontFamily(u8),
 Display(bool), Dimension(u8,Length), SelfAlign(u8,u8), Shrink(f32), Grow(f32), Wrap(bool), Tracks(u8,u8), Placement(u8,u16), AutoRowsMin,
 Background(ResolvedColor), Foreground(ResolvedColor), Corner(u8,f32), EdgeBorder(u8,f32), Overflow(bool), BorderColor(ResolvedColor),
 Align(u8), Items(u8), Justify(u8), Ring(ResolvedColor), RingWidth(f32), RingShadowColor(ResolvedColor),
}
impl Utility {
 pub fn category(self)->u8 {match self {Self::Row|Self::Column=>0,Self::GapAxis(axis,_)=>16+axis,Self::Padding(side,_)=>20+side,Self::Display(_)=>30,Self::Dimension(property,_)=>24+property,Self::SelfAlign(axis,_)=>31+axis,Self::Shrink(_)=>33,Self::Grow(_)=>34,Self::Wrap(_)=>35,Self::Tracks(axis,_)=>36+axis,Self::Placement(property,_)=>38+property,Self::AutoRowsMin=>42,Self::FontSize(_)=>4,Self::Weight(_)=>5,Self::DefaultLeading(_)=>45,Self::Leading(_)=>46,Self::FontFamily(_)=>47,Self::Background(_)=>6,Self::Foreground(_)=>7,Self::Corner(corner,_)=>48+corner,Self::EdgeBorder(side,_)=>52+side,Self::Overflow(_)=>56,Self::BorderColor(_)=>10,Self::Align(_)=>11,Self::Items(_)=>12,Self::Justify(_)=>13,Self::Ring(_)=>14,Self::RingWidth(_)=>43,Self::RingShadowColor(_)=>44}}
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
  Self::FontSize(v)=>format!("font-size:{v}px"),Self::Weight(v)=>format!("font-weight:{v}"),Self::DefaultLeading(v)=>format!("line-height:var(--ui-leading,{v}px)"),Self::Leading(v)=>format!("--ui-leading:{v};line-height:{v}"),Self::FontFamily(v)=>format!("font-family:{}",["system-ui,sans-serif","serif","monospace"][v as usize]),Self::Corner(c,v)=>format!("border-{}-radius:{v}px",["top-left","top-right","bottom-right","bottom-left"][c as usize]),Self::Overflow(v)=>format!("overflow:{}",if v{"hidden"}else{"visible"}),
  Self::Background(v)=>format!("background:{}",v.css()),Self::Foreground(v)=>format!("color:{}",v.css()),
  Self::EdgeBorder(s,v)=>{let side=["top","right","bottom","left"][s as usize];format!("border-{side}-width:{v}px;border-{side}-style:solid")},Self::BorderColor(v)=>format!("border-color:{}",v.css()),
  Self::Align(v)=>format!("text-align:{}",["left","center","right"][v as usize]),Self::Items(v)=>format!("align-items:{}",["flex-start","center","stretch"][v as usize]),Self::Justify(v)=>format!("justify-content:{}",["flex-start","center","space-between"][v as usize]),
  Self::Ring(v)=>format!("outline:2px solid {};outline-offset:2px",v.css()),
  Self::RingWidth(v)=>format!("--ui-shadow-ring-width:{v}px;box-shadow:0 0 0 var(--ui-shadow-ring-width) var(--ui-shadow-ring-color,var(--ui-ring))"),
  Self::RingShadowColor(v)=>format!("--ui-shadow-ring-color:{}",v.css()),
 }}
}
fn build_tokens()->Vec<String> {
 let mut tokens=["flex-row","flex-col","font-bold","font-normal","rounded","rounded-none","rounded-sm","rounded-lg","border","border-0","text-left","text-center","text-right","items-start","items-center","items-stretch","justify-start","justify-center","justify-between","text-base","text-xl","text-sm","font-medium","font-semibold","leading-snug","leading-normal","font-sans","font-serif","font-mono","rounded-xl","ring"].into_iter().map(str::to_owned).collect::<Vec<_>>();
 tokens.extend(layout::tokens());tokens.extend(edges::tokens());
 tokens.extend(["ring-0","ring-1","ring-2","ring-3"].map(str::to_owned));
 for color in COLORS.iter().copied().chain(["transparent"]) {for prefix in ["bg","text","border","ring"] {tokens.push(format!("{prefix}-{color}"));}}
 tokens
}
fn split(token:&str)->Result<(State,&str),String> {let parts=structural::parts(token)?;Ok((parts.state,parts.token))}
fn known_tokens()->&'static std::collections::BTreeSet<String> {
 static TOKENS:std::sync::OnceLock<std::collections::BTreeSet<String>>=std::sync::OnceLock::new();
 TOKENS.get_or_init(||build_tokens().into_iter().collect())
}
pub fn tokens()->Vec<String> {known_tokens().iter().cloned().collect()}
pub fn validate(classes:&str)->Result<(),String> {
 let known=known_tokens(); for class in classes.split_whitespace() {let (state,token)=split(class)?;if custom_spacing::declaration(token).is_some() && state!=State::Base{return Err("Spacing variable declarations cannot use interaction states".into());}if !known.contains(token) && opacity_token(token).is_none() && !structural::marker(token) && custom_spacing::declaration(token).is_none() && custom_spacing::reference(token).is_none() {return Err(format!("Unsupported UI utility: {class}"));}}Ok(())
}
pub(crate) fn resolved(classes:&str,theme:&Theme,mode:&str)->Vec<(State,Utility)> {
 resolved_with_variables(classes,theme,mode,&std::collections::BTreeMap::new())
}
fn resolved_with_variables(classes:&str,theme:&Theme,mode:&str,variables:&std::collections::BTreeMap<String,f32>)->Vec<(State,Utility)> {
 let mut rules=std::collections::BTreeMap::new();
 for class in classes.split_whitespace() {
  let (state,token)=split(class).expect("validated utility");
  if structural::marker(token)||custom_spacing::declaration(token).is_some(){continue;}
  let (token,opacity)=opacity_token(token).unwrap_or((token,1.));
  let mut put=|value:Utility|{rules.insert((state,value.category()),value);};
  if let Some(values)=custom_spacing::resolve(token,theme,variables).or_else(||edges::resolve(token,theme)).or_else(||layout::resolve(token,theme)){for value in values {put(value);}continue;}
  if token=="text-sm"{put(Utility::FontSize(theme.typography.base as f32*0.875));put(Utility::DefaultLeading(theme.typography.base as f32*1.25));continue;}
  let value=match token {
   "flex-row"=>Utility::Row,"flex-col"=>Utility::Column,"font-bold"=>Utility::Weight(700.),"font-normal"=>Utility::Weight(400.),"font-medium"=>Utility::Weight(500.),"font-semibold"=>Utility::Weight(600.),"font-sans"=>Utility::FontFamily(0),"font-serif"=>Utility::FontFamily(1),"font-mono"=>Utility::FontFamily(2),"leading-snug"=>Utility::Leading(1.375),"leading-normal"=>Utility::Leading(1.5),
   "text-base"=>Utility::FontSize(theme.typography.base as f32),"text-xl"=>Utility::FontSize(theme.typography.xl as f32),
   "ring"=>Utility::Ring(theme.color(mode,"ring").unwrap()),
   "ring-0"=>Utility::RingWidth(0.),"ring-1"=>Utility::RingWidth(1.),"ring-2"=>Utility::RingWidth(2.),"ring-3"=>Utility::RingWidth(3.),
   "text-left"=>Utility::Align(0),"text-center"=>Utility::Align(1),"text-right"=>Utility::Align(2),
   "items-start"=>Utility::Items(0),"items-center"=>Utility::Items(1),"items-stretch"=>Utility::Items(2),
   "justify-start"=>Utility::Justify(0),"justify-center"=>Utility::Justify(1),"justify-between"=>Utility::Justify(2),
   _=>{let (kind,color)=token.split_once('-').unwrap();let color=if color=="transparent"{ResolvedColor::transparent()}else{theme.color(mode,color).expect("validated theme token")}.opacity(opacity);match kind {"bg"=>Utility::Background(color),"text"=>Utility::Foreground(color),"border"=>Utility::BorderColor(color),"ring"=>Utility::RingShadowColor(color),_=>unreachable!()}},
  };put(value);
 }
 rules.into_iter().map(|((state,_),value)|(state,value)).collect()
}
// Keep semantic references in browser CSS so nested theme scopes inherit correctly.
pub(crate) fn css(classes:&str,theme:&Theme,mode:&str,id:usize,scope:&str)->String {
 structural::css(classes,theme,mode,id,scope)
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
 let (_,token)=split(class).unwrap();let token=opacity_token(token).map_or(token,|(base,_)|base);
 if let Some((_,scale))=custom_spacing::declaration(token){return layout::theme_keys(&format!("p-{scale}")).into_iter().map(|key|vec!["spacing".into(),key.into()]).collect();}
 if custom_spacing::reference(token).is_some(){return vec![];}
 let color=if token=="ring" {Some("ring")}else{token.strip_prefix("bg-").or_else(||token.strip_prefix("text-")).or_else(||token.strip_prefix("border-")).or_else(||token.strip_prefix("ring-"))};
 if let Some(color)=color.filter(|color|COLORS.contains(color)) {return ["light","dark"].into_iter().map(|mode|vec![mode.into(),crate::theme::key_name(color)]).collect();}
 if token.starts_with("rounded") {let key=if token.ends_with("-none"){return vec![];}else if token.ends_with("-sm"){"sm"}else if token.ends_with("-lg"){"lg"}else if token.ends_with("-xl"){"xl"}else{"md"};let mut paths=vec![vec!["radius".into(),key.into()]];if key=="xl"{paths.push(vec!["radius".into(),"lg".into()]);}return paths;}
 let field=match token {"rounded"=>Some(("radius","md")),"rounded-sm"=>Some(("radius","sm")),"rounded-lg"=>Some(("radius","lg")),"rounded-xl"=>Some(("radius","xl")),"text-sm"|"text-base"=>Some(("typography","base")),"text-xl"=>Some(("typography","xl")),_=>None};
 if let Some((group,key))=field{return vec![vec![group.into(),key.into()]];}
 layout::theme_keys(token).into_iter().map(|key|vec!["spacing".into(),key.into()]).collect()
}

/// Observable utility behavior, shared with completion and hover.
pub fn documentation(class:&str)->String {
 if validate(class).is_err(){return String::new();}
 let (_,token)=split(class).unwrap();let default=Theme::default();
 let (purpose,example)=if edges::resolve(token,&default).is_some() {
  ("Sets independent border sides, corner radii or overflow clipping. border and border-t/r/b/l use 1px, -0 removes it and -2 uses 2px; widths participate in layout. rounded and rounded-t/r/b/l/tl/tr/br/bl select md, with -none/sm/lg/xl selecting those slots. radius.xl defaults to lg for older themes. Corner radii scale together to fit the box. Later source classes win independently per side/corner. overflow-hidden clips descendant paint and pointer targets to the rounded padding box; overflow-visible restores overflow. Clipping preserves source/keyboard/accessibility order and does not make clipped controls disabled.",format!("<View className=\"{class}\" />"))
 }else if token=="text-sm" || token.starts_with("leading-") || matches!(token,"font-medium"|"font-semibold"|"font-sans"|"font-serif"|"font-mono") {
  ("text-sm uses 7/8 of typography.base for size and 5/4 of base for default line height (14px/20px at base=16). leading-snug and leading-normal select inherited font-size ratios 1.375 and 1.5; explicit leading wins over a text-sm default regardless of class order. font-medium/semibold select numeric weights 500/600. font-sans/serif/mono select system UI, serif or monospace generic families. Installed font selection and glyph rasterization vary by platform; arbitrary font assets and named families remain unsupported.",format!("<Text className=\"{class}\">Title</Text>"))
 }else if custom_spacing::declaration(token).is_some() || custom_spacing::reference(token).is_some() {
  ("Declares an inherited numeric spacing variable with [--NAME:--spacing(N)], or uses it with p/px/py/pt/pr/pb/pl/gap/gap-x/gap-y-(--NAME). N uses the checked project spacing scale; declarations resolve in their own theme scope and inherit as logical pixels. Later matching declarations win. A missing variable resolves to zero, replacing earlier padding/gap values. Metadata conditions are allowed; interaction-state declarations, arithmetic, aliases and cycles are unsupported. Variables affect spacing only, not accessibility.",format!("<View className=\"[--panel-spacing:--spacing(4)] p-(--panel-spacing)\" />"))
 }else if structural::marker(token) {
  ("Names an ancestor group for checked group-data-[slot=VALUE]/NAME and group-data-[size=VALUE]/NAME selectors. Marker alone changes no visual property. Group names use ASCII letters, digits and hyphens.",format!("<View className=\"{class}\" />"))
 }else if matches!(token,"ring-0"|"ring-1"|"ring-2"|"ring-3") {
  ("Sets an outer surface ring width of 0–3 logical pixels without changing layout. ring-COLOR selects its color and defaults to the theme ring token. This surface ring is independent of the button's keyboard focus outline.",format!("<View className=\"{class} ring-foreground/10\" />"))
 }else if color_token(opacity_token(token).map_or(token,|(base,_)|base)) {
  ("Sets a semantic theme color. Optional /N multiplies its alpha by an integer percentage 0–100; fractional, negative and out-of-range percentages fail. transparent is zero-alpha black. Ring colors need a ring-0/1/2/3 width to paint a surface ring. Color changes do not hide semantic content from accessibility. Native output has an 8-bit sRGB paint boundary; theme strings remain unchanged.",format!("<View className=\"{class}\" />"))
 }else if token.starts_with("grid-cols-") || token.starts_with("grid-rows-") {
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
 let declarations=custom_spacing::css(token,&default).unwrap_or_else(||resolved(class,&default,"light").into_iter().map(|(_,value)|value.css()).collect::<Vec<_>>().join("; "));
 format!("`{class}`\n\nShared web/native utility. {purpose} Container display/track/wrap utilities require View or a composed Button; text leaves reject them. No function arguments. Metadata variants accept data-[slot=VALUE], data-[size=VALUE], has-data-[slot=VALUE], has-data-[size=VALUE], group-data-[slot=VALUE]/NAME, group-data-[size=VALUE]/NAME and [.CLASS]. has-data matches descendants, excluding self. Group variants match any named ancestor, excluding self. Values/names use ASCII letters, digits and hyphens. Multiple metadata predicates compose by conjunction; at most one interaction state is supported. Metadata predicates change styling, never accessibility or source order. Unknown utilities and unsupported state prefixes fail at check/construction time. Example: `{example}`. With the default light theme: `{declarations}`. The enclosing theme supplies custom values; layout uses logical pixels. Arbitrary CSS, breakpoints, named grid lines and subgrid remain unsupported. Ownership: changes layout/style without changing child ownership.")
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
  assert!(css("grid grid-cols-2 gap-4",&theme,"light",1,"").contains("grid-template-columns:repeat(2,minmax(0,1fr))"));
 }
}

/// Text leaves have no child boxes to lay out as a flex/grid container.
pub fn requires_container(classes:&str)->bool {
 classes.split_whitespace().any(|class|split(class).is_ok_and(|(_,token)|matches!(token,"flex"|"grid"|"flex-wrap"|"flex-nowrap"|"auto-rows-min") || token.starts_with("grid-cols-") || token.starts_with("grid-rows-")))
}

fn color_token(token:&str)->bool {
 ["bg-","text-","border-","ring-"].iter().any(|prefix|token.strip_prefix(prefix).is_some_and(|name|name=="transparent" || COLORS.contains(&name)))
}
fn opacity_token(token:&str)->Option<(&str,f32)> {
 let (base,percentage)=token.rsplit_once('/')?;
 if !color_token(base) || percentage.is_empty() || !percentage.bytes().all(|v|v.is_ascii_digit()){return None;}
 let value=percentage.parse::<u8>().ok()?;
 (value<=100 && value.to_string()==percentage).then_some((base,value as f32/100.))
}
/// Suggest common percentages after slash without multiplying every ordinary completion list.
pub fn completion_tokens(prefix:&str)->Vec<String> {
 if let Some((base,_))=prefix.rsplit_once('/') && color_token(base) {
  return [0,5,10,15,20,25,30,40,50,60,70,75,80,90,95,100].into_iter().map(|value|format!("{base}/{value}")).collect();
 }
 tokens()
}

#[cfg(test)] mod color_tests {
 use super::*;
 #[test] fn alpha_modifiers_merge_colors_and_preserve_their_theme_references() {
  for token in ["bg-primary/0","bg-primary/23","hover:bg-primary/50","text-foreground/100","border-input/50","ring-foreground/10","bg-transparent"]{assert!(validate(token).is_ok(),"{token}");}
  for token in ["bg-primary/-1","bg-primary/101","bg-primary/50.5","bg-primary/050","w-4/50","ring-1/50"]{assert!(validate(token).is_err(),"{token}");}
  let theme=Theme::default();let values=resolved("bg-primary/50 bg-accent ring-1 ring-foreground/10",&theme,"light");
  assert!(values.iter().any(|(_,v)|matches!(v,Utility::Background(v) if v.alpha()==1.)));
  assert!(values.iter().any(|(_,v)|matches!(v,Utility::RingWidth(1.))));
  assert!(values.iter().any(|(_,v)|matches!(v,Utility::RingShadowColor(v) if (v.alpha()-0.1).abs()<0.000001)));
  assert_eq!(theme_paths("hover:bg-primary/50"),vec![vec!["light","primary"],vec!["dark","primary"]]);
  assert!(completion_tokens("bg-primary/").contains(&"bg-primary/50".into()));assert!(!tokens().iter().any(|v|v=="bg-primary/50"));
 }
 #[test] fn token_alpha_and_modifier_multiply_without_byte_quantization() {
  let mut raw:serde_json::Value=serde_json::from_str(super::super::theme::DEFAULT_JSON).unwrap();raw["light"]["primary"]="#12345680".into();let theme=Theme::parse(&raw.to_string()).unwrap();
  let values=resolved("bg-primary/25",&theme,"light");let Utility::Background(color)=values[0].1 else{panic!("background expected")};assert!((color.alpha()-128./255.*0.25).abs()<0.0000001);
  assert!(css("ring-1 ring-foreground/10",&theme,"light",1,"").contains("box-shadow:"));
 }
}
