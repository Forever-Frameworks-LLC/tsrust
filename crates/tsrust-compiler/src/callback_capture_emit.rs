//! Generic native parameters for independently captured callback inputs.
use super::*;

impl Emitter<'_> {
    pub(super) fn local_callback_environment_type(&self, ty: Type) -> String {
        let rendered = self.native_type(ty);
        let Type::Closure(origin) = ty else {
            return rendered;
        };
        let parameters = self
            .captured_callback_bounds(&self.program.closures[origin])
            .iter()
            .map(|bound| {
                (
                    bound
                        .split(':')
                        .next()
                        .expect("capture generic")
                        .trim()
                        .to_owned(),
                    "_".to_owned(),
                )
            })
            .collect();
        substitute_parameters(&rendered, &parameters)
    }

    pub(super) fn captured_callback_bounds(
        &self,
        closure: &crate::analysis::CheckedClosure,
    ) -> Vec<String> {
        if !matches!(
            closure.forward,
            Some(ClosureForward::RestAdapter { .. } | ClosureForward::Abstract(_))
        ) {
            return closure
                .captures
                .iter()
                .filter_map(|capture| {
                    let Type::Callback(id) = closure.function.bindings[capture.inner].ty else {
                        return None;
                    };
                    Some(self.callback_capture_bounds(id, &format!("K{}", capture.inner)))
                })
                .flatten()
                .collect();
        }
        let Type::Callback(id) = closure.function.bindings[0].ty else {
            return Vec::new();
        };
        self.callback_capture_bounds(id, &format!("C{id}"))
    }

    fn callback_capture_bounds(&self, id: usize, parameter: &str) -> Vec<String> {
        let contract = &self.program.callback_contracts[id];
        let (args, lifetimes) = self.native_argument_tuple(&contract.params);
        let args = args.replace("'p", "'capture_p");
        let lifetimes = lifetimes
            .into_iter()
            .map(|name| name.replace("'p", "'capture_p"))
            .collect::<Vec<_>>();
        let borrow = if lifetimes.is_empty() {
            String::new()
        } else {
            format!("for{} ", lifetime_parameters(&lifetimes))
        };
        let trait_name = if matches!(contract.kind, CallbackKind::Read | CallbackKind::SharedRead) {
            "ReadCallback"
        } else if contract.kind.is_once() {
            "OnceCallback"
        } else {
            "Callback"
        };
        let mut bounds = vec![format!(
            "{parameter}: {borrow}{trait_name}<{args}, {}>",
            self.callback_result_type(id, contract.result)
        )];
        bounds.extend(self.nullable_callback_bounds([Type::Callback(id)]));
        bounds
    }
}

/// Substitute complete generic identifiers without changing similarly named slots.
pub(super) fn substitute_parameters(code: &str, parameters: &HashMap<String, String>) -> String {
    let boundary = |ch: char| !ch.is_ascii_alphanumeric() && ch != '_';
    code.split_inclusive(boundary)
        .map(|part| {
            let name = part.trim_end_matches(boundary);
            parameters.get(name).map_or_else(
                || part.to_owned(),
                |replacement| format!("{replacement}{}", &part[name.len()..]),
            )
        })
        .collect()
}
