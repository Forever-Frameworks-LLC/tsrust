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

pub(crate) const NULL: NativeBinding = NativeBinding {
    name: "__null", result: NativeType::Structured(&Signature::Applied(NativeTypeId::Schema, &[Signature::Null])),
    backend: NativeBackend::Bundled { symbol: "tsrust_schema_null" }, ..STRING
};
pub(crate) const SAFE_PARSE_NULL: NativeBinding = NativeBinding {
    name: "__safeParseNull",
    parameters: &[
        crate::native_bindings::NativeParameter { name: "schema", ty: NativeType::Structured(&Signature::Applied(NativeTypeId::Schema, &[Signature::Null])), access: crate::Mode::Read, optional: false, rest: false },
        SAFE_PARSE.parameters[1],
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Null, &Signature::Named(NativeTypeId::SchemaError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_schema_safe_parse_null" }, ..SAFE_PARSE
};
pub(crate) const SAFE_PARSE_NULL_VALUE: NativeBinding = NativeBinding {
    name: "__safeParseNullValue",
    parameters: &[
        SAFE_PARSE_NULL.parameters[0],
        crate::native_bindings::NativeParameter { name: "input", ty: NativeType::Structured(&Signature::Null), access: crate::Mode::Copy, optional: false, rest: false },
    ],
    backend: NativeBackend::Bundled { symbol: "tsrust_schema_safe_parse_null_value" }, features: &[], ..SAFE_PARSE_NULL
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

pub(crate) const NUMBER: NativeBinding = NativeBinding {
    name: "__number", result: NativeType::Structured(&Signature::Applied(NativeTypeId::Schema, &[Signature::Number])),
    backend: NativeBackend::Bundled { symbol: "tsrust_schema_number" }, ..STRING
};
pub(crate) const SAFE_PARSE_NUMBER: NativeBinding = NativeBinding {
    name: "__safeParseNumber",
    parameters: &[
        crate::native_bindings::NativeParameter { name: "schema", ty: NativeType::Structured(&Signature::Applied(NativeTypeId::Schema, &[Signature::Number])), access: crate::Mode::Read, optional: false, rest: false },
        SAFE_PARSE.parameters[1],
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Number, &Signature::Named(NativeTypeId::SchemaError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_schema_safe_parse_number" }, ..SAFE_PARSE
};
pub(crate) const SAFE_PARSE_NUMBER_VALUE: NativeBinding = NativeBinding {
    name: "__safeParseNumberValue",
    parameters: &[
        SAFE_PARSE_NUMBER.parameters[0],
        crate::native_bindings::NativeParameter { name: "input", ty: NativeType::Number, access: crate::Mode::Copy, optional: false, rest: false },
    ],
    backend: NativeBackend::Bundled { symbol: "tsrust_schema_safe_parse_number_value" }, features: &[], ..SAFE_PARSE_NUMBER
};

pub(crate) const SAFE_PARSE_NUMBER_INT: NativeBinding = NativeBinding {
    name: "__safeParseNumberInt",
    parameters: &[
        SAFE_PARSE_NUMBER.parameters[0],
        crate::native_bindings::NativeParameter { name: "input", ty: NativeType::Int, access: crate::Mode::Copy, optional: false, rest: false },
    ],
    backend: NativeBackend::Bundled { symbol: "tsrust_schema_safe_parse_number_int" }, features: &[], ..SAFE_PARSE_NUMBER
};

pub(crate) const MIN_STRING: NativeBinding = NativeBinding {
    module: "std:schema", name: "__minString",
    parameters: &[
        crate::native_bindings::NativeParameter { name: "schema", access: crate::Mode::Consume, ..SAFE_PARSE.parameters[0] },
        crate::native_bindings::NativeParameter { name: "limit", ty: NativeType::Int, access: crate::Mode::Copy, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Applied(NativeTypeId::Schema, &[Signature::String]), &Signature::Named(NativeTypeId::SchemaConstructionError))),
    returned_borrow: None, backend: NativeBackend::Bundled { symbol: "tsrust_schema_min_string" }, features: &[],
};
pub(crate) const MAX_STRING: NativeBinding = NativeBinding {
    name: "__maxString", backend: NativeBackend::Bundled { symbol: "tsrust_schema_max_string" }, ..MIN_STRING
};

pub(crate) const MIN_INT: NativeBinding = NativeBinding {
    module: "std:schema", name: "__minInt",
    parameters: &[
        crate::native_bindings::NativeParameter { name: "schema", access: crate::Mode::Consume, ..SAFE_PARSE_INT.parameters[0] },
        crate::native_bindings::NativeParameter { name: "limit", ty: NativeType::Int, access: crate::Mode::Copy, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Applied(NativeTypeId::Schema, &[Signature::Int]), &Signature::Named(NativeTypeId::SchemaConstructionError))),
    returned_borrow: None, backend: NativeBackend::Bundled { symbol: "tsrust_schema_min_int" }, features: &[],
};
pub(crate) const MAX_INT: NativeBinding = NativeBinding {
    name: "__maxInt", backend: NativeBackend::Bundled { symbol: "tsrust_schema_max_int" }, ..MIN_INT
};

pub(crate) const GT_INT: NativeBinding = NativeBinding {
    name: "__gtInt", backend: NativeBackend::Bundled { symbol: "tsrust_schema_gt_int" }, ..MIN_INT
};
pub(crate) const LT_INT: NativeBinding = NativeBinding {
    name: "__ltInt", backend: NativeBackend::Bundled { symbol: "tsrust_schema_lt_int" }, ..MIN_INT
};

pub(crate) const MIN_NUMBER: NativeBinding = NativeBinding {
    name:"__minNumber",
    parameters:&[
        crate::native_bindings::NativeParameter{access:crate::Mode::Consume,..SAFE_PARSE_NUMBER.parameters[0]},
        crate::native_bindings::NativeParameter{ty:NativeType::Number,..MIN_INT.parameters[1]},
    ],
    result:NativeType::Structured(&Signature::Result(&Signature::Applied(NativeTypeId::Schema,&[Signature::Number]),&Signature::Named(NativeTypeId::SchemaConstructionError))),
    backend:NativeBackend::Bundled{symbol:"tsrust_schema_min_number"},..MIN_INT
};
pub(crate) const MAX_NUMBER: NativeBinding = NativeBinding {name:"__maxNumber",backend:NativeBackend::Bundled{symbol:"tsrust_schema_max_number"},..MIN_NUMBER};
pub(crate) const GT_NUMBER: NativeBinding = NativeBinding {name:"__gtNumber",backend:NativeBackend::Bundled{symbol:"tsrust_schema_gt_number"},..MIN_NUMBER};
pub(crate) const LT_NUMBER: NativeBinding = NativeBinding {name:"__ltNumber",backend:NativeBackend::Bundled{symbol:"tsrust_schema_lt_number"},..MIN_NUMBER};

pub(crate) fn prepare(program: &mut Program) -> Result<(), Diagnostic> {
    let mut spans = Vec::new();
    let mut uses_safe_parse = false;
    let mut uses_bounds = false;
    for function in &mut program.functions {
        crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |expression| {
            let ExprKind::Field(base, member) = &expression.kind else { return Ok(()); };
            uses_safe_parse |= member == "safeParse";
            uses_bounds |= matches!(member.as_str(),"min"|"max"|"gt"|"lt");
            if !matches!(&base.kind, ExprKind::Name(name) if name == crate::native_exports::Namespace::Schema.symbol()) { return Ok(()); }
            if matches!(member.as_str(),"array"|"object"|"literal"|"enum") {if matches!(member.as_str(),"literal"|"enum"){spans.push((&STRING,expression.span));spans.push((&NULL,expression.span));}expression.kind=ExprKind::Name(format!("@schema:{member}"));return Ok(());}
            let binding = match member.as_str() {
                "string" => &STRING,
                "boolean" => &BOOLEAN,
                "null" => &NULL,
                "int" => &INT,
                "number" => &NUMBER,
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
        crate::native_binding_types::prepare(&mut program.records, &SAFE_PARSE_NULL_VALUE, crate::Span::default())?;
        crate::native_binding_types::prepare(&mut program.records, &SAFE_PARSE_INT_VALUE, crate::Span::default())?;
        crate::native_binding_types::prepare(&mut program.records, &SAFE_PARSE_NUMBER_VALUE, crate::Span::default())?;
        crate::native_binding_types::prepare(&mut program.records, &SAFE_PARSE_NUMBER_INT, crate::Span::default())?;
    }
    // Current validation consumes JsonValue. Do not synthesize JSON storage solely
    // because an unrelated object also declares a method named safeParse.
    if uses_safe_parse
        && program.records.iter().any(|record| record.native_type == Some(NativeTypeId::Schema))
        && program.records.iter().any(|record| record.native_type == Some(NativeTypeId::JsonValue))
    {
        crate::native_binding_types::prepare(&mut program.records, &SAFE_PARSE, crate::Span::default())?;
        crate::native_binding_types::prepare(&mut program.records, &SAFE_PARSE_BOOLEAN, crate::Span::default())?;
        crate::native_binding_types::prepare(&mut program.records, &SAFE_PARSE_NULL, crate::Span::default())?;
        crate::native_binding_types::prepare(&mut program.records, &SAFE_PARSE_INT, crate::Span::default())?;
        crate::native_binding_types::prepare(&mut program.records, &SAFE_PARSE_NUMBER, crate::Span::default())?;
    }
    if uses_bounds && program.records.iter().any(|record| record.native_type == Some(NativeTypeId::Schema)) {
        for binding in [&MIN_STRING,&MAX_STRING,&MIN_INT,&MAX_INT,&GT_INT,&LT_INT,&MIN_NUMBER,&MAX_NUMBER,&GT_NUMBER,&LT_NUMBER] {
            crate::native_binding_types::prepare(&mut program.records,binding,crate::Span::default())?;
            let wrapper=super::builders::adapter(program,binding)?;
            program.functions.push(wrapper);
        }
    }
    Ok(())
}
