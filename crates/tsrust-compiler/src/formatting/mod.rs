//! Whitespace-only formatting; token spelling and comment trivia remain intact.
use crate::syntax::TokenKind;
use crate::{Diagnostic, Span};
mod types;
mod match_layout;

pub fn format(source: &str) -> Result<String, Diagnostic> {
    let tokens = types::normalized_tokens(source)?;
    let real = &tokens[..tokens.len().saturating_sub(1)];
    // Prefix chains do not end an operand. Postfix chains inherit the operand
    // before them. Classify once so arbitrarily long chains stay linear-time.
    let mut operand_ends = Vec::with_capacity(real.len());
    for token in real {
        let ends = match token.kind {
            TokenKind::Symbol => match token.text.as_str() {
                ")" | "]" | "}" => true,
                "!" | "++" | "--" => operand_ends.last().copied().unwrap_or(false),
                _ => false,
            },
            TokenKind::Name => !matches!(token.text.as_str(), "return" | "throw" | "typeof" | "void" | "case" | "of" | "in" | "await"),
            _ => true,
        };
        operand_ends.push(ends);
    }
    let ends_operand = |index: usize| operand_ends[index];
    let (match_headers, match_commas) = match_layout::layout(real);
    let generic_spans = types::generic_spans(&tokens);
    let generic = |at: usize| types::contains(&generic_spans, at);
    if real.is_empty() {
        return Ok(source.to_owned());
    }
    // Tokens of a destructuring pattern in a const or let declaration stay on
    // one line. A declaration can hold several declarators separated by commas.
    let mut pattern = vec![false; real.len()];
    let opens = |index: usize| real.get(index).is_some_and(|token| token.kind == TokenKind::Symbol && matches!(token.text.as_str(), "[" | "{"));
    // The matching close of the bracket at `open`.
    let closing = |open: usize| {
        let mut nesting = 0usize;
        for (offset, inner) in real[open..].iter().enumerate() {
            if inner.kind == TokenKind::Symbol {
                match inner.text.as_str() {
                    "[" | "{" | "(" => nesting += 1,
                    "]" | "}" | ")" => nesting = nesting.saturating_sub(1),
                    _ => {}
                }
            }
            if nesting == 0 {
                return open + offset;
            }
        }
        real.len() - 1
    };
    for (i, token) in real.iter().enumerate() {
        if token.kind != TokenKind::Name || !matches!(token.text.as_str(), "const" | "let") {
            continue;
        }
        let mut declarator = i + 1;
        loop {
            if opens(declarator) {
                let close = closing(declarator);
                // After a comma, `[` or `{` is a pattern only when `=` follows it.
                if declarator == i + 1 || real.get(close + 1).is_some_and(|next| next.text == "=") {
                    for inside in &mut pattern[declarator..=close] {
                        *inside = true;
                    }
                }
                declarator = close + 1;
            }
            // Skip the annotation and initializer to the next declarator.
            let mut nesting = 0usize;
            let mut next = None;
            for (offset, inner) in real[declarator.min(real.len())..].iter().enumerate() {
                if inner.kind == TokenKind::Name && inner.text == "of" && nesting == 0 {
                    break;
                }
                if inner.kind != TokenKind::Symbol {
                    continue;
                }
                match inner.text.as_str() {
                    "[" | "{" | "(" => nesting += 1,
                    "]" | "}" | ")" if nesting == 0 => break,
                    "]" | "}" | ")" => nesting -= 1,
                    ";" if nesting == 0 => break,
                    "," if nesting == 0 => {
                        next = Some(declarator + offset + 1);
                        break;
                    }
                    _ => {}
                }
            }
            match next {
                Some(following) => declarator = following,
                None => break,
            }
        }
    }
    // A record pattern in a parameter list also stays on one line: it follows
    // `(` or `,` and is followed by its annotation, or its parameter list is
    // followed by `=>`.
    let mut open_parentheses = Vec::new();
    for i in 0..real.len() {
        if real[i].kind != TokenKind::Symbol {
            continue;
        }
        match real[i].text.as_str() {
            "(" => open_parentheses.push(i),
            ")" => {
                open_parentheses.pop();
            }
            "{" if i > 0 && real[i - 1].kind == TokenKind::Symbol && matches!(real[i - 1].text.as_str(), "(" | ",") && !pattern[i] => {
                let Some(&list) = open_parentheses.last() else { continue };
                let close = closing(i);
                let annotated = real.get(close + 1).is_some_and(|next| next.kind == TokenKind::Symbol && next.text == ":");
                let arrow = real.get(closing(list) + 1).is_some_and(|next| next.text == "=>");
                let separated = real.get(close + 1).is_some_and(|next| matches!(next.text.as_str(), ")" | "," | "="));
                // `({ x, y } = value)` assigns through a record pattern.
                let assigned = real.get(close + 1).is_some_and(|next| next.kind == TokenKind::Symbol && next.text == "=");
                if annotated || assigned || arrow && separated {
                    for inside in &mut pattern[i..=close] {
                        *inside = true;
                    }
                }
            }
            _ => {}
        }
    }
    // Switch bodies: each case or default label ends its line, and the
    // statements of a clause are indented one level under their label.
    let mut switch_open = vec![false; real.len()];
    let mut label_start = vec![false; real.len()];
    let mut label_colon = vec![false; real.len()];
    {
        let mut bodies: Vec<usize> = Vec::new();
        let mut braces: Vec<bool> = Vec::new();
        let mut i = 0;
        while i < real.len() {
            let token = &real[i];
            if token.kind == TokenKind::Symbol && token.text == "{" && !pattern[i] {
                // `switch (...) {`: the brace follows the discriminant's closing parenthesis.
                let opens = i > 0 && real[i - 1].text == ")" && {
                    let mut nesting = 0usize;
                    let mut start = i - 1;
                    loop {
                        match real[start].text.as_str() {
                            ")" => nesting += 1,
                            "(" => nesting -= 1,
                            _ => {}
                        }
                        if nesting == 0 || start == 0 {
                            break;
                        }
                        start -= 1;
                    }
                    start > 0 && real[start - 1].kind == TokenKind::Name && real[start - 1].text == "switch"
                };
                switch_open[i] = opens;
                braces.push(opens);
                if opens {
                    bodies.push(braces.len());
                }
            } else if token.kind == TokenKind::Symbol && token.text == "}" && !pattern[i] {
                if braces.pop() == Some(true) {
                    bodies.pop();
                }
            } else if token.kind == TokenKind::Name
                && matches!(token.text.as_str(), "case" | "default")
                && bodies.last() == Some(&braces.len())
                && i > 0
                && (matches!(real[i - 1].text.as_str(), "{" | ";" | "}") || label_colon[i - 1])
            {
                label_start[i] = true;
                // The label ends at the first colon outside brackets and conditionals.
                let (mut nesting, mut conditionals) = (0usize, 0usize);
                let mut end = i + 1;
                while end < real.len() {
                    match real[end].text.as_str() {
                        "(" | "[" | "{" => nesting += 1,
                        ")" | "]" | "}" => nesting = nesting.saturating_sub(1),
                        "?" if nesting == 0 => conditionals += 1,
                        ":" if nesting == 0 && conditionals > 0 => conditionals -= 1,
                        ":" if nesting == 0 => break,
                        _ => {}
                    }
                    end += 1;
                }
                if end < real.len() {
                    label_colon[end] = true;
                }
            }
            i += 1;
        }
    }
    // The colon of a conditional expression has a space on both sides.
    let mut conditional_colon = vec![false; real.len()];
    {
        let mut nesting = 0usize;
        let mut questions: Vec<usize> = Vec::new();
        for (i, token) in real.iter().enumerate() {
            if token.kind != TokenKind::Symbol {
                continue;
            }
            match token.text.as_str() {
                "(" | "[" | "{" => nesting += 1,
                ")" | "]" | "}" => {
                    nesting = nesting.saturating_sub(1);
                    while questions.last().is_some_and(|level| *level > nesting) {
                        questions.pop();
                    }
                }
                // `name?: type` marks an optional parameter, not a conditional.
                "?" if real.get(i + 1).is_none_or(|next| next.text != ":") => questions.push(nesting),
                ":" if questions.last() == Some(&nesting) => {
                    questions.pop();
                    conditional_colon[i] = true;
                }
                ";" => questions.retain(|level| *level < nesting),
                _ => {}
            }
        }
    }
    // Only commas separating direct interface fields end a line. Commas in
    // inherited type lists, generic arguments and tuple/function types stay tight.
    let mut interface_field_comma = vec![false; real.len()];
    for (i, token) in real.iter().enumerate() {
        if token.kind != TokenKind::Name || token.text != "interface" {
            continue;
        }
        let Some(open) = (i + 1..real.len()).find(|&at| real[at].text == "{") else {
            continue;
        };
        let close = closing(open);
        let mut nesting = 0usize;
        for at in open + 1..close {
            if generic(real[at].span.start) || real[at].kind != TokenKind::Symbol {
                continue;
            }
            match real[at].text.as_str() {
                "(" | "[" | "{" => nesting += 1,
                ")" | "]" | "}" => nesting = nesting.saturating_sub(1),
                "," if nesting == 0 => interface_field_comma[at] = true,
                _ => {}
            }
        }
    }
    let mut switch_depths: Vec<usize> = Vec::new();
    let mut output = String::new();
    let mut depth: usize = 0;
    let mut parentheses: usize = 0;
    let mut import = false;
    let mut offset = 0;
    for (i, token) in real.iter().enumerate() {
        let gap = &source[offset..token.span.start];
        let previous = i.checked_sub(1).map(|i| &real[i]);
        let symbol = token.kind == TokenKind::Symbol;
        if depth == 0 && token.kind == TokenKind::Name && token.text == "import" {
            import = true;
        }
        if symbol && token.text == "}" && !pattern[i] {
            depth = depth.saturating_sub(1);
            if switch_depths.last().is_some_and(|body| depth < *body) {
                switch_depths.pop();
            }
        }
        // Comments are source trivia, not lexer tokens; retain their exact content.
        if !gap.chars().all(char::is_whitespace) {
            output.push_str(gap);
        } else if i == 0 {
            output.push_str(&"\n".repeat(gap.bytes().filter(|&byte| byte == b'\n').count()));
        } else if let Some(previous) = previous {
            let prev = previous.text.as_str();
            let current = token.text.as_str();
            let previous_symbol = previous.kind == TokenKind::Symbol;
            let empty_pair = previous_symbol && symbol && matches!((prev, current), ("{", "}") | ("[", "]") | ("(", ")"));
            let existing_newlines = gap.bytes().filter(|&byte| byte == b'\n').count();
            let exception_handler = previous_symbol && prev == "}" && matches!(current,"catch"|"finally");
            let type_continuation = previous_symbol && prev == "}" && matches!(current, "as" | "satisfies");
            let newline = if empty_pair || pattern[i - 1] {
                false
            } else if label_colon[i - 1] || interface_field_comma[i - 1] || match_commas[i - 1] {
                true
            } else if import && prev != ";" {
                existing_newlines > 0 && (prev == "{" || prev == "," || current == "}")
            } else {
                (previous_symbol && (prev == "{" || (prev == ";" && parentheses == 0)))
                    || (symbol && current == "}" && prev != "{")
                    || (previous_symbol
                        && prev == "}"
                        && !matches!(current, ";" | "," | ")" | "]" | "." | "else" | "catch" | "finally" | "(" | "as" | "satisfies"))
            };
            let separate_blocks =
                !import && previous_symbol && prev == "}" && newline && current != "}";
            let separate_declarations = previous_symbol
                && prev == ";"
                && depth == 0
                && matches!(current, "function" | "type" | "export");
            if newline || (!empty_pair && !exception_handler && !type_continuation && existing_newlines > 1 && (!import || prev == ";")) {
                let required = if separate_blocks || separate_declarations {
                    2
                } else {
                    1
                };
                let count = if import && prev != ";" {
                    required
                } else {
                    existing_newlines.max(required)
                };
                output.push_str(&"\n".repeat(count));
                // Inside a switch body, everything except a label sits one level deeper.
                let clauses = switch_depths.iter().filter(|body| depth >= **body).count();
                output.push_str(&"  ".repeat(depth + clauses.saturating_sub(usize::from(label_start[i]))));
            } else {
                let postfix_assertion = symbol && current == "!" && ends_operand(i - 1);
                // Signs after postfix operators are binary; signs after prefixes
                // still bind to their operand.
                let unary_sign = previous_symbol
                    && matches!(prev, "-" | "+")
                    && i.checked_sub(2).is_none_or(|before| !ends_operand(before));
                // `- -a` and `+ +a` keep their space: joined they would lex as `--` or `++`.
                let sign_operand = unary_sign && !(symbol && (current.starts_with('-') || current.starts_with('+')));
                let generic_open = symbol && current == "<" && generic(token.span.start);
                let generic_close = symbol && matches!(current, ">" | ">>" | ">>>") && generic(token.span.start);
                let after_generic_open = previous_symbol && prev == "<" && generic(previous.span.start);
                let optional_parameter = symbol && current == "?" && real.get(i + 1).is_some_and(|t| t.text == ":");
                let tight = empty_pair || sign_operand || generic_open || generic_close || after_generic_open || optional_parameter || postfix_assertion || (symbol
                    && matches!(current, "." | "?." | "," | ";" | ")" | "]" | ":" | "++" | "--")
                    && !conditional_colon[i])
                    || (previous_symbol
                        && (matches!(prev, "." | "?." | "(" | "[" | "..." | "~") || (matches!(prev, "!" | "++" | "--") && !ends_operand(i - 1))))
                    || (symbol
                        && current == "("
                        && !match_headers[i]
                        // A call or grouped callee; an operator or keyword keeps its space.
                        && (!previous_symbol || matches!(prev, ")" | "]" | ">" | "?.") || (prev == "!" && ends_operand(i - 1)) || unary_sign)
                        && !matches!(prev, "if" | "while" | "for" | "switch" | "catch" | "return" | "typeof" | "of" | "in" | "else" | "const" | "let" | "function" | "as" | "satisfies"))
                    || (symbol && current == "[" && ((previous.kind == TokenKind::Name && !matches!(prev, "const" | "let" | "of" | "return" | "as" | "satisfies")) || (previous_symbol && (matches!(prev, "]" | ")" | ">" | ">>" | ">>>") || (prev == "!" && ends_operand(i - 1))))))
                    || (symbol && current == "}" && prev == "{")
                    || previous.kind == TokenKind::TemplateHead
                    || previous.kind == TokenKind::TemplateMiddle
                    || token.kind == TokenKind::TemplateMiddle
                    || token.kind == TokenKind::TemplateTail;
                if !tight {
                    output.push(' ');
                }
            }
        }
        output.push_str(&source[token.span.start..token.span.end]);
        if symbol && token.text == ";" {
            import = false;
        }
        if symbol {
            match token.text.as_str() {
                "{" if !pattern[i] => {
                    depth += 1;
                    if switch_open[i] {
                        switch_depths.push(depth);
                    }
                }
                "(" => parentheses += 1,
                ")" => parentheses = parentheses.saturating_sub(1),
                _ => {}
            }
        }
        offset = token.span.end;
    }
    let trailing = &source[offset..];
    if !trailing.chars().all(char::is_whitespace) {
        output.push_str(trailing);
    } else {
        output.push_str(&"\n".repeat(trailing.bytes().filter(|&byte| byte == b'\n').count()));
    }
    if !output.ends_with('\n') {
        output.push('\n');
    }
    let formatted = types::normalized_tokens(&output)?;
    if tokens
        .iter()
        .map(|t| (t.kind, &t.text))
        .ne(formatted.iter().map(|t| (t.kind, &t.text)))
    {
        return Err(Diagnostic::new(
            "format",
            "Formatting would change source tokens; this syntax is not yet supported by the formatter.",
            Span::new(0, 0),
        ));
    }
    Ok(output)
}

#[cfg(test)]
mod fixture_tests {
    #[test]
    fn lexable_fixtures_preserve_tokens_and_format_idempotently() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let mut pending = vec![root];
        let mut checked = 0;
        let mut lexical_rejections = 0;
        let mut checked_contracts = 0;
        let mut failures = Vec::new();
        while let Some(path) = pending.pop() {
            if path.is_dir() {
                pending.extend(std::fs::read_dir(path).unwrap().map(|entry| entry.unwrap().path()));
                continue;
            }
            if path.extension().is_none_or(|extension| extension != "tsr") { continue; }
            let source = std::fs::read_to_string(&path).unwrap();
            // Deliberately malformed lexical fixtures remain rejection witnesses.
            if crate::syntax::lex(&source).is_err() { lexical_rejections += 1; continue; }
            match super::format(&source) {
                Ok(formatted) => match super::format(&formatted) {
                    Ok(second) if second == formatted => {
                        checked += 1;
                        if let Ok(original) = crate::check(&source) {
                            match crate::check(&formatted) {
                                Ok(after) if original.contracts() == after.contracts() => checked_contracts += 1,
                                Ok(_) => failures.push(format!("{}: formatting changed callable contracts",path.display())),
                                Err(error) => failures.push(format!("{}: formatted source no longer checks: {}",path.display(),error.message)),
                            }
                        }
                    },
                    Ok(_) => failures.push(format!("{}: second pass changed formatting",path.display())),
                    Err(error) => failures.push(format!("{}: {}",path.display(),error.message)),
                },
                Err(error) => failures.push(format!("{}: {}",path.display(),error.message)),
            }
        }
        assert!(checked > 0);
        assert!(failures.is_empty(),"Formatter corpus failures:\n{}",failures.join("\n"));
        println!("Checked {checked} lexable fixtures and {checked_contracts} standalone contracts; retained {lexical_rejections} lexical rejection witnesses");
    }
}
