//! Deque operations preserve moves and borrowed element storage.
use super::*;
impl Analyzer {
    pub(super) fn scoped_deque_carriers(&mut self, context: &mut Context, body: &mut [Stmt]) -> Result<(), Diagnostic> {
        if !context.bindings.iter().any(|binding| matches!(binding.ty, Type::Record(id) if self.program.records[id].native_element().is_some() && (self.aggregate_callable_facts(binding.ty).scoped || self.native_shape_contains_view(binding.ty)))) { return Ok(()); }
        let mut results = context.bindings.iter().enumerate().map(|(id, binding)| (id, binding.ty)).collect::<HashMap<_, _>>();
        for _ in 0..context.bindings.len() {
            let mut changed = false;
            for (id, binding) in context.bindings.iter().enumerate() {
                if let Some(initializer) = &binding.initializer
                    && let Some(ty) = self.planned_scoped_map_result(initializer, &results)
                    && results.get(&id) != Some(&ty)
                { results.insert(id, ty); changed = true; }
            }
            crate::fixed_storage::visit_statements(body, &mut |statement| {
                if let StmtKind::Let { binding: Some(binding), value, .. } = &statement.kind
                    && let Some(ty) = self.planned_scoped_map_result(value, &results)
                    && results.get(binding) != Some(&ty)
                { results.insert(*binding, ty); changed = true; }
                if let StmtKind::ForOf { iterable, binding: Some(binding), .. } = &statement.kind
                    && let Some(Type::Record(id)) = self.planned_scoped_map_result(iterable, &results)
                    && let Some(element) = self.program.records[id].native_element()
                    && results.get(binding) != Some(&element)
                { results.insert(*binding, element); changed = true; }
            });
            if !changed { break; }
        }
        for (&id, &ty) in &results { context.bindings[id].ty = ty; }
        self.retarget_scoped_map_body(context, body, &results, &HashMap::new())
    }
    pub(super) fn deque_receiver(&self, context: &Context, base: &Expr) -> Option<usize> {
        let Type::Record(id) = self.expression_type_hint(context, base)? else { return None; };
        self.program.records[id].native_element().map(|_| id)
    }
    pub(super) fn deque_method_type(&self, id: usize, name: &str, span: Span) -> Result<Type, Diagnostic> {
        match name {
            "pushFront" | "pushBack" | "clear" => Ok(Type::Void),
            "popFront" | "popBack" | "front" | "back" | "at" => self.unions.intern([self.program.records[id].native_element().unwrap(), Type::Undefined], span),
            _ => Err(Diagnostic::new("deque-member", "Unsupported Deque method.", span)),
        }
    }
    pub(super) fn deque_size(&mut self, context: &mut Context, base: &mut Expr, name: &str, mode: Mode, loans: &Loans, span: Span) -> Result<Type, Diagnostic> {
        self.expression(context, base, Mode::Read, loans, None)?;
        if name != "size" { return Err(Diagnostic::new("deque-member", "Deque has only the size value property; call methods with parentheses.", span)); }
        if mode == Mode::Mutate { return Err(Diagnostic::new("readonly", "Deque size is read-only.", span)); }
        Ok(Type::Int)
    }
    pub(super) fn deque_method_call(&mut self, context: &mut Context, expression: &mut Expr, loans: &Loans, expected: Option<Type>) -> Result<Type, Diagnostic> {
        let ExprKind::Call(callee, arguments) = &mut expression.kind else { unreachable!() };
        let ExprKind::Field(base, name) = &mut callee.kind else { unreachable!() };
        let method = match name.as_str() {
            "pushFront" => DequeMethod::PushFront, "pushBack" => DequeMethod::PushBack,
            "popFront" => DequeMethod::PopFront, "popBack" => DequeMethod::PopBack,
            "front" => DequeMethod::Front, "back" => DequeMethod::Back,
            "at" => DequeMethod::At, "clear" => DequeMethod::Clear,
            _ => return Err(Diagnostic::new("deque-member", "Unsupported Deque method.", callee.span)),
        };
        let push = matches!(method, DequeMethod::PushFront | DequeMethod::PushBack);
        let read = matches!(method, DequeMethod::Front | DequeMethod::Back | DequeMethod::At);
        Self::arity(arguments.len(), usize::from(push || method == DequeMethod::At), expression.span)?;
        let Type::Record(mut id) = self.expression(context, base, Mode::Read, loans, None)? else { unreachable!() };
        let element = self.program.records[id].native_element().unwrap();
        if !read { Self::storage_access(context, base, Mode::Mutate, expression.span, loans)?; }
        let mut active = loans.clone();
        for root in Self::loan_roots(base, if read { Mode::Read } else { Mode::Mutate }) {
            active.entry(root).or_default().push((if read { Mode::Read } else { Mode::Mutate }, base.span));
        }
        if let Some(argument) = arguments.first_mut() {
            let expected = if push {
                if let Type::Record(record) = element && self.aggregate_callable_facts(element).scoped { Type::Record(self.scoped_records.declaration(record)) } else { element }
            } else { Type::Int };
            let actual = self.expression(context, argument, if push { Mode::Consume } else { Mode::Read }, &active, Some(expected))?;
            self.require(actual, Some(expected), argument.span)?;
            if push && (self.aggregate_callable_facts(actual).scoped || self.native_shape_contains_view(actual))
                && let (Type::Record(previous), Type::Record(incoming)) = (element, actual)
                && let Some((binding, path)) = Self::scoped_array_storage_path(base)
            {
                let value = if previous == self.scoped_records.declaration(previous) || previous == incoming { actual }
                    else { self.join_record_types(previous, incoming, argument.span)?.ok_or_else(|| Diagnostic::new("type", "Deque callback values require compatible environment contracts.", argument.span))? };
                id = self.scoped_records.materialize_native_element(id, value, &mut self.program.records);
                while self.fields.len() < self.program.records.len() { self.fields.push(HashMap::new()); }
                if let Some(root) = self.scoped_array_replace_field(context.bindings[binding].ty, &path, Type::Record(id)) { context.bindings[binding].ty = root; }
                base.ty = Some(Type::Record(id));
                self.append_target_capture_loans(context, base, argument);
            }
        }
        if push {
            Self::append_native_view_element(context, base, &arguments[0])?;
            self.record_native_element_store(context, base, &arguments[0])?;
        }
        let mut result = self.deque_method_type(id, name, expression.span)?;
        if read && !result.is_copy() {
            if !Self::stable_at_receiver(base) { return Err(Diagnostic::new("borrow", "Borrowed Deque reads require a stored receiver.", base.span)); }
            result = Type::BorrowedUnion(result.union().unwrap());
        }
        self.require(result, expected, expression.span)?;
        expression.target = Some(CallTarget::DequeMethod(method));
        Ok(result)
    }
}
