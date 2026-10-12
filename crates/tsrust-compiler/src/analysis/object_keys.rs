use super::*;

impl Analyzer {
    #[inline(never)]
    pub(super) fn object_has_own(&mut self, context: &mut Context, args: &mut [Expr], loans: &Loans, span: Span, in_operator: bool) -> Result<Type, Diagnostic> {
        if args.len()!=2 { return Err(Diagnostic::new("arity","Object.hasOwn expects a value and a property key.",span)); }
        let mut key_loans = loans.clone();
        let early_key = if in_operator {
            let key = self.expression(context, &mut args[1], Mode::Read, loans, None)?;
            if !matches!(key, Type::String | Type::Int | Type::Number) {
                return Err(Diagnostic::new("type", "The in key must be a string, int or number.", args[1].span));
            }
            for root in Self::loan_roots(&args[1], Mode::Read) {
                key_loans.entry(root).or_default().push((Mode::Read,args[1].span));
            }
            Some(key)
        } else { None };
        let prior=self.builtin_literals;self.builtin_literals=true;
        let value=self.expression(context,&mut args[0],Mode::Read,&key_loans,None);
        self.builtin_literals=prior;let value=value?;
        self.require_native_capability(value, crate::native_types::Capability::PropertyReflection, args[0].span, false)?;
        if in_operator && !matches!(value, Type::Record(_) | Type::Array(_)) {
            return Err(Diagnostic::new("type", "The in value must be a record, tuple or array.", args[0].span));
        }
        if !matches!(value,Type::Record(_)|Type::Array(_)|Type::String|Type::Int|Type::Number|Type::Boolean) {
            return Err(Diagnostic::new("unsupported","Object.hasOwn accepts records, tuples, arrays, strings, numbers and booleans; nullable values and functions are unsupported.",args[0].span));
        }
        let mut active=loans.clone();
        for root in Self::loan_roots(&args[0],Mode::Read) { active.entry(root).or_default().push((Mode::Read,args[0].span)); }
        let key=if let Some(key)=early_key { key } else { self.expression(context,&mut args[1],Mode::Read,&active,None)? };
        let members=key.union().map_or_else(||vec![key],|id|self.unions.members(id));
        if members.iter().any(|ty|!matches!(ty,Type::String|Type::Int|Type::Number|Type::Boolean|Type::Null|Type::Undefined)) {
            return Err(Diagnostic::new("unsupported","Object.hasOwn property keys must be strings or primitive scalar values.",args[1].span));
        }
        Ok(Type::Boolean)
    }

    pub(super) fn object_keys(
        &mut self,
        context: &mut Context,
        args: &mut [Expr],
        loans: &Loans,
        span: Span,
    ) -> Result<Type, Diagnostic> {
        if args.len() != 1 {
            return Err(Diagnostic::new(
                "arity",
                "Object.keys expects one value.",
                span,
            ));
        }
        let prior = self.builtin_literals;
        self.builtin_literals = true;
        let actual = self.expression(context, &mut args[0], Mode::Read, loans, None);
        self.builtin_literals = prior;
        let actual = actual?;
        self.require_native_capability(actual, crate::native_types::Capability::PropertyReflection, args[0].span, false)?;
        if !matches!(
            actual,
            Type::Record(_)
                | Type::Array(_)
                | Type::String
                | Type::Int
                | Type::Number
                | Type::Boolean
        ) {
            return Err(Diagnostic::new(
                "unsupported",
                "Object.keys accepts records, tuples, arrays, strings, numbers and booleans; nullable values and functions are unsupported.",
                args[0].span,
            ));
        }
        Ok(Type::Array(ArrayElement::String(0)))
    }
}
