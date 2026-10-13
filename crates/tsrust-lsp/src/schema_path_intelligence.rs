//! Refinement path completion uses checked schema output and field identities.
use super::*;

impl SourceIndex {
    pub(super) fn schema_path_completion(&self, offset: usize) -> Option<Value> {
        let array = (2..self.tokens.len()).rev().find(|&id| {
            self.text(id) == "[" && self.text(id - 1) == ":" && self.text(id - 2) == "path"
                && self.tokens[id].span.end <= offset
                && self.pairs[id].is_none_or(|end| offset <= self.tokens[end].span.start)
        })?;
        let call = (2..array).rev().find(|&id| {
            self.text(id) == "(" && matches!(self.text(id - 1), "refine" | "addIssue")
                && self.text(id - 2) == "."
                && self.pairs[id].is_none_or(|end| end > array)
        })?;
        let add_issue = self.text(call - 1) == "addIssue";
        let mut cursor = if add_issue { call } else { call + 1 };
        while !add_issue && cursor < array && self.text(cursor) != "," {
            cursor = self.pairs[cursor].filter(|end| *end > cursor)
                .map_or(cursor + 1, |end| end + 1);
        }
        if cursor + 1 >= array || self.text(cursor + 1) != "{" {
            return None;
        }
        if self.pairs[cursor + 1].is_some_and(|end| end < array) { return None; }
        let receiver = self.receiver_type(call - 2);
        let mut output = if add_issue { self.validation_context_element(&receiver).map(str::to_owned) } else { self.schema_pipeline_types.get(&receiver)
            .map(|(_, output, _, _, _, _)| output.clone())
            .or_else(|| self.schema_element(&receiver).map(str::to_owned)) }?;
        // Only completed segments advance the path. The current segment may be
        // an unfinished string and must not require successful compilation.
        let mut segment = array + 1;
        for comma in array + 1..self.tokens.len() {
            if self.tokens[comma].span.start >= offset { break; }
            if self.text(comma) != "," { continue; }
            if comma != segment + 1 { return Some(json!([])); }
            let written = self.text(segment);
            if written.parse::<usize>().is_ok() {
                let elements = self.schema_path_alternatives(&output).into_iter()
                    .map(|ty| array_element_type(&ty).map(str::to_owned)).collect::<Option<Vec<_>>>()?;
                output = elements.join(" | ");
            } else {
                use tsrust_compiler::type_syntax::{ValueAnnotationKind, LiteralAnnotation};
                let parsed = tsrust_compiler::type_syntax::parse(written).ok()?;
                let ValueAnnotationKind::Literal(LiteralAnnotation::String(field)) = parsed.kind else { return Some(json!([])); };
                output = self.schema_path_fields(&output).remove(&field)?;
            }
            segment = comma + 1;
        }
        let token = self.tokens.get(segment).filter(|token| token.span.start < offset);
        let (prefix, start, end, quoted) = if let Some(token) = token {
            let quote = token.text.starts_with('\'') || token.text.starts_with('"');
            if !quote { return Some(json!([])); }
            let start = token.span.start + 1;
            let end = if token.unterminated_string { token.span.end } else { token.span.end.saturating_sub(1) };
            (self.source.get(start..offset).unwrap_or(""), start, end, true)
        } else { ("", offset, offset, false) };
        let items = self.schema_path_fields(&output).into_iter()
            .filter(|(name, _)| name.starts_with(prefix)).map(|(name, ty)| {
            let text = if quoted { name.clone() } else { serde_json::to_string(&name).unwrap() };
            json!({"label":name,"kind":5,"detail":self.display_type(&ty),
                "textEdit":{"range":self.range(Span::new(start,end)),"newText":text}})
        }).collect::<Vec<_>>();
        Some(json!(items))
    }
    fn schema_path_alternatives(&self, ty: &str) -> Vec<String> {
        self.record_alternatives(ty, true)
    }
    fn record_alternatives(&self, ty: &str, omit_absent: bool) -> Vec<String> {
        use tsrust_compiler::type_syntax::{ValueAnnotation, ValueAnnotationKind};
        fn flatten(value: &ValueAnnotation, source: &str, output: &mut Vec<String>, omit_absent: bool) {
            match &value.kind {
                ValueAnnotationKind::Union(members) => for member in members { flatten(member, source, output, omit_absent); },
                ValueAnnotationKind::Group(inner) => flatten(inner, source, output, omit_absent),
                _ => {
                    let text = source[value.span.start..value.span.end].trim();
                    if !omit_absent || !matches!(text, "null" | "undefined") { output.push(text.to_owned()); }
                }
            }
        }
        let ty = self.type_aliases.get(ty).map_or(ty, String::as_str);
        let mut result = Vec::new();
        if let Ok(value) = tsrust_compiler::type_syntax::parse(ty) { flatten(&value, ty, &mut result, omit_absent); }
        else { result.push(ty.to_owned()); }
        result
    }

    fn schema_path_fields(&self, ty: &str) -> std::collections::BTreeMap<String, String> {
        self.common_record_fields(ty, true)
    }
    pub(super) fn common_record_fields(&self, ty: &str, omit_absent: bool) -> std::collections::BTreeMap<String, String> {
        let mut alternatives = self.record_alternatives(ty, omit_absent).into_iter();
        let fields = |owner: &str| self.symbols.iter()
            .filter(|symbol| symbol.owner.as_deref() == Some(owner))
            .map(|symbol| (symbol.name.clone(), symbol.ty.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();
        let Some(first) = alternatives.next() else { return Default::default(); };
        let mut result = fields(&first);
        for owner in alternatives {
            let next = fields(&owner);
            result.retain(|name, ty| {
                let Some(other) = next.get(name) else { return false; };
                if ty != other { *ty = format!("{ty} | {other}"); }
                true
            });
        }
        result
    }

}
