//! Check field values before preserving concrete callable payload layout.
use super::*;

impl Analyzer {
    #[inline(never)]
    pub(super) fn scoped_record_field_value(
        &mut self,
        context: &mut Context,
        value: &mut Expr,
        field_type: Type,
        loans: &Loans,
    ) -> Result<Type, Diagnostic> {
        let field_type=if let Type::BorrowedCallable(family)=field_type{Type::Callback(self.callables.storage.declaration(family).expect("borrowed field declaration"))}else{field_type};
        let payload = self.unions.payload(field_type).unwrap_or(field_type);
        let expected = if matches!(payload, Type::Callback(_))
            && !matches!(value.kind, ExprKind::Closure { .. })
        {
            None
        } else {
            Some(if matches!(payload, Type::Callback(_)) {
                payload
            } else {
                field_type
            })
        };
        let actual = self.expression(context, value, Mode::Consume, loans, expected)?;
        let Type::Callback(declaration) = payload else {
            return Ok(actual);
        };
        if !matches!(actual, Type::Null | Type::Undefined) {
            return Ok(actual);
        }
        // Verify the written absence alternative before constructing an empty
        // native callback payload. The signature remains available to call help.
        self.require(actual, Some(field_type), value.span)?;
        let absent = self.absent_callback(declaration);
        let alternatives = self
            .unions
            .members(field_type.union().expect("checked nullable field"))
            .into_iter()
            .map(|ty| if ty == payload { absent } else { ty });
        let concrete = self.unions.intern(alternatives, value.span)?;
        value.inject(concrete.union().expect("nullable concrete callback"));
        Ok(concrete)
    }
}
