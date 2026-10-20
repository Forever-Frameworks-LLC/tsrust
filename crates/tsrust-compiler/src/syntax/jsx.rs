//! JSX uses the ordinary expression scheduler and lowers to checked component calls.
use super::*;
use super::scheduler::{Machine, Work};

#[derive(Clone, Copy)]
pub(super) enum Mode { Tag { closing: bool }, Children, Expression { braces: usize } }

pub(super) fn lex_step(source: &str, i: &mut usize, tokens: &mut Vec<Token>, modes: &mut Vec<Mode>, policy: NestingPolicy) -> Result<bool, Diagnostic> {
    let start = *i;
    let rest = &source[start..];
    let children = matches!(modes.last(), Some(Mode::Children));
    let opening = rest.starts_with('<') && rest.as_bytes().get(1).is_some_and(u8::is_ascii_uppercase)
        && (children || !matches!(modes.last(), Some(Mode::Tag { .. })) && tokens.last().is_none_or(|t| matches!(t.text.as_str(), "return" | "=>" | "=" | "(" | "[" | "," | ":" | "?" | "{" | "&&" | "||" | "??")));
    let mut push = |text: String, end: usize, kind: TokenKind| {
        tokens.push(Token { text, span: Span::new(start, end), kind });
        *i = end;
    };
    if opening {
        if policy.reached(modes.len()) { return Err(Diagnostic::new("unsupported", "JSX nesting exceeds the prototype limit of 128.", Span::new(start,start+1))); }
        modes.push(Mode::Tag { closing: false });
        push("<".into(), start+1, TokenKind::JsxOpen);
        return Ok(true);
    }
    if children {
        if rest.starts_with("</") {
            modes.push(Mode::Tag { closing: true });
            push("</".into(),start+2,TokenKind::JsxClose);
        } else if rest.starts_with('{') {
            modes.push(Mode::Expression { braces: 0 });
            push("{".into(),start+1,TokenKind::Symbol);
        } else {
            let length=rest.find(['<','{']).unwrap_or(rest.len());
            if length==0 { return Err(Diagnostic::new("unsupported","JSX requires a named uppercase component; fragments and intrinsic HTML tags are unsupported.",Span::new(start,start+1))); }
            push(rest[..length].into(),start+length,TokenKind::JsxText);
        }
        return Ok(true);
    }
    if let Some(Mode::Tag { closing }) = modes.last().copied() {
        if rest.starts_with('>') || rest.starts_with("/>") {
            let empty=rest.starts_with("/>");
            if closing && empty { return Err(Diagnostic::new("syntax","Closing JSX tags cannot be self-closing.",Span::new(start,start+2))); }
            modes.pop();
            if closing { modes.pop(); } else if !empty { modes.push(Mode::Children); }
            push(if empty {"/>"} else {">"}.into(),start+if empty {2}else{1},TokenKind::JsxEnd);
            return Ok(true);
        }
        if rest.starts_with('{') {
            modes.push(Mode::Expression { braces: 0 });
            push("{".into(),start+1,TokenKind::Symbol);
            return Ok(true);
        }
        if rest.as_bytes().first().is_some_and(|ch| ch.is_ascii_alphabetic() || *ch == b'_') {
            let length = rest.bytes().take_while(|ch| ch.is_ascii_alphanumeric() || matches!(ch, b'_' | b'-')).count();
            push(rest[..length].into(), start + length, TokenKind::Name);
            return Ok(true);
        }
    } else if let Some(Mode::Expression { braces })=modes.last_mut() {
        if rest.starts_with('}') && *braces==0 {
            modes.pop();
            push("}".into(),start+1,TokenKind::Symbol);
            return Ok(true);
        }
        if rest.starts_with('{') { *braces+=1; }
        if rest.starts_with('}') { *braces-=1; }
    }
    Ok(false)
}

pub(super) struct Frame { name: Token, start: usize, fields: Vec<(Token,Expr)>, children: Vec<Expr> }

impl Parser<'_> {
    pub(super) fn jsx_start(&mut self, machine: &mut Machine) -> Result<(),Diagnostic> {
        let start=self.take().span.start;
        let name=self.name()?;
        machine.work.push(Work::JsxAttributes(Box::new(Frame {name,start,fields:Vec::new(),children:Vec::new()})));
        Ok(())
    }
    pub(super) fn jsx_step(&mut self, work: Work, machine: &mut Machine) -> Result<(),Diagnostic> {
        match work {
            Work::JsxAttributes(mut frame) => {
                if self.accept("/>") { self.jsx_finish(frame,machine)?; }
                else if self.accept(">") { machine.work.push(Work::JsxChildren(frame)); }
                else if self.accept("{") {
                    let mut name=self.expect("...")?;name.text="record#spread".into();name.kind=TokenKind::JsxOpen;
                    machine.request(Work::JsxAttribute(frame,Box::new(name)),Work::Parse(0));
                }
                else {
                    let mut name=self.name()?;name.kind=TokenKind::JsxOpen;
                    if frame.fields.iter().any(|(key,_)|key.text==name.text) {
                        return Err(Diagnostic::new("syntax","JSX attributes must be unique.",name.span));
                    }
                    if self.accept("=") {
                        if self.accept("{") {
                            machine.request(Work::JsxAttribute(frame,Box::new(name)),Work::Parse(0));
                        } else if self.peek().kind==TokenKind::String {
                            let value=self.take();
                            let text=entities(&value.text,value.span,true)?;
                            if name.text=="className" {tsrust_ui::style::validate(&text).map_err(|message|Diagnostic::new("ui-class",message,value.span))?;}
                            frame.fields.push((name,Expr::new(ExprKind::String(text),value.span)));
                            machine.work.push(Work::JsxAttributes(frame));
                        } else { return Err(Diagnostic::new("syntax","Expected a quoted JSX attribute or {expression}.",self.peek().span)); }
                    } else {
                        let span=name.span;
                        frame.fields.push((name,Expr::new(ExprKind::Boolean(true),span)));
                        machine.work.push(Work::JsxAttributes(frame));
                    }
                }
            }
            Work::JsxAttribute(mut frame,name) => {
                let value=machine.value(); self.expect("}")?;
                if name.text=="className" && let ExprKind::String(text)=&value.kind {tsrust_ui::style::validate(text).map_err(|message|Diagnostic::new("ui-class",message,value.span))?;}
                frame.fields.push((*name,value)); machine.work.push(Work::JsxAttributes(frame));
            }
            Work::JsxChildren(mut frame) => {
                if self.peek().kind==TokenKind::JsxClose {
                    self.take(); let close=self.name()?;
                    if close.text!=frame.name.text { return Err(Diagnostic::new("syntax",format!("Expected closing JSX tag {}.",frame.name.text),close.span)); }
                    self.expect(">")?; self.jsx_finish(frame,machine)?;
                } else if self.peek().kind==TokenKind::JsxText {
                    let text=self.take();
                    let value=entities(&child_text(&text.text),text.span,false)?;
                    if !value.is_empty() { frame.children.push(Expr::new(ExprKind::String(value),text.span)); }
                    machine.work.push(Work::JsxChildren(frame));
                } else if self.accept("{") {
                    machine.request(Work::JsxChild(frame,true),Work::Parse(0));
                } else if self.peek().kind==TokenKind::JsxOpen {
                    machine.request(Work::JsxChild(frame,false),Work::Parse(0));
                } else { return Err(Diagnostic::new("syntax","Expected JSX child or closing tag.",self.peek().span)); }
            }
            Work::JsxChild(mut frame,braced) => {
                let value=machine.value(); if braced { self.expect("}")?; }
                frame.children.push(value); machine.work.push(Work::JsxChildren(frame));
            }
            _=>unreachable!("JSX work"),
        }
        Ok(())
    }
    fn jsx_finish(&mut self, mut frame: Box<Frame>, machine: &mut Machine) -> Result<(),Diagnostic> {
        let span=Span::new(frame.start,self.tokens[self.i-1].span.end);
        if !frame.children.is_empty() {
            if let Some((name,_))=frame.fields.iter().find(|(name,_)|name.text=="children"){
                return Err(Diagnostic::new("syntax","Supply JSX children either as a children attribute or between the tags, not both.",name.span));
            }
            let child_span=Span::new(frame.children[0].span.start,frame.children.last().unwrap().span.end);
            frame.fields.push((Token{text:"children".into(),span:child_span,kind:TokenKind::Name},Expr::new(ExprKind::Array(frame.children),child_span)));
        }
        let props=Expr::new(ExprKind::Record(frame.fields),span);
        let callee=Expr::new(ExprKind::Name(frame.name.text),frame.name.span);
        machine.values.push(Expr::new(ExprKind::Call(Box::new(callee),vec![props]),span));
        machine.work.push(Work::Postfix{optional:false});
        Ok(())
    }
}

// JSX indentation is not text; explicit {" "} keeps an intentional separator.
fn child_text(text: &str) -> String {
    let lines=text.split('\n').collect::<Vec<_>>();
    let last=lines.len()-1;
    lines.into_iter().enumerate().filter_map(|(index,line)| {
        let line=line.trim_end_matches('\r').replace('\t'," ");
        let line=if index>0 {line.trim_start_matches(' ')} else {&line};
        let line=if index<last {line.trim_end_matches(' ')} else {line};
        (!line.is_empty()).then(||line.to_owned())
    }).collect::<Vec<_>>().join(" ")
}

fn entities(text: &str, span: Span,attribute:bool) -> Result<String,Diagnostic> {
    let mut output=String::new(); let mut remaining=text;
    while let Some(at)=remaining.find('&') {
        output.push_str(&remaining[..at]); remaining=&remaining[at+1..];
        let end=remaining.find(';');
        if attribute && !remaining.starts_with('#') && end.is_none_or(|end|!remaining[..end].bytes().all(|ch|ch.is_ascii_alphanumeric()||ch==b'#')) {output.push('&');continue;}
        let Some(end)=end else {return Err(Diagnostic::new("syntax","Unterminated JSX entity; use {\"&\"} for a literal ampersand.",span));};
        let entity=&remaining[..end];
        let value=match entity {
            "amp"=>Some('&'),"lt"=>Some('<'),"gt"=>Some('>'),"quot"=>Some('"'),"apos"=>Some('\''),
            _=>entity.strip_prefix("#x").and_then(|digits|u32::from_str_radix(digits,16).ok()).or_else(||entity.strip_prefix('#').and_then(|digits|digits.parse::<u32>().ok())).and_then(char::from_u32),
        }.ok_or_else(||Diagnostic::new("syntax","Unsupported or invalid JSX entity; use a braced string for exact text.",span))?;
        output.push(value); remaining=&remaining[end+1..];
    }
    output.push_str(remaining); Ok(output)
}

/// Shared tolerant tag context for editor attribute help; expressions retain normal help.
pub fn attribute_context(source: &str, offset: usize) -> Option<(Span, Vec<String>)> {
    let tokens=super::lex(source.get(..offset)?).ok()?;
    let mut open=None; let mut braces=0usize;
    for (index,token) in tokens.iter().enumerate() {
        match token.kind {
            TokenKind::JsxOpen=>{open=Some(index);braces=0;},
            TokenKind::JsxEnd=>open=None,
            TokenKind::Symbol if token.text=="{"=>braces+=1,
            TokenKind::Symbol if token.text=="}"=>braces=braces.saturating_sub(1),
            _=>{},
        }
    }
    if braces!=0 {return None;}
    let open=open?; let component=tokens.get(open+1)?;
    if component.kind!=TokenKind::Name {return None;}
    let mut written=Vec::new(); let mut depth=0usize;
    for token in tokens.iter().skip(open+2) {
        if token.kind==TokenKind::Symbol && token.text=="{" {depth+=1;}
        if token.kind==TokenKind::Symbol && token.text=="}" {depth=depth.saturating_sub(1);continue;}
        if depth==0 && token.kind==TokenKind::Name && token.span.end<offset {written.push(token.text.clone());}
    }
    Some((component.span,written))
}
