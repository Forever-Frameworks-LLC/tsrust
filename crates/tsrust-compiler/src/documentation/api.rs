use crate::project::{ProjectAnalysis,ProjectType,SourceId};
use crate::syntax;
use super::Range;
use serde::{Deserialize,Serialize};
use std::collections::BTreeMap;
#[derive(Clone,Debug,Deserialize,Serialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
pub struct ApiField {pub name:String,pub optional:bool,pub signature:String,pub documentation:String,pub source:String,pub range:Range}
#[derive(Clone,Debug,Deserialize,Serialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
pub struct ApiDefinition {pub name:String,pub kind:String,pub source:String,pub range:Option<Range>,pub signature:String,pub documentation:String,pub fields:Vec<ApiField>}
fn before(source:&str,start:usize)->String {
    let prefix=&source[..start];let Some(end)=prefix.rfind("*/") else{return String::new();};
    let tail=&prefix[end+2..];
    if tail.split_whitespace().any(|word|!matches!(word,"export"|"default"|"async"|"function"|"interface"|"type")){return String::new();}
    let Some(begin)=prefix[..end].rfind("/**") else{return String::new();};
    prefix[begin+3..end].lines().map(|line|line.trim().trim_start_matches('*').trim_start()).collect::<Vec<_>>().join("\n").trim().to_string()
}
fn header(source:&str,start:usize)->Result<String,String> {
    let tokens=syntax::lex(&source[start..]).map_err(|error|error.message)?;
    let mut parentheses=0;let mut brackets=0;
    for token in tokens {
        match token.text.as_str(){"("=>parentheses+=1,")"=>parentheses-=1,"["=>brackets+=1,"]"=>brackets-=1,"{" if parentheses==0&&brackets==0=>return Ok(source[start..start+token.span.start].trim().to_string()),_=>{}}
    }
    Err("Public documentation declaration has no body".into())
}
/// Declaration/field identities come from a successful project check, not a parallel type parser.
pub fn api(project:&ProjectAnalysis,sources:&[SourceId])->Result<Vec<ApiDefinition>,String> {
    let mut parsed=BTreeMap::new();
    for (id,module) in &project.graph.modules {
        if module.data.is_none()&&module.rust.is_none(){
            let program=syntax::parse_module(&module.declaration_source,crate::project::is_test_entry(&id.0),Default::default(),Default::default()).map_err(|error|format!("{}: {}",id.0,error.message))?;
            parsed.insert(id.clone(),program);
        }
    }
    let docs:serde_json::Value=serde_json::from_str(include_str!("../../../tsrust-lsp/src/builtins.json")).map_err(|error|error.to_string())?;
    let facts=project.checked_type_facts();let mut output=Vec::new();
    for id in sources {
        let module=project.graph.modules.get(id).ok_or_else(||format!("Documentation API module is outside the checked graph: {}",id.0))?;
        let program=parsed.get(id).ok_or("Documentation API source must be Tsrust")?;
        let source=&module.source.text;
        let aliases=syntax::TypeAliasCatalog::parse(&module.declaration_source).map_err(|error|error.message)?;
        for (export,span) in &module.exports {
            if export=="default"{continue;}
            let (kind,signature)=if let Some(function)=program.functions.iter().find(|value|value.name.span==*span||value.name.text==*export){
                ("function",format!("function {}",header(source,function.name.span.start)?))
            }else if let Some(record)=program.records.iter().find(|value|value.name.span==*span||value.name.text==*export){
                ("interface",format!("interface {}",header(source,record.name.span.start)?))
            }else if let Some(alias)=aliases.declarations().into_iter().find(|value|value.name==*export){("type",alias.signature)}else{continue;};
            let mut fields=Vec::new();
            for (owner,name,location,_,optional) in &facts.record_fields {
                let ProjectType::Named{declaration:Some(declaration),..}=owner else{continue;};
                if declaration.source!=*id||declaration.span!=*span{continue;}
                let original=project.graph.modules.get(&location.source).ok_or("Missing checked prop declaration")?;
                let record=parsed.get(&location.source).and_then(|program|program.records.iter().find(|record|record.fields.iter().any(|field|field.name.span==location.span)));
                let field=record.and_then(|record|record.fields.iter().find(|field|field.name.span==location.span)).ok_or_else(||format!("Missing source prop annotation for {}.{} at {}:{}..{}",export,name,location.source.0,location.span.start,location.span.end))?;
                let annotation=&original.source.text[field.annotation.span.start..field.annotation.span.end];
                let mut documentation=before(&original.source.text,location.span.start);
                if location.source.0=="std:ui" {
                    if let Some(record)=record {if let Some(entry)=docs["std:ui"].as_array().and_then(|entries|entries.iter().find(|entry|entry["name"].as_str()==Some(record.name.text.as_str()))){
                        if let Some(text)=entry["fields"][name]["documentation"].as_str(){documentation=text.into();}
                    }}
                }
                fields.push(ApiField{name:name.clone(),optional:*optional,signature:format!("{}{}: {}",if name.bytes().all(|ch|ch.is_ascii_alphanumeric()||ch==b'_'){name.clone()}else{crate::json_string(name)},if *optional{"?"}else{""},annotation),documentation,source:location.source.0.clone(),range:Range{start:location.span.start,end:field.annotation.span.end}});
            }
            output.push(ApiDefinition{name:export.clone(),kind:kind.into(),source:id.0.clone(),range:Some(Range{start:span.start,end:span.end}),signature,documentation:before(source,span.start),fields});
        }
    }
    Ok(output)
}

/// The editor catalog is the single behavioral documentation source for built-ins.
pub fn builtin(reference:&super::ApiReference)->Result<ApiDefinition,String>{
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../../tsrust-lsp/src/builtins.json")).map_err(|error|error.to_string())?;
 let entry=catalog[&reference.receiver].as_array().and_then(|entries|entries.iter().find(|entry|entry["name"].as_str()==Some(&reference.name))).ok_or_else(||format!("Unknown built-in documentation {}.{}",reference.receiver,reference.name))?;
 Ok(ApiDefinition{name:format!("{}.{}",reference.receiver,reference.name),kind:"method".into(),source:format!("builtin:{}",reference.receiver),range:None,signature:entry["signature"].as_str().ok_or("Built-in has no signature")?.into(),documentation:entry["documentation"].as_str().ok_or("Built-in has no documentation")?.into(),fields:vec![]})
}
