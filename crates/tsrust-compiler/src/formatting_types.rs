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
    let mut spans: Vec<Span> = Vec::new();
    for (index, token) in real.iter().enumerate() {
        if spans
            .iter()
            .any(|span| span.start <= token.span.start && token.span.start < span.end)
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
        let mut annotation_tokens = tokens.to_vec();
        if let Ok((_, annotation, end)) =
            crate::type_syntax::parse_parameter_prefix(&mut annotation_tokens, index, 0)
        {
            let call = application
                && annotation_tokens
                    .get(end)
                    .is_some_and(|next| next.text == "(");
            if type_position || call {
                spans.push(Span::new(
                    token.span.start,
                    annotation_tokens[end - 1].span.end.max(annotation.span.end),
                ));
            }
        }
    }
    spans
}
