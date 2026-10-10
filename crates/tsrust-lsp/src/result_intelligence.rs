//! Result-specific editor presentation; type validity remains the compiler's job.
use super::*;
use tsrust_compiler::type_syntax::{ValueAnnotationKind, parse};

impl SourceIndex {
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
