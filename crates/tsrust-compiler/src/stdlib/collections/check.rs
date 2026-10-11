//! Deque operations preserve moves and borrowed element storage.
use super::*;
impl Analyzer {
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
        let Type::Record(id) = self.expression(context, base, Mode::Read, loans, None)? else { unreachable!() };
        let element = self.program.records[id].native_element().unwrap();
        if !read { Self::storage_access(context, base, Mode::Mutate, expression.span, loans)?; }
        let mut active = loans.clone();
        for root in Self::loan_roots(base, if read { Mode::Read } else { Mode::Mutate }) {
            active.entry(root).or_default().push((if read { Mode::Read } else { Mode::Mutate }, base.span));
        }
        if let Some(argument) = arguments.first_mut() {
            let expected = if push { element } else { Type::Int };
            let actual = self.expression(context, argument, if push { Mode::Consume } else { Mode::Read }, &active, Some(expected))?;
            self.require(actual, Some(expected), argument.span)?;
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
