//! Member completion while the document does not check.
//!
//! A member access that is still being typed (`value.` or `value.pre`) makes
//! the document invalid, so the checker has no facts for it. The editor then
//! knows the cursor: it removes that one unfinished access, closes what the
//! access left open, and asks the checker about the repaired text. Types and
//! members come from that check, never from a guess about the broken text.
use super::*;

/// A document with the unfinished member access at the cursor removed.
pub(crate) struct Repair {
    pub(crate) text: String,
    /// First byte that differs from the original text.
    pub(crate) start: usize,
    /// End of the receiver expression in `text`.
    pub(crate) receiver_end: usize,
}

fn closer(open: &str) -> &'static str {
    match open {
        "(" => ")",
        "[" => "]",
        _ => "}",
    }
}

impl SourceIndex {
    /// The checker accepted this exact text; its facts describe it.
    pub(crate) fn checked(&self) -> bool {
        self.checked
    }

    pub(crate) fn mark_checked(&mut self) {
        self.checked = true;
    }

    pub(crate) fn source_range(&self, start: usize, end: usize) -> Range {
        self.range(Span::new(start, end))
    }

    pub(crate) fn position_offset(&self, params: &Value) -> Option<usize> {
        let position: Position = serde_json::from_value(params.get("position").cloned()?).ok()?;
        self.offset(position)
    }

    /// The separator before the cursor and the start of the typed member
    /// prefix, when the cursor is in a member position.
    pub(crate) fn member_position(&self, offset: usize) -> Option<(usize, usize)> {
        let prefix_start = self
            .tokens
            .iter()
            .rposition(|token| token.span.end == offset && name(&token.text) && !token.literal)
            .map_or(offset, |id| self.tokens[id].span.start);
        let before = self.tokens.iter().rposition(|token| token.span.end <= prefix_start)?;
        (before > 0 && member_separator(self.text(before)) && self.source[self.tokens[before].span.end..prefix_start].trim().is_empty()).then_some((before, prefix_start))
    }

    /// First token of the postfix expression that ends at token `end`.
    fn operand_start(&self, end: usize) -> Option<usize> {
        let mut at = end;
        loop {
            let text = self.text(at);
            if text == "!" {
                at = at.checked_sub(1)?;
                continue;
            }
            if matches!(text, ")" | "]") {
                let open = self.pairs[at]?;
                // A call or an index continues the operand before it.
                if open > 0 && (matches!(self.text(open - 1), ")" | "]") || (name(self.text(open - 1)) && !matches!(self.text(open - 1), "if" | "while" | "for" | "switch" | "match" | "catch" | "return" | "typeof" | "in" | "of"))) {
                    at = open - 1;
                    continue;
                }
                return Some(open);
            }
            let token = &self.tokens[at];
            let operand = name(text) || token.literal || self.templates.iter().any(|(_, close)| *close == at);
            if !operand {
                return None;
            }
            if let Some((open, _)) = self.templates.iter().find(|(_, close)| *close == at) {
                return Some(*open);
            }
            if at >= 2 && member_separator(self.text(at - 1)) {
                at -= 2;
                continue;
            }
            return Some(at);
        }
    }

    /// Closers for the brackets that the unfinished code left open. Closers
    /// after the cursor belong to the outermost open brackets; the inner ones
    /// are still open and close here.
    fn missing_closers(&self, before: usize, offset: usize) -> String {
        let mut open: Vec<&str> = Vec::new();
        for token in &self.tokens[..before] {
            match token.text.as_str() {
                "(" | "[" | "{" | "${" => open.push(token.text.as_str()),
                ")" | "]" | "}" => {
                    open.pop();
                }
                _ => {}
            }
        }
        let mut nested = 0usize;
        let mut closing = 0usize;
        for token in self.tokens.iter().filter(|token| token.span.start >= offset) {
            match token.text.as_str() {
                "(" | "[" | "{" | "${" => nested += 1,
                ")" | "]" | "}" if nested > 0 => nested -= 1,
                ")" | "]" | "}" => closing += 1,
                _ => {}
            }
        }
        open.iter().skip(closing).rev().map(|open| closer(open)).collect()
    }

    /// Keep complete declarations around the cursor when a later declaration is broken.
    /// Every candidate still needs a full compiler check, including its dependencies.
    pub(crate) fn declaration_prefix_repairs(&self, offset: usize) -> Vec<Repair> {
        let Some((separator, _)) = self.member_position(offset) else { return Vec::new(); };
        let receiver_end = self.tokens[separator - 1].span.end;
        self.tokens.iter().filter(|token| token.scope == 0 && token.span.start > offset
            && !token.literal && matches!(token.text.as_str(), "function" | "export" | "type" | "interface" | "enum" | "const" | "let"))
            .flat_map(|token| {
                let text = self.source[..token.span.start].to_owned();
                let mut repairs = SourceIndex::new(&text).member_repairs(offset);
                repairs.push(Repair { text, start: token.span.start, receiver_end });
                repairs
            }).collect()
    }

    /// Repaired documents to try, most likely first.
    pub(crate) fn member_repairs(&self, offset: usize) -> Vec<Repair> {
        let Some((separator, _)) = self.member_position(offset) else {
            return Vec::new();
        };
        let end = separator - 1;
        let Some(begin) = self.operand_start(end) else {
            return Vec::new();
        };
        let start = self.tokens[begin].span.start;
        let receiver = &self.source[start..self.tokens[end].span.end];
        let closers = self.missing_closers(begin, offset);
        let rest = &self.source[offset..];
        // A receiver that is a whole statement is read through `typeof`: the
        // checker then reports its narrowed view and accepts it unused.
        let statement = begin == 0 || matches!(self.text(begin - 1), "{" | ";" | "}");
        let (head, receiver_end) = if statement {
            (format!("{}typeof {receiver};", &self.source[..start]), start + "typeof ".len() + receiver.len())
        } else {
            (format!("{}{receiver}", &self.source[..start]), start + receiver.len())
        };
        // The statement probe already ends in a semicolon; avoid an invalid empty statement.
        let rest=if statement && rest.trim_start().starts_with(';'){&rest[rest.find(';').unwrap()+1..]}else{rest};
        let terminated = rest.trim_start().starts_with([';', ')', ',', ']']);
        let mut tails = Vec::new();
        if statement {
            tails.push(if closers.is_empty() { String::new() } else { format!("{closers};") });
            tails.push(closers.clone());
        } else if terminated && closers.is_empty() {
            tails.push(String::new());
        } else {
            tails.push(format!("{closers};"));
            tails.push(closers.clone());
        }
        tails.dedup();
        tails.into_iter().map(|tail| Repair { text: format!("{head}{tail}{rest}"), start, receiver_end }).collect()
    }

    /// Members of the expression that ends at byte `receiver_end` of this
    /// checked text, filtered by `prefix`. `replace` is a range of the
    /// document the user is editing.
    pub(crate) fn members_ending_at(&self, receiver_end: usize, prefix: &str, replace: Range, receiver: &str, optional: bool) -> Option<Value> {
        let end = self.tokens.iter().position(|token| token.span.end == receiver_end)?;
        // The separator is gone from the repaired text; `end + 1` stands for it.
        let ty = if matches!(self.text(end), "Math" | "Number" | "console" | "Array" | "Object" | "String" | "Map" | "JSON")
            && (end == 0 || !member_separator(self.text(end - 1)))
            && self.resolve_token(end).is_none()
        {
            self.text(end).to_owned()
        } else {
            self.expression_type(end)
        };
        // `?.` reads the member of the present value only.
        let ty = if optional { self.record_alternatives(&ty, true).join(" | ") } else { ty };
        Some(json!(self.member_items(&ty, prefix, replace, receiver)))
    }

    /// Answer a member completion from a repaired single document.
    pub(crate) fn repaired_member_completion(&self, offset: usize) -> Option<Value> {
        let (separator, prefix_start) = self.member_position(offset)?;
        let prefix = &self.source[prefix_start..offset];
        let replace = self.range(Span::new(prefix_start, offset));
        let receiver = self.receiver_text(separator);
        self.declaration_prefix_repairs(offset).into_iter().chain(self.member_repairs(offset)).find_map(|repair| {
            let repaired = SourceIndex::new(&repair.text);
            repaired.checked().then(|| repaired.members_ending_at(repair.receiver_end, prefix, replace, &receiver, self.text(separator) == "?.")).flatten()
        })
    }

    /// Source text of the receiver before `separator`, for edits that
    /// replace the whole access.
    pub(crate) fn optional_separator(&self, separator: usize) -> bool {
        self.text(separator) == "?."
    }

    pub(crate) fn receiver_text(&self, separator: usize) -> String {
        let end = separator - 1;
        self.operand_start(end).map_or_else(String::new, |begin| self.source[self.tokens[begin].span.start..self.tokens[end].span.end].to_owned())
    }
}
