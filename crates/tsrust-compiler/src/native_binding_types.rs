//! Prepare recursive native signatures before ordinary generic specialization.
//! Private schema fields reuse the checked arena's Result/array resolution, so
//! native calls do not maintain a second type or tagged-union implementation.
use crate::{Diagnostic, Span, native_bindings::{NativeBinding, NativeType}, native_signatures::Signature,
    syntax::{Record, RecordField, Token, TokenKind, Type}};

fn key(signature: &Signature) -> Result<String, Diagnostic> {
    signature.render(|id| Some(crate::native_type_resolution::linked_name(id)))
        .map(|text| format!("{}native-signature#{text}", crate::type_aliases::LINKED_PREFIX))
        .map_err(|_| Diagnostic::new("native-type-limit", "Native signature exceeds its rendering limit.", Span::default()))
}

pub(crate) fn prepare(records: &mut Vec<Record>, binding: &NativeBinding, span: Span) -> Result<(), Diagnostic> {
    binding.validate_returned_borrow().map_err(|error| Diagnostic::new(
        "native-return-borrow-contract", format!("Invalid native returned-borrow contract: {error:?}."), span,
    ))?;
    // Fail closed until source provenance and lifetime-bearing emission are integrated.
    // A valid contract must never be projected as an ordinary owned result.
    if binding.returned_borrow.is_some() {
        return Err(Diagnostic::new("native-return-borrow", "Native returned borrows require source provenance and a lifetime-bearing backend representation.", span));
    }
    for ty in binding.parameters.iter().map(|parameter| parameter.ty).chain(std::iter::once(binding.result)) {
        let NativeType::Structured(signature) = ty else { continue; };
        let name = key(signature)?;
        if records.iter().any(|record| record.name.text == name) { continue; }
        let annotation = crate::native_type_resolution::project(records, signature, span)?;
        records.push(Record {
            native_type: None, map_definition: None, set_definition: None, enum_definition: None,
            application: None, name: Token { text: name, kind: TokenKind::Symbol, span }, type_parameters: Vec::new(),
            fields: vec![RecordField {
                name: Token { text: "signature#value".into(), kind: TokenKind::Symbol, span },
                annotation, ty: None, callable_declaration: None, optional: false,
            }],
        });
    }
    Ok(())
}

pub(crate) fn resolve(records: &[Record], signature: &Signature) -> Option<Type> {
    let name = key(signature).ok()?;
    records.iter().find(|record| record.name.text == name)?.fields.first()?.ty
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_bindings::{NativeBackend, NativeParameter};
    use crate::native_signatures::NativeTypeId;
    #[test]
    fn recursive_native_signatures_share_checked_result_identity() {
        static RESULT: Signature = Signature::Result(&Signature::Named(NativeTypeId::Bytes), &Signature::Named(NativeTypeId::ByteError));
        let binding = NativeBinding { returned_borrow: None, module: "std:bytes", name: "probe", parameters: &[NativeParameter {
            name: "values", ty: NativeType::Structured(&Signature::Array(&Signature::Int)), access: crate::Mode::Read, optional: false, rest: false,
        }], result: NativeType::Structured(&RESULT), backend: NativeBackend::Bundled { symbol: "unused" }, features: &[] };
        let mut program = crate::syntax::parse("function main(){}").unwrap();
        prepare(&mut program.records, &binding, Span::new(0, 0)).unwrap();
        let count = program.records.len();
        prepare(&mut program.records, &binding, Span::new(0, 0)).unwrap();
        assert_eq!(program.records.len(), count);
        let checked = crate::analysis::analyze(program).unwrap();
        let ty = resolve(&checked.records, &RESULT).unwrap();
        let Type::Union(id) = ty else { panic!("checked Result union"); };
        let variants = crate::result_identity::variants(&checked.records, &checked.unions[id.index()].members).unwrap();
        for (variant, identity) in variants.into_iter().zip([NativeTypeId::Bytes, NativeTypeId::ByteError]) {
            let Type::Record(payload) = crate::result_identity::payload(&checked.records[variant]).unwrap() else { panic!("nominal payload"); };
            assert_eq!(checked.records[payload].native_type, Some(identity));
        }
    }
}
