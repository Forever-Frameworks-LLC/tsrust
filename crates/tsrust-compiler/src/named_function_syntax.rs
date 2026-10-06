//! Shared header and body parser for named function declarations.
use super::*;

impl Parser<'_> {
    #[inline(never)]
    pub(super) fn local_function_statement(&mut self) -> Result<Stmt, Diagnostic> {
        let keyword = self.expect("function")?;
        if self.unresolved_local_function.is_none() {
            self.unresolved_local_function = Some(self.peek().clone());
        }
        let function = self.named_function()?;
        let span = Span::new(keyword.span.start, function.scope_span.end);
        Ok(Stmt {
            labels: Vec::new(),
            control_target: None,
            span,
            kind: StmtKind::LocalFunction(Box::new(function)),
        })
    }

    /// The function keyword has already been consumed.
    #[inline(never)]
    pub(super) fn named_function(&mut self) -> Result<Function, Diagnostic> {
        let mut function = Box::new(self.named_function_header()?);
        function.body.append(&mut self.block()?);
        function.scope_span.end = self.tokens[self.i - 1].span.end;
        Ok(*function)
    }

    #[inline(never)]
    fn named_function_header(&mut self) -> Result<Function, Diagnostic> {
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
        let mut body = Vec::new();
        Self::open_with_patterns(patterns, &mut body);
        let scope_span = name.span;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_declarations_retain_name_header_body_and_scope() {
        let source = "function main(){function twice(value:int):int{return value+value;} console.log(twice(3));}";
        let program =
            super::super::parse_raw(source, false, Default::default(), Default::default()).unwrap();
        assert_eq!(program.functions.len(), 1);
        let StmtKind::LocalFunction(function) = &program.functions[0].body[0].kind else {
            panic!("lexical declaration AST");
        };
        assert_eq!(function.name.text, "twice");
        assert_eq!(function.params[0].0.text, "value");
        assert!(function.result.is_some());
        assert_eq!(function.body.len(), 1);
        assert_eq!(
            &source[function.scope_span.start..function.scope_span.end],
            "twice(value:int):int{return value+value;}"
        );
        assert_eq!(program.functions[0].body.len(), 2);
    }
}

#[cfg(test)]
mod depth_tests {
    #[test]
    fn nested_named_declaration_depth_preserves_normal_stack_and_source_limit() {
        let source = format!(
            "function main(){{{}{}{}}}",
            "function next():int{".repeat(32),
            "return 1;",
            "}".repeat(32)
        );
        assert!(
            super::super::parse_raw(&source, false, Default::default(), Default::default()).is_ok()
        );
        let source = format!(
            "function main(){{{}{}{}}}",
            "function next():int{".repeat(200),
            "return 1;",
            "}".repeat(200)
        );
        let error = super::super::parse_raw(&source, false, Default::default(), Default::default())
            .unwrap_err();
        assert_eq!(error.code, "unsupported");
        assert!(error.message.contains("128"));
    }
}
