//! UI values use the compiler's utility vocabulary and checked component props.
use super::*;
impl SourceIndex {
 fn ui_attribute(&self,token:usize)->Option<usize> {
  if token>=2 && self.text(token-1)=="=" {Some(token-2)}else if token>=3 && self.text(token-1)=="{" && self.text(token-2)=="=" {Some(token-3)}else{None}
 }
 pub(super) fn ui_value(&self,offset:usize)->Option<(usize,String,usize,String)> {
  let token=self.tokens.iter().position(|token|token.literal && token.span.start<offset && offset<token.span.end)?;
  let attribute=self.ui_attribute(token)?;
  let (_,field)=self.jsx_attribute_at(self.tokens[attribute].span.end)?;
  let start=self.tokens[token].span.start+1;let prefix=self.source.get(start..offset)?.to_owned();Some((token,field,start,prefix))
 }
 pub(super) fn ui_value_completion(&self,offset:usize)->Option<Value> {
  let (token,field,start,prefix)=self.ui_value(offset)?;
  if field=="className" {
   let start=start+prefix.rfind(char::is_whitespace).map_or(0,|at|at+1);let prefix=&self.source[start..offset];
   let (state,word)=prefix.split_once(':').map_or(("",prefix),|(state,word)|(state,word));
   if !state.is_empty() && tsrust_compiler::ui_project::style::validate(&format!("{state}:p-0")).is_err(){return Some(json!([]));}
   let range=self.range(Span::new(start,offset));
   let values=tsrust_compiler::ui_project::style::tokens().into_iter().map(|token|if state.is_empty(){token}else{format!("{state}:{token}")}).filter(|v|v.starts_with(prefix)).map(|value|json!({"label":value,"kind":12,"detail":"Shared web/native UI utility; later classes override earlier properties","textEdit":{"range":range,"newText":value}})).collect::<Vec<_>>();
   let _=word;return Some(json!(values));
  }
  let (_,fields,_)=self.jsx_props(self.tokens[self.ui_attribute(token)?].span.end)?;let (_,ty)=fields.iter().find(|(name,_)|*name==field)?;
  let ty=self.display_type(self.type_aliases.get(ty).unwrap_or(ty));let range=self.range(Span::new(start,offset));
  let annotation=tsrust_compiler::type_syntax::parse(&ty).ok()?;let mut pending=vec![&annotation];let mut literals=vec![];
  while let Some(value)=pending.pop() {use tsrust_compiler::type_syntax::{ValueAnnotationKind as Kind,LiteralAnnotation};match &value.kind {Kind::Literal(LiteralAnnotation::String(value))=>literals.push(value.clone()),Kind::Union(values)=>pending.extend(values.iter().rev()),Kind::Group(value)=>pending.push(value),_=>{}}}
  let values=literals.into_iter().filter(|v|v.starts_with(&prefix)).map(|value|json!({"label":value,"kind":12,"detail":format!("{field}: {}",self.display_type(&ty)),"textEdit":{"range":range,"newText":value}})).collect::<Vec<_>>();
  Some(json!(values))
 }
 pub(crate) fn ui_class_at(&self,offset:usize)->Option<(String,Span)> {
  let (token,field,start,_)=self.ui_value(offset)?;if field!="className" {return None;}
  let end=self.tokens[token].span.end-1;let text=&self.source[start..end];let at=offset-start;
  let begin=text[..at].rfind(char::is_whitespace).map_or(0,|v|v+1);let finish=text[at..].find(char::is_whitespace).map_or(text.len(),|v|at+v);
  let token=text[begin..finish].to_owned();tsrust_compiler::ui_project::style::validate(&token).ok()?;Some((token,Span::new(start+begin,start+finish)))
 }
}

impl SourceIndex {
 pub(super) fn ui_theme_request(&self,method:&str,offset:usize,uri:&str)->Option<Value> {
  if !self.theme_document && !uri.ends_with("/ui.theme.json") && !uri.ends_with("/tsrust.config.json") {return None;}
  if !matches!(method,"textDocument/completion"|"textDocument/hover"){return None;}
  let token=self.tokens.iter().position(|token|token.literal && token.span.start<offset && offset<token.span.end)?;
  let literal=|index:usize|->Option<String>{serde_json::from_str(&self.tokens.get(index)?.text).ok()};
  let mut path=vec![];
  for id in 0..token {
   if self.text(id)=="{" {path.push(if id>=2 && self.text(id-1)==":" {literal(id-2)}else{None});}
   if self.text(id)=="}" {path.pop();}
  }
  let config_uri=uri.ends_with("/tsrust.config.json");
  let config_section=config_uri && path.len()==2 && path.get(1).and_then(Option::as_deref)==Some("ui");
  let target_section=config_uri && path.len()==3 && path.get(1).and_then(Option::as_deref)==Some("ui") && path.get(2).and_then(Option::as_deref)==Some("target");
  let value:Value=if target_section{path=vec![None];serde_json::to_value(tsrust_compiler::ui_project::Targets::default()).ok()?}
   else if config_section{path=vec![None];serde_json::to_value(tsrust_compiler::ui_project::Config::default()).ok()?}
   else{
    if config_uri{if path.get(1).and_then(Option::as_deref)!=Some("ui") || path.get(2).and_then(Option::as_deref)!=Some("theme"){return None;}path.drain(1..3);}
    serde_json::from_str(tsrust_compiler::ui_project::theme::DEFAULT_JSON).ok()?
   };
  let mut object=&value;for key in path.iter().flatten(){object=object.get(key)?;}
  let start=self.tokens[token].span.start+1;let end=self.tokens[token].span.end-1;
  let prefix=self.source.get(start..offset)?;let range=self.range(Span::new(start,offset));
  let is_value=token>0 && self.text(token-1)==":";
  if method=="textDocument/completion" {
   let entries=if is_value && prefix.starts_with('@') && matches!(path.last().and_then(Option::as_deref),Some("light"|"dark")) {object.as_object()?.keys().map(|key|format!("@{key}")).collect::<Vec<_>>()}else if !is_value {object.as_object()?.keys().cloned().collect()}else{return Some(json!([]));};
   return Some(json!(entries.into_iter().filter(|key|key.starts_with(prefix)).map(|key|json!({"label":key,"kind":10,"detail":"Checked UI theme token","textEdit":{"range":range,"newText":key}})).collect::<Vec<_>>()));
  }
  let key=if is_value {literal(token.checked_sub(2)?)?}else{literal(token)?};
  let sample=object.get(&key)?;
  let detail=if target_section && key=="embedded" {"Boolean. Must remain false: embedded rendering and hardware profiles are not implemented."}else if target_section {"Boolean. true enables this UI deployment backend; false disables it. At least native or web must be enabled. Multiple targets can be enabled explicitly."}else if config_section && key=="target" {"Explicit UI deployment choices: native, web and embedded booleans. Enable native and/or web. Embedded must remain false. Missing choices prompt during ui add; build/run/dev never write setup."}else if config_section && matches!(key.as_str(),"components"|"lock") {"Project-relative source directory or installation lock path. Defaults are src/components/ui and ui-components.lock.json; parent traversal and overlapping destinations are rejected."}else if key=="schemaVersion" {"Theme schema version. Must be 1."}else if path.len()==2 && matches!(path.last().and_then(Option::as_deref),Some("light"|"dark")) {"Opaque sRGB #RRGGBB or @token alias in this mode. Missing aliases and cycles are errors."}else if sample.is_number(){"Finite logical pixels. Spacing: 0–512; radius: 0–256; typography: 8–128."}else{"Required UI theme section. Unknown and missing fields are errors. Changes affect both web and native output."};
  Some(json!({"contents":{"kind":"markdown","value":format!("`{key}`\n\n{detail}\n\nProject-owned data. Editing tokens does not recompile renderer dependencies.")},"range":self.range(Span::new(start,end))}))
 }
}
