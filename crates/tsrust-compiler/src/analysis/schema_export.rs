//! Export borrows the exact reusable plan and has its own checked error domain.
use super::*;

impl Analyzer {
    pub(super) fn schema_export(&mut self, context: &mut Context, expr: &mut Expr, loans: &Loans, expected: Option<Type>) -> Result<Option<Type>, Diagnostic> {
        let ExprKind::Call(callee, args) = &mut expr.kind else { return Ok(None); };
        let ExprKind::Field(base, method) = &callee.kind else { return Ok(None); };
        if method != "toJsonSchema" { return Ok(None); }
        if !args.is_empty() {
            return Err(Diagnostic::new("arity", "toJsonSchema takes no arguments.", expr.span));
        }
        let mut receiver = (**base).clone();
        let ty = self.expression(context, &mut receiver, Mode::Read, loans, None)?;
        let Type::Record(root) = ty else {
            return Err(Diagnostic::new("schema-type", "toJsonSchema requires a schema receiver.", callee.span));
        };
        if self.schema_pipelines.types(root).is_none()
            && self.program.records[root].native_type != Some(crate::native_signatures::NativeTypeId::Schema) {
            return Err(Diagnostic::new("schema-type", "toJsonSchema requires a schema receiver.", callee.span));
        }
        let output = self.schema_native_dependency(crate::native_signatures::NativeTypeId::JsonValue, expr.span)?;
        let failure = self.schema_native_dependency(crate::native_signatures::NativeTypeId::SchemaExportError, expr.span)?;
        self.require(Type::Record(output), expected, expr.span)?;
        *args = vec![receiver];
        callee.kind = ExprKind::Name("@schema:export".into());
        expr.target = Some(CallTarget::SchemaExport(failure));
        Ok(Some(Type::Record(output)))
    }
}
