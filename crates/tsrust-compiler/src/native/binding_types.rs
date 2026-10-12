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
    // A returned borrow is published only with a validated contract: the analyzer
    // attaches its source loans and the emitter keeps the real slice lifetime.
    let borrowed_shape = binding.contains_borrowed_signatures().map_err(|_| Diagnostic::new(
        "native-type-limit", "Native signature exceeds its borrowed-shape traversal limit.", span,
    ))?;
    if borrowed_shape && binding.returned_borrow.is_none() {
        return Err(Diagnostic::new("native-return-borrow", "A native result that borrows storage requires a returned-borrow contract.", span));
    }
    project_shapes(records, binding, span)
}

fn project_shapes(records: &mut Vec<Record>, binding: &NativeBinding, span: Span) -> Result<(), Diagnostic> {
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

#[cfg(test)]
pub(crate) fn prepare_private_view_test(records: &mut Vec<Record>, binding: &NativeBinding) -> Result<(), Diagnostic> {
    binding.validate_returned_borrow().unwrap();
    project_shapes(records, binding, Span::default())
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
    fn native_view_factory_bindings_are_valid_and_published() {
        for (name,access,payload) in [
            ("view",crate::native_return_borrows::BorrowAccess::ReadOnly,NativeTypeId::ByteView),
            ("viewMut",crate::native_return_borrows::BorrowAccess::Exclusive,NativeTypeId::ByteViewMut),
        ] {
            let binding=crate::native_bindings::lookup("std:bytes",name).unwrap();
            binding.validate_returned_borrow().unwrap();
            let contract=binding.returned_borrow.unwrap();
            assert_eq!(contract.source_parameter,0);
            assert_eq!(contract.access,access);
            assert_eq!(contract.payload,payload);
            assert_eq!(contract.slot,crate::native_return_borrows::ReturnSlot::ResultOk);
            let mut records=Vec::new();
            prepare(&mut records,binding,Span::default()).unwrap();
            assert!(!records.is_empty());
            assert!(crate::native_types::lookup("std:bytes",crate::native_types::declaration(payload).name).is_some());
        }
    }
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
