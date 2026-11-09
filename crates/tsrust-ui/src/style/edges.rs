//! Edge properties expand before merging, so source order remains property order.
use super::{Theme,Utility};
pub(super) fn tokens()->Vec<String> {
 let mut out=vec!["border-dashed".into(),"border-solid".into(),"overflow-hidden".into(),"overflow-visible".into(),"overflow-auto".into()];
 for corner in ["","t-","r-","b-","l-","tl-","tr-","br-","bl-"] {for size in ["none","sm","lg","xl","2xl","3xl","4xl"] {out.push(format!("rounded-{corner}{size}"));}if !corner.is_empty(){out.push(format!("rounded-{}",corner.trim_end_matches('-')));}}
 for edge in ["","t","r","b","l"] {let base=if edge.is_empty(){"border".to_owned()}else{format!("border-{edge}")};out.extend([base.clone(),format!("{base}-0"),format!("{base}-2")]);}out
}
pub(super) fn resolve(token:&str,theme:&Theme)->Option<Vec<Utility>> {
 if token=="border-dashed"{return Some(vec![Utility::BorderDashed(true)]);}
 if token=="border-solid"{return Some(vec![Utility::BorderDashed(false)]);}
 if token=="overflow-hidden"{return Some(vec![Utility::Overflow(1)]);}
 if token=="overflow-auto"{return Some(vec![Utility::Overflow(2)]);}
 if token=="overflow-visible"{return Some(vec![Utility::Overflow(0)]);}
 if token=="rounded" || token.starts_with("rounded-") {
  let rest=token.strip_prefix("rounded-").unwrap_or("");
  let (corners,size)=if rest.is_empty(){(&[0,1,2,3][..],"")}else{
   let (first,tail)=rest.split_once('-').unwrap_or((rest,""));
   match first {"t"=>(&[0,1][..],tail),"r"=>(&[1,2][..],tail),"b"=>(&[2,3][..],tail),"l"=>(&[3,0][..],tail),"tl"=>(&[0][..],tail),"tr"=>(&[1][..],tail),"br"=>(&[2][..],tail),"bl"=>(&[3][..],tail),_=>(&[0,1,2,3][..],rest)}
  };
  let value=match size {""=>theme.radius.md,"none"=>0.,"sm"=>theme.radius.sm,"lg"=>theme.radius.lg,"xl"=>theme.radius.extra_large(),"2xl"=>theme.radius.extra_large()*1.25,"3xl"=>theme.radius.extra_large()*1.5,"4xl"=>theme.radius.extra_large()*2.,_=>return None} as f32;
  return Some(corners.iter().map(|&corner|Utility::Corner(corner,value)).collect());
 }
 if token=="border" || token.starts_with("border-") {
  let rest=token.strip_prefix("border-").unwrap_or("");let (first,tail)=rest.split_once('-').unwrap_or((rest,""));
  let (sides,size)=match first {"t"=>(&[0][..],tail),"r"=>(&[1][..],tail),"b"=>(&[2][..],tail),"l"=>(&[3][..],tail),_=>(&[0,1,2,3][..],rest)};
  let value=match size {""=>1.,"0"=>0.,"2"=>2.,_=>return None};return Some(sides.iter().map(|&side|Utility::EdgeBorder(side,value)).collect());
 }
 None
}

#[cfg(test)] mod tests {
 use super::*;
 #[test] fn card_edge_and_corner_overrides_expand_in_source_order() {
  let theme=Theme::default();for token in tokens(){assert!(super::super::validate(&token).is_ok(),"{token}");assert!(resolve(&token,&theme).is_some(),"{token}");}
  let rules=super::super::resolved("rounded-t-xl rounded-none rounded-bl-sm border-t-2 border border-r-0",&theme,"light");
  for (corner,radius) in [(0,0.),(1,0.),(2,0.),(3,4.)]{assert!(rules.iter().any(|(_,v)|matches!(v,Utility::Corner(c,r) if *c==corner && *r==radius)));}
  for (side,width) in [(0,1.),(1,0.),(2,1.),(3,1.)]{assert!(rules.iter().any(|(_,v)|matches!(v,Utility::EdgeBorder(s,w) if *s==side && *w==width)));}
 }
}
