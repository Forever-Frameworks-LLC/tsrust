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
            let Some(end) = self.pairs.get(close + 1).copied().flatten() else { continue; };
            let start = self.tokens[keyword].span.start;
            let Ok(syntax) = tsrust_compiler::match_syntax::parse(&self.source[start..self.tokens[end].span.end]) else { continue; };
            for arm in syntax.arms {
                let tsrust_compiler::match_syntax::Pattern::Variant { binding: Some(binding), .. } = arm.pattern else { continue; };
                let span = Span::new(start + binding.start, start + binding.end);
                let Some(token) = self.tokens.iter().position(|token| token.span == span) else { continue; };
                let Some(scope) = self.scopes.iter().position(|scope| scope.span.start == start + arm.body.start) else { continue; };
                // Lowered payload extraction shares the exact binding span with
                // destructuring facts. Never infer a payload from the variant name.
                let ty = self.checked_method_result(token).unwrap_or("").to_owned();
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
        }
    }

    pub(super) fn specialize_result_constructor_help(&self, callee: usize, open: usize, builtin: &mut Value) {
        let name = self.text(callee);
        if !matches!(name, "Ok" | "Err") || self.text(callee + 1) != "<" || self.text(open - 1) != ">" {
            return;
        }
        let source = format!("Result<{}>", self.snippet(callee + 2, open - 1));
        let Ok(annotation) = parse(&source) else { return; };
        let ValueAnnotationKind::Applied { arguments, .. } = annotation.kind else { return; };
        if arguments.len() != 2 { return; }
        let types = arguments.iter().map(|argument| source[argument.span.start..argument.span.end].trim()).collect::<Vec<_>>();
        let payload = types[usize::from(name == "Err")];
        let parameter = if name == "Ok" { "value" } else { "error" };
        let parameters = if ungroup_callable_type(payload) == "void" { vec![] } else { vec![format!("{parameter}: {payload}")] };
        builtin["signature"] = json!(format!("{name}<{}, {}>({}): Result<{}, {}>", types[0], types[1], parameters.join(", "), types[0], types[1]));
        builtin["parameters"] = json!(parameters);
    }
}
