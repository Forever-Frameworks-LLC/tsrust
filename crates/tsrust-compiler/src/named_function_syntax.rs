//! Shared header and body parser for named function declarations.
use super::*;

impl Parser<'_> {
    /// The function keyword has already been consumed.
    #[inline(never)]
    pub(super) fn named_function(&mut self) -> Result<Function, Diagnostic> {
        let name = self.name()?;
        let type_parameters = self.type_parameters()?;
        self.expect("(")?;
        let mut params = Vec::new();
        let mut parameter_initializers = Vec::new();
        let mut parameter_access = Vec::new();
        let mut rest_parameter = false;
        if !self.is(")") {
            loop {
                let rest = self.accept("...");
                rest_parameter = rest;
                let pattern = matches!(self.peek().text.as_str(), "[" | "{");
                let name = if pattern {
                    self.parameter_pattern()?
                } else {
                    self.name()?
                };
                let optional = if self.accept("?") {
                    Some(self.tokens[self.i - 1].span)
                } else {
                    None
                };
                self.expect(":")?;
                let (annotation, access) = if self.access_wrapper_start() {
                    self.wrapped_parameter_annotation()?
                } else {
                    (self.parameter_annotation()?, None)
                };
                let access = access.or_else(|| pattern.then(|| Self::pattern_access(&name)));
                parameter_access.push(access.map(|token| {
                    (
                        match token.text.as_str() {
                            "read" => crate::Mode::Read,
                            "mutate" => crate::Mode::Mutate,
                            _ => crate::Mode::Consume,
                        },
                        token.span,
                    )
                }));
                let (ty, callback) = match annotation {
                    ParameterAnnotation::Named(ty) => (*ty, None),
                    ParameterAnnotation::Callback(callback) => {
                        (callback.value_annotation(), Some(*callback))
                    }
                    ParameterAnnotation::Inferred(_) => {
                        unreachable!("declared parameter has an annotation")
                    }
                };
                let initializer = self.parameter_initializer(optional, true)?;
                Self::rest_initializer(rest, &initializer, name.span)?;
                parameter_initializers.push(initializer);
                params.push((name, ty, callback));
                if rest && self.is(",") {
                    return Err(Diagnostic::new(
                        "rest-parameter",
                        "A rest parameter must be last.",
                        self.peek().span,
                    ));
                }
                if !self.accept(",") {
                    break;
                }
            }
        }
        self.expect(")")?;
        Self::parameter_order(
            &parameter_initializers[..parameter_initializers.len() - usize::from(rest_parameter)],
            params.iter().map(|(name, _, _)| name.span),
        )?;
        let result = if self.accept(":") {
            Some(self.result_annotation()?)
        } else {
            None
        };
        let patterns = std::mem::take(&mut self.parameter_patterns);
        let mut body = self.block()?;
        Self::open_with_patterns(patterns, &mut body);
        let scope_span = Span::new(name.span.start, self.tokens[self.i - 1].span.end);
        Ok(Function {
            application: None,
            scope_span,
            name,
            type_parameters,
            params,
            parameter_initializers,
            parameter_access,
            rest_parameter,
            result,
            body,
        })
    }
}
