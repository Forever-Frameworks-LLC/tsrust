//! JSX layout uses lexer boundaries; child text keeps the parser's exact meaning.
use super::FormatOptions;
use super::trivia::{comments, indent_lines};
use crate::{Diagnostic, Span, syntax::{Token, TokenKind}};

fn ends(tokens: &[Token]) -> Result<Vec<usize>, Diagnostic> {
    let mut result = vec![0; tokens.len()];
    let mut stack = Vec::new();
    let mut closing = false;
    for (index, token) in tokens.iter().enumerate() {
        match token.kind {
            TokenKind::JsxOpen => { stack.push(index); closing = false; }
            TokenKind::JsxClose => closing = true,
            TokenKind::JsxEnd if closing || token.text == "/>" => {
                let Some(open) = stack.pop() else { return Err(invalid(token.span)); };
                result[open] = index;
                closing = false;
            }
            _ => {}
        }
    }
    if let Some(open) = stack.pop() { return Err(invalid(tokens[open].span)); }
    Ok(result)
}
fn invalid(span: Span) -> Diagnostic {
    Diagnostic::new("format", "Cannot lay out an incomplete TSRX element.", span)
}

struct Layout<'a> {
    source: &'a str,
    tokens: &'a [Token],
    ends: &'a [usize],
    options: &'a FormatOptions,
}

impl Layout<'_> {
    fn expression(&self, open: usize) -> Result<(String, usize), Diagnostic> {
        let mut depth = 1;
        let mut at = open + 1;
        while at < self.tokens.len() {
            let token = &self.tokens[at];
            if token.kind == TokenKind::JsxOpen { at = self.ends[at] + 1; continue; }
            if token.kind == TokenKind::Symbol {
                if token.text == "{" { depth += 1; }
                if token.text == "}" { depth -= 1; }
            }
            if depth == 0 {
                let content = &self.source[self.tokens[open].span.end..token.span.start];
                let formatted = super::format_with_options(content, self.options)?;
                let parsed = crate::syntax::lex(&formatted)?;
                let last = parsed.iter().rev().find(|token| token.kind != TokenKind::End).map_or(0, |token| token.span.end);
                let trailing_line_comment = comments(&formatted[last..])?.iter().any(|comment| comment.starts_with("//"));
                return Ok((format!("{{{}{}}}", formatted.trim(), if trailing_line_comment { "\n" } else { "" }), at + 1));
            }
            at += 1;
        }
        Err(invalid(self.tokens[open].span))
    }

    fn element(&self, open: usize, indent: usize) -> Result<String, Diagnostic> {
        let end = self.ends[open];
        let raw = &self.source[self.tokens[open].span.start..self.tokens[end].span.end];
        // shortcut: retain mixed text and multiline literals until trivia-aware reindentation is available.
        let mut scan = open + 1;
        while scan <= end {
            let token = &self.tokens[scan];
            if token.kind == TokenKind::JsxOpen { scan = self.ends[scan] + 1; continue; }
            if token.kind == TokenKind::JsxText && !crate::syntax::jsx::child_text(&token.text).is_empty()
                || matches!(token.kind, TokenKind::String | TokenKind::TemplateHead | TokenKind::TemplateMiddle | TokenKind::TemplateTail)
                    && token.text.contains('\n') { return Ok(raw.to_owned()); }
            scan += 1;
        }

        let name = self.tokens.get(open + 1).ok_or_else(|| invalid(self.tokens[open].span))?;
        let mut at = open + 2;
        let mut attributes = Vec::new();
        let mut previous_end = name.span.end;
        while at <= end && self.tokens[at].kind != TokenKind::JsxEnd {
            let gap = &self.source[previous_end..self.tokens[at].span.start];
            let mut attribute = if gap.chars().all(char::is_whitespace) { String::new() } else { format!("{}\n", comments(gap)?.join("\n")) };
            if self.tokens[at].text == "{" {
                let (expression, next) = self.expression(at)?;
                attribute.push_str(&expression);
                at = next;
            } else {
                attribute.push_str(&self.tokens[at].text);
                at += 1;
                if at <= end && self.tokens[at].text == "=" {
                    attribute.push('=');
                    at += 1;
                    if self.tokens[at].text == "{" {
                        let (expression, next) = self.expression(at)?;
                        attribute.push_str(&expression);
                        at = next;
                    } else {
                        attribute.push_str(&self.source[self.tokens[at].span.start..self.tokens[at].span.end]);
                        at += 1;
                    }
                }
            }
            previous_end = self.tokens[at - 1].span.end;
            attributes.push(attribute);
        }
        if at > end { return Err(invalid(self.tokens[open].span)); }
        let trailing = &self.source[previous_end..self.tokens[at].span.start];
        if !trailing.chars().all(char::is_whitespace) { attributes.push(comments(trailing)?.join("\n")); }
        let mut output = format!("<{}", name.text);
        let multiline = attributes.iter().any(|attribute| attribute.contains('\n') || attribute.starts_with("//"));
        for attribute in attributes {
            if multiline {
                output.push('\n');
                output.push_str(&indent_lines(&attribute, indent + self.options.indent_width)?);
            } else {
                output.push(' ');
                output.push_str(&attribute);
            }
        }
        if multiline { output.push('\n'); output.push_str(&" ".repeat(indent)); }
        if self.tokens[at].text == "/>" {
            if !multiline { output.push(' '); }
            output.push_str("/>");
            return Ok(output);
        }
        output.push('>');
        at += 1;
        let mut children = Vec::new();
        while at < end && self.tokens[at].kind != TokenKind::JsxClose {
            match self.tokens[at].kind {
                TokenKind::JsxText => at += 1,
                TokenKind::JsxOpen => {
                    children.push(self.element(at, indent + self.options.indent_width)?);
                    at = self.ends[at] + 1;
                }
                TokenKind::Symbol if self.tokens[at].text == "{" => {
                    let (expression, next) = self.expression(at)?;
                    let indentation = indent + self.options.indent_width;
                    let indented = indent_lines(&expression, indentation)?;
                    children.push(indented[indentation..].to_owned());
                    at = next;
                }
                _ => return Err(invalid(self.tokens[at].span)),
            }
        }
        if self.options.jsx_children_on_new_lines && !children.is_empty() {
            for child in children {
                output.push('\n');
                // Nested elements already carry their own continuation indentation.
                output.push_str(&" ".repeat(indent + self.options.indent_width));
                output.push_str(&child);
            }
            output.push('\n');
            output.push_str(&" ".repeat(indent));
        } else { output.push_str(&children.join("")); }
        output.push_str(&format!("</{}>", name.text));
        Ok(output)
    }
}

pub(super) fn layout(source: &str, options: &FormatOptions) -> Result<String, Diagnostic> {
    let tokens = crate::syntax::lex(source)?;
    let ends = ends(&tokens)?;
    let layout = Layout { source, tokens: &tokens, ends: &ends, options };
    let mut output = String::new();
    let mut offset = 0;
    let mut at = 0;
    while at < tokens.len() {
        if tokens[at].kind != TokenKind::JsxOpen { at += 1; continue; }
        let end = ends[at];
        let parenthesized = at >= 2 && tokens[at - 1].text == "(" && tokens[at - 2].text == "return"
            && tokens.get(end + 1).is_some_and(|token| token.text == ")");
        let direct = at > 0 && tokens[at - 1].text == "return"
            && tokens.get(end + 1).is_some_and(|token| matches!(token.text.as_str(), ";" | "}"));
        let wrap = options.jsx_return_parentheses && (direct || parenthesized);
        let unwrap = !options.jsx_return_parentheses && parenthesized;
        let start = if (wrap || unwrap) && parenthesized { tokens[at - 1].span.start } else { tokens[at].span.start };
        let line = &source[source[..start].rfind('\n').map_or(0, |index| index + 1)..start];
        let indent = line.bytes().take_while(|byte| *byte == b' ').count();
        output.push_str(&source[offset..start]);
        if wrap {
            output.push_str("(\n");
            output.push_str(&" ".repeat(indent + options.indent_width));
        }
        output.push_str(&layout.element(at, indent + if wrap { options.indent_width } else { 0 })?);
        if wrap { output.push('\n'); output.push_str(&" ".repeat(indent)); output.push(')'); }
        let consumed = end + usize::from((wrap || unwrap) && parenthesized);
        offset = tokens[consumed].span.end;
        at = consumed + 1;
    }
    output.push_str(&source[offset..]);
    Ok(output)
}

fn canonical(source: &str) -> Result<Vec<(TokenKind, String)>, Diagnostic> {
    let tokens = super::types::normalized_jsx_tokens(source)?;
    let ends = ends(&tokens)?;
    let mut skip = vec![false; tokens.len()];
    for at in 2..tokens.len() {
        if tokens[at].kind == TokenKind::JsxOpen && tokens[at - 1].text == "(" && tokens[at - 2].text == "return"
            && tokens.get(ends[at] + 1).is_some_and(|token| token.text == ")") {
            skip[at - 1] = true;
            skip[ends[at] + 1] = true;
        }
    }
    Ok(tokens.into_iter().enumerate().filter_map(|(index, token)| {
        if skip[index] { return None; }
        if token.kind == TokenKind::JsxText {
            let text = crate::syntax::jsx::child_text(&token.text);
            if text.is_empty() { return None; }
            return Some((token.kind, text));
        }
        Some((token.kind, token.text))
    }).collect())
}

pub(super) fn verify(before: &str, after: &str) -> Result<(), Diagnostic> {
    if canonical(before)? != canonical(after)? || comments(before)? != comments(after)? {
        return Err(Diagnostic::new("format", "Formatting would change source tokens or JSX text.", Span::new(0, 0)));
    }
    Ok(())
}
