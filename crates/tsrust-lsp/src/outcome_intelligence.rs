//! Schema safeParse outcomes: `SafeParseSuccess<T> | SafeParseError<E>`.
//! The editor knows both alternatives and narrows a local through
//! `if (name.success)` and `if (!name.success)`, as the checker does.
use super::*;

/// Fields of one outcome alternative written by its public name.
pub(super) fn outcome_fields(alternative: &str) -> Option<std::collections::BTreeMap<String, String>> {
    let (success, field, inner) = if let Some(inner) = alternative.strip_prefix("SafeParseSuccess<") {
        ("true", "data", inner)
    } else if let Some(inner) = alternative.strip_prefix("SafeParseError<") {
        ("false", "error", inner)
    } else {
        return None;
    };
    let inner = inner.strip_suffix('>')?;
    Some([("success".to_owned(), success.to_owned()), (field.to_owned(), inner.to_owned())].into_iter().collect())
}

impl SourceIndex {
    /// The alternative selected by an enclosing success check on `name`.
    pub(super) fn narrow_outcome(&self, ty: String, n: usize) -> String {
        if !ty.contains("SafeParseSuccess<") || !ty.contains("SafeParseError<") { return ty; }
        let name = self.text(n);
        let alternatives = self.record_alternatives(&ty, false);
        let pick = |success: bool| alternatives.iter().find(|alternative| alternative.starts_with(if success { "SafeParseSuccess<" } else { "SafeParseError<" })).cloned();
        // Condition `(name.success)` or `(!name.success)` ending at `close`.
        let condition = |close: usize| -> Option<bool> {
            let open = self.pairs.get(close).copied().flatten()?;
            if open == 0 || self.text(open - 1) != "if" { return None; }
            let inside = (open + 1..close).map(|index| self.text(index)).collect::<Vec<_>>();
            match inside.as_slice() {
                [receiver, ".", "success"] if *receiver == name => Some(true),
                ["!", receiver, ".", "success"] if *receiver == name => Some(false),
                _ => None,
            }
        };
        for block in (0..n).rev().filter(|&index| self.text(index) == "{") {
            let Some(close) = (block + 1..self.tokens.len()).find(|&index| self.pairs.get(index).copied().flatten() == Some(block)) else { continue };
            if close < n { continue; }
            if block == 0 { break; }
            let selected = if self.text(block - 1) == ")" {
                condition(block - 1)
            } else if self.text(block - 1) == "else" && block >= 2 && self.text(block - 2) == "}" {
                self.pairs[block - 2].filter(|&open| open > 0 && self.text(open - 1) == ")").and_then(|open| condition(open - 1)).map(|success| !success)
            } else { None };
            if let Some(success) = selected { return pick(success).unwrap_or(ty); }
        }
        ty
    }
}
