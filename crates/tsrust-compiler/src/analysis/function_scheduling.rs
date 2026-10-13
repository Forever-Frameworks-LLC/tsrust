//! Function body scheduling and lexical capture dependencies.
use super::*;

impl Analyzer {
    /// Check completed owners' uncalled nested bodies in the same depth-first
    /// order as direct invocation, without retaining a Rust frame per owner.
    /// Direct call dependencies inside a body still use the existing checker.
    pub(super) fn function(&mut self, index: usize) -> Result<(), Diagnostic> {
        let mut pending = self.check_function_body(index)?;
        pending.reverse();
        while let Some(index) = pending.pop() {
            let uncalled = self.check_function_body(index)?;
            pending.extend(uncalled.into_iter().rev());
        }
        Ok(())
    }

    /// Resolve lexical owners before installing an unused-body placeholder.
    /// The continuation list preserves the original owner-first checks; owner
    /// completion can prepare or even check a waiting descendant.
    pub(super) fn capturing_function_is_unused(&mut self, index: usize) -> Result<bool, Diagnostic> {
        let mut pending = Vec::new();
        let mut current = index;
        let mut unused = loop {
            if !self.named_captures.needs_context(current) || self.named_captures.ready(current) {
                break false;
            }
            if let Some(owner) = self.named_captures.declaring_function(current, &self.program, &self.function_names)
                && owner != current
                && self.checked[owner].is_none()
                && !self.visiting[owner]
            {
                pending.push((current, owner));
                current = owner;
            } else {
                break self.defer_unused_capturing_function(current);
            }
        };
        while let Some((current, owner)) = pending.pop() {
            if !unused {
                self.function(owner)?;
            }
            unused = self.defer_unused_capturing_function(current);
        }
        Ok(unused)
    }

    fn defer_unused_capturing_function(&mut self, index: usize) -> bool {
        if self.named_captures.ready(index) || self.checked[index].is_some() {
            return false;
        }
        self.checked[index] = Some(CheckedFunction {
            storage_exits: Default::default(), native_returns: None, native_mutations: None,
            template: true,
            specialization: false,
            name: self.program.functions[index].name.text.clone(),
            bindings: Vec::new(),
            parameter_count: 0,
            result: self.signatures[index].result,
            body: Vec::new(),
        });
        true
    }

    fn check_function_body(&mut self, index: usize) -> Result<Vec<usize>, Diagnostic> {
        if self.discovering_results {
            return Ok(Vec::new());
        }
        if self.checked[index].is_some() {
            return Ok(Vec::new());
        }
        if self.named_captures.needs_context(index) && !self.named_captures.ready(index) {
            return Err(self.program.local_capture_plans.iter().find_map(crate::local_capture_plan::Plan::pending_diagnostic).expect("pending named capture"));
        }
        let name = self.program.functions[index].name.clone();
        if self.visiting[index] {
            if self.signatures[index].inferred_result {
                // A return that does not depend on the recursive call fixes the
                // type; the finished body is then checked against it as usual.
                let Some(&provisional) = self.provisional_results.get(&index) else {
                    return Err(Diagnostic::new("result-inference", "The result type of this recursive function cannot be inferred: every return depends on the recursive call or on a local variable. Add a result annotation.", name.span));
                };
                self.signatures[index].result = provisional;
            }
            let start = self
                .function_stack
                .iter()
                .position(|&id| id == index)
                .expect("active function");
            for &id in &self.function_stack[start..] {
                if self.signatures[id]
                    .params
                    .iter()
                    .any(|ty| matches!(ty, Type::Callback(_)))
                {
                    self.recursive_callbacks.insert(id);
                }
            }
            return if self.assumed_modes.is_some() {
                Ok(Vec::new())
            } else {
                Err(Diagnostic::new(
                    "recursion-inference",
                    "Recursive access inference is required.",
                    name.span,
                ))
            };
        }
        if self.depth >= 128 {
            return Err(Diagnostic::new(
                "unsupported",
                "Call graph depth exceeds the prototype limit of 128.",
                name.span,
            ));
        }
        if self.callable_result_type(self.signatures[index].result)
            && self.assumed_modes.is_some()
            && !self.known_results.contains_key(&index)
        {
            self.discover_results()?;
        }
        self.visiting[index] = true;
        self.function_stack.push(index);
        self.depth += 1;
        let mut body = std::mem::take(&mut self.program.functions[index].body);
        let mut context = self.function_context(index, &mut body)?;
        for (position, access) in self.program.functions[index].parameter_access.iter().enumerate() {
            if matches!(access, Some((Mode::Read, _))) && !context.bindings[position].ty.is_copy() {
                context.readonly_parameters.insert(position);
            }
        }
        if self.signatures[index].inferred_result && !self.provisional_results.contains_key(&index)
            && let Some(provisional) = self.provisional_result(&context, &body)
        {
            // Type hints inside the body read the signature before the
            // recursive call is checked, so publish the result now.
            self.provisional_results.insert(index, provisional);
            self.signatures[index].result = provisional;
        }
        // Move the body into analysis rather than clone the syntax tree.
        let mut result_type = self.signatures[index].result;
        context.inferred_body_result = self.signatures[index].inferred_result;
        let returned = self
            .check_source_callable_body(&mut context, Self::uninitialized_body(&mut body), result_type)
            .map_err(|error| self.filter_ownership_fixes(&context, error))?;
        if context.inferred_body_result {
            result_type = self.inferred_body_result(&body)?;
            self.signatures[index].result = result_type;
        }
        if result_type != Type::Void && !returned {
            return Err(Diagnostic::new(
                "type",
                format!(
                    "Function {:?} must return {}.",
                    self.program.functions[index]
                        .application
                        .as_ref()
                        .map(|application| application.declaration.text.as_str())
                        .unwrap_or(&name.text),
                    self.type_name(result_type)
                ),
                name.span,
            ));
        }
        self.finalize_nullable_locals(&mut context.bindings, &mut body)?;
        // A record annotation names its declaration; the checked result must
        // retain the concrete callback environment selected by its returns.
        if let Type::Array(ArrayElement::Record(base, depth)) = result_type
            && let Some(joined) = self.joined_array_result(&mut body, base, depth)?
        {
            result_type = joined;
        }
        if let Type::Record(base) = result_type
            && let Some(joined) = self.joined_record_result(&mut body, base)?
        {
            result_type = joined;
        } else if let Type::Record(base) = result_type {
            let actual = self.inferred_body_result(&body)?;
            if let Type::Record(instance) = actual {
                if self.scoped_records.declaration(instance) == base {
                    result_type = actual;
                }
            }
        }
        let result_type = if self.callable_result_type(result_type) {
            context.returned_callable.expect("checked callable return")
        } else {
            result_type
        };
        let result_type = self.nullable_record_result(&mut body, result_type)?;
        self.finalize_nullable_returns(&mut body, result_type)?;
        self.declared_parameter_access(index, &mut context)?;
        self.validate_native_returns(&context)?;
        if self.assumed_modes.is_some() && self.callable_result_type(self.signatures[index].result)
        {
            self.known_results.insert(index, result_type);
        }
        let uncalled = self.prepare_uncalled_capturing_functions(index, &context)?;
        self.checked[index] = Some(CheckedFunction {
                storage_exits: context.storage_exits.clone(), native_returns: context.native_returns.clone(), native_mutations: context.native_mutations.clone(),
            template: context.template,
            specialization: index >= self.original_count,
            name: name.text,
            bindings: context.bindings,
            parameter_count: self.signatures[index].params.len(),
            result: result_type,
            body,
        });
        self.visiting[index] = false;
        self.function_stack.pop();
        self.depth -= 1;
        Ok(uncalled)
    }

}
