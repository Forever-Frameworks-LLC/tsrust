//! Build-time documentation data. Runtime renderers receive data, never executable Markdown.
mod markdown;
mod api;
pub use api::{ApiDefinition,ApiField,api,builtin};
mod code;
pub use code::{CodeToken, highlight};
pub use markdown::parse;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Range { pub start: usize, pub end: usize }
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all="kebab-case")]
pub enum Kind { Paragraph, Heading, Quote, CodeBlock, HtmlLiteral, List, Item, Table, TableHead, TableRow, TableCell, Emphasis, Strong, Strike, Link, Image, Text, Code, SoftBreak, HardBreak, Rule, Task }
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all="camelCase")]
pub struct Node {
    pub kind: Kind,
    pub source: Range,
    /// Preorder index of the containing node; -1 denotes a document root.
    pub parent: i64,
    pub text: String,
    pub raw: String,
    pub destination: String,
    pub title: String,
    pub anchor: String,
    pub language: String,
    pub level: u8,
    pub ordinal: Option<u64>,
    pub checked: Option<bool>,
    pub alignment: Vec<String>,
    pub tokens: Vec<CodeToken>,
}
impl Node {
    fn new(kind: Kind, source: std::ops::Range<usize>, parent: i64) -> Self {
        Self {kind, source: Range {start: source.start, end: source.end}, parent, text: String::new(), raw: String::new(), destination: String::new(), title: String::new(), anchor: String::new(), language: String::new(), level: 0, ordinal: None, checked: None, alignment: Vec::new(), tokens: Vec::new()}
    }
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all="camelCase")]
pub struct Heading { pub anchor: String, pub title: String, pub level: u8, pub node: usize, pub source: Range }
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all="camelCase")]
pub struct Document { pub nodes: Vec<Node>, pub headings: Vec<Heading>, pub title: String, pub summary: String }
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all="camelCase")]
pub struct Metadata {
    pub id: String,
    pub group: String,
    pub order: u32,
    pub status: Status,
    pub targets: Vec<Target>,
    #[serde(default)] pub examples: Vec<String>,
    #[serde(default)] pub component: Option<String>,
    #[serde(default)] pub api:Vec<ApiReference>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all="kebab-case")]
pub enum Status { Ready, Partial, Pending, Proposal }
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all="lowercase")]
pub enum Target { Web, Native }
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all="camelCase")]
pub struct Page {
    pub metadata: Metadata, pub route: String, pub source: String, pub sha256: String, pub markdown:String,
    pub document: Document,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all="camelCase")]
pub struct Example { pub kind:ExampleKind, pub id: String, pub source: String, pub export: String, pub sha256: String, pub code: String, pub tokens: Vec<CodeToken> }
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all="camelCase")]
pub struct Catalog {
    pub schema_version: u32, pub pages: Vec<Page>, pub examples: Vec<Example>,
    pub inputs: BTreeMap<String,String>,
    pub api: Vec<ApiDefinition>,
    pub library: serde_json::Value,
}
/// Metadata is a JSON object in the first `tsrust-docs` HTML comment. Other HTML stays literal.
pub fn metadata(source: &str) -> Result<(Metadata,usize),String> {
    let content=source.strip_prefix("<!-- tsrust-docs\n").ok_or("Documentation needs a leading <!-- tsrust-docs JSON metadata comment")?;
    let end=content.find("-->").ok_or("Unclosed documentation metadata comment")?;
    let metadata:Metadata=serde_json::from_str(&content[..end]).map_err(|error|format!("Invalid documentation metadata: {error}"))?;
    if !identifier(&metadata.id) || !identifier(&metadata.group) {return Err("Documentation id/group must use lowercase ASCII letters, digits and single hyphens".into());}
    if metadata.targets.is_empty() || metadata.targets.iter().collect::<std::collections::BTreeSet<_>>().len()!=metadata.targets.len(){return Err("Documentation needs distinct explicit web/native targets".into());}
    let offset=source.len()-content.len()+end+3;
    Ok((metadata,offset))
}
pub fn identifier(value:&str)->bool {
    !value.is_empty() && !value.starts_with('-') && !value.ends_with('-') && !value.contains("--") && value.bytes().all(|byte|byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte==b'-')
}
/// URI safety is conservative; internal existence and fragments are resolved by the content build.
pub fn safe_destination(value:&str)->Result<(),String> {
    if value.is_empty() || value.chars().any(|ch|ch.is_whitespace()||ch.is_control()) || value.contains('\\') || value.starts_with("//") {return Err(format!("Unsafe or empty documentation URL: {value:?}"));}
    let lower=value.to_ascii_lowercase();
    if lower.starts_with("https://") || lower.starts_with("http://") {
        let host=value.split_once("://").expect("scheme").1.split(['/', '?', '#']).next().unwrap_or("");
        if host.is_empty(){return Err("HTTP documentation URLs need a host".into());}
    } else if lower.starts_with("mailto:") {
        if value[7..].is_empty(){return Err("Empty mailto URL".into());}
    } else if value.split(['/', '?', '#']).next().unwrap_or("").contains(':') {return Err(format!("Unsupported documentation URL scheme: {value}"));}
    if value.bytes().any(|byte|matches!(byte,b'<'|b'>'|b'"')) {return Err(format!("Unsafe documentation URL characters: {value:?}"));}
    Ok(())
}

#[derive(Clone,Debug,Deserialize,Serialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
pub struct ExampleSource {pub id:String,pub source:String,pub export:String,pub kind:ExampleKind}
#[derive(Clone,Debug,Deserialize,Serialize,PartialEq,Eq)]
#[serde(rename_all="lowercase")]
pub enum ExampleKind {Component,Code}
#[derive(Clone,Debug,Deserialize,Serialize,PartialEq,Eq)]
#[serde(deny_unknown_fields)]
pub struct ApiReference {pub receiver:String,pub name:String}
#[derive(Clone,Debug,Deserialize,Serialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
pub struct Config {
    pub content:String,
    pub examples:Vec<ExampleSource>,
    pub api_sources:Vec<String>,
    pub inventory:Option<String>,
}
impl Config {
    pub fn validate(&self)->Result<(),String> {
        for path in std::iter::once(self.content.as_str()).chain(self.inventory.as_deref()).chain(self.examples.iter().map(|example|example.source.as_str())).chain(self.api_sources.iter().filter(|path|path.as_str()!="std:ui").map(String::as_str)) {
            crate::ui_project::safe_relative(std::path::Path::new(path))?;
        }
        let mut ids=std::collections::BTreeSet::new();
        for example in &self.examples {
            if !identifier(&example.id)||!ids.insert(&example.id){return Err("Example IDs must be distinct lowercase identifiers".into());}
            if example.export.is_empty()||!example.export.bytes().all(|ch|ch.is_ascii_alphanumeric()||ch==b'_'){return Err("Example export must name a checked local export".into());}
        }
        Ok(())
    }
}

/// Shared checked declarations used by generated content and editor witnesses.
pub const MODEL_SOURCE:&str=include_str!("model.tsr");
