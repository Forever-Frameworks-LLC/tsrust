//! Type punctuation is recognized by the shared annotation parser.
use crate::syntax::{Token, TokenKind};
use crate::Span;

pub(super) fn generic_spans(tokens: &[Token]) -> Vec<Span> {
    let real = &tokens[..tokens.len().saturating_sub(1)];
    let mut spans: Vec<Span> = Vec::new();
    for (index, token) in real.iter().enumerate() {
        if spans.iter().any(|span| span.start <= token.span.start && token.span.start < span.end)
            || token.kind != TokenKind::Name
            || real.get(index + 1).is_none_or(|next| next.text != "<") {
            continue;
        }
        let previous = index.checked_sub(1).map(|before| real[before].text.as_str());
        let alias_result = previous == Some("=") && index >= 3 && real[index-3].text == "type";
        let type_position = matches!(previous,Some(":"|"function"|"type"|"interface"|"new"|"extends"|"as"|"satisfies"))
            || alias_result
            || matches!(token.text.as_str(),"Function"|"Mutable"|"ReadOnly"|"Once"|"Owned");
        let mut annotation_tokens = tokens.to_vec();
        if let Ok((_, annotation, end)) = crate::type_syntax::parse_parameter_prefix(&mut annotation_tokens,index,0) {
            // A parsed type application followed by call parentheses is explicit
            // call syntax. Comparisons and shifts retain their operator spacing.
            let call = annotation_tokens.get(end).is_some_and(|next| next.text == "(");
            if type_position || call {
                spans.push(Span::new(token.span.start,annotation_tokens[end-1].span.end.max(annotation.span.end)));
            }
        }
    }
    spans
}
