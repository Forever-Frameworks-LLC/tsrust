//! Resolve only linked schema namespace identities; source aliases remain lexical.
use crate::{Diagnostic, syntax::{Program, ExprKind}, native_bindings::{NativeBinding, NativeBackend, NativeType}, native_signatures::{Signature, NativeTypeId}};

pub(crate) const STRING: NativeBinding = NativeBinding {
    module: "std:schema", name: "__string", parameters: &[],
    result: NativeType::Structured(&Signature::Applied(NativeTypeId::Schema, &[Signature::String])),
    returned_borrow: None, backend: NativeBackend::Bundled { symbol: "tsrust_schema_string" }, features: &[],
};

pub(crate) const SAFE_PARSE: NativeBinding = NativeBinding {
    module: "std:schema", name: "__safeParseString",
    parameters: &[
        crate::native_bindings::NativeParameter { name: "schema", ty: NativeType::Structured(&Signature::Applied(NativeTypeId::Schema, &[Signature::String])), access: crate::Mode::Read, optional: false, rest: false },
        crate::native_bindings::NativeParameter { name: "input", ty: NativeType::Structured(&Signature::Named(NativeTypeId::JsonValue)), access: crate::Mode::Consume, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::String, &Signature::Named(NativeTypeId::SchemaError))),
    returned_borrow: None, backend: NativeBackend::Bundled { symbol: "tsrust_schema_safe_parse_string" }, features: &["json"],
};

pub(crate) const SAFE_PARSE_OWNED: NativeBinding = NativeBinding {
    name: "__safeParseOwnedString",
    parameters: &[
        crate::native_bindings::NativeParameter { name: "schema", ty: NativeType::Structured(&Signature::Applied(NativeTypeId::Schema, &[Signature::String])), access: crate::Mode::Read, optional: false, rest: false },
        crate::native_bindings::NativeParameter { name: "input", ty: NativeType::String, access: crate::Mode::Consume, optional: false, rest: false },
    ],
    backend: NativeBackend::Bundled { symbol: "tsrust_schema_safe_parse_owned_string" }, features: &[],
    ..SAFE_PARSE
};

pub(crate) const BOOLEAN: NativeBinding = NativeBinding {
    name: "__boolean", result: NativeType::Structured(&Signature::Applied(NativeTypeId::Schema, &[Signature::Boolean])),
    backend: NativeBackend::Bundled { symbol: "tsrust_schema_boolean" }, ..STRING
};
pub(crate) const SAFE_PARSE_BOOLEAN: NativeBinding = NativeBinding {
    name: "__safeParseBoolean",
    parameters: &[
        crate::native_bindings::NativeParameter { name: "schema", ty: NativeType::Structured(&Signature::Applied(NativeTypeId::Schema, &[Signature::Boolean])), access: crate::Mode::Read, optional: false, rest: false },
        SAFE_PARSE.parameters[1],
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Boolean, &Signature::Named(NativeTypeId::SchemaError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_schema_safe_parse_boolean" }, ..SAFE_PARSE
};
pub(crate) const SAFE_PARSE_BOOLEAN_VALUE: NativeBinding = NativeBinding {
    name: "__safeParseBooleanValue",
    parameters: &[
        SAFE_PARSE_BOOLEAN.parameters[0],
        crate::native_bindings::NativeParameter { name: "input", ty: NativeType::Boolean, access: crate::Mode::Copy, optional: false, rest: false },
    ],
    backend: NativeBackend::Bundled { symbol: "tsrust_schema_safe_parse_boolean_value" }, features: &[], ..SAFE_PARSE_BOOLEAN
};

pub(crate) const INT: NativeBinding = NativeBinding {
    name: "__int", result: NativeType::Structured(&Signature::Applied(NativeTypeId::Schema, &[Signature::Int])),
    backend: NativeBackend::Bundled { symbol: "tsrust_schema_int" }, ..STRING
};
pub(crate) const SAFE_PARSE_INT: NativeBinding = NativeBinding {
    name: "__safeParseInt",
    parameters: &[
        crate::native_bindings::NativeParameter { name: "schema", ty: NativeType::Structured(&Signature::Applied(NativeTypeId::Schema, &[Signature::Int])), access: crate::Mode::Read, optional: false, rest: false },
        SAFE_PARSE.parameters[1],
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Int, &Signature::Named(NativeTypeId::SchemaError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_schema_safe_parse_int" }, ..SAFE_PARSE
};
pub(crate) const SAFE_PARSE_INT_VALUE: NativeBinding = NativeBinding {
    name: "__safeParseIntValue",
    parameters: &[
        SAFE_PARSE_INT.parameters[0],
        crate::native_bindings::NativeParameter { name: "input", ty: NativeType::Int, access: crate::Mode::Copy, optional: false, rest: false },
    ],
    backend: NativeBackend::Bundled { symbol: "tsrust_schema_safe_parse_int_value" }, features: &[], ..SAFE_PARSE_INT
};

pub(crate) fn prepare(program: &mut Program) -> Result<(), Diagnostic> {
    let mut spans = Vec::new();
    let mut uses_safe_parse = false;
    for function in &mut program.functions {
        crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |expression| {
            let ExprKind::Field(base, member) = &expression.kind else { return Ok(()); };
            uses_safe_parse |= member == "safeParse";
            if !matches!(&base.kind, ExprKind::Name(name) if name == crate::native_exports::Namespace::Schema.symbol()) { return Ok(()); }
            let binding = match member.as_str() {
                "string" => &STRING,
                "boolean" => &BOOLEAN,
                "int" => &INT,
                _ => return Err(Diagnostic::new("schema-member", format!("Unknown schema builder {member:?}."), expression.span)),
            };
            spans.push((binding, expression.span));
            expression.kind = ExprKind::Name(format!("@native:std:schema#{}", binding.name));
            Ok(())
        })?;
    }
    for (binding, span) in spans { crate::native_binding_types::prepare(&mut program.records, binding, span)?; }
    if uses_safe_parse && program.records.iter().any(|record| record.native_type == Some(NativeTypeId::Schema)) {
        crate::native_binding_types::prepare(&mut program.records, &SAFE_PARSE_OWNED, crate::Span::default())?;
        crate::native_binding_types::prepare(&mut program.records, &SAFE_PARSE_BOOLEAN_VALUE, crate::Span::default())?;
        crate::native_binding_types::prepare(&mut program.records, &SAFE_PARSE_INT_VALUE, crate::Span::default())?;
    }
    // Current validation consumes JsonValue. Do not synthesize JSON storage solely
    // because an unrelated object also declares a method named safeParse.
    if uses_safe_parse
        && program.records.iter().any(|record| record.native_type == Some(NativeTypeId::Schema))
        && program.records.iter().any(|record| record.native_type == Some(NativeTypeId::JsonValue))
    {
        crate::native_binding_types::prepare(&mut program.records, &SAFE_PARSE, crate::Span::default())?;
        crate::native_binding_types::prepare(&mut program.records, &SAFE_PARSE_BOOLEAN, crate::Span::default())?;
        crate::native_binding_types::prepare(&mut program.records, &SAFE_PARSE_INT, crate::Span::default())?;
    }
    Ok(())
}
