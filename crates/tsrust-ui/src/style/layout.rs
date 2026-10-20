//! Bounded portable layout utilities; all dimensions use logical pixels.
use super::{Theme, Utility};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length { Pixels(f32), Percent(f32), Auto, None, Fit, Viewport(u8) }
impl Length {
    pub(super) fn css(self) -> String { match self {
        Self::Pixels(value) => format!("{value}px"), Self::Percent(value) => format!("{}%",value*100.),
        Self::Viewport(axis)=>if axis==0{"100vw".into()}else{"100vh".into()},
        Self::Auto => "auto".into(), Self::None => "none".into(), Self::Fit=>"fit-content".into(),
    }}
}
pub(super) const SPACING: &[&str] = &["0","px","0.5","1","1.5","2","2.5","3","3.5","4","5","6","8","12","16","18","24","32","48","64","96","128","160","192","256"];
pub(super) fn spacing(size:&str,theme:&Theme)->f32 { match size {
    "px"=>1., "2"=>theme.spacing.sm as f32,"4"=>theme.spacing.md as f32,"6"=>theme.spacing.lg as f32,
    "1"=>theme.spacing.sm as f32/2.,"3"=>(theme.spacing.sm+theme.spacing.md) as f32/2.,
    _=>size.parse::<f32>().expect("validated size")*theme.spacing.md as f32/4.,
}}
/// A normalized ratio shared by CSS serialization and native logical geometry.
pub(super) fn aspect(token:&str)->Option<Option<f32>> {
    if token=="aspect-auto" {return Some(None);}
    let value=match token {"aspect-square"=>"1","aspect-video"=>"16/9",_=>token.strip_prefix("aspect-[")?.strip_suffix(']')?};
    let mut parts=value.split('/');
    let width=parts.next()?.parse::<f64>().ok()?;
    let height=parts.next().map(|part|part.parse::<f64>()).transpose().ok()?.unwrap_or(1.);
    if parts.next().is_some() || !width.is_finite() || !height.is_finite() || width<=0. || height<=0. {return None;}
    let ratio=(width/height) as f32;
    (ratio.is_finite() && ratio>0.).then_some(Some(ratio))
}
pub(super) fn tokens()->Vec<String> {
    let mut values=["table","table-header-group","table-row-group","table-footer-group","table-row","table-cell","table-caption","caption-top","caption-bottom","align-top","align-middle","align-bottom","flex-1","max-w-sm","relative","absolute","aspect-auto","aspect-square","aspect-video","aspect-[16/9]","aspect-[3/4]","aspect-[1.5]","hidden","flex","inline-flex","grid","auto-rows-min","flex-wrap","flex-nowrap","grow","grow-0","shrink","shrink-0"].into_iter().map(str::to_owned).collect::<Vec<_>>();
    for prefix in ["inset","inset-x","inset-y","top","right","bottom","left"] {
        for size in SPACING.iter().copied().chain(["full","1/2","1/3","2/3","1/4","3/4"]) {values.push(format!("{prefix}-{size}"));values.push(format!("-{prefix}-{size}"));}
        values.push(format!("{prefix}-auto"));
    }
    for prefix in ["m","mx","my","mt","mr","mb","ml","p","px","py","pt","pr","pb","pl","gap","gap-x","gap-y"] {
        for size in SPACING {values.push(format!("{prefix}-{size}"));}
    }
    for prefix in ["w","h","min-w","min-h","max-w","max-h","size"] {
        for size in SPACING.iter().copied().chain(["auto","full","screen","1/2","1/3","2/3","1/4","3/4"]) {
            if prefix.starts_with("max-") && size=="auto" {continue;}
            values.push(format!("{prefix}-{size}"));
        }
        if prefix.starts_with("max-"){values.push(format!("{prefix}-none"));}
        if matches!(prefix,"w"|"h"|"size"){values.push(format!("{prefix}-fit"));}
    }
    for prefix in ["grid-cols","grid-rows"] {
        values.push(format!("{prefix}-none"));
        for count in 1..=12 {values.push(format!("{prefix}-{count}"));}
    }
    values.extend(["grid-cols-[1fr_auto]","grid-rows-[auto_auto]"].map(str::to_owned));
    for prefix in ["col-start","row-start"] {for count in 1..=13 {values.push(format!("{prefix}-{count}"));}}
    for prefix in ["col-span","row-span"] {for count in 1..=12 {values.push(format!("{prefix}-{count}"));}}
    for prefix in ["self","justify-self"] {for align in ["start","center","end","stretch","auto"] {values.push(format!("{prefix}-{align}"));}}
    values
}
pub(super) fn resolve(token:&str,theme:&Theme)->Option<Vec<Utility>> {
    if let Some(index)=["table","table-header-group","table-row-group","table-footer-group","table-row","table-cell","table-caption"].iter().position(|value|*value==token){return Some(vec![Utility::TableDisplay(index as u8)]);}
    if let Some(index)=["caption-top","caption-bottom"].iter().position(|value|*value==token){return Some(vec![Utility::CaptionBottom(index==1)]);}
    if let Some(index)=["align-top","align-middle","align-bottom"].iter().position(|value|*value==token){return Some(vec![Utility::VerticalAlign(index as u8)]);}
    if token=="flex-1"{return Some(vec![Utility::Grow(1.),Utility::Shrink(1.),Utility::BasisZero]);}
    if token=="max-w-sm"{return Some(vec![Utility::Dimension(4,Length::Pixels(theme.typography.base as f32*24.))]);}
    for (prefix,sides) in [("m-",&[0,1,2,3][..]),("mx-",&[1,3][..]),("my-",&[0,2][..]),("mt-",&[0][..]),("mr-",&[1][..]),("mb-",&[2][..]),("ml-",&[3][..])]{if let Some(size)=token.strip_prefix(prefix){return Some(sides.iter().map(|&side|Utility::Margin(side,spacing(size,theme))).collect());}}
    if let Some(ratio)=aspect(token){return Some(vec![Utility::AspectRatio(ratio)]);}
    if let Some(values)=offset(token,theme){return Some(values);}
    let simple=match token {
        "relative"=>Some(Utility::Relative),"absolute"=>Some(Utility::Absolute),
        "hidden"=>Some(Utility::Hidden),"flex"=>Some(Utility::Display(false)),"inline-flex"=>Some(Utility::InlineFlex),"grid"=>Some(Utility::Display(true)),"auto-rows-min"=>Some(Utility::AutoRowsMin),
        "flex-wrap"=>Some(Utility::Wrap(true)),"flex-nowrap"=>Some(Utility::Wrap(false)),
        "grow"=>Some(Utility::Grow(1.)),"grow-0"=>Some(Utility::Grow(0.)),"shrink"=>Some(Utility::Shrink(1.)),"shrink-0"=>Some(Utility::Shrink(0.)),
        "grid-cols-[1fr_auto]"=>Some(Utility::Tracks(0,255)),"grid-rows-[auto_auto]"=>Some(Utility::Tracks(1,254)),_=>None,
    };
    if let Some(value)=simple{return Some(vec![value]);}
    for (prefix,axes) in [("p-",&[0,1,2,3][..]),("px-",&[1,3][..]),("py-",&[0,2][..]),("pt-",&[0][..]),("pr-",&[1][..]),("pb-",&[2][..]),("pl-",&[3][..])] {
        if let Some(size)=token.strip_prefix(prefix){return Some(axes.iter().map(|&side|Utility::Padding(side,spacing(size,theme))).collect());}
    }
    for (prefix,axes) in [("gap-x-",&[1][..]),("gap-y-",&[0][..]),("gap-",&[0,1][..])] {
        if let Some(size)=token.strip_prefix(prefix){return Some(axes.iter().map(|&axis|Utility::GapAxis(axis,spacing(size,theme))).collect());}
    }
    for (prefix,axes) in [("min-w-",&[2][..]),("min-h-",&[3][..]),("max-w-",&[4][..]),("max-h-",&[5][..]),("size-",&[0,1][..]),("w-",&[0][..]),("h-",&[1][..])] {
        if let Some(size)=token.strip_prefix(prefix){
            if size=="screen"{return Some(axes.iter().map(|&axis|Utility::Dimension(axis,Length::Viewport(axis%2))).collect());}
            let value=if let Some(value)=pixels(size){Length::Pixels(value)}else{match size {"auto"=>Length::Auto,"none"=>Length::None,"fit"=>Length::Fit,"full"=>Length::Percent(1.),"1/2"=>Length::Percent(0.5),"1/3"=>Length::Percent(1./3.),"2/3"=>Length::Percent(2./3.),"1/4"=>Length::Percent(0.25),"3/4"=>Length::Percent(0.75),_=>Length::Pixels(spacing(size,theme))}};
            return Some(axes.iter().map(|&axis|Utility::Dimension(axis,value)).collect());
        }
    }
    for (prefix,axis) in [("grid-cols-",0),("grid-rows-",1)] {if let Some(count)=token.strip_prefix(prefix){return Some(vec![Utility::Tracks(axis,if count=="none"{0}else{count.parse().expect("validated tracks")})]);}}
    for (prefix,property) in [("col-start-",0),("col-span-",1),("row-start-",2),("row-span-",3)] {if let Some(count)=token.strip_prefix(prefix){return Some(vec![Utility::Placement(property,count.parse().expect("validated placement"))]);}}
    for (prefix,axis) in [("self-",0),("justify-self-",1)] {if let Some(align)=token.strip_prefix(prefix){return Some(vec![Utility::SelfAlign(axis,match align {"start"=>0,"center"=>1,"end"=>2,"stretch"=>3,"auto"=>4,_=>unreachable!()})]);}}
    None
}

fn offset(token:&str,theme:&Theme)->Option<Vec<Utility>> {
    let (token,sign)=token.strip_prefix('-').map_or((token,1.),|token|(token,-1.));
    for (prefix,sides) in [("inset-x-",&[1,3][..]),("inset-y-",&[0,2][..]),("inset-",&[0,1,2,3][..]),("top-",&[0][..]),("right-",&[1][..]),("bottom-",&[2][..]),("left-",&[3][..])] {
        let Some(size)=token.strip_prefix(prefix) else{continue;};
        let value=match size {"auto" if sign>0.=>Length::Auto,"full"=>Length::Percent(sign),"1/2"=>Length::Percent(sign/2.),"1/3"=>Length::Percent(sign/3.),"2/3"=>Length::Percent(sign*2./3.),"1/4"=>Length::Percent(sign/4.),"3/4"=>Length::Percent(sign*3./4.),size if SPACING.contains(&size)=>Length::Pixels(sign*spacing(size,theme)),_=>return None};
        return Some(sides.iter().map(|&side|Utility::Inset(side,value)).collect());
    }
    None
}

pub(super) fn theme_keys(token:&str)->Vec<&'static str> {
    for prefix in ["m-","mx-","my-","mt-","mr-","mb-","ml-","gap-x-","gap-y-","gap-","min-w-","min-h-","max-w-","max-h-","size-","w-","h-","px-","py-","pt-","pr-","pb-","pl-","p-"] {
        if let Some(size)=token.strip_prefix(prefix) {if pixels(size).is_some(){return vec![];}return match size {"1"|"2"=>vec!["sm"],"3"=>vec!["sm","md"],"6"=>vec!["lg"],"0"|"px"|"auto"|"none"|"fit"|"full"|"screen"|"1/2"|"1/3"|"2/3"|"1/4"|"3/4"=>vec![],_=>vec!["md"]};}
    }
    vec![]
}

#[cfg(test)]mod aspect_tests {
 use super::*;
 #[test]fn aspect_ratios_normalize_and_reject_invalid_native_geometry_inputs(){
  for (class,expected) in [("aspect-square",1.),("aspect-video",16./9.),("aspect-[3/4]",0.75),("aspect-[1.5]",1.5)] {assert_eq!(aspect(class),Some(Some(expected)));}
  assert_eq!(aspect("aspect-auto"),Some(None));
  for class in ["aspect-[0]","aspect-[-1]","aspect-[NaN]","aspect-[inf]","aspect-[1/0]","aspect-[1/2/3]","aspect-[1e100]","aspect-[1e-100]","aspect-[]","aspect-[1;display:none]"]{assert!(aspect(class).is_none(),"{class}");assert!(super::super::validate(class).is_err(),"{class}");}
  let rules=super::super::resolved("aspect-video aspect-square",&Theme::default(),"light");assert_eq!(rules.iter().filter(|(_,rule)|matches!(rule,Utility::AspectRatio(_))).count(),1);assert!(rules.iter().any(|(_,rule)|matches!(rule,Utility::AspectRatio(Some(1.)))));
  assert!(super::super::documentation("aspect-[3/4]").contains("f32"));assert!(super::super::documentation("relative").contains("unsupported"));
 }
}

// Pixel dimensions have no theme dependency and never accept CSS expressions.
pub(super) fn pixels(size:&str)->Option<f32>{
 let text=size.strip_prefix('[')?.strip_suffix("px]").or_else(||size.strip_prefix('[')?.strip_suffix(']'))?;
 if text.is_empty()||!text.bytes().all(|b|b.is_ascii_digit()||b==b'.'){return None;}
 let value=text.parse::<f32>().ok()?;(value.is_finite()&&value>=0.).then_some(value)
}
pub(super) fn pixel_dimension(token:&str)->bool{
 ["min-w-","min-h-","max-w-","max-h-","size-","w-","h-"].iter().any(|prefix|token.strip_prefix(prefix).is_some_and(|size|pixels(size).is_some()))
}
