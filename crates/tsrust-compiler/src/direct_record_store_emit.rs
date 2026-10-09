//! Expose exact capture installation to native borrow checking at the call site.
use super::*;

impl Emitter<'_> {
    #[inline(never)]
    pub(super) fn direct_record_store_call(
        &self,
        caller: &CheckedFunction,
        callee: usize,
        arguments: &[Expr],
    ) -> Option<String> {
        let function = &self.program.functions[callee];
        if function.template
            || function.result != Type::Void
            || arguments.len() != function.parameter_count
            || self.result_functions.borrow().contains_key(&callee)
            || self
                .program
                .named_capture_access
                .get(&callee)
                .is_some_and(|captures| !captures.is_empty())
            || function.bindings[..function.parameter_count]
                .iter()
                .any(|binding| binding.omission.is_omittable() || binding.rest)
            || function.bindings[..function.parameter_count]
                .iter()
                .zip(arguments)
                .any(|(binding, argument)| {
                    binding.mode == Mode::Consume && argument.ty.is_none_or(|ty| !ty.is_copy())
                })
            || arguments.iter().any(|argument| {
                !matches!(argument.kind, ExprKind::Name(_)) || argument.binding.is_none()
            })
        {
            return None;
        }
        let [
            Stmt {
                kind:
                    StmtKind::Assign {
                        target,
                        value,
                        operator: None,
                    },
                ..
            },
        ] = function.body.as_slice()
        else {
            return None;
        };
        let (root, steps) = self.storage_steps(target);
        let parameter = root.binding?;
        if parameter >= function.parameter_count
            || function.bindings[parameter].mode != Mode::Mutate
        {
            return None;
        }
        let id = arguments[parameter].binding?;
        self.owned_record_layout(caller, id)?;
        let path = steps
            .iter()
            .map(|step| match step {
                StorageStep::Field(field) => Some(*field),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?;
        if path.is_empty() {
            return None;
        }
        let mut source = value;
        while let ExprKind::Inject { value, .. } | ExprKind::TypeAssertion { value, .. } =
            &source.kind
        {
            source = value;
        }
        let (ExprKind::Closure { .. }, Some(Type::Closure(origin))) = (&source.kind, source.ty)
        else {
            return None;
        };
        let closure = &self.program.closures[origin];
        if self.program.polymorphic_storage.contains(origin) {
            return None;
        }
        let mut fields = Vec::new();
        for capture in &closure.captures {
            if capture.restored
                || capture.outer >= function.parameter_count
                || self.record_capture_is_view(capture, closure)
            {
                return None;
            }
            let argument = &arguments[capture.outer];
            let actual = argument.ty?;
            if capture.storage == Mode::Consume && !actual.is_copy() {
                return None;
            }
            if actual.union().is_some() {
                return None;
            }
            fields.push(format!(
                "capture{}:{}",
                capture.inner,
                self.expr(caller, argument, capture.storage)
            ));
        }
        let constructor = format!("Closure{origin}{{{}}}", fields.join(","));
        let mut destination = self.native_captures.ty(caller, id);
        for &index in &path {
            let Type::Record(record) = destination else {
                return None;
            };
            destination = self.program.records[record].fields[index].ty?;
        }
        let converted = if let Type::Union(union) = destination {
            let members = &self.program.unions[union.index()].members;
            let payload = self.callback_payload(destination)?;
            let selected = members.iter().position(|member| *member == payload)?;
            format!(
                "Union{}::V{selected}({})",
                union.index(),
                self.record_carrier_conversion(Type::Closure(origin), payload, &constructor)
            )
        } else {
            self.record_carrier_conversion(Type::Closure(origin), destination, &constructor)
        };
        let place = self.owned_record_leaf_place(caller, id, &path)?;
        Some(format!("{{{place}={converted};()}}"))
    }
}
