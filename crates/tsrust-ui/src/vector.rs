//! Checked vector data; source strings stay separate from renderer geometry.
use crate::{Child,Kind,Node};
#[cfg(any(test,feature="native"))]
use crate::theme::{Theme,ResolvedColor};
use kurbo::{Affine,BezPath};
const MAX_PATH_BYTES:usize=1_048_576;
const MAX_PATH_ELEMENTS:usize=100_000;
#[derive(Clone,Default,serde::Deserialize)]
#[serde(default,deny_unknown_fields,rename_all="camelCase")]
pub(crate) struct Paint {
 pub fill:Option<String>,pub stroke:Option<String>,pub stroke_width:Option<f64>,pub stroke_linecap:Option<String>,pub stroke_linejoin:Option<String>,pub stroke_miterlimit:Option<f64>,pub fill_rule:Option<String>,
}
impl Paint {
 fn validate(&self)->Result<(),String>{
  for value in [&self.fill,&self.stroke].into_iter().flatten(){paint(value)?;}
  if self.stroke_width.is_some_and(|value|!value.is_finite()||value<0.||!(value as f32).is_finite()){return Err("SVG strokeWidth must be finite, nonnegative and fit native layout precision".into());}
  if self.stroke_miterlimit.is_some_and(|value|!value.is_finite()||value<1.||!(value as f32).is_finite()){return Err("SVG strokeMiterlimit must be finite and at least 1".into());}
  if self.stroke_linecap.as_deref().is_some_and(|value|!matches!(value,"butt"|"round"|"square")){return Err("SVG strokeLinecap supports butt, round or square".into());}
  if self.stroke_linejoin.as_deref().is_some_and(|value|!matches!(value,"miter"|"round"|"bevel")){return Err("SVG strokeLinejoin supports miter, round or bevel".into());}
  if self.fill_rule.as_deref().is_some_and(|value|!matches!(value,"nonzero"|"evenodd")){return Err("SVG fillRule supports nonzero or evenodd".into());}Ok(())
 }
 pub(crate) fn inherited(&self,parent:&Self)->Self {Self{fill:self.fill.clone().or_else(||parent.fill.clone()),stroke:self.stroke.clone().or_else(||parent.stroke.clone()),stroke_width:self.stroke_width.or(parent.stroke_width),stroke_linecap:self.stroke_linecap.clone().or_else(||parent.stroke_linecap.clone()),stroke_linejoin:self.stroke_linejoin.clone().or_else(||parent.stroke_linejoin.clone()),stroke_miterlimit:self.stroke_miterlimit.or(parent.stroke_miterlimit),fill_rule:self.fill_rule.clone().or_else(||parent.fill_rule.clone())}}
 fn attributes(&self)->Vec<(&'static str,String)>{
  let mut values=Vec::new();for (key,value) in [("fill",&self.fill),("stroke",&self.stroke),("stroke-linecap",&self.stroke_linecap),("stroke-linejoin",&self.stroke_linejoin),("fill-rule",&self.fill_rule)]{if let Some(value)=value{values.push((key,if matches!(key,"fill"|"stroke")&&crate::theme::COLORS.contains(&value.as_str()){format!("var(--ui-{value})")}else{value.clone()}));}}
  for (key,value) in [("stroke-width",self.stroke_width),("stroke-miterlimit",self.stroke_miterlimit)]{if let Some(value)=value{values.push((key,value.to_string()));}}values
 }
}
fn paint(value:&str)->Result<(),String>{if matches!(value,"none"|"currentColor")||crate::theme::COLORS.contains(&value){Ok(())}else{crate::theme::literal_color(value).map(|_|())}}
#[cfg(any(test,feature="native"))]
pub(crate) fn color(value:&str,current:ResolvedColor,theme:&Theme,mode:&str)->Result<Option<ResolvedColor>,String>{match value{"none"=>Ok(None),"currentColor"=>Ok(Some(current)),value if crate::theme::COLORS.contains(&value)=>theme.color(mode,value).map(Some),value=>crate::theme::literal_color(value).map(Some)}}
#[derive(Clone,serde::Deserialize)]
#[serde(default,deny_unknown_fields,rename_all="camelCase")]
pub(crate) struct Root {
 pub view_box:String,pub width:f64,pub height:f64,pub preserve_aspect_ratio:String,
 pub paint:Paint,
 #[serde(skip)]pub bounds:[f64;4],#[serde(skip)]pub aspect:Aspect,
}
impl Default for Root {fn default()->Self{Self{view_box:"0 0 24 24".into(),width:24.,height:24.,preserve_aspect_ratio:"xMidYMid meet".into(),paint:Paint{fill:Some("currentColor".into()),stroke:Some("none".into()),stroke_width:Some(1.),stroke_linecap:Some("butt".into()),stroke_linejoin:Some("miter".into()),stroke_miterlimit:Some(4.),fill_rule:Some("nonzero".into())},bounds:[0.,0.,24.,24.],aspect:Default::default()}}}
#[derive(Clone,Copy)]
pub(crate) struct Aspect {pub align:Option<(u8,u8)>,pub slice:bool}
impl Default for Aspect {fn default()->Self{Self{align:Some((1,1)),slice:false}}}
impl Root {
 fn parse(options:&str)->Result<Self,String>{
  let mut value:Self=serde_json::from_str(options).map_err(|error|format!("Invalid Svg options: {error}"))?;value.paint=value.paint.inherited(&Self::default().paint);value.paint.validate()?;
  if [value.width,value.height].iter().any(|value|!value.is_finite()||*value<0.||!(*value as f32).is_finite()){return Err("SVG width/height must be finite nonnegative logical pixels that fit native layout precision".into());}
  let bounds=value.view_box.parse::<svgtypes::ViewBox>().map_err(|error|format!("Invalid SVG viewBox: {error}"))?;
  value.bounds=[bounds.x,bounds.y,bounds.w,bounds.h];
  if value.bounds.iter().any(|value|!value.is_finite()||!(*value as f32).is_finite())||bounds.w<=0.||bounds.h<=0.||bounds.w as f32==0.||bounds.h as f32==0.{return Err("SVG viewBox requires finite x/y and positive width/height that fit native layout precision".into());}
  if value.preserve_aspect_ratio.trim().is_empty(){return Err("SVG preserveAspectRatio cannot be empty".into());}
  let aspect=value.preserve_aspect_ratio.parse::<svgtypes::AspectRatio>().map_err(|error|format!("Invalid SVG aspect policy: {error}"))?;
  if aspect.defer{return Err("SVG defer aspect policy is not implemented".into());}
  use svgtypes::Align;
  value.aspect=Aspect{align:match aspect.align {Align::None=>None,Align::XMinYMin=>Some((0,0)),Align::XMidYMin=>Some((1,0)),Align::XMaxYMin=>Some((2,0)),Align::XMinYMid=>Some((0,1)),Align::XMidYMid=>Some((1,1)),Align::XMaxYMid=>Some((2,1)),Align::XMinYMax=>Some((0,2)),Align::XMidYMax=>Some((1,2)),Align::XMaxYMax=>Some((2,2))},slice:aspect.slice};
  value.transform(value.width,value.height)?;Ok(value)
 }
 pub(crate) fn transform(&self,width:f64,height:f64)->Result<Affine,String>{
  let [x,y,w,h]=self.bounds;let (sx,sy)=(width/w,height/h);
  let transform=if let Some((ax,ay))=self.aspect.align{let scale=if self.aspect.slice{sx.max(sy)}else{sx.min(sy)};Affine::translate(((width-w*scale)*ax as f64/2.,(height-h*scale)*ay as f64/2.))*Affine::scale(scale)*Affine::translate((-x,-y))}
  else{Affine::scale_non_uniform(sx,sy)*Affine::translate((-x,-y))};
  if transform.as_coeffs().iter().any(|value|!value.is_finite()||!(*value as f32).is_finite()){return Err("SVG viewport transform exceeds native geometry precision".into());}Ok(transform)
 }
}
pub(crate) struct Path {pub d:String,pub geometry:BezPath,pub paint:Paint,pub binding:Option<Box<dyn FnMut()->Result<String,String>>>}
fn geometry(value:&str)->Result<BezPath,String>{
 if value.len()>MAX_PATH_BYTES{return Err(format!("SVG path data exceeds {MAX_PATH_BYTES} UTF-8 bytes"));}
 // Validate source operands before arc conversion; the geometry library is deliberately permissive.
 for (index,segment) in svgtypes::PathParser::from(value).enumerate(){
  if index>=MAX_PATH_ELEMENTS{return Err(format!("SVG path exceeds {MAX_PATH_ELEMENTS} source segments"));}
  use svgtypes::PathSegment::*;
  let values=match segment.map_err(|error|format!("Invalid SVG path: {error}"))? {
   MoveTo{x,y,..}|LineTo{x,y,..}|SmoothQuadratic{x,y,..}=>[x,y,0.,0.,0.,0.,0.],HorizontalLineTo{x,..}=>[x,0.,0.,0.,0.,0.,0.],VerticalLineTo{y,..}=>[y,0.,0.,0.,0.,0.,0.],
   CurveTo{x1,y1,x2,y2,x,y,..}=>[x1,y1,x2,y2,x,y,0.],SmoothCurveTo{x2,y2,x,y,..}=>[x2,y2,x,y,0.,0.,0.],Quadratic{x1,y1,x,y,..}=>[x1,y1,x,y,0.,0.,0.],
   EllipticalArc{rx,ry,x_axis_rotation,x,y,..}=>{if [rx,ry].iter().any(|value|value.abs()>0.&&value.abs()<=0.00001){return Err("Nonzero SVG arc radii must exceed 0.00001 viewBox units for native geometry".into());}[rx,ry,x_axis_rotation,x,y,0.,0.]},ClosePath{..}=>[0.;7],
  };
  if values.iter().any(|value|!value.is_finite()||!(*value as f32).is_finite()){return Err("SVG path operands must be finite and fit native geometry precision".into());}
 }
 let mut path=BezPath::new();
 for segment in svgtypes::SimplifyingPathParser::from(value){
  use svgtypes::SimplePathSegment::*;
  match segment.map_err(|error|format!("Invalid SVG path: {error}"))?{MoveTo{x,y}=>path.move_to((x,y)),LineTo{x,y}=>path.line_to((x,y)),CurveTo{x1,y1,x2,y2,x,y}=>path.curve_to((x1,y1),(x2,y2),(x,y)),Quadratic{x1,y1,x,y}=>path.quad_to((x1,y1),(x,y)),ClosePath=>path.close_path()}
  if path.elements().len()>MAX_PATH_ELEMENTS{return Err(format!("SVG path exceeds {MAX_PATH_ELEMENTS} elements"));}
 }
 if !path.is_finite()||path.elements().iter().any(|element|match element{kurbo::PathEl::MoveTo(p)|kurbo::PathEl::LineTo(p)=>![p.x,p.y].iter().all(|v|(*v as f32).is_finite()),kurbo::PathEl::QuadTo(a,b)=>![a.x,a.y,b.x,b.y].iter().all(|v|(*v as f32).is_finite()),kurbo::PathEl::CurveTo(a,b,c)=>![a.x,a.y,b.x,b.y,c.x,c.y].iter().all(|v|(*v as f32).is_finite()),kurbo::PathEl::ClosePath=>false}){return Err("SVG path coordinates must be finite and fit native geometry precision".into());}
 if path.elements().len()>MAX_PATH_ELEMENTS{return Err(format!("SVG path exceeds {MAX_PATH_ELEMENTS} elements"));}Ok(path)
}
pub(crate) enum Data {Root(Root),Path(Path)}
impl Data {
 pub(crate) fn attributes(&self)->Vec<(&'static str,String)>{match self{
  Self::Root(value)=>{let mut attrs=value.paint.attributes();attrs.extend([("viewBox",value.view_box.clone()),("width",value.width.to_string()),("height",value.height.to_string()),("preserveAspectRatio",value.preserve_aspect_ratio.clone()),("xmlns","http://www.w3.org/2000/svg".into())]);attrs},
  Self::Path(value)=>{let mut attrs=value.paint.attributes();attrs.push(("d",value.d.clone()));attrs},
 }}
 #[cfg(any(test,feature="native",all(target_arch="wasm32",feature="ui-web")))]
 pub(crate) fn refresh(&mut self)->Result<usize,String>{if let Self::Path(value)=self{if let Some(binding)=&mut value.binding{let next=binding()?;if next!=value.d{let parsed=geometry(&next)?;value.geometry=parsed;value.d=next;return Ok(1);}}}Ok(0)}
}
impl Node {
 pub fn svg(children:Vec<Child>,class_name:String,options:&str)->Result<Self,String>{
  let options=Root::parse(options)?;let mut paths=Vec::new();for child in children{match child{Child::Node(node) if node.kind==Kind::SvgPath=>paths.push(node),_=>return Err("Svg children must be Path nodes; use an empty d value to hide a retained path".into())}}
  let mut node=Self::new(Kind::Svg,class_name,String::new(),paths)?;node.vector=Some(Box::new(Data::Root(options)));node.semantics.hidden=true;node.semantics.role="presentation".into();Ok(node)
 }
 pub fn svg_path(d:&str,class_name:String,options:&str)->Result<Self,String>{let paint:Paint=serde_json::from_str(options).map_err(|error|format!("Invalid Path options: {error}"))?;paint.validate()?;let geometry=geometry(d)?;let mut node=Self::new(Kind::SvgPath,class_name,String::new(),vec![])?;node.vector=Some(Box::new(Data::Path(Path{d:d.into(),geometry,paint,binding:None})));Ok(node)}
 pub fn with_path_binding(mut self,mut binding:impl FnMut()->Result<String,String>+'static)->Result<Self,String>{let Some(Data::Path(value))=self.vector.as_deref_mut() else{return Err("A path binding requires Path".into());};let next=binding()?;let parsed=geometry(&next)?;value.geometry=parsed;value.d=next;value.binding=Some(Box::new(binding));Ok(self)}
 pub fn validate_root(&self)->Result<(),String>{if self.kind==Kind::SvgPath{return Err("Path requires an enclosing Svg".into());}self.validate_structure()}
}

#[cfg(test)]mod tests {
 use super::*;
 use std::{cell::{Cell,RefCell},rc::Rc};
 fn path(d:&str)->Node{Node::svg_path(d,String::new(),"{}").unwrap()}
 #[test]fn viewport_mapping_preserves_offsets_alignment_and_aspect_policy(){
  for (policy,expected) in [("xMidYMid meet",[0.,50.,200.,150.]),("xMinYMin meet",[0.,0.,200.,100.]),("xMaxYMax meet",[0.,100.,200.,200.]),("xMidYMid slice",[-100.,0.,300.,200.]),("none",[0.,0.,200.,200.])]{
   let root=Root::parse(&format!("{{\"viewBox\":\"10 20 40 20\",\"width\":200,\"height\":200,\"preserveAspectRatio\":\"{policy}\"}}")).unwrap();let matrix=root.transform(200.,200.).unwrap();let a=matrix*kurbo::Point::new(10.,20.);let b=matrix*kurbo::Point::new(50.,40.);assert_eq!([a.x,a.y,b.x,b.y],expected,"{policy}");println!("{}",serde_json::json!({"case":"vector-viewport","policy":policy,"width":200.,"height":200.,"corners":[a.x,a.y,b.x,b.y]}));
  }
 }
 #[test]fn paints_resolve_against_the_current_scoped_theme(){
  let theme=Theme::default();let current=theme.color("dark","foreground").unwrap();
  assert_eq!(color("none",current,&theme,"dark").unwrap(),None);
  assert_eq!(color("currentColor",current,&theme,"dark").unwrap(),Some(current));
  assert_eq!(color("primary",current,&theme,"dark").unwrap(),Some(theme.color("dark","primary").unwrap()));
  assert!(color("url(secret)",current,&theme,"dark").is_err());
 }
 #[test]fn geometry_and_paint_reject_instead_of_emitting_invalid_svg(){
  for data in ["M0", "M0 0 L NaN 1","M0 0 L1e100 1","M0 0 X2 2", "<script/>", "M0 0 ?", "M0 0 A0.000001 2 0 0 1 10 10"]{assert!(Node::svg_path(data,String::new(),"{}").is_err(),"{data}");}
  for options in [r#"{"viewBox":"0 0 0 24"}"#,r#"{"viewBox":"0,,0,24,24"}"#,r#"{"width":-1}"#,r#"{"viewBox":"0 0 1e-38 1e-38"}"#,r#"{"preserveAspectRatio":"none garbage"}"#,r#"{"paint":{"strokeWidth":-1}}"#,r#"{"paint":{"fill":"url(https://example.com/x)"}}"#]{assert!(Node::svg(vec![],String::new(),options).is_err(),"{options}");}
  for classes in ["w-auto","h-fit","min-w-fit"]{assert!(Node::svg(vec![],classes.into(),"{}").is_err());}
  assert!(Node::svg_path("",String::new(),"{}").is_ok());assert!(Node::svg_path("M0 0L1 1", "hover:text-primary".into(), "{}").is_err());
  assert!(Node::view(vec![Child::Node(path(""))],String::new()).is_err());assert!(path("").validate_root().is_err());assert!(Node::svg(vec![Child::Node(Node::text(vec![],String::new()).unwrap())],String::new(),"{}").is_err());
 }
 struct Release(Rc<Cell<usize>>);impl Drop for Release{fn drop(&mut self){self.0.set(self.0.get()+1);}}
 #[test]fn strict_parser_handles_implicit_plus_commands_and_curved_geometry(){
  use kurbo::Shape;
  let line=geometry("M0 0 +1+2 +3+4").unwrap();assert_eq!(line.elements().last(),Some(&kurbo::PathEl::LineTo(kurbo::Point::new(3.,4.))));
  let circle=geometry("M2 12 A10 10 0 1 0 22 12 A10 10 0 1 0 2 12 Z").unwrap();let bounds=circle.bounding_box();for (actual,wanted) in [(bounds.x0,2.),(bounds.y0,2.),(bounds.x1,22.),(bounds.y1,22.)]{assert!((actual-wanted).abs()<0.01,"{bounds:?}");}
  assert!(geometry("M0 0 +").is_err());assert!(geometry("M0 0,,").is_err());
 }
 #[test]fn live_paths_keep_identity_preserve_source_and_release_evaluators(){
  let source=Rc::new(RefCell::new("M0 0 L 12 24".to_owned()));let released=Rc::new(Cell::new(0));let probe=Release(released.clone());let state=source.clone();
  let child=path("").with_path_binding(move||{let _keep=&probe;Ok(state.borrow().clone())}).unwrap();let mut root=Node::svg(vec![Child::Node(child)],String::new(),"{}").unwrap();root.refresh().unwrap();let identity=root.children[0].identity.get();
  *source.borrow_mut()="M1 2 L 8 9".into();assert_eq!(root.refresh().unwrap(),1);assert_eq!(root.children[0].identity.get(),identity);let Data::Path(data)=root.children[0].vector.as_deref().unwrap()else{panic!("path")};assert_eq!(data.d,"M1 2 L 8 9");
  *source.borrow_mut()="M0 0 ?".into();assert!(root.refresh().is_err());let Data::Path(data)=root.children[0].vector.as_deref().unwrap()else{panic!("path")};assert_eq!(data.d,"M1 2 L 8 9");assert_eq!(released.get(),0);drop(root);assert_eq!(released.get(),1);
 }
 #[test]fn named_images_and_decorative_icons_have_explicit_semantics(){
  let root=Node::svg(vec![Child::Node(path("M2 12 L10 20 L22 4"))],"size-6".into(),"{}").unwrap();assert!(root.semantics.hidden);let html=crate::document(&root,"Icon");assert!(html.contains("xmlns=\"http://www.w3.org/2000/svg\""));assert!(html.contains("aria-hidden=\"true\""));assert!(html.contains("d=\"M2 12 L10 20 L22 4\""));
  let root=root.with_semantics(r#"{"role":"img","label":"Saved"}"#).unwrap();assert!(!root.semantics.hidden);assert_eq!(root.content_text(),"Saved");assert!(Node::svg(vec![],String::new(),"{}").unwrap().with_semantics(r#"{"role":"img"}"#).is_err());
 }
}
