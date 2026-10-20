use super::{Document, Heading, Kind, Node, highlight, safe_destination};
use pulldown_cmark::{BrokenLink,CodeBlockKind,Event,Options,Parser,Tag};
use std::collections::BTreeSet;

fn content(nodes:&[Node],root:usize)->String {
    let mut output=String::new();
    for index in root+1..nodes.len() {
        let mut parent=nodes[index].parent;let mut inside=false;
        while parent>=0 {if parent==root as i64 {inside=true;break;}parent=nodes[parent as usize].parent;}
        if !inside {break;}
        match nodes[index].kind {Kind::Text|Kind::Code|Kind::HtmlLiteral=>output.push_str(&nodes[index].text),Kind::SoftBreak|Kind::HardBreak=>output.push(' '),_=>{}}
    }
    output
}
fn anchor(title:&str,used:&mut BTreeSet<String>)->String {
    let mut base=String::new();let mut separator=false;
    for ch in title.chars().flat_map(char::to_lowercase) {
        if ch.is_alphanumeric()||ch=='_' {if separator&&!base.is_empty(){base.push('-');}base.push(ch);separator=false;}
        else if ch.is_whitespace()||ch=='-' {separator=true;}
    }
    if base.is_empty(){base="section".into();}
    let mut value=base.clone();let mut suffix=2;
    while !used.insert(value.clone()){value=format!("{base}-{suffix}");suffix+=1;}
    value
}
/// CommonMark plus tables, task lists and strikethrough. Byte ranges reference the original source.
pub fn parse(source:&str,offset:usize)->Result<Document,String> {
    parse_inner(source,offset,true)
}
/// Hover prose uses ordinary CommonMark: unmatched [labels] remain literal text.
/// Authored page files retain strict unresolved-reference diagnostics.
pub(super) fn parse_api(source:&str)->Result<Document,String>{parse_inner(source,0,false)}
fn parse_inner(source:&str,offset:usize,strict_references:bool)->Result<Document,String> {
    if !source.is_char_boundary(offset){return Err("Markdown offset is not a UTF-8 boundary".into());}
    let mut nodes=Vec::<Node>::new();let mut stack=Vec::<usize>::new();let mut headings=Vec::new();let mut used=BTreeSet::new();let mut title=String::new();let mut summary=String::new();let mut code_ranges=std::collections::BTreeMap::<usize,(usize,usize)>::new();
    let options=Options::ENABLE_TABLES|Options::ENABLE_TASKLISTS|Options::ENABLE_STRIKETHROUGH;
    let mut unresolved=Vec::new();
    let callback=|link:BrokenLink<'_>|{unresolved.push(link.reference.to_string());None};
    let parser=Parser::new_with_broken_link_callback(&source[offset..],options,Some(callback));
    for (event,range) in parser.into_offset_iter() {
        let range=offset+range.start..offset+range.end;
        let parent=stack.last().copied().map(|id|id as i64).unwrap_or(-1);
        match event {
            Event::Start(tag)=>{
                if stack.len()>=128{return Err(format!("Markdown nesting exceeds 128 at byte {}",range.start));}
                let kind=match &tag {
                    Tag::Paragraph=>Kind::Paragraph,Tag::Heading{..}=>Kind::Heading,Tag::BlockQuote(_)=>Kind::Quote,
                    Tag::CodeBlock(_)=>Kind::CodeBlock,Tag::HtmlBlock=>Kind::HtmlLiteral,Tag::List(_)=>Kind::List,Tag::Item=>Kind::Item,
                    Tag::Table(_)=>Kind::Table,Tag::TableHead=>Kind::TableHead,Tag::TableRow=>Kind::TableRow,Tag::TableCell=>Kind::TableCell,
                    Tag::Emphasis=>Kind::Emphasis,Tag::Strong=>Kind::Strong,Tag::Strikethrough=>Kind::Strike,Tag::Link{..}=>Kind::Link,Tag::Image{..}=>Kind::Image,
                    _=>return Err(format!("Unsupported Markdown extension at byte {}",range.start)),
                };
                let mut node=Node::new(kind,range,parent);
                match tag {
                    Tag::Heading{level,..}=>node.level=level as u8,
                    Tag::List(start)=>node.ordinal=start,
                    Tag::CodeBlock(CodeBlockKind::Fenced(info))=>node.language=info.split_whitespace().next().unwrap_or("").to_string(),
                    Tag::Table(alignment)=>node.alignment=alignment.into_iter().map(|value|format!("{value:?}").to_ascii_lowercase()).collect(),
                    Tag::Link{dest_url,title,..}|Tag::Image{dest_url,title,..}=>{safe_destination(&dest_url)?;node.destination=dest_url.to_string();node.title=title.to_string();},
                    _=>{},
                }
                stack.push(nodes.len());nodes.push(node);
            },
            Event::End(_)=>{
                let index=stack.pop().ok_or("Unbalanced Markdown parser output")?;
                nodes[index].source.end=nodes[index].source.end.max(range.end);
                match nodes[index].kind {
                    Kind::Heading=>{
                        let text=content(&nodes,index);let id=anchor(&text,&mut used);
                        nodes[index].anchor=id.clone();
                        if nodes[index].level==1&&title.is_empty(){title=text.clone();}
                        headings.push(Heading{anchor:id,title:text,level:nodes[index].level,node:index,source:nodes[index].source.clone()});
                    },
                    Kind::Paragraph if summary.is_empty()&&nodes[index].parent<0=>summary=content(&nodes,index),
                    Kind::CodeBlock=>{if let Some((start,end))=code_ranges.remove(&index){nodes[index].raw=source[start..end].to_string();}nodes[index].tokens=highlight(&nodes[index].text,&nodes[index].language)?;},
                    Kind::Image=>{nodes[index].text=content(&nodes,index);if nodes[index].text.trim().is_empty(){return Err(format!("Documentation images need nonempty alt text at byte {}",nodes[index].source.start));}},
                    _=>{},
                }
            },
            Event::Text(text) if stack.last().is_some_and(|id|nodes[*id].kind==Kind::CodeBlock)=>{
                let index=*stack.last().unwrap();code_ranges.entry(index).and_modify(|span|span.1=range.end).or_insert((range.start,range.end));
                nodes[index].text.push_str(&text);
            },
            event=>{
                let (kind,text,checked)=match event {
                    Event::Text(value)=>(Kind::Text,value.to_string(),None),Event::Code(value)=>(Kind::Code,value.to_string(),None),
                    Event::Html(value)|Event::InlineHtml(value)=>(Kind::HtmlLiteral,value.to_string(),None),
                    Event::SoftBreak=>(Kind::SoftBreak,String::new(),None),Event::HardBreak=>(Kind::HardBreak,String::new(),None),
                    Event::Rule=>(Kind::Rule,String::new(),None),Event::TaskListMarker(value)=>(Kind::Task,String::new(),Some(value)),
                    _=>return Err(format!("Unsupported Markdown event at byte {}",range.start)),
                };
                let mut node=Node::new(kind,range,parent);node.text=text;node.checked=checked;nodes.push(node);
            },
        }
    }
    if strict_references&&!unresolved.is_empty(){return Err(format!("Unresolved Markdown references: {}",unresolved.join(", ")));}
    if !stack.is_empty(){return Err("Unclosed Markdown parser output".into());}
    Ok(Document{nodes,headings,title,summary})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]fn api_prose_keeps_bracketed_utility_labels_literal_without_weakening_pages_or_urls(){
        let text="Use aspect-[W/H] and [--NAME:--spacing(N)] with `text-primary`.";
        assert!(parse(text,0).is_err());let doc=parse_api(text).unwrap();assert_eq!(content(&doc.nodes,0),text.replace('`',""));
        assert!(parse_api("[bad](javascript:alert)").is_err());assert!(parse("[missing][reference]",0).is_err());
    }
    #[test]
    fn commonmark_extensions_ranges_anchors_and_code_remain_structured() {
        let source="# Héllo 🦀\n\nText **bold** and *soft* and ~~gone~~ with `x`.\n\n## Same\n\n## Same\n\n## Same-2\n\n> Quote\n\n3. Third\n4. Fourth\n\n- [x] Done\n- [ ] Next\n\n| Left | Right |\n| :--- | ---: |\n| a | b |\n\n```tsrx\nconst n = <View><Text>🦀</Text></View>;\n```\n\n---\n\n<script>alert('x')</script>\n";
        let doc=parse(source,0).unwrap();assert_eq!(doc.title,"Héllo 🦀");
        assert_eq!(doc.headings.iter().map(|heading|heading.anchor.as_str()).collect::<Vec<_>>(),vec!["héllo","same","same-2","same-2-2"]);
        for kind in [Kind::Paragraph,Kind::Strong,Kind::Emphasis,Kind::Strike,Kind::Code,Kind::Quote,Kind::List,Kind::Task,Kind::Table,Kind::Rule,Kind::HtmlLiteral] {assert!(doc.nodes.iter().any(|node|node.kind==kind),"{kind:?}");}
        assert!(doc.nodes.iter().any(|node|node.ordinal==Some(3)));
        let code=doc.nodes.iter().find(|node|node.kind==Kind::CodeBlock).unwrap();assert!(code.text.contains("</View>"));assert!(code.tokens.iter().any(|token|token.class=="component"&&&code.text[token.source.start..token.source.end]=="View"));
        for (id,node) in doc.nodes.iter().enumerate(){assert!(node.parent<id as i64);assert!(node.source.start<=node.source.end&&node.source.end<=source.len());assert!(source.is_char_boundary(node.source.start)&&source.is_char_boundary(node.source.end));}
        assert!(doc.nodes.iter().any(|node|node.kind==Kind::HtmlLiteral&&node.text.contains("script")));
    }
    #[test]
    fn unsafe_links_missing_references_and_empty_image_labels_reject() {
        for source in ["[x](javascript:alert)","[x](JaVaScRiPt:alert)","[x](data:text/html,x)","[x](//outside.test/path)","![ ](image.png)","[label][missing]"] {assert!(parse(source,0).is_err(),"{source}");}
        let doc=parse("# Links\n\n[x](../guide.md#hello) [web](https://example.com/docs) <https://example.com> ![A crab](crab.png)\n",0).unwrap();assert_eq!(doc.nodes.iter().filter(|node|matches!(node.kind,Kind::Link|Kind::Image)).count(),4);
        assert!(parse("é",1).is_err());
    }
    #[test]
    fn exact_unicode_source_and_crlf_are_independent_of_display_normalization() {
        let source="# Source\r\n\r\n```tsrust\r\nconst value = '🦀';\r\n```\r\n";
        let doc=parse(source,0).unwrap();let code=doc.nodes.iter().find(|node|node.kind==Kind::CodeBlock).unwrap();
        assert_eq!(code.raw,"const value = '🦀';\r\n");
        assert_eq!(code.tokens.iter().map(|token|&code.text[token.source.start..token.source.end]).collect::<String>(),code.text);
    }
}
