//! One property merge policy: later classes override earlier classes in each state.
use crate::theme::{Theme,COLORS,ResolvedColor};
mod layout;
mod structural;
mod custom_spacing;
mod edges;
mod text;
pub use text::WhiteSpace;
#[cfg(any(test,feature="native"))]
pub(crate) use structural::scoped_at_width as resolved_scoped_node_at_width;
#[cfg(test)]
pub(crate) use structural::resolved as resolved_node;
pub use layout::Length;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum State { Base, Hover, Focus, Active, Disabled }
#[derive(Debug, Clone, Copy)]
pub enum Utility {
 BasisZero, Tracking(f32), Balance(bool), BorderDashed(bool), Margin(u8,f32), WhiteSpace(WhiteSpace), Underline(bool), UnderlineOffset(Option<f32>),
 Row, Column, GapAxis(u8,f32), Padding(u8,f32), FontSize(f32), Weight(f32), DefaultLeading(f32), Leading(f32), FontFamily(u8),
 TableDisplay(u8), CaptionBottom(bool), VerticalAlign(u8), Display(bool), Hidden, InlineFlex, Dimension(u8,Length), SelfAlign(u8,u8), Shrink(f32), Grow(f32), Wrap(bool), Tracks(u8,u8), Placement(u8,u16), AutoRowsMin,
 Background(ResolvedColor), Foreground(ResolvedColor), Corner(u8,f32), EdgeBorder(u8,f32), Overflow(u8), BorderColor(ResolvedColor),
 Align(u8), Items(u8), Justify(u8), Ring(ResolvedColor), RingWidth(f32), RingShadowColor(ResolvedColor),
 Placeholder(ResolvedColor),Opacity(f32),OutlineNone,Cursor(u8),SelectNone,PointerEvents(bool),FieldSizingContent,AspectRatio(Option<f32>),Relative,Absolute,Inset(u8,Length),
}
impl Utility {
 pub fn category(self)->u8 {match self {Self::TableDisplay(_)=>30,Self::CaptionBottom(_)=>81,Self::VerticalAlign(_)=>82,Self::BasisZero=>80,Self::Tracking(_)=>73,Self::Balance(_)=>74,Self::BorderDashed(_)=>75,Self::Margin(side,_)=>76+side,Self::WhiteSpace(_)=>70,Self::Underline(_)=>71,Self::UnderlineOffset(_)=>72,Self::Row|Self::Column=>0,Self::GapAxis(axis,_)=>16+axis,Self::Padding(side,_)=>20+side,Self::Display(_)|Self::Hidden|Self::InlineFlex=>30,Self::Dimension(property,_)=>24+property,Self::SelfAlign(axis,_)=>31+axis,Self::Shrink(_)=>33,Self::Grow(_)=>34,Self::Wrap(_)=>35,Self::Tracks(axis,_)=>36+axis,Self::Placement(property,_)=>38+property,Self::AutoRowsMin=>42,Self::FontSize(_)=>4,Self::Weight(_)=>5,Self::DefaultLeading(_)=>45,Self::Leading(_)=>46,Self::FontFamily(_)=>47,Self::Background(_)=>6,Self::Foreground(_)=>7,Self::Corner(corner,_)=>48+corner,Self::EdgeBorder(side,_)=>52+side,Self::Overflow(_)=>56,Self::BorderColor(_)=>10,Self::Align(_)=>11,Self::Items(_)=>12,Self::Justify(_)=>13,Self::Ring(_)=>14,Self::RingWidth(_)=>43,Self::RingShadowColor(_)=>44,Self::Placeholder(_)=>57,Self::Opacity(_)=>58,Self::OutlineNone=>14,Self::Cursor(_)=>60,Self::SelectNone=>61,Self::PointerEvents(_)=>62,Self::FieldSizingContent=>63,Self::AspectRatio(_)=>64,Self::Relative|Self::Absolute=>65,Self::Inset(side,_)=>66+side}}
 pub(crate) fn css(self)->String {match self {
  Self::BasisZero=>"flex-basis:0%".into(),Self::Tracking(v)=>format!("letter-spacing:{v}em"),Self::Balance(v)=>format!("text-wrap-style:{}",if v{"balance"}else{"auto"}),Self::BorderDashed(v)=>format!("border-style:{}",if v{"dashed"}else{"solid"}),Self::Margin(side,v)=>format!("margin-{}:{v}px",["top","right","bottom","left"][side as usize]),
  Self::WhiteSpace(v)=>format!("white-space:{}",v.css()),Self::Underline(v)=>format!("text-decoration-line:{}",if v{"underline"}else{"none"}),Self::UnderlineOffset(v)=>format!("text-underline-offset:{}",v.map_or_else(||"auto".into(),|v|format!("{v}px"))),
  Self::AspectRatio(value)=>format!("aspect-ratio:{}",value.map_or_else(||"auto".into(),|value|value.to_string())),Self::Relative=>"position:relative".into(),Self::Absolute=>"position:absolute".into(),Self::Inset(side,value)=>format!("{}:{}",["top","right","bottom","left"][side as usize],value.css()),
  Self::Placeholder(v)=>format!("color:{};opacity:1",v.css()),Self::Opacity(v)=>format!("opacity:{v}"),Self::OutlineNone=>"outline:none".into(),Self::Cursor(v)=>format!("cursor:{}",["auto","text","not-allowed"][v as usize]),Self::SelectNone=>"user-select:none".into(),Self::PointerEvents(v)=>format!("pointer-events:{}",if v{"auto"}else{"none"}),Self::FieldSizingContent=>"field-sizing:content;height:auto".into(),
  Self::Row=>"flex-direction:row".into(),Self::Column=>"flex-direction:column".into(),Self::GapAxis(axis,v)=>format!("{}-gap:{v}px",if axis==0 {"row"}else{"column"}),
  Self::TableDisplay(index)=>format!("display:{}{}",["table","table-header-group","table-row-group","table-footer-group","table-row","table-cell","table-caption"][index as usize],if index==0{";border-collapse:collapse;border-spacing:0"}else{""}),
  Self::CaptionBottom(bottom)=>format!("caption-side:{}",if bottom{"bottom"}else{"top"}),
  Self::VerticalAlign(index)=>format!("vertical-align:{}",["top","middle","bottom"][index as usize]),
  Self::InlineFlex=>"display:inline-flex".into(),
  Self::Hidden=>"display:none".into(),
  Self::Display(grid)=>format!("display:{}",if grid{"grid"}else{"flex"}),
  Self::Dimension(property,value)=>format!("{}:{}",["width","height","min-width","min-height","max-width","max-height"][property as usize],value.css()),
  Self::SelfAlign(axis,value)=>format!("{}-self:{}",if axis==0{"align"}else{"justify"},["start","center","end","stretch","auto"][value as usize]),
  Self::Shrink(value)=>format!("flex-shrink:{value}"),Self::Grow(value)=>format!("flex-grow:{value}"),Self::Wrap(value)=>format!("flex-wrap:{}",if value{"wrap"}else{"nowrap"}),
  Self::Tracks(axis,count)=>format!("grid-template-{}:{}",if axis==0{"columns"}else{"rows"},match count {0=>"none".into(),255=>"1fr auto".into(),254=>"auto auto".into(),_=>format!("repeat({count},minmax(0,1fr))")}),
  Self::Placement(property,count)=>format!("{}:{}{count}",["grid-column-start","grid-column-end","grid-row-start","grid-row-end"][property as usize],if property%2==1{"span "}else{""}),
  Self::AutoRowsMin=>"grid-auto-rows:min-content".into(),
  Self::Padding(side,v)=>format!("padding-{}:{v}px",["top","right","bottom","left"][side as usize]),
  Self::FontSize(v)=>format!("font-size:{v}px"),Self::Weight(v)=>format!("font-weight:{v}"),Self::DefaultLeading(v)=>format!("line-height:var(--ui-leading,{v}px)"),Self::Leading(v)=>format!("--ui-leading:{v};line-height:{v}"),Self::FontFamily(v)=>format!("font-family:{}",["system-ui,sans-serif","serif","monospace"][v as usize]),Self::Corner(c,v)=>format!("border-{}-radius:{v}px",["top-left","top-right","bottom-right","bottom-left"][c as usize]),Self::Overflow(v)=>format!("overflow:{}",["visible","hidden","auto"][v as usize]),
  Self::Background(v)=>format!("background:{}",v.css()),Self::Foreground(v)=>format!("color:{}",v.css()),
  Self::EdgeBorder(s,v)=>{let side=["top","right","bottom","left"][s as usize];format!("border-{side}-width:{v}px")},Self::BorderColor(v)=>format!("border-color:{}",v.css()),
  Self::Align(v)=>format!("text-align:{}",["left","center","right"][v as usize]),Self::Items(v)=>format!("align-items:{}",["flex-start","center","stretch"][v as usize]),Self::Justify(v)=>format!("justify-content:{}",["flex-start","center","space-between"][v as usize]),
  Self::Ring(v)=>format!("outline:2px solid {};outline-offset:2px",v.css()),
  Self::RingWidth(v)=>format!("--ui-shadow-ring-width:{v}px;box-shadow:0 0 0 var(--ui-shadow-ring-width) var(--ui-shadow-ring-color,var(--ui-ring))"),
  Self::RingShadowColor(v)=>format!("--ui-shadow-ring-color:{}",v.css()),
 }}
}
fn build_tokens()->Vec<String> {
 let mut tokens=["flex-row","flex-col","font-bold","font-normal","rounded","rounded-none","rounded-sm","rounded-lg","border","border-0","text-left","text-center","text-right","items-start","items-center","items-stretch","justify-start","justify-center","justify-between","text-base","text-xl","text-2xl","text-3xl","text-4xl","text-5xl","text-6xl","text-7xl","text-sm","text-xs","font-medium","font-semibold","leading-tight","leading-snug","leading-normal","leading-relaxed","text-sm/relaxed","font-heading","font-sans","font-serif","font-mono","rounded-xl","ring"].into_iter().map(str::to_owned).collect::<Vec<_>>();
 tokens.extend(["opacity-0","opacity-50","opacity-100","outline-none","cursor-auto","cursor-text","cursor-not-allowed","select-none","pointer-events-none","pointer-events-auto","field-sizing-content"].map(str::to_owned));
 tokens.extend(text::tokens());tokens.extend(layout::tokens());tokens.extend(edges::tokens());
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
 if classes.split_whitespace().any(|class|split(class).is_ok_and(|(_,token)|token=="table-cell")) && classes.split_whitespace().any(|class|split(class).is_ok_and(|(_,token)|["w-full","w-1/2","w-1/3","w-2/3","w-1/4","w-3/4","min-w-full","max-w-full","min-w-1/2","min-w-1/3","min-w-2/3","min-w-1/4","min-w-3/4","max-w-1/2","max-w-1/3","max-w-2/3","max-w-1/4","max-w-3/4"].contains(&token))){return Err("Percentage table-cell widths are unsupported; use intrinsic or pixel widths".into());}
 let known=known_tokens(); for class in classes.split_whitespace() {let parsed=structural::parts(class)?;let (state,token)=(parsed.state,parsed.token);if parsed.placeholder && !token.strip_prefix("text-").is_some_and(|color|color=="transparent"||COLORS.contains(&color)||opacity_token(token).is_some()){return Err("Placeholder variants require a text color".into());}if custom_spacing::declaration(token).is_some() && state!=State::Base{return Err("Spacing variable declarations cannot use interaction states".into());}if !known.contains(token) && opacity_token(token).is_none() && layout::aspect(token).is_none() && !layout::pixel_dimension(token) && !structural::marker(token) && custom_spacing::declaration(token).is_none() && custom_spacing::reference(token).is_none() {return Err(format!("Unsupported UI utility: {class}"));}}Ok(())
}
pub(crate) fn resolved(classes:&str,theme:&Theme,mode:&str)->Vec<(State,Utility)> {
 resolved_with_variables(classes,theme,mode,&std::collections::BTreeMap::new())
}
fn resolved_with_variables(classes:&str,theme:&Theme,mode:&str,variables:&std::collections::BTreeMap<String,f32>)->Vec<(State,Utility)> {
 let mut rules=std::collections::BTreeMap::new();
 for class in classes.split_whitespace() {
  let parsed=structural::parts(class).expect("validated utility");let (state,token)=(parsed.state,parsed.token);
  if structural::marker(token)||custom_spacing::declaration(token).is_some(){continue;}
  let (token,opacity)=opacity_token(token).unwrap_or((token,1.));
  let mut put=|value:Utility|{let value=if parsed.placeholder{if let Utility::Foreground(color)=value{Utility::Placeholder(color)}else{value}}else{value};rules.insert((state,value.category()),value);};
  if let Some(values)=custom_spacing::resolve(token,theme,variables).or_else(||edges::resolve(token,theme)).or_else(||layout::resolve(token,theme)){for value in values {put(value);}continue;}
  if token=="text-sm/relaxed"{put(Utility::FontSize(theme.typography.base as f32*0.875));put(Utility::Leading(1.625));continue;}
  if matches!(token,"text-sm"|"text-xs"){let (size,leading)=if token=="text-xs"{(0.75,1.)}else{(0.875,1.25)};put(Utility::FontSize(theme.typography.base as f32*size));put(Utility::DefaultLeading(theme.typography.base as f32*leading));continue;}
  let value=if let Some(value)=text::resolve(token){value}else{match token {
   "opacity-0"=>Utility::Opacity(0.),"opacity-50"=>Utility::Opacity(0.5),"opacity-100"=>Utility::Opacity(1.),"outline-none"=>Utility::OutlineNone,"cursor-auto"=>Utility::Cursor(0),"cursor-text"=>Utility::Cursor(1),"cursor-not-allowed"=>Utility::Cursor(2),"select-none"=>Utility::SelectNone,"pointer-events-none"=>Utility::PointerEvents(false),"pointer-events-auto"=>Utility::PointerEvents(true),"field-sizing-content"=>Utility::FieldSizingContent,
   "flex-row"=>Utility::Row,"flex-col"=>Utility::Column,"font-bold"=>Utility::Weight(700.),"font-normal"=>Utility::Weight(400.),"font-medium"=>Utility::Weight(500.),"font-semibold"=>Utility::Weight(600.),"font-sans"=>Utility::FontFamily(0),"font-serif"=>Utility::FontFamily(1),"font-mono"=>Utility::FontFamily(2),"leading-tight"=>Utility::Leading(1.1),"leading-snug"=>Utility::Leading(1.375),"leading-normal"=>Utility::Leading(1.5),"leading-relaxed"=>Utility::Leading(1.625),"font-heading"=>Utility::FontFamily(theme.typography.heading_family.index()),
   "text-2xl"=>Utility::FontSize(theme.typography.base as f32*1.875),"text-3xl"=>Utility::FontSize(theme.typography.base as f32*2.25),"text-4xl"=>Utility::FontSize(theme.typography.base as f32*3.),"text-5xl"=>Utility::FontSize(theme.typography.base as f32*3.75),"text-6xl"=>Utility::FontSize(theme.typography.base as f32*4.5),"text-7xl"=>Utility::FontSize(theme.typography.base as f32*6.),
   "text-base"=>Utility::FontSize(theme.typography.base as f32),"text-xl"=>Utility::FontSize(theme.typography.xl as f32),
   "ring"=>Utility::Ring(theme.color(mode,"ring").unwrap()),
   "ring-0"=>Utility::RingWidth(0.),"ring-1"=>Utility::RingWidth(1.),"ring-2"=>Utility::RingWidth(2.),"ring-3"=>Utility::RingWidth(3.),
   "text-left"=>Utility::Align(0),"text-center"=>Utility::Align(1),"text-right"=>Utility::Align(2),
   "items-start"=>Utility::Items(0),"items-center"=>Utility::Items(1),"items-stretch"=>Utility::Items(2),
   "justify-start"=>Utility::Justify(0),"justify-center"=>Utility::Justify(1),"justify-between"=>Utility::Justify(2),
   _=>{let (kind,color)=token.split_once('-').unwrap();let color=if color=="transparent"{ResolvedColor::transparent()}else{theme.color(mode,color).expect("validated theme token")}.opacity(opacity);match kind {"bg"=>Utility::Background(color),"text"=>Utility::Foreground(color),"border"=>Utility::BorderColor(color),"ring"=>Utility::RingShadowColor(color),_=>unreachable!()}},
  }};put(value);
 }
 rules.into_iter().map(|((state,_),value)|(state,value)).collect()
}
// Keep semantic references in browser CSS so nested theme scopes inherit correctly.
pub(crate) fn css(classes:&str,theme:&Theme,mode:&str,id:u64,scope:&str)->String {
 structural::css(classes,theme,mode,id,scope)
}

#[cfg(test)] mod tests {
 use super::*;
 #[test] fn caller_overrides_merge_by_property_and_state() {
  let theme=Theme::default();let values=resolved("p-4 px-2 bg-primary bg-accent hover:bg-primary",&theme,"light");
  assert!(values.iter().any(|(s,v)|*s==State::Base && matches!(v,Utility::Padding(3,v) if *v==8.)));
  assert!(values.iter().any(|(s,v)|*s==State::Base && matches!(v,Utility::Padding(0,v) if *v==16.)));
  assert_eq!(values.iter().filter(|(s,v)|*s==State::Base && matches!(v,Utility::Background(_))).count(),1);
  assert!(validate("hover:unknown").is_err());assert!(validate("sm:p-2").is_ok());
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
 if token.starts_with("rounded") {let key=if token.ends_with("-none"){return vec![];}else if token.ends_with("-sm"){"sm"}else if token.ends_with("-lg"){"lg"}else if ["-xl","-2xl","-3xl","-4xl"].iter().any(|suffix|token.ends_with(suffix)){"xl"}else{"md"};let mut paths=vec![vec!["radius".into(),key.into()]];if key=="xl"{paths.push(vec!["radius".into(),"lg".into()]);}return paths;}
 let field=match token {"rounded"=>Some(("radius","md")),"rounded-sm"=>Some(("radius","sm")),"rounded-lg"=>Some(("radius","lg")),"rounded-xl"=>Some(("radius","xl")),"text-xs"|"text-sm"|"text-sm/relaxed"|"max-w-sm"|"text-base"|"text-2xl"|"text-3xl"|"text-4xl"|"text-5xl"|"text-6xl"|"text-7xl"=>Some(("typography","base")),"text-xl"=>Some(("typography","xl")),"font-heading"=>Some(("typography","headingFamily")),_=>None};
 if let Some((group,key))=field{return vec![vec![group.into(),key.into()]];}
 layout::theme_keys(token).into_iter().map(|key|vec!["spacing".into(),key.into()]).collect()
}

/// Observable utility behavior, shared with completion and hover.
pub fn documentation(class:&str)->String {
 if validate(class).is_err(){return String::new();}
 let (_,token)=split(class).unwrap();let default=Theme::default();
 let (purpose,example)=if matches!(token,"table"|"table-header-group"|"table-row-group"|"table-footer-group"|"table-row"|"table-cell"|"table-caption"|"caption-top"|"caption-bottom"|"align-top"|"align-middle"|"align-bottom") {
  ("Selects portable shared-column table layout, caption placement or cell vertical alignment. Semantic tags alone do not change layout. Portable rows require th/td table-cell children, and groups require tr table-row children; malformed structures throw before rendering. Table rows share intrinsic column widths across header/body/footer; colSpan covers 1–1000 columns. Tables can exceed a narrow overflow-auto wrapper. Caption-bottom moves captions visually without changing source/ownership/focus order. align-top/middle/bottom places cell content within its row. Percentage cell widths, rowSpan, nested tables, fixed table layout, separate spacing and border-conflict resolution are unsupported. Font metrics and surplus width distribution can differ by target. No arguments or ownership change.",format!("<View as=\"table\" className=\"table caption-bottom\"><View as=\"tr\" className=\"table-row\"><View as=\"td\" className=\"table-cell align-middle\">Value</View></View></View> // one shared column"))
 }else if layout::pixel_dimension(token) {
  ("Sets a nonnegative finite logical-pixel box dimension with w/h/min-w/min-h/max-w/max-h/size-[Npx] (or [N]). No theme scaling. Fractions, negatives, CSS expressions and nonfinite/native-overflow values reject. Example w-[100px] prefers 100 logical pixels; content and min/max constraints can require more space. No arguments, text mutation or ownership change.",format!("<View className=\"{class}\"/>"))
 }else if token=="hidden" {
  ("Removes the element and its subtree from layout, painting, pointer input, keyboard focus and accessibility. Responsive flex/grid utilities can show the same retained subtree at a wider viewport. Unlike conditional children, this preserves state and ownership and does not dispose handlers. No arguments. Example: hidden md:flex hides below 768 logical pixels and shows at or above 768.",format!("<View className=\"{class} md:flex\"/>"))
 }else if matches!(token,"text-balance"|"text-wrap-auto") {
  ("Balances short wrapped text without changing its value or hard breaks. text-balance asks both renderers for balanced line lengths; text-wrap-auto restores ordinary wrapping. Native balancing preserves the ordinary line count for blocks of 2–6 lines using a bounded width search; longer blocks use ordinary wrapping. Browsers choose their own balancing algorithm and limits, so exact break positions can differ with fonts. Whitespace/nowrap rules still apply. Measurement, painting, selection and accessibility geometry share the native layout. Does not change width constraints or editable-control behavior. No arguments or ownership change.",format!("<Text className=\"w-64 text-center {class}\">Build a workspace for your next project</Text>"))
 }else if matches!(token,"tracking-tight"|"tracking-normal") {
  ("Sets inherited letter spacing relative to the current font size. tracking-tight uses -0.025em (-0.4 logical pixels at font-size 16); tracking-normal restores zero extra spacing. Native shaping, measurement, selection and painting use the same spacing. Stored text is unchanged; font shaping can differ across platforms. No arguments or ownership change.",format!("<Text className=\"{class}\">Project settings</Text>"))
 }else if token=="font-heading" {
  ("Selects typography.headingFamily from the enclosing project/theme scope. Omission uses sans; serif and mono are also accepted generic families. Explicit font-sans/serif/mono classes can override it. Does not load fonts; named font assets remain unsupported and actual system fonts can differ across platforms. Inherited by child text. No arguments or ownership change.",format!("<Text className=\"{class}\">Projects</Text>"))
 }else if matches!(token,"border-dashed"|"border-solid") {
  ("Selects dashed or solid border painting independently of border widths and colors. Defaults to solid; border-dashed alone adds no width. Later border width utilities preserve this style. Native dashes use three times the largest side width for each dash and gap, clipped to the same rounded border ring as solid paint. Browser dash phase/corner distribution can differ. Zero-width sides remain unpainted. No arguments or ownership change.",format!("<View className=\"border rounded-xl {class} p-6\"/>"))
 }else if token=="flex-1" {
  ("Lets a flex item grow and shrink with factors 1 and starts its flex basis at 0%. This distributes available main-axis space while respecting min/max dimensions and intrinsic minimums. It does not change the item into a flex container. Later grow/shrink utilities override their own factors without clearing the basis. No arguments or ownership change.",format!("<View className=\"flex-row\"><View className=\"{class}\"/></View>"))
 }else if token=="max-w-sm" {
  ("Caps box width at 24 times typography.base from the enclosing theme: 384 logical pixels at base 16, 480 at base 20. Width can still shrink to fit its parent. This does not center a box or change its minimum width. Inline text runs reject box utilities. No arguments or ownership change.",format!("<View className=\"w-full {class}\"/>"))
 }else if token.starts_with("m-")||["mx-","my-","mt-","mr-","mb-","ml-"].iter().any(|prefix|token.starts_with(prefix)) {
  ("Adds nonnegative outer spacing using the project's spacing scale. m sets all sides; mx/my set horizontal/vertical sides; mt/mr/mb/ml set one side. Later matching classes override each side independently. 2/4/6 use spacing.sm/md/lg; px means one logical pixel and other numeric values follow the shared spacing scale. Negative, percentage and auto margins remain unsupported. Flex/grid margins do not collapse. No arguments, text mutation or ownership change.",format!("<View className=\"{class}\"/>"))
 }else if text::resolve(token).is_some() {
  ("Controls inherited text flow or adds a local underline. whitespace-normal collapses spaces, tabs and segment breaks and allows wrapping; whitespace-nowrap collapses them and prevents soft wraps. whitespace-pre preserves whitespace and hard breaks without soft wraps; whitespace-pre-wrap preserves whitespace with soft wrapping, the project default. No text value or editable value is mutated. Selection uses displayed text. Input and Textarea keep their editing whitespace rules. underline/no-underline adds/removes this element's underline; ancestor decorations remain. underline-offset-auto uses font metrics; -0/1/2/4/8 uses that many logical pixels below the baseline, independent of theme spacing. Later matching classes win per property. Native font metrics and rasterization can differ from browser fonts. No ownership change.",format!("<Text className=\"{class}\">Ready to ship</Text>"))
 }else if layout::aspect(token).is_some() {
  ("Sets a preferred width/height ratio on a layout box; inline text runs have no separate ratio box. aspect-square uses 1, aspect-video uses 16/9, aspect-[W/H] and aspect-[R] accept finite positive numbers, and aspect-auto removes the preference. Invalid, zero, negative, nonfinite or native-overflow/underflow ratios reject. Both targets normalize the ratio to f32 logical layout precision. A definite width with automatic height derives height; two explicit dimensions win. Min/max dimensions constrain the result. Children can overflow unless clipping/scrolling is requested. No accessibility role or ownership change. Later matching aspect classes win.",format!("<View className=\"w-64 {class}\" />"))
 }else if matches!(token,"relative"|"absolute") || layout::resolve(token,&default).is_some_and(|values|values.iter().any(|value|matches!(value,Utility::Inset(_,_)))) {
  ("Positions a layout box within its immediate UI parent. Every UI box establishes a containing block. relative keeps normal flow; absolute removes the box from flow. top/right/bottom/left and inset/inset-x/inset-y set sides independently; auto clears a side. Spacing uses project tokens, fractions/full use the parent's corresponding dimension, and a leading minus negates a numeric/fraction offset. Later matching classes win independently per side. A definite parent size is needed for predictable percentage placement. Inline text descendants reject box utilities. Paint order, clipping and keyboard order remain source order; z-index, fixed/sticky positioning and transforms are unsupported. No ownership change.",format!("<View className=\"relative w-64 h-32\"><View className=\"absolute top-2 right-2 w-8 h-8\" /></View> // 8px from the parent's top/right at default spacing"))
 }else if matches!(token,"opacity-0"|"opacity-50"|"opacity-100"|"outline-none"|"cursor-auto"|"cursor-text"|"cursor-not-allowed"|"select-none"|"pointer-events-none"|"pointer-events-auto"|"field-sizing-content") {
  ("opacity-0/50/100 selects whole-subtree paint opacity 0/0.5/1 without changing layout, focus or accessibility. outline-none removes the fallback focus outline; supply a visible focus ring when using it. cursor-auto/text/not-allowed chooses the pointer cursor. pointer-events-none/auto removes/restores pointer targeting and is inherited; keyboard focus stays available. select-none prevents selection of static content, while editable fields keep editing selection. field-sizing-content makes Textarea height follow wrapped content/placeholder subject to min/max bounds; other nodes ignore it. No function arguments. Later matching classes override the same property.",format!("<Input className=\"{class}\" aria-label=\"Name\" />"))
 }else if edges::resolve(token,&default).is_some() {
  ("Sets independent border sides, corner radii or overflow clipping. border and border-t/r/b/l use 1px, -0 removes it and -2 uses 2px; widths participate in layout. rounded and rounded-t/r/b/l/tl/tr/br/bl select md, with -none/sm/lg/xl selecting those slots. -2xl/3xl/4xl use 1.25/1.5/2 times radius.xl. radius.xl defaults to lg for older themes. Corner radii scale together to fit the box. Later source classes win independently per side/corner. overflow-auto creates a bounded two-axis scroll container; constrain its dimensions. Wheel/trackpad deltas bubble after an inner container reaches its limit. Focus reveals controls through nested containers. Use View tabIndex=0 and an accessible region name for keyboard scrolling. overflow-hidden clips to the rounded padding box and permits focus reveal but no wheel scrolling; overflow-visible restores overflow. Clipping preserves source/keyboard/accessibility order and does not make clipped controls disabled.",format!("<View className=\"{class}\" />"))
 }else if matches!(token,"text-xs"|"text-sm"|"text-sm/relaxed"|"text-2xl"|"text-3xl"|"text-4xl"|"text-5xl"|"text-6xl"|"text-7xl") || token.starts_with("leading-") || matches!(token,"font-medium"|"font-semibold"|"font-sans"|"font-serif"|"font-mono") {
  ("text-2xl/3xl/4xl/5xl/6xl/7xl use 1.875/2.25/3/3.75/4.5/6 times typography.base (30/36/48/60/72/96px at base=16). text-xs uses 3/4 of typography.base and one base unit for default line height (12px/16px at base=16). text-sm uses 7/8 of typography.base for size and 5/4 of base for default line height (14px/20px at base=16). leading-tight, leading-snug leading-normal and leading-relaxed select inherited font-size ratios 1.1, 1.375, 1.5 and 1.625; text-sm/relaxed combines the small font size with 1.625 leading; explicit leading wins over a text-sm default regardless of class order. font-medium/semibold select numeric weights 500/600. font-sans/serif/mono select system UI, serif or monospace generic families. Installed font selection and glyph rasterization vary by platform; arbitrary font assets and named families remain unsupported.",format!("<Text className=\"{class}\">Title</Text>"))
 }else if custom_spacing::declaration(token).is_some() || custom_spacing::reference(token).is_some() {
  ("Declares an inherited numeric spacing variable with [--NAME:--spacing(N)], or uses it with p/px/py/pt/pr/pb/pl/gap/gap-x/gap-y-(--NAME). N uses the checked project spacing scale; declarations resolve in their own theme scope and inherit as logical pixels. Later matching declarations win. A missing variable resolves to zero, replacing earlier padding/gap values. Metadata conditions are allowed; interaction-state declarations, arithmetic, aliases and cycles are unsupported. Variables affect spacing only, not accessibility.",format!("<View className=\"[--panel-spacing:--spacing(4)] p-(--panel-spacing)\" />"))
 }else if structural::marker(token) {
  ("Names an ancestor group for checked group-data-[slot=VALUE]/NAME and group-data-[size=VALUE]/NAME or group-data-[variant=VALUE]/NAME selectors. Marker alone changes no visual property. Group names use ASCII letters, digits and hyphens.",format!("<View className=\"{class}\" />"))
 }else if matches!(token,"ring-0"|"ring-1"|"ring-2"|"ring-3") {
  ("Sets an outer surface ring width of 0–3 logical pixels without changing layout. ring-COLOR selects its color and defaults to the theme ring token. This surface ring is independent of the button's keyboard focus outline.",format!("<View className=\"{class} ring-foreground/10\" />"))
 }else if color_token(opacity_token(token).map_or(token,|(base,_)|base)) {
  ("Sets a semantic theme color. Optional /N multiplies its alpha by an integer percentage 0–100; fractional, negative and out-of-range percentages fail. placeholder:text-COLOR applies that color only to empty editable hints with full token opacity. transparent is zero-alpha black. Ring colors need a ring-0/1/2/3 width to paint a surface ring. Color changes do not hide semantic content from accessibility. Native output has an 8-bit sRGB paint boundary; theme strings remain unchanged.",format!("<View className=\"{class}\" />"))
 }else if token.starts_with("grid-cols-") || token.starts_with("grid-rows-") {
  ("Sets explicit grid tracks. Numeric counts 1–12 divide available space equally with zero minimum; none removes explicit tracks. The two written track lists preserve their auto/fraction sizing. Requires a grid container.",format!("<View className=\"grid {class}\" />"))
 }else if token.starts_with("col-") || token.starts_with("row-") {
  ("Places a child in its parent grid. Starts are one-based lines 1–13; spans cover 1–12 tracks. Placement changes visual layout without changing source, keyboard or accessibility order.",format!("<View className=\"grid grid-cols-2\"><View className=\"{class}\" /></View>"))
 }else if token=="grid" || token=="flex" || token=="inline-flex" || token.starts_with("flex-") {
  ("Selects grid/flex display, flex direction or flex wrapping. Containers default to flex column. Wrapping needs a finite width and children that cannot shrink to one line; grid uses source-order auto placement.",format!("<View className=\"{class}\" />"))
 }else if token.starts_with("gap-") || token.starts_with('p') && token.contains('-') {
  ("Sets row/column gaps or inner padding. p/px/py expand to independent sides; later classes win per side and state. 2/4/6 use spacing.sm/md/lg, 1 is sm/2, 3 averages sm/md; other numeric sizes use spacing.md/4 per unit. px is exactly one logical pixel and 0 is zero.",format!("<View className=\"{class}\" />"))
 }else if token.starts_with("w-") || token.starts_with("h-") || token.starts_with("min-") || token.starts_with("max-") || token.starts_with("size-") {
  ("Sets preferred, minimum or maximum width/height; size sets both preferred dimensions. Numeric sizes use the project spacing scale; px is one logical pixel. full and fractions use the parent content box. fit clamps the available size between min-content and max-content; unbreakable content can overflow. Percentage height needs a definite parent height; otherwise it resolves as auto. screen uses the native logical window size or browser viewport (100vw/100vh), including for min/max dimensions. auto uses content sizing; max-*-none removes that bound. Min/max constraints can override preferred size.",format!("<View className=\"{class}\" />"))
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
 format!("`{class}`\n\nShared web/native utility. {purpose} Container display/track/wrap utilities require View or a composed Button; text leaves reject them. No function arguments. aria-current matches a local link whose destination is the current page, an ancestor path segment or the current fragment; it does not change navigation. Metadata variants accept data-[slot=VALUE], data-[size=VALUE], data-[orientation=VALUE], data-[state=VALUE], data-horizontal, data-vertical, has-data-[slot=VALUE], has-data-[size=VALUE], in-data-[slot=VALUE], group-data-[slot=VALUE]/NAME, group-data-[size=VALUE]/NAME [.CLASS] and [a]. [a] matches only the current Link, including replacement-rendered links. Orientation metadata belongs to separator semantics. has-data matches descendants, excluding self. in-data matches any ancestor, excluding self; group variants additionally require the named group marker. Values/names use ASCII letters, digits and hyphens. Multiple metadata predicates compose by conjunction; at most one interaction state is supported. Metadata predicates change styling, never accessibility or source order. Bounded descendant selectors [&_svg] and [&_svg:not([class*='size-'])] accept pointer events, shrink and numeric size utilities; direct [&>a] and [&>a:hover] selectors accept text colors and decoration. Descendant rules style matching children, not the declaring parent. Local child classes override matching defaults. Descendant numeric sizes follow the target's nearest theme. Table row selectors [&_tr], [&_tr:last-child] and [&>tr:last-child] support border widths only; last-child refers to the actual final element sibling, including display-hidden siblings. has-aria-expanded matches an expanded Button descendant. These rules preserve retained children and apply on both targets. Other descendant forms reject. Unknown utilities and unsupported state prefixes fail at check/construction time. Example: `{example}`. With the default light theme: `{declarations}`. The enclosing theme supplies custom values; layout uses logical pixels. Viewport sm/md/lg/xl/2xl variants use 640/768/1024/1280/1536 logical pixels; dark matches the enclosing theme mode. Arbitrary CSS, container queries, named grid lines and subgrid remain unsupported. Ownership: changes layout/style without changing child ownership.")
}

#[cfg(test)] mod layout_tests {
 use super::*;
 #[test] fn table_header_height_uses_the_shared_spacing_scale() {
  assert!(tokens().iter().any(|token|token=="h-10"));
  let mut theme=Theme::default();
  for (spacing,height) in [(16.,40.),(20.,50.)] {
   theme.spacing.md=spacing;
   let rules=resolved("h-10",&theme,"light");
   assert!(rules.iter().any(|(_,value)|matches!(value,Utility::Dimension(1,Length::Pixels(actual)) if *actual==height)),"{rules:?}");
   assert!(rules.iter().any(|(_,value)|value.css()==format!("height:{height}px")));
  }
 }
 #[test] fn every_layout_token_resolves_and_keeps_property_order() {
  let theme=Theme::default();for token in layout::tokens(){assert!(validate(&token).is_ok(),"{token}");assert!(!resolved(&token,&theme,"light").is_empty(),"{token}");assert!(!documentation(&token).is_empty());}
  let rules=resolved("pl-6 p-4 pr-2 gap-x-6 gap-2 gap-y-4 size-8 w-12",&theme,"light");
  assert!(rules.iter().any(|(_,value)|matches!(value,Utility::Padding(3,16.))));
  assert!(rules.iter().any(|(_,value)|matches!(value,Utility::Padding(1,8.))));
  assert!(rules.iter().any(|(_,value)|matches!(value,Utility::GapAxis(0,16.))));
  assert!(rules.iter().any(|(_,value)|matches!(value,Utility::GapAxis(1,8.))));
  assert!(rules.iter().any(|(_,value)|matches!(value,Utility::Dimension(0,Length::Pixels(48.)))));
  assert!(rules.iter().any(|(_,value)|matches!(value,Utility::Dimension(1,Length::Pixels(32.)))));
  for token in ["grid-cols-0","grid-cols-13","col-start-0","col-span-13","max-w-auto","w-none","w-[-2px]","grid-cols-[2fr_1fr]"]{assert!(validate(token).is_err(),"{token}");}
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
 classes.split_whitespace().any(|class|split(class).is_ok_and(|(_,token)|matches!(token,"table"|"table-header-group"|"table-row-group"|"table-footer-group"|"table-row"|"table-cell"|"table-caption"|"flex"|"inline-flex"|"grid"|"flex-wrap"|"flex-nowrap"|"auto-rows-min") || token.starts_with("grid-cols-") || token.starts_with("grid-rows-")))
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

/// Inline text has typography, not independent layout boxes on either renderer.
pub(crate) fn validate_inline(classes:&str)->Result<(),String> {
 validate(classes)?;
 if resolved(classes,&Theme::default(),"light").iter().any(|(_,value)|!matches!(value,Utility::Tracking(_)|Utility::Balance(_)|Utility::WhiteSpace(_)|Utility::Underline(_)|Utility::UnderlineOffset(_)|Utility::FontSize(_)|Utility::Weight(_)|Utility::DefaultLeading(_)|Utility::Leading(_)|Utility::FontFamily(_)|Utility::Foreground(_))) {
  return Err("Inline Text and Link support typography and text colors; box/layout utilities require a surrounding View".into());
 }
 Ok(())
}

#[cfg(test)] mod positioning_tests {
 use super::*;
 #[test] fn position_sides_merge_independently_with_theme_percentages_and_resets(){
  let mut theme=Theme::default();theme.spacing.sm=10.;theme.spacing.md=20.;
  let rules=resolved("absolute inset-4 left-1/2 top-auto right-2 -bottom-px relative",&theme,"light");
  assert!(rules.iter().any(|(_,value)|matches!(value,Utility::Relative)));
  for (side,expected) in [(0,Length::Auto),(1,Length::Pixels(10.)),(2,Length::Pixels(-1.)),(3,Length::Percent(0.5))]{assert!(rules.iter().any(|(_,value)|matches!(value,Utility::Inset(actual,offset)if *actual==side&&*offset==expected)),"{rules:?}");}
  for token in ["absolute","inset-0","inset-x-auto","top-1/2","-left-full","right-18","-inset-y-2.5"]{assert!(validate(token).is_ok(),"{token}");assert!(documentation(token).contains("parent"));}
  for token in ["static","fixed","sticky","top-NaN","top-screen","top-[12px]","-top-auto","z-10"]{assert!(validate(token).is_err(),"{token}");}
  assert!(validate_inline("absolute").is_err());assert!(validate_inline("left-2").is_err());
 }
 #[test] fn variant_predicates_keep_ancestor_and_caller_override_rules(){
  let child=super::super::Node::view(vec![],"text-muted-foreground group-data-[variant=destructive]/alert:text-destructive/90 text-primary".into()).unwrap();
  let root=super::super::Node::view(vec![],"group/alert".into()).unwrap().with_semantics(r#"{"variant":"destructive","slot":"alert"}"#).unwrap();let theme=Theme::default();
  let base=super::super::Node::view(vec![],"text-muted-foreground group-data-[variant=destructive]/alert:text-destructive/90".into()).unwrap();let selected=resolved_node(&base,&[&root],&theme,"light");assert!(selected.iter().any(|(_,value)|matches!(value,Utility::Foreground(color)if *color==theme.color("light","destructive").unwrap().opacity(0.9))));
  let rules=resolved_node(&child,&[&root],&theme,"light");assert!(rules.iter().any(|(_,value)|matches!(value,Utility::Foreground(color)if *color==theme.color("light","primary").unwrap())));
  assert!(super::super::Node::view(vec![],String::new()).unwrap().with_semantics(r#"{"variant":"bad value"}"#).is_err());
  let html=super::super::document(&root,"Variant");assert!(html.contains("data-variant=\"destructive\""));
 }
}

#[cfg(test)] mod responsive_navigation_tests {
 use super::*;
 #[test] fn hidden_shares_display_precedence_and_documents_retained_ownership() {
  assert!(validate("hidden lg:flex").is_ok());assert_eq!(resolved("flex hidden",&Theme::default(),"light").last().unwrap().1.css(),"display:none");
  assert_eq!(resolved("hidden flex",&Theme::default(),"light").last().unwrap().1.css(),"display:flex");
  assert!(documentation("hidden").contains("does not dispose handlers"));
  assert!(css("hidden lg:flex",&Theme::default(),"light",1,"").contains("display:none"));
 }
}
