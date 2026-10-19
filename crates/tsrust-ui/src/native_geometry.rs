//! CSS-style elliptical inner corners and rounded overflow geometry.
use vello::kurbo::{BezPath,Rect};
pub(super) fn normalized(rect:Rect,mut radii:[f64;4])->[f64;4] {
 let mut factor=1f64;
 for (size,sum) in [(rect.width(),radii[0]+radii[1]),(rect.width(),radii[3]+radii[2]),(rect.height(),radii[0]+radii[3]),(rect.height(),radii[1]+radii[2])]{if sum>0.{factor=factor.min(size.max(0.)/sum);}}
 for radius in &mut radii{*radius*=factor;}radii
}
fn ellipse_path(rect:Rect,mut rx:[f64;4],mut ry:[f64;4])->BezPath {
 let mut factor=1f64;
 for (size,sum) in [(rect.width(),rx[0]+rx[1]),(rect.width(),rx[3]+rx[2]),(rect.height(),ry[0]+ry[3]),(rect.height(),ry[1]+ry[2])]{if sum>0.{factor=factor.min(size.max(0.)/sum);}}
 for value in rx.iter_mut().chain(&mut ry){*value*=factor;}
 // Cubic quarter-circle approximation, matching the same bounded geometry for clipping and hit testing.
 let k=0.5522847498307936;let [tl,tr,br,bl]=rx;let [ytl,ytr,ybr,ybl]=ry;
 let mut p=BezPath::new();p.move_to((rect.x0+tl,rect.y0));p.line_to((rect.x1-tr,rect.y0));p.curve_to((rect.x1-tr+k*tr,rect.y0),(rect.x1,rect.y0+ytr-k*ytr),(rect.x1,rect.y0+ytr));
 p.line_to((rect.x1,rect.y1-ybr));p.curve_to((rect.x1,rect.y1-ybr+k*ybr),(rect.x1-br+k*br,rect.y1),(rect.x1-br,rect.y1));
 p.line_to((rect.x0+bl,rect.y1));p.curve_to((rect.x0+bl-k*bl,rect.y1),(rect.x0,rect.y1-ybl+k*ybl),(rect.x0,rect.y1-ybl));
 p.line_to((rect.x0,rect.y0+ytl));p.curve_to((rect.x0,rect.y0+ytl-k*ytl),(rect.x0+tl-k*tl,rect.y0),(rect.x0+tl,rect.y0));p.close_path();p
}
pub(super) fn rounded(rect:Rect,radii:[f64;4])->BezPath {let radii=normalized(rect,radii);ellipse_path(rect,radii,radii)}
pub(super) fn inner(rect:Rect,radii:[f64;4],border:[f64;4])->BezPath {
 let [top,right,bottom,left]=border;let r=normalized(rect,radii);
 let x0=(rect.x0+left).min(rect.x1);let y0=(rect.y0+top).min(rect.y1);
 let inner=Rect::new(x0,y0,(rect.x1-right).max(x0),(rect.y1-bottom).max(y0));
 ellipse_path(inner,[(r[0]-left).max(0.),(r[1]-right).max(0.),(r[2]-right).max(0.),(r[3]-left).max(0.)],[(r[0]-top).max(0.),(r[1]-top).max(0.),(r[2]-bottom).max(0.),(r[3]-bottom).max(0.)])
}
pub(super) fn outset(radii:[f64;4],amount:f64)->[f64;4] {radii.map(|r|if r==0.{0.}else{r+amount})}

#[cfg(test)] mod tests {
 use super::*;
 use vello::kurbo::{Point,Shape};
 #[test] fn card_corner_scaling_and_elliptical_padding_clips_match_css_geometry() {
  let rect=Rect::new(0.,0.,100.,20.);assert_eq!(normalized(rect,[40.,40.,0.,0.]),[20.,20.,0.,0.]);
  let rect=Rect::new(0.,0.,64.,64.);let path=inner(rect,[14.;4],[2.,1.,4.,8.]);
  assert!(!path.contains(Point::new(8.,2.)));assert!(path.contains(Point::new(20.,20.)));assert!(!path.contains(Point::new(63.5,20.)));assert!(!path.contains(Point::new(20.,61.)));
  assert!(!rounded(rect,[14.;4]).contains(Point::new(1.,1.)));
 }
}
