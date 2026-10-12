//! Reporting-only suppression ranges. Never changes syntax or checked programs.
use super::{Parser, Token, TokenKind, lex};
use crate::Span;

#[derive(Default)]
pub struct DiagnosticSuppressions { ranges: Vec<Span> }
impl DiagnosticSuppressions {
    pub fn new(source: &str) -> Self {
        if !source.contains("@tsrust ignore") { return Self::default(); }
        let Some(tokens) = reporting_tokens(source) else { return Self::default(); };
        let mut ranges = Vec::new();
        let mut previous = 0;
        for (index, token) in tokens.iter().enumerate() {
            let gap = &source[previous..token.span.start];
            if directive(gap) && token.kind != TokenKind::End {
                let mut parser = Parser::new(source, Default::default(), Default::default()).ok();
                let parsed_end = parser.as_mut().and_then(|parser| {
                    parser.i = index;
                    if parser.is("export") { parser.take(); parser.accept("default"); }
                    parser.block_statement().ok()?;
                    Some(parser.tokens[parser.i.saturating_sub(1)].span.end)
                });
                ranges.push(Span::new(token.span.start, parsed_end.unwrap_or_else(|| boundary(&tokens[index..]))));
            }
            previous = token.span.end;
        }
        Self { ranges }
    }
    pub fn suppresses(&self, span: Span) -> bool {
        self.ranges.iter().any(|range| range.start <= span.start && span.start < range.end)
    }
}

// Preserve opaque invalid tokens while recovering later statement boundaries.
// If a prefix cannot be lexed independently (for example an unfinished template
// interpolation), keep diagnostics visible rather than guessing at comments.
fn reporting_tokens(source: &str) -> Option<Vec<Token>> {
    let mut output = Vec::new();
    let mut offset = 0;
    loop {
        match lex(&source[offset..]) {
            Ok(mut tokens) => {
                for token in &mut tokens { token.span.start += offset; token.span.end += offset; }
                output.extend(tokens);
                return Some(output);
            }
            Err(error) => {
                let mut prefix = lex(&source[offset..offset + error.span.start]).ok()?;
                prefix.pop();
                for token in &mut prefix { token.span.start += offset; token.span.end += offset; }
                output.extend(prefix);
                if error.span.end == 0 { return None; }
                output.push(Token { kind: TokenKind::Symbol, text: "<invalid>".into(),
                    span: Span::new(offset + error.span.start, offset + error.span.end) });
                offset += error.span.end;
            }
        }
    }
}

fn directive(mut trivia: &str) -> bool {
    let mut found = false;
    while !trivia.is_empty() {
        trivia = trivia.trim_start();
        if let Some(rest) = trivia.strip_prefix("/*") {
            let Some(end) = rest.find("*/") else { break; };
            trivia = &rest[end + 2..];
        } else if let Some(rest) = trivia.strip_prefix("//") {
            let end = rest.find(['\n', '\r', '\u{2028}', '\u{2029}']).unwrap_or(rest.len());
            if let Some(reason) = rest[..end].trim().strip_prefix("@tsrust ignore -") {
                found |= !reason.trim().is_empty();
            }
            trivia = &rest[end..];
        } else { break; }
    }
    found
}

// Recovery boundary for malformed statements and type/module declarations.
// Delimiters inside literals are token contents, not structural punctuation.
fn boundary(tokens: &[Token]) -> usize {
    let block = tokens.iter().take(3).any(|t| t.kind == TokenKind::Name && matches!(t.text.as_str(), "function" | "interface" | "enum" | "if" | "for" | "while" | "switch" | "try"));
    let mut stack = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind == TokenKind::End { return token.span.start; }
        if token.kind != TokenKind::Symbol { continue; }
        match token.text.as_str() {
            "(" => stack.push(")"), "[" => stack.push("]"), "{" => stack.push("}"),
            ")" | "]" | "}" => {
                if stack.last().copied() != Some(token.text.as_str()) { return token.span.start; }
                stack.pop();
                if stack.is_empty() && block && token.text == "}" {
                    if tokens.get(index + 1).is_some_and(|t| matches!(t.text.as_str(), "else" | "catch" | "finally")) { continue; }
                    return token.span.end;
                }
            }
            ";" if stack.is_empty() => return token.span.end,
            _ => {}
        }
    }
    tokens.last().map_or(0, |t| t.span.end)
}
