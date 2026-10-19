//! Checked project data shared by the compiler, editor and both renderers.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
    pub fn color(&self,mode:&str,key:&str)->Result<[u8;3],String> {
        let values=match mode {"light"=>&self.light,"dark"=>&self.dark,_=>return Err("UI theme mode must be light or dark".into())};
        let first=key_name(key);let mut key=first.as_str(); let mut seen=std::collections::BTreeSet::new();
        loop {
            if !seen.insert(key) {return Err(format!("UI theme alias cycle: {mode}.{key}"));}
            let value=values.get(key).ok_or_else(||format!("Missing UI theme token: {mode}.{key}"))?;
            if let Some(alias)=value.strip_prefix('@') {key=alias;continue;}
            // Restrict to exact opaque sRGB bytes; reject rather than clip wide-gamut or alpha data.
            if value.len()!=7 || !value.starts_with('#') || !value[1..].bytes().all(|v|v.is_ascii_hexdigit()) {return Err(format!("UI theme {mode}.{key} must be #RRGGBB or @token"));}
            let parsed=color::parse_color(value).map_err(|e|format!("Invalid UI theme color: {e}"))?.to_alpha_color::<color::Srgb>().to_rgba8();
            return Ok([parsed.r,parsed.g,parsed.b]);
        }
    }
    pub(crate) fn variables(&self,mode:&str)->String {
        let mut result=String::new();
        for key in COLORS {let [r,g,b]=self.color(mode,key).expect("validated theme");result.push_str(&format!("--ui-{key}:#{r:02x}{g:02x}{b:02x};"));}
        for (name,scale) in [("spacing",&self.spacing),("radius",&self.radius)] {for (key,value) in [("sm",scale.sm),("md",scale.md),("lg",scale.lg)] {result.push_str(&format!("--ui-{name}-{key}:{}px;",value as f32));}}
        result.push_str(&format!("--ui-text-base:{}px;--ui-text-xl:{}px;",self.typography.base as f32,self.typography.xl as f32));result
    }
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn rejects_missing_unknown_alias_cycles_and_lossy_colors() {
  let original:serde_json::Value=serde_json::from_str(DEFAULT_JSON).unwrap();
  for (key,value) in [("primary","@primary"),("primary","@absent"),("primary","#ffffff80"),("surprise","#000000")] {
   let mut theme=original.clone();theme["light"][key]=value.into();assert!(Theme::parse(&theme.to_string()).is_err());
  }
  for value in [512.000001,-1e-100] {let mut theme=original.clone();theme["spacing"]["sm"]=serde_json::json!(value);assert!(Theme::parse(&theme.to_string()).is_err());}
  let mut theme=original.clone();theme["light"].as_object_mut().unwrap().remove("primary");assert!(Theme::parse(&theme.to_string()).is_err());
  let mut theme=original;theme["light"]["primary"]="@foreground".into();assert_eq!(Theme::parse(&theme.to_string()).unwrap().color("light","primary").unwrap(),[23,32,51]);
 }
}
