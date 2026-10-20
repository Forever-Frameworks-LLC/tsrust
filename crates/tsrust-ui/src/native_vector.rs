//! SVG viewport mapping and paths use Vello's existing geometry pipeline.
use super::*;
use crate::vector::{Data,Root};
use kurbo::Shape;
pub(super) struct Vector {root:Root,paths:Vec<Path>}
struct Path {geometry:kurbo::BezPath,fill:Option<crate::theme::ResolvedColor>,stroke:Option<crate::theme::ResolvedColor>,style:kurbo::Stroke,rule:Fill,opacity:f32}
pub(super) fn collect(node:&Node,current:crate::theme::ResolvedColor,theme:&crate::theme::Theme,mode:&str,ancestors:&[&Node],variables:&std::collections::BTreeMap<String,f32>,width:f32)->Result<Option<Box<Vector>>,String>{
 let Some(Data::Root(root))=node.vector.as_deref() else{return Ok(None)};let mut paths=Vec::new();
 for child in &node.children {
  let Some(Data::Path(path))=child.vector.as_deref() else{return Err("Svg requires Path children".into())};let mut color=current;let mut opacity=1.;
  for (_,value) in crate::style::resolved_scoped_node_at_width(child,ancestors,theme,mode,variables,width).0 {match value{Utility::Foreground(value)=>color=value,Utility::Opacity(value)=>opacity=value,_=>{}}}
  let paint=path.paint.inherited(&root.paint);let stroke=kurbo::Stroke::new(paint.stroke_width.unwrap_or(1.)).with_caps(match paint.stroke_linecap.as_deref(){Some("round")=>kurbo::Cap::Round,Some("square")=>kurbo::Cap::Square,_=>kurbo::Cap::Butt}).with_join(match paint.stroke_linejoin.as_deref(){Some("round")=>kurbo::Join::Round,Some("bevel")=>kurbo::Join::Bevel,_=>kurbo::Join::Miter}).with_miter_limit(paint.stroke_miterlimit.unwrap_or(4.));
  paths.push(Path{geometry:path.geometry.clone(),fill:crate::vector::color(paint.fill.as_deref().unwrap_or("currentColor"),color,theme,mode)?,stroke:crate::vector::color(paint.stroke.as_deref().unwrap_or("none"),color,theme,mode)?,style:stroke,rule:if paint.fill_rule.as_deref()==Some("evenodd"){Fill::EvenOdd}else{Fill::NonZero},opacity});
 }
 Ok(Some(Box::new(Vector{root:root.clone(),paths})))
}
pub(super) fn paint(vector:&Vector,scene:&mut Scene,transform:Affine,width:f64,height:f64)->Result<(),String>{
 if width<=0.||height<=0.{return Ok(());}
 let transform=transform*vector.root.transform(width,height)?;
 if transform.as_coeffs().iter().any(|value|!value.is_finite()||!(*value as f32).is_finite()){return Err("SVG viewport transform exceeds native geometry precision".into());}
 for path in &vector.paths {
  if path.opacity==0.||path.geometry.elements().is_empty(){continue;}
  if path.opacity!=1.{let bounds=path.geometry.bounding_box().inflate(path.style.width*path.style.miter_limit+1.,path.style.width*path.style.miter_limit+1.);scene.push_layer(Fill::NonZero,vello::peniko::BlendMode::default(),path.opacity,transform,&bounds);}
  if let Some(color)=path.fill {scene.fill(path.rule,transform,paint_color(color),None,&path.geometry);}
  if let Some(color)=path.stroke {if path.style.width>0.{scene.stroke(&path.style,transform,paint_color(color),None,&path.geometry);}}
  if path.opacity!=1.{scene.pop_layer();}
 }
 Ok(())
}

#[cfg(test)]mod tests {
 use super::*;
 fn make_image(data:&str)->Node {
  let path=Node::svg_path(data,"text-destructive opacity-50".into(),r#"{"stroke":"currentColor","strokeWidth":2,"strokeLinecap":"round","strokeLinejoin":"bevel"}"#).unwrap().with_semantics(r#"{"id":"geometry-only"}"#).unwrap();
  Node::svg(vec![crate::Child::Node(path)],"w-48 h-24 text-primary".into(),r#"{"paint":{"fill":"none"}}"#).unwrap().with_semantics(r#"{"role":"img","label":"Builds","describedBy":"geometry-only caption"}"#).unwrap()
 }
 #[test]fn native_vectors_draw_geometry_with_layout_and_accessible_image_semantics(){
  let image=make_image("M2 2L22 22");let caption=Node::text(vec![crate::TextPart::Literal("Six data values".into())],String::new()).unwrap().with_semantics(r#"{"id":"caption"}"#).unwrap();let mut root=Node::view(vec![crate::Child::Node(image),crate::Child::Node(caption)],String::new()).unwrap();root.refresh().unwrap();
  let mut painter=Painter::new();let (scene,tree)=painter.scene(&root,400.,1.,None).unwrap();let (_,image)=tree.nodes.iter().find(|(_,node)|node.role()==accesskit::Role::Image).unwrap();assert_eq!(image.label(),Some("Builds"));assert!(image.children().is_empty());assert_eq!(image.described_by().len(),1);let caption=tree.nodes.iter().find(|(id,_)|Some(id)==image.described_by().first()).unwrap();assert_eq!(caption.1.label(),Some("Six data values"));
  let item=painter.node(&root,TextStyle::default()).unwrap();let vector=item.children[0].vector.as_ref().unwrap();assert_eq!(vector.paths.len(),1);assert_eq!(vector.paths[0].style.width,2.);assert_eq!(vector.paths[0].style.start_cap,kurbo::Cap::Round);assert_eq!(vector.paths[0].opacity,0.5);assert_eq!(vector.paths[0].stroke,Some(crate::theme::Theme::default().color("light","destructive").unwrap()));
  let blank_image=make_image("");let caption=Node::text(vec![crate::TextPart::Literal("Six data values".into())],String::new()).unwrap().with_semantics(r#"{"id":"caption"}"#).unwrap();let mut blank_root=Node::view(vec![crate::Child::Node(blank_image),crate::Child::Node(caption)],String::new()).unwrap();blank_root.refresh().unwrap();
  let (blank,blank_tree)=painter.scene(&blank_root,400.,1.,None).unwrap();assert!(blank_tree.nodes.iter().any(|(_,node)|node.role()==accesskit::Role::Image));assert!(scene.encoding().path_data.len()>blank.encoding().path_data.len());
 }
 #[test]fn svg_icons_contribute_only_their_intended_button_name(){
  let decorative=Node::svg(vec![crate::Child::Node(Node::svg_path("M2 12L10 20L22 4",String::new(),"{}").unwrap())],"size-4".into(),"{}").unwrap();
  let mut button=Node::button_children(vec![crate::Child::Node(decorative),crate::Child::Text(crate::TextPart::Literal("Save".into()))],"flex-row".into(),false,None).unwrap();button.refresh().unwrap();let mut painter=Painter::new();let (_,tree)=painter.scene(&button,300.,1.,None).unwrap();assert_eq!(tree.nodes.iter().find(|(_,node)|node.role()==accesskit::Role::Button).unwrap().1.label(),Some("Save"));
 }
}
