//! Contextual Result construction reuses ordinary owned record checking and
//! tagged-union injection. No inactive payload is initialized or copied.
use super::*;

impl Analyzer {
    #[inline(never)]
    pub(super) fn result_constructor(
        &mut self,
        context: &mut Context,
        expression: &mut Expr,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Option<Type>, Diagnostic> {
        let ExprKind::Call(callee, _) = &expression.kind else { return Ok(None); };
        let ExprKind::Name(name) = &callee.kind else { return Ok(None); };
        let explicit = matches!(name.as_str(), "@result:Ok" | "@result:Err");
        if !explicit && (!matches!(name.as_str(), "Ok" | "Err")
            || context.names.contains_key(name) || context.pending_names.contains(name)
            || self.function_names.contains_key(name) || self.generic_functions.template(name).is_some()
        ) { return Ok(None); }
        let variant = name.strip_prefix("@result:").unwrap_or(name).to_owned();
        let Some(Type::Union(union)) = expected else {
            return Err(Diagnostic::new("annotation-required", "A Result constructor needs a Result<T, E> annotation or call/return context.", expression.span));
        };
        let members = self.unions.members(union);
        let marker = format!("Result#{variant}");
        let selected = members.iter().find_map(|member| {
            let Type::Record(id) = member else { return None; };
            let application = self.program.records[*id].application.as_ref()?;
            (application.declaration.text == marker).then_some(*id)
        });
        let Some(record) = selected.filter(|_| members.len() == 2) else {
            return Err(Diagnostic::new("type", "Ok and Err construct Result<T, E> values; this context requires another type.", expression.span));
        };
        let required = self.program.records[record].fields.len();
        let ExprKind::Call(_, arguments) = &mut expression.kind else { unreachable!(); };
        if arguments.len() != required {
            return Err(Diagnostic::new("arity", format!("{variant} expects {required} payload arguments, found {}.", arguments.len()), expression.span));
        }
        let mut fields = arguments.pop().map(|value| vec![(Token { text: "payload".into(), span: value.span, kind: TokenKind::Name }, value)]).unwrap_or_default();
        let actual = self.record_literal(context, &mut fields, loans, Some(Type::Record(record)), expression.span)?;
        self.require(actual, Some(Type::Union(union)), expression.span)?;
        *expression = Expr::checked(ExprKind::Record(fields), expression.span, Some(actual));
        expression.inject(union);
        Ok(Some(Type::Union(union)))
    }
}
