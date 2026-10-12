//! Schema method lowering preserves ordinary native argument ownership checks.
use super::*;
impl Analyzer {
    pub(super) fn schema_method_call(&self, context: &Context, expr: &mut Expr) -> Result<(), Diagnostic> {
        let ExprKind::Call(callee, args) = &mut expr.kind else { return Ok(()); };
        let ExprKind::Field(base, member) = &callee.kind else { return Ok(()); };
        let Some(Type::Record(id)) = self.expression_type_hint(context, base) else { return Ok(()); };
        let record = &self.program.records[id];
        if record.native_type != Some(crate::native_signatures::NativeTypeId::Schema) { return Ok(()); }
        if member != "safeParse" { return Err(Diagnostic::new("schema-member", "Unsupported schema method.", callee.span)); }
        if args.len() != 1 { return Err(Diagnostic::new("arity", "safeParse expects one input value.", expr.span)); }
        if record.application.as_ref().and_then(|application| application.resolved_arguments.first()).copied() != Some(Type::String) {
            return Err(Diagnostic::new("schema-type", "safeParse currently supports string schemas.", callee.span));
        }
        args.insert(0, (**base).clone());
        callee.kind = ExprKind::Name("@native:std:schema#__safeParseString".into());
        Ok(())
    }
}
