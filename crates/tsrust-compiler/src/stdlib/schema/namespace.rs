//! Resolve only linked schema namespace identities; source aliases remain lexical.
use crate::{Diagnostic, syntax::{Program, ExprKind}, native_bindings::{NativeBinding, NativeBackend, NativeType}, native_signatures::{Signature, NativeTypeId}};

pub(crate) const STRING: NativeBinding = NativeBinding {
    module: "std:schema", name: "__string", parameters: &[],
    result: NativeType::Structured(&Signature::Applied(NativeTypeId::Schema, &[Signature::String])),
    returned_borrow: None, backend: NativeBackend::Bundled { symbol: "tsrust_schema_string" }, features: &[],
};

pub(crate) fn prepare(program: &mut Program) -> Result<(), Diagnostic> {
    let mut spans = Vec::new();
    for function in &mut program.functions {
        crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |expression| {
            let ExprKind::Field(base, member) = &expression.kind else { return Ok(()); };
            if !matches!(&base.kind, ExprKind::Name(name) if name == crate::native_exports::Namespace::Schema.symbol()) { return Ok(()); }
            if member != "string" {
                return Err(Diagnostic::new("schema-member", format!("Unknown schema builder {member:?}."), expression.span));
            }
            spans.push(expression.span);
            expression.kind = ExprKind::Name("@native:std:schema#__string".into());
            Ok(())
        })?;
    }
    for span in spans { crate::native_binding_types::prepare(&mut program.records, &STRING, span)?; }
    Ok(())
}
