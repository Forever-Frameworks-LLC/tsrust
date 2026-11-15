//! Normalize whitespace outside comments without changing any comment bytes.
use crate::{Diagnostic, Span};

pub(super) fn comment_spans(source: &str) -> Result<Vec<Span>, Diagnostic> {
    let mut comments = Vec::new();
    let mut offset = 0;
    for token in crate::syntax::lex(source)? {
        let mut gap = &source[offset..token.span.start];
        let mut base = offset;
        while !gap.trim().is_empty() {
            let trimmed = gap.trim_start();
            base += gap.len() - trimmed.len();
            gap = trimmed;
            let length = if gap.starts_with("//") { gap.find('\n').unwrap_or(gap.len()) }
                else if gap.starts_with("/*") { gap.find("*/").map_or(gap.len(), |end| end + 2) }
                else { gap.len() };
            comments.push(Span::new(base, base + length));
            gap = &gap[length..];
            base += length;
        }
        offset = token.span.end;
    }
    Ok(comments)
}

pub(super) fn comments(source: &str) -> Result<Vec<String>, Diagnostic> {
    Ok(comment_spans(source)?.into_iter().map(|span| source[span.start..span.end].to_owned()).collect())
}

pub(super) fn indent_lines(text: &str, indent: usize) -> Result<String, Diagnostic> {
    let protected = comment_spans(text)?;
    let mut offset = 0;
    let mut output = String::new();
    for line in text.split_inclusive('\n') {
        let inside_comment = protected.iter().any(|span| span.start < offset && offset < span.end);
        if !inside_comment && !line.trim().is_empty() { output.push_str(&" ".repeat(indent)); }
        output.push_str(line);
        offset += line.len();
    }
    Ok(output)
}

fn whitespace(output: &mut String, text: &str, indent: usize) {
    let lines = text.bytes().filter(|byte| *byte == b'\n').count();
    if lines == 0 { output.push_str(text); }
    else { output.push_str(&"\n".repeat(lines)); output.push_str(&" ".repeat(indent)); }
}

pub(super) fn reindent_gap(gap: &str, comment_indent: usize, code_indent: usize) -> Result<String, Diagnostic> {
    let mut output = String::new();
    let mut offset = 0;
    for span in comment_spans(gap)? {
        whitespace(&mut output, &gap[offset..span.start], comment_indent);
        output.push_str(&gap[span.start..span.end]);
        offset = span.end;
    }
    whitespace(&mut output, &gap[offset..], code_indent);
    Ok(output)
}
