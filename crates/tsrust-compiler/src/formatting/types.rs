//! Type punctuation is recognized by the shared annotation parser.
use crate::Span;
use crate::syntax::{Token, TokenKind};

pub(super) fn generic_spans(tokens: &[Token]) -> Vec<Span> {
    let real = &tokens[..tokens.len().saturating_sub(1)];
    let mut starts = vec![false; real.len()];
    let (mut braces, mut angles) = (0usize, 0usize);
    let (mut alias_header, mut interface_header) = (false, false);
    for (index, token) in real.iter().enumerate() {
        let previous = index
            .checked_sub(1)
            .map(|before| real[before].text.as_str());
        if braces == 0
            && token.kind == TokenKind::Name
            && matches!(previous, None | Some(";" | "}" | "export"))
        {
            alias_header = token.text == "type";
            interface_header = token.text == "interface";
            angles = 0;
        }
        if alias_header || interface_header {
            match token.text.as_str() {
                "<" => angles += 1,
                ">" | ">>" | ">>>" => angles = angles.saturating_sub(token.text.len()),
                "=" if alias_header && angles == 0 => {
                    if index + 1 < starts.len() {
                        starts[index + 1] = true;
                    }
                    alias_header = false;
                }
                "," if interface_header && angles == 0 => {
                    if index + 1 < starts.len() {
                        starts[index + 1] = true;
                    }
                }
                _ => {}
            }
        }
        if token.kind == TokenKind::Symbol {
            match token.text.as_str() {
                "{" => {
                    braces += 1;
                    alias_header = false;
                    interface_header = false;
                }
                "}" => braces = braces.saturating_sub(1),
                ";" => {
                    alias_header = false;
                    interface_header = false;
                }
                _ => {}
            }
        }
    }
    let headers = crate::syntax::alias_parameter_spans(tokens);
    let mut spans: Vec<Span> = Vec::new();
    let mut annotation_tokens = tokens.to_vec();
    for (index, token) in real.iter().enumerate() {
        if let Some(span) = headers.iter().find(|span| span.start == token.span.start) { spans.push(*span); continue; }
        if spans.last().is_some_and(|span| span.start <= token.span.start && token.span.start < span.end)
        {
            continue;
        }
        let previous = index
            .checked_sub(1)
            .map(|before| real[before].text.as_str());
        let type_position = starts[index]
            || matches!(
                previous,
                Some(
                    ":" | "function"
                        | "type"
                        | "interface"
                        | "new"
                        | "extends"
                        | "as"
                        | "satisfies"
                )
            )
            || matches!(
                token.text.as_str(),
                "Function" | "Mutable" | "ReadOnly" | "Once" | "Owned"
            );
        let application = token.kind == TokenKind::Name
            && real.get(index + 1).is_some_and(|next| next.text == "<");
        if !type_position && !application {
            continue;
        }
        if let Ok((span, call)) = crate::type_syntax::parameter_prefix_span(&mut annotation_tokens, index) {
            if type_position || application && call { spans.push(span); }
        }
    }
    spans
}

pub(super) fn contains(spans: &[Span], at: usize) -> bool {
    let end = spans.partition_point(|span| span.start <= at);
    end > 0 && at < spans[end - 1].end
}

/// Normalize lexer-packed generic closers on both sides of token preservation.
/// Comparisons and shifts outside parsed type spans retain exact operator tokens.
pub(super) fn normalized_tokens(source: &str) -> Result<Vec<Token>, crate::Diagnostic> {
    let mut tokens = crate::syntax::lex(source)?;
    // Preserve JSX islands byte-for-byte until their dedicated layout rules land.
    // Surrounding declarations still use the ordinary formatter.
    let mut at=0;
    while at<tokens.len() {
        if tokens[at].kind != TokenKind::JsxOpen { at+=1; continue; }
        let start=tokens[at].span.start;
        let mut depth=0usize; let mut closing=false; let mut end=at;
        for (index,token) in tokens.iter().enumerate().skip(at) {
            match token.kind {
                TokenKind::JsxOpen => {depth+=1; closing=false;}
                TokenKind::JsxClose => closing=true,
                TokenKind::JsxEnd if closing || token.text=="/>" => { depth-=1; closing=false; },
                _=>{},
            }
            if depth==0 {end=index;break;}
        }
        if end==at { return Err(crate::Diagnostic::new("syntax","Unterminated JSX element.",tokens[at].span)); }
        let span=Span::new(start,tokens[end].span.end);
        tokens.splice(at..=end,[Token{text:source[span.start..span.end].into(),span,kind:TokenKind::String}]);
        at+=1;
    }
    let spans = generic_spans(&tokens);
    let mut index = 0;
    while index < tokens.len() {
        if matches!(tokens[index].text.as_str(), ">>" | ">>>" | ">=" | ">>=" | ">>>=")
            && contains(&spans, tokens[index].span.start)
        {
            crate::type_syntax::split_closer(&mut tokens, index);
        }
        index += 1;
    }
    Ok(tokens)
}
