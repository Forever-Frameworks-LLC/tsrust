//! Schema method lowering preserves ordinary native argument ownership checks.
use super::*;
impl Analyzer {
    pub(super) fn schema_method_call(&self, context: &Context, expr: &mut Expr) -> Result<(), Diagnostic> {
        let ExprKind::Call(callee, args) = &mut expr.kind else { return Ok(()); };
        let ExprKind::Field(base, member) = &callee.kind else { return Ok(()); };
        let Some(Type::Record(id)) = self.expression_type_hint(context, base) else { return Ok(()); };
        let record = &self.program.records[id];
        if record.native_type != Some(crate::native_signatures::NativeTypeId::Schema) { return Ok(()); }
        if matches!((member.as_str(),record.application.as_ref().and_then(|application|application.resolved_arguments.first())), ("min"|"max",Some(Type::String|Type::Int)) | ("gt"|"lt",Some(Type::Int))) {
            if args.len()!=1 {return Err(Diagnostic::new("arity","Constraints expect one integer limit.",expr.span));}
            args.insert(0,(**base).clone());
            let kind=if record.application.as_ref().unwrap().resolved_arguments[0]==Type::Int {"Int"}else{"String"};
            callee.kind=ExprKind::Name(format!("@schema:__{}{kind}",member));
            return Ok(());
        }
        if member != "safeParse" { return Err(Diagnostic::new("schema-member", "Unsupported schema method.", callee.span)); }
        if args.len() != 1 { return Err(Diagnostic::new("arity", "safeParse expects one input value.", expr.span)); }
        let output = record.application.as_ref().and_then(|application| application.resolved_arguments.first()).copied();
        let input = self.expression_type_hint(context, &args[0]);
        let json = matches!(input, Some(Type::Record(input)) if self.program.records[input].native_type == Some(crate::native_signatures::NativeTypeId::JsonValue));
        let symbol = match (output, input, json) {
            (Some(Type::String), Some(Type::String), _) => "@native:std:schema#__safeParseOwnedString",
            (Some(Type::String), _, true) => "@native:std:schema#__safeParseString",
            (Some(Type::Boolean), Some(Type::Boolean), _) => "@native:std:schema#__safeParseBooleanValue",
            (Some(Type::Boolean), _, true) => "@native:std:schema#__safeParseBoolean",
            (Some(Type::Int), Some(Type::Int), _) => "@native:std:schema#__safeParseIntValue",
            (Some(Type::Int), _, true) => "@native:std:schema#__safeParseInt",
            (Some(Type::Number), Some(Type::Number), _) => "@native:std:schema#__safeParseNumberValue",
            (Some(Type::Number), Some(Type::Int), _) => "@native:std:schema#__safeParseNumberInt",
            (Some(Type::Number), _, true) => "@native:std:schema#__safeParseNumber",
            (Some(Type::Null), Some(Type::Null), _) => "@native:std:schema#__safeParseNullValue",
            (Some(Type::Null), _, true) => "@native:std:schema#__safeParseNull",
            (Some(Type::String | Type::Boolean | Type::Int | Type::Number | Type::Null), _, _) => return Err(Diagnostic::new("schema-input", "safeParse requires a matching concrete scalar or owned JsonValue input.", args[0].span)),
            _ => return Err(Diagnostic::new("schema-type", "safeParse currently supports string, boolean, integer, number and null schemas.", callee.span)),
        };
        args.insert(0, (**base).clone());
        callee.kind = ExprKind::Name(symbol.into());
        Ok(())
    }
}
