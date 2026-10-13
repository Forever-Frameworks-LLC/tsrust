//! Contextual completion for the final schema options object.
use super::*;

impl SourceIndex {
    pub(super) fn schema_options_completion(&self, offset: usize) -> Option<Value> {
        let object = (0..self.tokens.len()).rev().find(|&id| self.text(id) == "{"
            && self.tokens[id].span.end <= offset
            && self.pairs[id].is_none_or(|end| offset <= self.tokens[end].span.start))?;
        let call = (2..object).rev().find(|&id| self.text(id) == "("
            && self.text(id - 2) == "." && self.pairs[id].is_none_or(|end| end > object))?;
        let receiver = self.receiver_type(call - 2);
        let method = self.text(call - 1);
        let (argument, fields) = if receiver == "@namespace:std:schema#s" {
            let entry = builtins_for("schemaBuilder").into_iter().find(|entry| entry["name"] == method)?;
            let parameters = entry["parameters"].as_array()?;
            if !parameters.last()?["label"].as_str()?.starts_with("options?") { return None; }
            (parameters.len() - 1, vec![("message", "string", "Fallback validation message. Child custom messages take precedence.")])
        } else if matches!(method, "min" | "max" | "gt" | "lt")
            && (self.schema_element(&receiver).is_some() || self.schema_pipeline_types.contains_key(&receiver)) {
            (1, vec![("message", "string", "Validation message for this constraint only. Preserves the issue code and path; does not replace type errors or another constraint's message.")])
        } else if method == "refine" && (self.schema_element(&receiver).is_some() || self.schema_pipeline_types.contains_key(&receiver)) {
            (1, vec![("path", "checked field/index path", "Relative diagnostic path. Field names and array indices are checked against the refined value."),
                ("message", "string", "Message reported when the predicate returns false. Exceptions still propagate.")])
        } else { return None; };
        let mut cursor = call + 1;
        let mut index = 0;
        while cursor < object {
            if self.text(cursor) == "," { index += 1; cursor += 1; }
            else { cursor = self.pairs[cursor].filter(|end| *end > cursor).map_or(cursor + 1, |end| end + 1); }
        }
        if cursor != object || index != argument { return None; }
        let current = self.token_at(offset).filter(|&id| name(self.text(id)));
        let start = current.map_or(offset, |id| self.tokens[id].span.start);
        let before = self.tokens.iter().rposition(|token| token.span.end <= start)?;
        if !matches!(self.text(before), "{" | ",") { return None; }
        let prefix = self.source.get(start..offset)?;
        let written = (object + 1..self.tokens.len().saturating_sub(1))
            .take_while(|&id| self.tokens[id].span.start < offset)
            .filter(|&id| self.text(id + 1) == ":").map(|id| self.text(id)).collect::<Vec<_>>();
        Some(json!(fields.into_iter().filter(|(field, _, _)| field.starts_with(prefix) && !written.contains(field))
            .map(|(field, ty, docs)| json!({"label":field,"kind":5,"detail":format!("{field}?: {ty}"),
                "documentation":{"kind":"markdown","value":docs},
                "textEdit":{"range":self.range(Span::new(start,offset)),"newText":field}})).collect::<Vec<_>>()))
    }
}
