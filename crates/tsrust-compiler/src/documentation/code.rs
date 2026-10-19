use super::Range;
use serde::{Deserialize,Serialize};
use crate::syntax::{self,TokenKind};
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
pub struct CodeToken { pub source: Range, pub class: String, pub text:String }
/// Unknown fences remain exact readable text; Tsrust fences use the actual compiler lexer.
pub fn highlight(source:&str, language:&str)->Result<Vec<CodeToken>,String> {
    if !matches!(language,"tsrust"|"tsr"|"tsrx"){return Ok(vec![CodeToken{source:Range{start:0,end:source.len()},class:"plain".into(),text:source.into()}]);}
    let tokens=syntax::lex(source).map_err(|error|format!("Code fence cannot be tokenized: {} at byte {}",error.message,error.span.start))?;
    let mut output=Vec::new();let mut offset=0;let mut tag_name=false;
    for token in tokens.into_iter().filter(|token|token.kind!=TokenKind::End) {
        if offset<token.span.start {
            let gap=&source[offset..token.span.start];
            output.push(CodeToken{source:Range{start:offset,end:token.span.start},text:gap.into(),class:if gap.trim_start().starts_with("//")||gap.trim_start().starts_with("/*"){"comment"}else{"plain"}.into()});
        }
        let class=match token.kind {
            TokenKind::JsxOpen|TokenKind::JsxClose=>{tag_name=true;"punctuation"},
            TokenKind::JsxEnd=>{tag_name=false;"punctuation"},
            TokenKind::JsxText=>"plain",
            TokenKind::Name if tag_name=>{tag_name=false;"component"},
            TokenKind::String|TokenKind::TemplateHead|TokenKind::TemplateMiddle|TokenKind::TemplateTail=>"string",
            TokenKind::Integer|TokenKind::Number=>"number",
            TokenKind::Name if matches!(token.text.as_str(),"async"|"await"|"break"|"case"|"catch"|"class"|"const"|"continue"|"default"|"else"|"export"|"extends"|"false"|"finally"|"for"|"from"|"function"|"if"|"import"|"in"|"interface"|"let"|"match"|"new"|"null"|"of"|"return"|"switch"|"throw"|"true"|"try"|"type"|"typeof"|"undefined"|"while")=>"keyword",
            TokenKind::Name if matches!(token.text.as_str(),"boolean"|"int"|"number"|"string"|"void"|"ReadOnly"|"Mutable"|"Owned")=>"type",
            TokenKind::Name=>"name",
            _=>"punctuation",
        };
        offset=token.span.end;output.push(CodeToken{source:Range{start:token.span.start,end:offset},class:class.into(),text:source[token.span.start..offset].into()});
    }
    if offset<source.len(){let gap=&source[offset..];output.push(CodeToken{source:Range{start:offset,end:source.len()},text:gap.into(),class:if gap.trim_start().starts_with("//")||gap.trim_start().starts_with("/*"){"comment"}else{"plain"}.into()});}
    Ok(output)
}
