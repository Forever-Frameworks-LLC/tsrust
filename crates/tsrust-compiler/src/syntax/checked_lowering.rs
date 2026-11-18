//! `Checked<T>` and `Checked<T, E>` lower to ordinary syntax before parsing.
//!
//! A function whose declared result is `Checked<T, E>` (or `Task<Checked<T, E>>`)
//! returns `Result<T, RuntimeError | E>`. Its body runs inside `try`: each of its
//! own `return value` statements becomes `return Ok(value)`, and a fault or a
//! thrown `E` reaches the generated `catch`, which returns `Err(error)`. In a
//! function type (`Function<Checked<T>>`) only the result type is rewritten.
//! Inside `try`, local faults are already values and calls that may unwind are
//! caught one by one, so the lowering adds no new runtime machinery.
//!
//! The rewrite works on tokens: user tokens keep their spans, so diagnostics in
//! the body point at the source as written. Generated tokens carry the span of
//! the `Checked` name.
use super::{Token, TokenKind};
use crate::{Diagnostic, Span};

/// The catch binding in generated code; not a name users can collide with.
const FAILURE: &str = "tsrustCheckedFailure";

pub(crate) fn lower(mut tokens: Vec<Token>) -> Result<Vec<Token>, Diagnostic> {
    runtime_error_names(&mut tokens);
    if !tokens.iter().any(|token| token.kind == TokenKind::Name && token.text == "Checked") {
        return Ok(tokens);
    }
    let tokens = split_closers(tokens);
    let mut edits: Vec<Edit> = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        if is_name(&tokens[index], "Checked") && tokens.get(index + 1).is_some_and(|next| is_symbol(next, "<")) {
            index = checked(&tokens, index, &mut edits)?;
        } else {
            index += 1;
        }
    }
    Ok(apply(tokens, edits))
}

/// The public type name `RuntimeError` is the record `catch` receives for
/// faults, internally `#RuntimeError`, unless the module declares its own.
fn runtime_error_names(tokens: &mut [Token]) {
    let declared = tokens.windows(2).any(|pair| matches!(pair[0].text.as_str(), "type" | "interface" | "enum" | "class") && is_name(&pair[1], "RuntimeError"));
    if declared {
        return;
    }
    // Member names (`value.RuntimeError`) stay ordinary fields.
    for index in 0..tokens.len() {
        let member = index > 0 && matches!(tokens[index - 1].text.as_str(), "." | "?.") && tokens[index - 1].kind == TokenKind::Symbol;
        if is_name(&tokens[index], "RuntimeError") && !member {
            tokens[index].text = "#RuntimeError".into();
        }
    }
}

/// Replace `count` tokens at `at` with `tokens`; `count == 0` inserts.
struct Edit {
    at: usize,
    count: usize,
    tokens: Vec<Token>,
}

fn is_name(token: &Token, text: &str) -> bool {
    token.kind == TokenKind::Name && token.text == text
}

fn is_symbol(token: &Token, text: &str) -> bool {
    token.kind == TokenKind::Symbol && token.text == text
}

/// `>>` and `>>>` close several type argument lists at once; the rewrite needs
/// each `>` alone. Expressions never contain `Checked<`, so only closers that
/// follow one are split.
fn split_closers(tokens: Vec<Token>) -> Vec<Token> {
    let mut output = Vec::with_capacity(tokens.len());
    let mut open = 0usize;
    for token in tokens {
        if is_name(&token, "Checked") {
            open = open.max(1);
        }
        if open > 0 && token.kind == TokenKind::Symbol && token.text.len() > 1 && token.text.bytes().all(|byte| byte == b'>') {
            for offset in 0..token.text.len() {
                let start = token.span.start + offset;
                output.push(Token { text: ">".into(), span: Span::new(start, start + 1), kind: TokenKind::Symbol });
            }
            open = 0;
            continue;
        }
        if open > 0 && (is_symbol(&token, "{") || is_symbol(&token, ";") || is_symbol(&token, "=>")) {
            open = 0;
        }
        output.push(token);
    }
    output
}

fn generated(span: Span, text: &str) -> Vec<Token> {
    super::lex(text)
        .expect("generated Checked syntax lexes")
        .into_iter()
        .filter(|token| token.kind != TokenKind::End)
        .map(|token| Token { span, ..token })
        .collect()
}

fn apply(tokens: Vec<Token>, mut edits: Vec<Edit>) -> Vec<Token> {
    // Stable by position: insertions at one index keep the order they were planned in.
    edits.sort_by_key(|edit| edit.at);
    let mut output = Vec::with_capacity(tokens.len() + edits.iter().map(|edit| edit.tokens.len()).sum::<usize>());
    let tokens = tokens.into_iter().enumerate();
    let mut edits = edits.into_iter().peekable();
    let mut skip = 0usize;
    for (index, token) in tokens {
        while let Some(edit) = edits.next_if(|edit| edit.at == index) {
            skip = skip.max(edit.count);
            output.extend(edit.tokens);
        }
        if skip > 0 {
            skip -= 1;
            continue;
        }
        output.push(token);
    }
    output.extend(edits.flat_map(|edit| edit.tokens));
    output
}

/// Index of the token that closes the bracket opened at `open`. Angle
/// brackets count only angle brackets (closers were split); the others count
/// `(`, `[` and `{` together.
fn closing(tokens: &[Token], open: usize) -> Option<usize> {
    let angle = is_symbol(&tokens[open], "<");
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(open) {
        if token.kind != TokenKind::Symbol {
            continue;
        }
        let (opens, closes) = if angle {
            (token.text == "<", token.text == ">")
        } else {
            (matches!(token.text.as_str(), "(" | "[" | "{"), matches!(token.text.as_str(), ")" | "]" | "}"))
        };
        if opens {
            depth += 1;
        } else if closes {
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

/// Rewrite one `Checked<...>` starting at `at`; returns where scanning resumes.
fn checked(tokens: &[Token], at: usize, edits: &mut Vec<Edit>) -> Result<usize, Diagnostic> {
    let span = tokens[at].span;
    let close = closing(tokens, at + 1).ok_or_else(|| Diagnostic::new("checked", "Checked needs a closing `>`.", span))?;
    let arguments = &tokens[at + 2..close];
    let mut depth = 0usize;
    let mut comma = None;
    for (offset, token) in arguments.iter().enumerate() {
        match token.text.as_str() {
            "(" | "[" | "{" | "<" if token.kind == TokenKind::Symbol => depth += 1,
            ")" | "]" | "}" | ">" if token.kind == TokenKind::Symbol => depth = depth.saturating_sub(1),
            "," if depth == 0 && token.kind == TokenKind::Symbol => {
                if comma.is_some() {
                    return Err(Diagnostic::new("checked", "Checked takes a result type and an optional error type, such as Checked<int, ParseError>.", span));
                }
                comma = Some(offset);
            }
            _ => {}
        }
    }
    let (result, error) = match comma {
        Some(comma) => (&arguments[..comma], Some(&arguments[comma + 1..])),
        None => (arguments, None),
    };
    if result.is_empty() || error.is_some_and(<[Token]>::is_empty) {
        return Err(Diagnostic::new("checked", "Checked needs a result type, such as Checked<int>.", span));
    }
    if result.iter().chain(error.into_iter().flatten()).any(|token| is_name(token, "Checked")) {
        return Err(Diagnostic::new("checked", "Checked cannot be nested; one Checked result already carries every failure.", span));
    }
    // `Checked<T, E>` is `Result<T, RuntimeError | (E)>`.
    let mut replacement = generated(span, "Result<");
    replacement.extend(result.iter().cloned());
    replacement.extend(generated(span, ","));
    replacement.push(Token { text: "#RuntimeError".into(), span, kind: TokenKind::Name });
    if let Some(error) = error {
        replacement.extend(generated(span, "| ("));
        replacement.extend(error.iter().cloned());
        replacement.extend(generated(span, ")"));
    }
    replacement.extend(generated(span, ">"));
    edits.push(Edit { at, count: close - at + 1, tokens: replacement });
    let void = result.len() == 1 && is_name(&result[0], "void");
    match position(tokens, at) {
        Position::Result { after } => body(tokens, after, span, void, edits)?,
        Position::Type => {}
        Position::Main => return Err(Diagnostic::new("checked", "main cannot return Checked; handle failures inside main or let them end the program.", span)),
        Position::Other => return Err(Diagnostic::new("checked", "Checked is only allowed as a function's declared result type, such as function parse(text: string): Checked<int>.", span)),
    }
    Ok(close + 1)
}

enum Position {
    /// A function result; its body starts after token `after`.
    Result { after: usize },
    /// A function type's result (`Function<Checked<T>>`): the type alone.
    Type,
    Main,
    Other,
}

fn position(tokens: &[Token], at: usize) -> Position {
    // `Task<Checked<T>>` declares an async function's result.
    let (mut start, mut end) = (at, closing(tokens, at + 1).unwrap_or(at));
    if at >= 2 && is_symbol(&tokens[at - 1], "<") && is_name(&tokens[at - 2], "Task") {
        start = at - 2;
        match tokens.get(end + 1) {
            Some(token) if is_symbol(token, ">") => end += 1,
            _ => return Position::Other,
        }
    }
    if start >= 2 && is_symbol(&tokens[start - 1], "<") && is_name(&tokens[start - 2], "Function") {
        return Position::Type;
    }
    if start == 0 || !is_symbol(&tokens[start - 1], ":") || start < 2 || !is_symbol(&tokens[start - 2], ")") {
        return Position::Other;
    }
    // A callable type's result (`(text: string) => Checked<int>` is written
    // with `=>` before the result and never reaches here); a parameter list
    // followed by `:` is a function or arrow declaration.
    let Some(open) = opening(tokens, start - 2) else { return Position::Other };
    if open >= 1 && is_name(&tokens[open - 1], "main") && open >= 2 && is_name(&tokens[open - 2], "function") {
        return Position::Main;
    }
    match tokens.get(end + 1) {
        Some(token) if is_symbol(token, "{") || is_symbol(token, "=>") => Position::Result { after: end },
        _ => Position::Other,
    }
}

/// Index of the `(` that the `)` at `close` ends.
fn opening(tokens: &[Token], close: usize) -> Option<usize> {
    let mut depth = 0usize;
    for index in (0..=close).rev() {
        let token = &tokens[index];
        if token.kind != TokenKind::Symbol {
            continue;
        }
        match token.text.as_str() {
            ")" | "]" | "}" => depth += 1,
            "(" | "[" | "{" => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

/// Wrap the body that follows the result annotation ending at `after`.
fn body(tokens: &[Token], after: usize, span: Span, void: bool, edits: &mut Vec<Edit>) -> Result<(), Diagnostic> {
    let catch = format!("}} catch ({FAILURE}) {{ return Err({FAILURE}); }}");
    let first = after + 1;
    if is_symbol(&tokens[first], "=>") && !tokens.get(first + 1).is_some_and(|token| is_symbol(token, "{")) {
        // Expression body: `=> value` becomes a block that returns `Ok(value)`.
        let start = first + 1;
        let end = expression_end(tokens, start);
        edits.push(Edit { at: start, count: 0, tokens: generated(span, "{ try { return Ok(") });
        edits.push(Edit { at: end, count: 0, tokens: generated(span, &format!(");{catch} }}")) });
        return Ok(());
    }
    let open = if is_symbol(&tokens[first], "=>") { first + 1 } else { first };
    let close = closing(tokens, open).ok_or_else(|| Diagnostic::new("checked", "Checked function body has no closing `}`.", span))?;
    edits.push(Edit { at: open + 1, count: 0, tokens: generated(span, "try {") });
    let mut tail = String::new();
    if void {
        tail.push_str("return Ok();");
    }
    tail.push_str(&catch);
    edits.push(Edit { at: close, count: 0, tokens: generated(span, &tail) });
    returns(tokens, open + 1, close, span, edits);
    Ok(())
}

/// Rewrite the function's own `return` statements between `start` and `end`,
/// skipping the bodies of nested functions and arrows (each lowers its own).
fn returns(tokens: &[Token], start: usize, end: usize, span: Span, edits: &mut Vec<Edit>) {
    let arms = match_arms(tokens, start, end);
    let mut index = start;
    while index < end {
        let token = &tokens[index];
        if is_name(token, "function") {
            // `function name(parameters): Result {`: skip to the body and past it.
            let body = (index..end).find(|&at| is_symbol(&tokens[at], "(")).and_then(|open| closing(tokens, open)).and_then(|close| (close..end).find(|&at| is_symbol(&tokens[at], "{")));
            index = body.and_then(|open| closing(tokens, open)).map_or(end, |close| close + 1);
            continue;
        }
        if is_symbol(token, "=>") && !arms.contains(&index) && tokens.get(index + 1).is_some_and(|next| is_symbol(next, "{")) {
            index = closing(tokens, index + 1).map_or(end, |close| close + 1);
            continue;
        }
        if is_name(token, "return") {
            let value = index + 1;
            if tokens.get(value).is_some_and(|token| is_symbol(token, ";")) {
                edits.push(Edit { at: value, count: 0, tokens: generated(span, "Ok()") });
            } else {
                let end_of_value = expression_end(tokens, value);
                edits.push(Edit { at: value, count: 0, tokens: generated(span, "Ok(") });
                edits.push(Edit { at: end_of_value, count: 0, tokens: generated(span, ")") });
            }
        }
        index += 1;
    }
}

/// The `=>` tokens that separate `match` arms from their bodies (not arrows).
fn match_arms(tokens: &[Token], start: usize, end: usize) -> std::collections::HashSet<usize> {
    let mut arms = std::collections::HashSet::new();
    for index in start..end {
        if !is_name(&tokens[index], "match") || !tokens.get(index + 1).is_some_and(|token| is_symbol(token, "(")) {
            continue;
        }
        let Some(subject) = closing(tokens, index + 1) else { continue };
        if !tokens.get(subject + 1).is_some_and(|token| is_symbol(token, "{")) {
            continue;
        }
        let Some(close) = closing(tokens, subject + 1) else { continue };
        let mut depth = 0usize;
        for at in subject + 2..close {
            let token = &tokens[at];
            match token.text.as_str() {
                "(" | "[" | "{" if token.kind == TokenKind::Symbol => depth += 1,
                ")" | "]" | "}" if token.kind == TokenKind::Symbol => depth = depth.saturating_sub(1),
                "=>" if depth == 0 && token.kind == TokenKind::Symbol => {
                    arms.insert(at);
                }
                _ => {}
            }
        }
    }
    arms
}

/// End (exclusive) of the expression starting at `start`: the first `;`, `,`
/// or unmatched closer at depth zero.
fn expression_end(tokens: &[Token], start: usize) -> usize {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(start) {
        if token.kind == TokenKind::End {
            return index;
        }
        if token.kind != TokenKind::Symbol {
            continue;
        }
        match token.text.as_str() {
            "(" | "[" | "{" => depth += 1,
            ")" | "]" | "}" => {
                if depth == 0 {
                    return index;
                }
                depth -= 1;
            }
            ";" | "," if depth == 0 => return index,
            _ => {}
        }
    }
    tokens.len()
}
