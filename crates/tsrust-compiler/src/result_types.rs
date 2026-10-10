//! Result alternatives use the existing tagged-union storage. Each private
//! nominal wrapper has only its active payload; equal T and E remain distinct.
use super::*;

impl Context<'_> {
    #[inline(never)]
    pub(super) fn result_annotation(&mut self, value: &mut ValueAnnotation) -> Result<(), Diagnostic> {
        if self.depth >= 32 {
            return Err(Diagnostic::new("generic-limit", "Result specialization exceeds its nesting limit of 32.", value.span));
        }
        self.depth += 1;
        let result = self.result_annotation_inner(value);
        self.depth -= 1;
        result
    }

    #[inline(never)]
    fn result_annotation_inner(&mut self, value: &mut ValueAnnotation) -> Result<(), Diagnostic> {
        let Kind::Applied { arguments, .. } = &mut value.kind else { unreachable!("Result application"); };
        if arguments.len() != 2 {
            return Err(Diagnostic::new("arity", "Result requires success and error types.", value.span));
        }
        for argument in arguments.iter_mut() { self.annotation(argument, &HashMap::new())?; }
        let types = arguments.iter().map(|argument| {
            let mut resolver = self.callables.resolver(&self.names, self.unions);
            if let Some((ty, _)) = resolver.storage_declaration(argument)? { Ok(ty) }
            else { resolver.resolve_value_annotation(argument, true) }
        }).collect::<Result<Vec<_>, Diagnostic>>()?;
        let mut alternatives = Vec::with_capacity(2);
        for (variant, payload) in ["Ok", "Err"].into_iter().zip(types.iter().copied()) {
            let key = (format!("Result#{variant}"), types.clone());
            let id = if let Some(&id) = self.instances.get(&key) { id } else {
                if self.instances.len() >= 1024 || self.counts.get(&key.0).copied().unwrap_or(0) >= 128 {
                    return Err(Diagnostic::new("generic-limit", "Result specialization exceeds the per-variant (128) or program (1024) limit.", value.span));
                }
                let id = self.records.len();
                // `#` cannot appear in a source name. The identity includes both
                // type arguments, including the inactive alternative's type.
                let name = format!("Result#{variant}#{id}");
                let fields = if payload == Type::Void { Vec::new() } else {
                    let index = usize::from(variant == "Err");
                    vec![RecordField {
                        name: Token { text: "payload".into(), span: value.span, kind: TokenKind::Name },
                        annotation: arguments[index].clone(), ty: None, callable_declaration: None, optional: false,
                    }]
                };
                self.records.push(Record {
                    name: Token { text: name.clone(), span: value.span, kind: TokenKind::Symbol },
                    application: Some(Box::new(RecordApplication {
                        declaration: Token { text: key.0.clone(), span: value.span, kind: TokenKind::Symbol },
                        arguments: arguments.clone(),
                    })),
                    fields, type_parameters: Vec::new(), map_definition: None, set_definition: None, enum_definition: None,
                });
                self.names.insert(name, id);
                self.instances.insert(key.clone(), id);
                *self.counts.entry(key.0).or_default() += 1;
                id
            };
            alternatives.push(ValueAnnotation { kind: Kind::Named(TypeName { text: self.records[id].name.text.clone(), span: value.span }), span: value.span });
        }
        value.kind = Kind::Union(alternatives);
        Ok(())
    }
}
