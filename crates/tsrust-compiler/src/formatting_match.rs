//! Contextual match headers and direct arm separators, without parsing bodies.
use crate::syntax::{Token, TokenKind};

pub(super) fn layout(tokens: &[Token]) -> (Vec<bool>, Vec<bool>) {
    let mut pairs = vec![None; tokens.len()];
    let mut stack = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != TokenKind::Symbol { continue; }
        match token.text.as_str() {
            "(" | "[" | "{" => stack.push(index),
            ")" | "]" | "}" => {
                let expected = match token.text.as_str() { ")" => "(", "]" => "[", _ => "{" };
                if stack.last().is_some_and(|&open| tokens[open].text == expected) {
                    let open = stack.pop().unwrap();
                    pairs[open] = Some(index);
                }
            }
            _ => {},
        }
    }
    let mut headers = vec![false; tokens.len()];
    let mut bodies = vec![false; tokens.len()];
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != TokenKind::Name || token.text != "match"
            || index.checked_sub(1).is_some_and(|previous| matches!(tokens[previous].text.as_str(), "function" | "." | "?."))
            || !tokens.get(index + 1).is_some_and(|next| next.text == "(") { continue; }
        let Some(close) = pairs[index + 1] else { continue; };
        if tokens.get(close + 1).is_some_and(|next| next.text == "{") {
            headers[index + 1] = true;
            bodies[close + 1] = true;
        }
    }
    stack.clear();
    let mut commas = vec![false; tokens.len()];
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != TokenKind::Symbol { continue; }
        match token.text.as_str() {
            "(" | "[" | "{" => stack.push(index),
            ")" | "]" | "}" => { stack.pop(); },
            "," if stack.last().is_some_and(|&open| bodies[open]) => commas[index] = true,
            _ => {},
        }
    }
    (headers, commas)
}
