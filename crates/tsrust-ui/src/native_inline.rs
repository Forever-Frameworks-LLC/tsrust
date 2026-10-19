//! Inline descendants share shaping while retaining semantic identity and hit geometry.
use super::*;
use std::ops::Range;
#[derive(Clone,Copy,Default,PartialEq,Debug)]
pub(super) struct Brush {pub color:Option<crate::theme::ResolvedColor>,pub underline:bool,pub strike:bool}
pub(super) struct Inline {
    pub identity:u64,pub semantics:crate::Semantics,pub label:String,accessible_label:String,pub range:Range<usize>,pub children:Vec<Inline>,style:TextStyle,ring:crate::theme::ResolvedColor,
}
pub(super) fn collect(item:Item,text:&mut String,spans:&mut Vec<(Range<usize>,TextStyle)>)->Inline {
    let start=text.len();let mut children=Vec::new();
    if item.inline.is_empty() && item.children.is_empty(){text.push_str(&item.raw_text);spans.push((start..text.len(),item.text_style));}
    else if !item.inline.is_empty(){
        fn append(part:Inline,text:&mut String,spans:&mut Vec<(Range<usize>,TextStyle)>)->Inline {
            let start=text.len();let mut children=Vec::new();
            if part.children.is_empty(){text.push_str(&part.label);spans.push((start..text.len(),part.style));}
            else{for child in part.children {children.push(append(child,text,spans));}}
            Inline{range:start..text.len(),children,..part}
        }
        for part in item.inline{children.push(append(part,text,spans));}
    }else{for child in item.children{children.push(collect(child,text,spans));}}
    Inline{identity:item.identity,semantics:item.semantics,label:text[start..].to_owned(),accessible_label:item.label,range:start..text.len(),children,style:item.text_style,ring:item.ring}
}
pub(super) fn default_style(builder:&mut parley::RangedBuilder<'_,Brush>,style:TextStyle){
    builder.push_default(StyleProperty::Brush(Brush{color:None,underline:style.underline,strike:style.strike}));
    builder.push_default(StyleProperty::FontStyle(if style.italic{parley::style::FontStyle::Italic}else{parley::style::FontStyle::Normal}));
}
pub(super) fn range_style(builder:&mut parley::RangedBuilder<'_,Brush>,range:Range<usize>,style:TextStyle){
    builder.push(StyleProperty::FontSize(style.size),range.clone());
    builder.push(StyleProperty::FontWeight(FontWeight::new(style.weight)),range.clone());
    builder.push(StyleProperty::FontStyle(if style.italic{parley::style::FontStyle::Italic}else{parley::style::FontStyle::Normal}),range.clone());
    builder.push(StyleProperty::LineHeight(style.leading.map(parley::style::LineHeight::FontSizeRelative).unwrap_or(style.line_height)),range.clone());
    builder.push(StyleProperty::from(match style.family{1=>parley::style::GenericFamily::Serif,2=>parley::style::GenericFamily::Monospace,_=>parley::style::GenericFamily::SystemUi}),range.clone());
    builder.push(StyleProperty::Brush(Brush{color:Some(style.color),underline:style.underline,strike:style.strike}),range);
}
fn rects(part:&Inline,text:&Layout<Brush>,origin:(f64,f64))->Vec<vello::kurbo::Rect>{
    let selection=parley::editing::Selection::new(parley::editing::Cursor::from_byte_index(text,part.range.start,parley::Affinity::Downstream),parley::editing::Cursor::from_byte_index(text,part.range.end,parley::Affinity::Upstream));
    let mut rects=Vec::new();selection.geometry_with(text,|r,_|rects.push(vello::kurbo::Rect::new(origin.0+r.x0,origin.1+r.y0,origin.0+r.x1,origin.1+r.y1)));rects
}
pub(super) fn accessible(part:&Inline,text:&Layout<Brush>,origin:(f64,f64),scale:f64,nodes:&mut Vec<(accesskit::NodeId,accesskit::Node)>)->accesskit::NodeId {
    let id=accesskit::NodeId(part.identity);let role=if !part.semantics.href.is_empty(){accesskit::Role::Link}else{match part.semantics.tag.as_str(){"strong"=>accesskit::Role::Strong,"em"=>accesskit::Role::Emphasis,"del"=>accesskit::Role::ContentDeletion,"code"=>accesskit::Role::Code,_=>if part.children.is_empty(){accesskit::Role::Label}else{accesskit::Role::GenericContainer}}};
    let role=match part.semantics.role.as_str(){"alert"=>accesskit::Role::Alert,"status"=>accesskit::Role::Status,"group"=>accesskit::Role::Group,"note"=>accesskit::Role::Note,"region"=>accesskit::Role::Region,"separator"=>accesskit::Role::Splitter,"none"|"presentation"=>accesskit::Role::GenericContainer,_=>role};
    let mut node=accesskit::Node::new(role);
    if part.semantics.role=="alert"{node.set_live(accesskit::Live::Assertive);}else if part.semantics.role=="status"{node.set_live(accesskit::Live::Polite);}
    if part.semantics.invalid=="true"{node.set_invalid(accesskit::Invalid::True);}
    if part.semantics.role=="separator"{node.set_orientation(if part.semantics.orientation=="vertical"{accesskit::Orientation::Vertical}else{accesskit::Orientation::Horizontal});}
    if part.semantics.hidden{node.set_hidden();}
    if part.children.is_empty()||role==accesskit::Role::Link||!part.semantics.label.is_empty() {node.set_label(if part.semantics.label.is_empty(){part.accessible_label.clone()}else{part.semantics.label.clone()});}
    if role==accesskit::Role::Link && super::super::navigation::is_current(&part.semantics.href){node.set_aria_current(accesskit::AriaCurrent::Page);}
    if role==accesskit::Role::Link {node.add_action(accesskit::Action::Click);node.add_action(accesskit::Action::Focus);}
    if let Some(bounds)=rects(part,text,origin).into_iter().reduce(|a,b|a.union(b)){node.set_bounds(accesskit::Rect::new(bounds.x0*scale,bounds.y0*scale,bounds.x1*scale,bounds.y1*scale));}
    node.set_children(part.children.iter().map(|child|accessible(child,text,origin,scale,nodes)).collect::<Vec<_>>());nodes.push((id,node));id
}
pub(super) fn paint(part:&Inline,text:&Layout<Brush>,scene:&mut Scene,origin:(f64,f64),transform:Affine,clips:&[vello::kurbo::BezPath],hits:&mut std::collections::BTreeMap<u64,Vec<vello::kurbo::BezPath>>,cursors:&mut std::collections::BTreeMap<u64,u8>,focus:Option<u64>){
    let mut path=vello::kurbo::BezPath::new();
    for rect in rects(part,text,origin){path.extend(rect.to_path(0.1).elements().iter().copied());if focus==Some(part.identity){scene.stroke(&vello::kurbo::Stroke::new(2.),transform,paint_color(part.ring),None,&rect.inflate(1.,1.));}}
    path.apply_affine(transform);let mut regions=clips.to_vec();regions.push(path);if !part.style.pointer_events{regions.push(vello::kurbo::BezPath::new());}hits.insert(part.identity,regions);cursors.insert(part.identity,if part.semantics.href.is_empty(){1}else{3});
    for child in &part.children {paint(child,text,scene,origin,transform,clips,hits,cursors,focus);}
}
impl Painter {
    pub(super) fn inline_layout(&self,item:&Item)->Result<Layout<Brush>,String>{
        let layout=self.tree.layout(item.id).map_err(|e|e.to_string())?;
        let width=(layout.size.width-layout.padding.left-layout.padding.right-layout.border.left-layout.border.right).max(0.);
        let mut text=item.text.clone().ok_or("Inline content requires a text layout")?;layout_text(&mut text,width,false);text.align(match item.align{1=>Alignment::Center,2=>Alignment::End,_=>Alignment::Start},AlignmentOptions::default());Ok(text)
    }
}
#[cfg(test)]mod tests{
 use super::*;
 fn text(value:&str,tag:&str,class:&str)->Node{Node::text(vec![crate::TextPart::Literal(value.into())],class.into()).unwrap().with_semantics(&format!("{{\"tag\":\"{tag}\"}}" )).unwrap()}
 #[test]fn inline_wrapping_preserves_styles_link_geometry_and_accessibility(){
  use crate::Child;
  let location=crate::navigation::location(|value|value,|value:&String|value.clone()).unwrap();
  let decoration=text("↗ ","span","").with_semantics("{\"hidden\":true}").unwrap();
  let link=crate::navigation::link(vec![Child::Node(decoration),Child::Text(crate::TextPart::Literal("café 文 and a long destination".into()))],"text-primary".into(),"/docs").unwrap();
  let mut root=Node::text_children(vec![Child::Text(crate::TextPart::Literal("Before ".into())),Child::Node(text("bold ","strong","text-syntax-keyword")),Child::Node(text("italic ","em","")),Child::Node(link),Child::Text(crate::TextPart::Literal(" after".into()))],"text-sm".into()).unwrap().with_semantics("{\"tag\":\"p\"}").unwrap();root.refresh().unwrap();
  let mut painter=Painter::new();let (_,tree)=painter.scene(&root,120.,1.,None).unwrap();let (_,link)=tree.nodes.iter().find(|(_,node)|node.role()==accesskit::Role::Link).unwrap();assert_eq!(link.label(),Some("café 文 and a long destination"));assert!(link.supports_action(accesskit::Action::Click));assert!(link.bounds().unwrap().height()>16.);
  let item=painter.node(&root,TextStyle::default()).unwrap();assert!(item.children.is_empty());assert_eq!(item.inline[1].style.weight,700.);assert!(item.inline[2].style.italic);let mut paragraph=item.text.unwrap();paragraph.break_all_lines(Some(120.));assert!(paragraph.lines().count()>2);let mut colors=Vec::new();for line in paragraph.lines(){for item in line.items(){if let PositionedLayoutItem::GlyphRun(run)=item{if let Some(color)=run.style().brush.color{colors.push(color);}}}}assert!(colors.contains(&crate::theme::Theme::default().color("light","syntax-keyword").unwrap()));assert!(colors.contains(&crate::theme::Theme::default().color("light","primary").unwrap()));
  let link_id=tree.nodes.iter().find(|(_,node)|node.role()==accesskit::Role::Link).unwrap().0.0;assert!(painter.hit_regions.contains_key(&link_id));assert!(painter.scroll_chains.contains_key(&link_id));assert!(root.activate(link_id).unwrap());assert_eq!(location.get().unwrap(),"/docs");
 }
}

pub(super) struct Selectable {value:String,layout:Layout<Brush>,selection:parley::editing::Selection,origin:(f64,f64)}
impl Selectable {
 pub fn new(value:String,layout:Layout<Brush>,origin:(f64,f64))->Self{let selection=parley::editing::Selection::from_byte_index(&layout,0,parley::Affinity::Downstream);Self{value,layout,selection,origin}}
 pub fn sync(&mut self,value:&str,layout:&Layout<Brush>,origin:(f64,f64)){
  if self.value!=value {self.value=value.into();self.selection=parley::editing::Selection::from_byte_index(layout,0,parley::Affinity::Downstream);}else{self.selection=self.selection.refresh(layout);}
  self.layout=layout.clone();self.origin=origin;
 }
 pub fn point(&mut self,point:(f64,f64),extend:bool){let x=(point.0-self.origin.0) as f32;let y=(point.1-self.origin.1) as f32;self.selection=if extend{self.selection.extend_to_point(&self.layout,x,y)}else{parley::editing::Selection::from_point(&self.layout,x,y)};}
 pub fn selected(&self)->Option<String>{(!self.selection.is_collapsed()).then(||self.value[self.selection.text_range()].to_owned())}
 pub fn all(&mut self){self.selection=parley::editing::Selection::new(parley::editing::Cursor::from_byte_index(&self.layout,0,parley::Affinity::Downstream),parley::editing::Cursor::from_byte_index(&self.layout,self.value.len(),parley::Affinity::Upstream));}
 pub fn paint(&self,scene:&mut Scene,transform:Affine,ring:crate::theme::ResolvedColor){self.selection.geometry_with(&self.layout,|r,_|scene.fill(Fill::NonZero,transform,paint_color(ring.opacity(0.25)),None,&vello::kurbo::Rect::new(r.x0+self.origin.0,r.y0+self.origin.1,r.x1+self.origin.0,r.y1+self.origin.1)));}
 pub fn navigation(&mut self,key:&Key,extend:bool,word:bool)->bool {
  let s=self.selection;let text=&self.layout;self.selection=match key {
   Key::Named(NamedKey::ArrowLeft)=>if word{s.previous_visual_word(text,extend)}else{s.previous_visual(text,extend)},
   Key::Named(NamedKey::ArrowRight)=>if word{s.next_visual_word(text,extend)}else{s.next_visual(text,extend)},
   Key::Named(NamedKey::ArrowUp)=>s.previous_line(text,extend),Key::Named(NamedKey::ArrowDown)=>s.next_line(text,extend),
   Key::Named(NamedKey::Home)=>s.line_start(text,extend),Key::Named(NamedKey::End)=>s.line_end(text,extend),_=>return false,
  };true
 }
}

impl App {
 pub(super) fn select_text(&mut self,extend:bool){
  let scale=self.active.as_ref().map(|active|active.window.scale_factor()).unwrap_or(1.);
  if let Some(text)=self.painter.selected_text.and_then(|id|self.painter.readonly.get_mut(&id)){text.point((self.pointer.0/scale,self.pointer.1/scale),extend);}
 }
 pub(super) fn readonly_key(&mut self,event:&winit::event::KeyEvent)->bool {
  if event.state!=ElementState::Pressed{return false;}
  let Some(id)=self.painter.selected_text else{return false;};let Some(text)=self.painter.readonly.get_mut(&id) else{return false;};
  let command=if cfg!(target_os="macos"){self.modifiers.super_key()}else{self.modifiers.control_key()};let word=if cfg!(target_os="macos"){self.modifiers.alt_key()}else{command};
  let letter=match &event.logical_key{Key::Character(value)=>value.to_lowercase(),_=>String::new()};
  if command && letter=="a" {text.all();return true;}
  if command && letter=="c" {if let Some(value)=text.selected(){if let Err(error)=self.clipboard().and_then(|clipboard|clipboard.set_text(value).map_err(|error|error.to_string())){eprintln!("Clipboard copy failed: {error}");}}return true;}
  if event.logical_key==Key::Named(NamedKey::Escape){self.painter.selected_text=None;return true;}
  text.navigation(&event.logical_key,self.shift,word)
 }
}
#[cfg(test)]mod selection_tests{
 use super::*;
 #[test]fn select_all_keeps_unicode_crlf_across_styles_and_resets_on_value_change(){
  let raw="café 文\r\n  ";let mut painter=Painter::new();let node=Node::text(vec![crate::TextPart::Literal(raw.into())],"font-mono".into()).unwrap();let item=painter.node(&node,TextStyle::default()).unwrap();let mut layout=item.text.unwrap();layout.break_all_lines(Some(60.));let mut selection=Selectable::new(raw.into(),layout.clone(),(0.,0.));selection.all();assert_eq!(selection.selected().as_deref(),Some(raw));assert!(selection.navigation(&Key::Named(NamedKey::ArrowLeft),true,false));assert!(raw.is_char_boundary(selection.selection.focus().index()));selection.sync("new",&layout,(0.,0.));assert!(selection.selected().is_none());
 }
}
