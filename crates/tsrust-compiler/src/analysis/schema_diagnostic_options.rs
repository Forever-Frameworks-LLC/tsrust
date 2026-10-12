//! Constructor messages wrap the exact child plan without copying it.
use super::*;
use crate::stdlib::schema::pipelines::WrapperKind;

impl Analyzer {
    pub(super) fn schema_constructor_options(
        &mut self, context: &mut Context, expr: &mut Expr, loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Option<Type>, Diagnostic> {
        let ExprKind::Call(callee, args) = &expr.kind else { return Ok(None); };
        let ExprKind::Name(name) = &callee.kind else { return Ok(None); };
        let required = match name.as_str() {
            "@native:std:schema#__string" | "@native:std:schema#__int" |
            "@native:std:schema#__number" | "@native:std:schema#__boolean" |
            "@native:std:schema#__null" => 0,
            "@schema:array" | "@schema:object" | "@schema:record" | "@schema:set" |
            "@schema:literal" | "@schema:enum" | "@schema:union" => 1,
            "@schema:map" | "@schema:discriminatedUnion" => 2,
            _ => return Ok(None),
        };
        if args.len() != required + 1 { return Ok(None); }
        let mut receiver = expr.clone();
        let ExprKind::Call(_, args) = &mut receiver.kind else { unreachable!() };
        let options = args.pop().unwrap();
        let ExprKind::Record(fields) = options.kind else {
            return Err(Diagnostic::new("schema-options", "Schema options require an object containing a string message.", options.span));
        };
        let mut message = None;
        for (name, value) in fields {
            if name.text != "message" || message.is_some() {
                return Err(Diagnostic::new("schema-options", "Constructor options accept one message field.", name.span));
            }
            message = Some(value);
        }
        let ty = self.expression(context, &mut receiver, Mode::Consume, loans, None)?;
        let Type::Record(source) = ty else { unreachable!("schema constructor result") };
        let Some(mut message) = message else {
            self.require(ty, expected, expr.span)?;
            *expr = receiver;
            return Ok(Some(ty));
        };
        self.expression(context, &mut message, Mode::Consume, loans, Some(Type::String))?;
        self.schema_pipeline_wrapper(expr, source, receiver, WrapperKind::Message, Some(message), expected).map(Some)
    }
}
