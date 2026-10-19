//! Checked project data shared by the compiler, editor and both renderers.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
mod paint_color;
pub use paint_color::ResolvedColor;

pub const COLORS: &[&str] = &["background", "foreground", "card", "card-foreground", "primary", "primary-foreground", "secondary", "secondary-foreground", "muted", "muted-foreground", "accent", "accent-foreground", "destructive", "destructive-foreground", "border", "input", "ring"];
pub fn key_name(key:&str)->String {
 let mut out=String::new();let mut upper=false;
 for ch in key.chars() {if ch=='-' {upper=true;}else if upper {out.extend(ch.to_uppercase());upper=false;}else{out.push(ch);}}out
}
pub const DEFAULT_JSON: &str = include_str!("theme.json");
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Scale { pub sm:f64, pub md:f64, pub lg:f64 }
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Typography { pub base:f64, pub xl:f64 }
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all="camelCase")]
pub struct Theme {
    pub schema_version:u32,
    pub light:BTreeMap<String,String>,
    pub dark:BTreeMap<String,String>,
    pub spacing:Scale,
    pub radius:Scale,
    pub typography:Typography,
}
impl Default for Theme {fn default()->Self {Self::parse(DEFAULT_JSON).expect("valid bundled theme")}}
impl Theme {
    pub fn parse(text:&str)->Result<Self,String> {
        let theme:Self=serde_json::from_str(text).map_err(|e|format!("Invalid UI theme: {e}"))?;
        if theme.schema_version!=1 {return Err("UI theme schemaVersion must be 1".into());}
        for (name,values) in [("light",&theme.light),("dark",&theme.dark)] {
            for key in values.keys() {if !COLORS.iter().any(|token|key_name(token)==*key) {return Err(format!("Unknown UI theme token: {name}.{key}"));}}
            for key in COLORS {theme.color(name,key)?;}
            if theme.color(name,"background")?.alpha()!=1. {return Err(format!("UI theme {name}.background must be opaque; transparent native windows are unsupported"));}
        }
        for (name,scale,maximum) in [("spacing",&theme.spacing,512.),("radius",&theme.radius,256.)] {
            for (key,value) in [("sm",scale.sm),("md",scale.md),("lg",scale.lg)] {
                if !value.is_finite() || !(0. ..=maximum).contains(&value) {return Err(format!("UI theme {name}.{key} must be between 0 and {maximum} logical pixels"));}
            }
        }
        for (key,value) in [("base",theme.typography.base),("xl",theme.typography.xl)] {
            if !value.is_finite() || !(8. ..=128.).contains(&value) {return Err(format!("UI theme typography.{key} must be between 8 and 128 logical pixels"));}
        }
        Ok(theme)
    }
    pub fn color(&self,mode:&str,key:&str)->Result<ResolvedColor,String> {
        let values=match mode {"light"=>&self.light,"dark"=>&self.dark,_=>return Err("UI theme mode must be light or dark".into())};
        let mut key=key_name(key);let mut seen=std::collections::BTreeSet::new();
        loop {
            if !seen.insert(key.clone()){return Err(format!("UI theme alias cycle: {mode}.{key}"));}
            let value=values.get(&key).ok_or_else(||format!("Missing UI theme token: {mode}.{key}"))?.trim();
            if let Some(alias)=value.strip_prefix('@'){key=key_name(alias);continue;}
            return ResolvedColor::parse(value).map_err(|error|format!("UI theme {mode}.{key}: {error}"));
        }
    }
    pub(crate) fn variables(&self,mode:&str)->String {
        let mut result=String::new();
        for key in COLORS {let color=self.color(mode,key).expect("validated theme");result.push_str(&format!("--ui-{key}:{};",color.css()));}
        for (name,scale) in [("spacing",&self.spacing),("radius",&self.radius)] {for (key,value) in [("sm",scale.sm),("md",scale.md),("lg",scale.lg)] {result.push_str(&format!("--ui-{name}-{key}:{}px;",value as f32));}}
        result.push_str(&format!("--ui-text-base:{}px;--ui-text-xl:{}px;",self.typography.base as f32,self.typography.xl as f32));result
    }
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn rejects_missing_unknown_alias_cycles_and_lossy_colors() {
  let original:serde_json::Value=serde_json::from_str(DEFAULT_JSON).unwrap();
  for (key,value) in [("primary","@primary"),("primary","@absent"),("primary","#fffffg80"),("surprise","#000000")] {
   let mut theme=original.clone();theme["light"][key]=value.into();assert!(Theme::parse(&theme.to_string()).is_err());
  }
  for value in [512.000001,-1e-100] {let mut theme=original.clone();theme["spacing"]["sm"]=serde_json::json!(value);assert!(Theme::parse(&theme.to_string()).is_err());}
  let mut theme=original.clone();theme["light"].as_object_mut().unwrap().remove("primary");assert!(Theme::parse(&theme.to_string()).is_err());
  let mut theme=original;theme["light"]["primary"]="@foreground".into();assert_eq!(Theme::parse(&theme.to_string()).unwrap().color("light","primary").unwrap(),ResolvedColor::rgb8(23,32,51));
 }
}

#[cfg(test)] mod color_tests {
 use super::*;
 fn configured(key:&str,value:&str)->String{let mut theme:serde_json::Value=serde_json::from_str(DEFAULT_JSON).unwrap();theme["light"][key]=value.into();theme.to_string()}
 #[test] fn alpha_hex_aliases_and_oklch_keep_values_until_paint() {
  let mut value:serde_json::Value=serde_json::from_str(DEFAULT_JSON).unwrap();value["light"]["primary"]="#12345680".into();value["light"]["accent"]="@card-foreground".into();
  let theme=Theme::parse(&value.to_string()).unwrap();let rgba=theme.color("light","primary").unwrap().components();assert_eq!(rgba,[18./255.,52./255.,86./255.,128./255.]);assert_eq!(theme.light["primary"],"#12345680");
  assert_eq!(theme.color("light","accent").unwrap(),theme.color("light","foreground").unwrap());
  let neutral=Theme::parse(&configured("primary","oklch(50% 0 0 / 25%)")).unwrap().color("light","primary").unwrap();
  for channel in &neutral.components()[..3]{assert!((*channel-0.38857286).abs()<0.000002);}
  assert_eq!(neutral.alpha(),0.25);assert!(neutral.css().starts_with("color(srgb "));
  let vivid=Theme::parse(&configured("primary","oklch(0.6 0.12 260 / 0.8)")).unwrap();assert!((vivid.color("light","primary").unwrap().alpha()-0.8).abs()<0.000001);
 }
 #[test] fn invalid_ranges_gamut_and_transparent_window_backgrounds_fail() {
  for text in ["#abc","#1234567g","oklch(120% 0 0)","oklch(.6 -.1 30)","oklch(.6 .1 361)","oklch(.6 .1 30 / 1.01)","oklch(NaN 0 0)","oklch(.6 0.8 30)","oklch(.6 .1 30deg)","oklch(.6 10% 30)","color(display-p3 1 0 0)"]{assert!(Theme::parse(&configured("primary",text)).is_err(),"{text}");}
  for text in ["#ffffff00","oklch(.8 0 0 / .5)"] {assert!(Theme::parse(&configured("background",text)).unwrap_err().contains("must be opaque"));}
  let mut theme:serde_json::Value=serde_json::from_str(DEFAULT_JSON).unwrap();theme["light"]["card"]="@card-foreground".into();theme["light"]["cardForeground"]="@card".into();assert!(Theme::parse(&theme.to_string()).unwrap_err().contains("cycle"));
 }
 #[test] fn default_text_and_focus_pairs_have_accessible_contrast() {
  fn luminance(color:ResolvedColor)->f64{let [r,g,b,_]=color.components();let linear=|v:f32|{let v=v as f64;if v<=0.04045{v/12.92}else{((v+0.055)/1.055).powf(2.4)}};0.2126*linear(r)+0.7152*linear(g)+0.0722*linear(b)}
  let theme=Theme::default();
  for mode in ["light","dark"] {for (fg,bg,minimum) in [("foreground","background",4.5),("primary-foreground","primary",4.5),("muted-foreground","muted",4.5),("ring","background",3.)] {
   let fg=luminance(theme.color(mode,fg).unwrap());let bg=luminance(theme.color(mode,bg).unwrap());let ratio=(fg.max(bg)+0.05)/(fg.min(bg)+0.05);assert!(ratio>=minimum,"{mode}: contrast {ratio} below {minimum}");
  }}
 }
}
