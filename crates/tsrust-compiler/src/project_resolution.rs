//! Shared, filesystem-independent import candidate rules. Providers retain source
//! identity and snapshot ownership; they must try candidates in this exact order.
use std::{collections::BTreeMap,path::{Path,PathBuf}};
use serde_json::Value;

#[derive(Clone,Debug)]
pub struct Aliases { root:PathBuf, paths:BTreeMap<String,String> }
fn clean(text:&str)->bool { !text.is_empty() && !text.contains(['\\','\0','\n','\r']) }
impl Aliases {
 /// Parse only the paths field; the project configuration owns other settings.
 pub fn parse(root:&Path,value:Option<&Value>)->Result<Self,String> {
  let mut paths=BTreeMap::new();
  if let Some(value)=value {
   let entries=value.as_object().ok_or("paths must be an object of string mappings")?;
   for (key,value) in entries {
    let target=value.as_str().ok_or_else(||format!("paths.{key} must be a relative path string"))?;
    let wildcards=key.matches('*').count();
    if !clean(key)||key.starts_with('.')||key.starts_with('/')||key.starts_with("std:")||wildcards>1 {
     return Err(format!("Invalid path alias: {key}"));
    }
    if !clean(target)||Path::new(target).is_absolute()||target.matches('*').count()!=wildcards {
     return Err(format!("paths.{key} must be relative and have the same single wildcard as its alias"));
    }
    paths.insert(key.clone(),target.to_owned());
   }
  }
  Ok(Self{root:root.to_owned(),paths})
 }
 pub fn expand(&self,specifier:&str)->Result<Option<PathBuf>,String> {
  if let Some(target)=self.paths.get(specifier).filter(|_|!specifier.contains('*')) {return Ok(Some(self.root.join(target)));}
  let mut chosen:Option<(usize,PathBuf)>=None;
  let mut ambiguous=false;
  for (pattern,target) in &self.paths {
   let Some((prefix,suffix))=pattern.split_once('*') else {continue};
   let Some(middle)=specifier.strip_prefix(prefix).and_then(|text|text.strip_suffix(suffix)) else {continue};
   let priority=prefix.len();
   match &chosen {
    Some((old,_)) if *old>priority=>continue,
    Some((old,_)) if *old==priority=>ambiguous=true,
    _=>{chosen=Some((priority,self.root.join(target.replacen('*',middle,1))));ambiguous=false;},
   }
  }
  if ambiguous {return Err(format!("Ambiguous path aliases for {specifier}"));}
  Ok(chosen.map(|(_,path)|path))
 }
}
/// Data extensions remain explicit. No package search or filesystem access occurs.
pub fn candidates(importer:&Path,specifier:&str,aliases:Option<&Aliases>)->Result<Vec<PathBuf>,String> {
 if !clean(specifier)||Path::new(specifier).is_absolute()||specifier.starts_with("std:") {return Err(format!("Invalid file import: {specifier}"));}
 let path=if specifier.starts_with("./")||specifier.starts_with("../") {
  importer.parent().unwrap_or(Path::new(".")).join(specifier)
 } else {
  aliases.map(|aliases|aliases.expand(specifier)).transpose()?.flatten().ok_or_else(||format!("No path alias matches {specifier}"))?
 };
 if path.extension().is_some() {return Ok(vec![path]);}
 let mut source=path.as_os_str().to_os_string();source.push(".tsr");
 Ok(vec![PathBuf::from(source),path.join("index.tsr")])
}

#[cfg(test)]
mod tests {
 use super::*;
 use serde_json::json;
 #[test]
 fn paths_are_relative_to_config_and_exact_matches_win() {
  let aliases=Aliases::parse(Path::new("/project"),Some(&json!({"@/*":"./src/*","@/components":"./special","@/components/*":"./ui/*"}))).unwrap();
  assert_eq!(aliases.expand("@/components").unwrap(),Some(PathBuf::from("/project/./special")));
  assert_eq!(aliases.expand("@/components/button").unwrap(),Some(PathBuf::from("/project/./ui/button")));
  assert_eq!(aliases.expand("@/data/file.json").unwrap(),Some(PathBuf::from("/project/./src/data/file.json")));
  assert_eq!(aliases.expand("other").unwrap(),None);
 }
 #[test]
 fn source_candidates_prefer_files_and_preserve_explicit_formats() {
  let base=Path::new("/project/src/main.tsr");
  assert_eq!(candidates(base,"./components",None).unwrap(),vec![PathBuf::from("/project/src/./components.tsr"),PathBuf::from("/project/src/./components/index.tsr")]);
  assert_eq!(candidates(base,"./data.jsonc",None).unwrap(),vec![PathBuf::from("/project/src/./data.jsonc")]);
  assert_eq!(candidates(base,"./component.tsr",None).unwrap(),vec![PathBuf::from("/project/src/./component.tsr")]);
  assert!(candidates(base,"@/components",None).unwrap_err().contains("No path alias"));
 }
 #[test]
 fn invalid_or_ambiguous_aliases_fail_closed() {
  for value in [json!([]),json!({"@/*":["src/*"]}),json!({"@/**":"src/*"}),json!({"@/*":"src"}),json!({"name":"src/*"}),json!({"std:json":"src/json"}),json!({"./local":"src/local"}),json!({"name":"/absolute"}),json!({"":"src"}),json!({"name":""})] {
   assert!(Aliases::parse(Path::new("/project"),Some(&value)).is_err(),"{value}");
  }
  let aliases=Aliases::parse(Path::new("/project"),Some(&json!({"@/*end":"src/*","@/*nd":"other/*"}))).unwrap();
  assert!(aliases.expand("@/weekend").unwrap_err().contains("Ambiguous"));
  let specific=Aliases::parse(Path::new("/project"),Some(&json!({"@/*end":"src/*","@/*nd":"other/*","@/week*":"specific/*"}))).unwrap();
  assert_eq!(specific.expand("@/weekend").unwrap(),Some(PathBuf::from("/project/specific/end")));
 }
 #[test]
 fn wildcard_suffix_and_missing_config_are_supported() {
  let aliases=Aliases::parse(Path::new("/project"),Some(&json!({"@/*.tsr":"src/*.tsr"}))).unwrap();
  assert_eq!(candidates(Path::new("/elsewhere/main.tsr"),"@/greet.tsr",Some(&aliases)).unwrap(),vec![PathBuf::from("/project/src/greet.tsr")]);
  assert_eq!(Aliases::parse(Path::new("/project"),None).unwrap().expand("x").unwrap(),None);
  for path in ["/absolute","./bad\\path","./bad\npath", ""] {assert!(candidates(Path::new("/main.tsr"),path,None).is_err());}
 }
}
