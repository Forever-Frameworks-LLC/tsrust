//! Result-specific editor presentation; type validity remains the compiler's job.
use super::*;
use tsrust_compiler::type_syntax::{ValueAnnotationKind, parse};

impl SourceIndex {
    pub(super) fn is_match_arm_arrow(&self, arrow: usize) -> bool {
        let scope = &self.scopes[self.tokens[arrow].scope];
        let Some(body) = self.tokens.iter().position(|token| token.span.start == scope.span.start && token.text == "{") else { return false; };
        if body == 0 || self.text(body - 1) != ")" { return false; }
        let Some(open) = self.pairs.get(body - 1).copied().flatten() else { return false; };
        open > 0 && self.text(open - 1) == "match"
    }

    pub(super) fn refresh_result_match_bindings(&mut self) {
        for keyword in 0..self.tokens.len() {
            if self.text(keyword) != "match" || self.text(keyword + 1) != "(" { continue; }
            let Some(close) = self.pairs.get(keyword + 1).copied().flatten() else { continue; };
            if self.text(close + 1) != "{" { continue; }
            let start = self.tokens[keyword].span.start;
            let syntax = self.pairs.get(close + 1).copied().flatten().and_then(|end| tsrust_compiler::match_syntax::parse(&self.source[start..self.tokens[end].span.end]).ok());
            let Some(syntax) = syntax else {
                self.refresh_unfinished_match_bindings(close + 1);
                continue;
            };
            for arm in syntax.arms {
                let tsrust_compiler::match_syntax::Pattern::Variant { binding: Some(binding), .. } = arm.pattern else { continue; };
                let span = Span::new(start + binding.start, start + binding.end);
                let Some(token) = self.tokens.iter().position(|token| token.span == span) else { continue; };
                let Some(scope) = self.scopes.iter().position(|scope| scope.span.start == start + arm.body.start) else { continue; };
                // Lowered payload extraction shares the exact binding span with
                // destructuring facts. Never infer a payload from the variant name.
                self.install_match_binding(token, scope, true);
            }
        }
    }

    fn install_match_binding(&mut self, token: usize, scope: usize, checked: bool) {
        let span = self.tokens[token].span;
        let ty = if checked { self.checked_method_result(token).unwrap_or("") } else { "" }.to_owned();
        let signature = format!("(match) {}{}", self.text(token), if ty.is_empty() { String::new() } else { format!(": {ty}") });
        if let Some(id) = self.symbols.iter().position(|symbol| symbol.span == span) {
            self.symbols[id].ty = ty;
            self.symbols[id].signature = signature;
            self.symbols[id].scope = scope;
            self.parameter_symbols.remove(&id);
            self.parameter_available_from.remove(&id);
        } else {
            self.add(token, 13, ty, signature, span, span.start, scope);
        }
    }

    fn refresh_unfinished_match_bindings(&mut self, body: usize) {
        let Some(scope) = self.scopes.iter().position(|scope| scope.span.start == self.tokens[body].span.start) else { return; };
        let end = self.pairs.get(body).copied().flatten().unwrap_or(self.tokens.len());
        for pattern in body + 1..end {
            if self.tokens[pattern].scope != scope || !matches!(self.text(pattern), "Ok" | "Err") { continue; }
            if pattern != body + 1 && !matches!(self.text(pattern - 1), "," | "}") { continue; }
            if self.text(pattern + 1) != "(" || self.text(pattern + 3) != ")" || self.text(pattern + 4) != "=>" || self.text(pattern + 5) != "{" { continue; }
            let binding = pattern + 2;
            if self.text(binding) == "_" || !name(self.text(binding)) || self.tokens[binding].literal { continue; }
            let Some(arm_scope) = self.scopes.iter().position(|scope| scope.span.start == self.tokens[pattern + 5].span.start) else { continue; };
            // A repaired-document check may provide authoritative facts for this
            // unchanged header. Without those facts, retain navigation only.
            self.install_match_binding(binding, arm_scope, true);
        }
    }

    pub(super) fn specialize_result_constructor_help(&self, callee: usize, open: usize, builtin: &mut Value) {
        let name = self.text(callee);
        if !matches!(name, "Ok" | "Err") || self.text(callee + 1) != "<" || self.text(open - 1) != ">" {
            return;
        }
        let source = format!("Result<{}>", self.snippet(callee + 2, open - 1));
        let Ok(annotation) = parse(&source) else { return; };
        let ValueAnnotationKind::Applied { arguments, .. } = annotation.into_kind() else { return; };
        if arguments.len() != 2 { return; }
        let types = arguments.iter().map(|argument| source[argument.span.start..argument.span.end].trim()).collect::<Vec<_>>();
        let payload = types[usize::from(name == "Err")];
        let parameter = if name == "Ok" { "value" } else { "error" };
        let parameters = if ungroup_callable_type(payload) == "void" { vec![] } else { vec![format!("{parameter}: {payload}")] };
        builtin["signature"] = json!(format!("{name}<{}, {}>({}): Result<{}, {}>", types[0], types[1], parameters.join(", "), types[0], types[1]));
        builtin["parameters"] = json!(parameters);
    }

    /// `Result<T, E>` as its two alternatives, when `ty` is one.
    pub(super) fn result_alternatives(&self, ty: &str) -> Option<(String, String)> {
        let ty = self.type_aliases.get(ty).map_or(ty, String::as_str);
        let mut annotation = parse(ty).ok()?;
        loop {
            match annotation.into_kind() {
                ValueAnnotationKind::Group(inner) => annotation = *inner,
                ValueAnnotationKind::Applied { name, arguments } if arguments.len() == 2 => {
                    let wrapper = matches!(name.text.as_str(), "ReadOnly" | "Mutable" | "Owned");
                    if name.text != "Result" || wrapper { return None; }
                    let part = |index: usize| ty[arguments[index].span.start..arguments[index].span.end].trim().to_owned();
                    return Some((part(0), part(1)));
                }
                _ => return None,
            }
        }
    }

    /// The field that both alternatives of a Result hold, as the checker
    /// reads it: `Ok` and `Err` each store one `payload`, and a field that
    /// every alternative has is readable with the union of its types. A void
    /// alternative stores nothing, so the Result then has no common field.
    pub(super) fn result_payload(&self, ty: &str) -> Option<String> {
        let (ok, error) = self.result_alternatives(ty)?;
        if ungroup_callable_type(&ok) == "void" || ungroup_callable_type(&error) == "void" { return None; }
        Some(if ok == error { ok } else { format!("{ok} | {error}") })
    }

    /// Members of a Result receiver: its common payload field, and the
    /// `match` that narrows it to one alternative.
    pub(super) fn result_member_items(&self, ty: &str, prefix: &str, replace: Range, receiver: &str) -> Vec<Value> {
        let Some((ok, error)) = self.result_alternatives(ty) else { return Vec::new(); };
        let mut items = Vec::new();
        if let Some(payload) = self.result_payload(ty)
            && "payload".starts_with(prefix)
        {
            let payload = self.display_type(&payload);
            items.push(json!({"label":"payload","kind":5,"detail":format!("payload: {payload}"),
                "documentation":{"kind":"markdown","value":format!("The value of the active alternative, as `{payload}`. It is `Ok`'s value or `Err`'s error; use `match` to know which.")},
                "textEdit":{"range":replace,"newText":"payload"}}));
        }
        if !receiver.is_empty() && "match".starts_with(prefix) {
            let binding = |ty: &str, name: &str| if ungroup_callable_type(ty) == "void" { "_".to_owned() } else { name.to_owned() };
            let text = format!("match ({receiver}) {{\n  Ok({}) => {{\n    $0\n  }}\n  Err({}) => {{\n  }}\n}}", binding(&ok, "value"), binding(&error, "error"));
            // The edit replaces the whole access, so it starts at the receiver.
            let mut range = replace;
            let width = receiver.encode_utf16().count() as u32 + 1 + prefix.encode_utf16().count() as u32;
            if !receiver.contains('\n') && range.end.character >= width {
                range.start.character = range.end.character - width;
                items.push(json!({"label":"match","kind":15,"detail":format!("match ({receiver}) {{ Ok(…) => … Err(…) => … }}"),
                    "documentation":{"kind":"markdown","value":format!("Handle both alternatives of `{}`. `Ok` holds `{}`; `Err` holds `{}`.", self.display_type(ty), self.display_type(&ok), self.display_type(&error))},
                    "filterText":format!("{receiver}.match"),"insertTextFormat":2,
                    "textEdit":{"range":range,"newText":text}}));
            }
        }
        items
    }
}
