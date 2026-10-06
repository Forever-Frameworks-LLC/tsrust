//! Whitespace-only formatting; token spelling and comment trivia remain intact.
use crate::syntax::{TokenKind, lex};
use crate::{Diagnostic, Span};

pub fn format(source: &str) -> Result<String, Diagnostic> {
    let tokens = lex(source)?;
    let real = &tokens[..tokens.len().saturating_sub(1)];
    // Recognize type applications through the shared annotation parser, rather
    // than treating comparison and shift operators as generic punctuation.
    let mut generic_spans = Vec::new();
    for (i, token) in real.iter().enumerate() {
        if generic_spans.iter().any(|span: &Span| span.start <= token.span.start && token.span.start < span.end) {
            continue;
        }
        if token.kind != TokenKind::Name || real.get(i + 1).is_none_or(|t| t.text != "<") {
            continue;
        }
        let previous = i.checked_sub(1).map(|n| real[n].text.as_str());
        let alias_result = previous == Some("=") && i >= 3 && real[i - 3].text == "type";
        let type_position = matches!(previous, Some(":" | "function" | "type" | "interface" | "new" | "extends"))
            || alias_result
            || matches!(token.text.as_str(), "Function" | "Mutable" | "ReadOnly" | "Once" | "Owned");
        if type_position {
            let mut annotation_tokens = tokens.clone();
            if let Ok((_, annotation, end)) = crate::type_syntax::parse_parameter_prefix(&mut annotation_tokens, i, 0) {
                // Data ownership wrappers return the inner type's span. Include
                // the wrapper and its closing delimiter in formatting detection.
                generic_spans.push(Span::new(token.span.start, annotation_tokens[end - 1].span.end.max(annotation.span.end)));
            }
        }
    }
    let generic = |at: usize| generic_spans.iter().any(|span| span.start <= at && at < span.end);
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
            let existing_newlines = gap.bytes().filter(|&byte| byte == b'\n').count();
            let exception_handler = previous_symbol && prev == "}" && matches!(current,"catch"|"finally");
            let newline = if pattern[i - 1] {
                false
            } else if label_colon[i - 1] || interface_field_comma[i - 1] {
                true
            } else if import && prev != ";" {
                existing_newlines > 0 && (prev == "{" || prev == "," || current == "}")
            } else {
                (previous_symbol && (prev == "{" || (prev == ";" && parentheses == 0)))
                    || (symbol && current == "}" && prev != "{")
                    || (previous_symbol
                        && prev == "}"
                        && !matches!(current, ";" | "," | ")" | "]" | "." | "else" | "catch" | "finally" | "("))
            };
            let separate_blocks =
                !import && previous_symbol && prev == "}" && newline && current != "}";
            let separate_declarations = previous_symbol
                && prev == ";"
                && depth == 0
                && matches!(current, "function" | "type" | "export");
            if newline || (!exception_handler && existing_newlines > 1 && (!import || prev == ";")) {
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
                output.push_str(&"    ".repeat(depth + clauses.saturating_sub(usize::from(label_start[i]))));
            } else {
                let postfix_assertion = symbol && current == "!"
                    && (matches!(previous.kind, TokenKind::Number | TokenKind::String)
                        || (previous.kind == TokenKind::Name && !matches!(prev, "return" | "throw" | "typeof" | "void"))
                        || matches!(prev, ")" | "]" | "}" | "!"));
                // `-(a)` after an operator, an opening bracket or a keyword is a sign.
                let unary_sign = previous_symbol
                    && matches!(prev, "-" | "+")
                    && i.checked_sub(2).map(|before| &real[before]).is_none_or(|before| {
                        (before.kind == TokenKind::Symbol && !matches!(before.text.as_str(), ")" | "]" | "}"))
                            || (before.kind == TokenKind::Name && matches!(before.text.as_str(), "return" | "typeof" | "of" | "in"))
                    });
                let generic_open = symbol && current == "<" && generic(token.span.start);
                let generic_close = symbol && matches!(current, ">" | ">>" | ">>>") && generic(token.span.start);
                let after_generic_open = previous_symbol && prev == "<" && generic(previous.span.start);
                let optional_parameter = symbol && current == "?" && real.get(i + 1).is_some_and(|t| t.text == ":");
                let tight = generic_open || generic_close || after_generic_open || optional_parameter || postfix_assertion || (symbol
                    && matches!(current, "." | "," | ";" | ")" | "]" | ":" | "++" | "--")
                    && !conditional_colon[i])
                    || (previous_symbol
                        && matches!(prev, "." | "(" | "[" | "..." | "!" | "~" | "++" | "--"))
                    || (symbol
                        && current == "("
                        // A call or grouped callee; an operator or keyword keeps its space.
                        && (!previous_symbol || matches!(prev, ")" | "]" | ">" | "?.") || unary_sign)
                        && !matches!(prev, "if" | "while" | "for" | "switch" | "catch" | "return" | "typeof" | "of" | "in" | "else" | "const" | "let" | "function"))
                    || (symbol && current == "[" && previous.kind == TokenKind::Name && !matches!(prev, "const" | "let" | "of" | "return"))
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
    let formatted = lex(&output)?;
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
