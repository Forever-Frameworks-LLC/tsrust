//! Prepare recursive native signatures before ordinary generic specialization.
//! Private schema fields reuse the checked arena's Result/array resolution, so
//! native calls do not maintain a second type or tagged-union implementation.
use crate::{Diagnostic, Span, native_bindings::{NativeBinding, NativeType}, native_signatures::Signature,
    syntax::{Record, RecordField, Token, TokenKind, Type}};

fn key(signature:&Signature,parameters:&[Signature])->Result<String,Diagnostic>{
    let failure=||Diagnostic::new("native-type-limit","Native signature exceeds its rendering limit or has an unresolved parameter.",Span::default());
    let arguments=parameters.iter().map(|parameter|parameter.render(|id|Some(crate::native_type_resolution::linked_name(id))).map(|text|text.into_owned())).collect::<Result<Vec<_>,_>>().map_err(|_|failure())?;
    signature.render_with_parameters(|id|Some(crate::native_type_resolution::linked_name(id)),|index|arguments.get(index).cloned())
        .map(|text|format!("{}native-signature#{text}",crate::type_aliases::LINKED_PREFIX)).map_err(|_|failure())
}

/// Prepare a concrete type shape. Callable borrowed-result validation remains in
/// prepare; this helper does not register or authorize a native callable.
pub(crate) fn prepare_signature(records:&mut Vec<Record>,signature:&Signature,parameters:&[Signature],span:Span)->Result<(),Diagnostic>{
    let name=key(signature,parameters)?;
    if records.iter().any(|record|record.name.text==name){return Ok(());}
    let annotation=crate::native_type_resolution::project_with_parameters(records,signature,parameters,span)?;
    records.push(Record {
        native_type:None,map_definition:None,set_definition:None,enum_definition:None,literal_definition:None,
        application:None,name:Token{text:name,kind:TokenKind::Symbol,span},type_parameters:Vec::new(),
        fields:vec![RecordField{name:Token{text:"signature#value".into(),kind:TokenKind::Symbol,span},annotation,ty:None,callable_declaration:None,optional:false}],
    });
    Ok(())
}

pub(crate) fn resolve_signature(records:&[Record],signature:&Signature,parameters:&[Signature])->Option<Type>{
    let name=key(signature,parameters).ok()?;
    records.iter().find(|record|record.name.text==name)?.fields.first()?.ty
}

pub(crate) fn prepare(records: &mut Vec<Record>, binding: &NativeBinding, span: Span) -> Result<(), Diagnostic> {
    binding.validate_returned_borrow().map_err(|error| Diagnostic::new(
        "native-return-borrow-contract", format!("Invalid native returned-borrow contract: {error:?}."), span,
    ))?;
    let borrowed_shape = binding.contains_borrowed_signatures().map_err(|_| Diagnostic::new(
        "native-type-limit", "Native signature exceeds its borrowed-shape traversal limit.", span,
    ))?;
    // SQLite has a lifetime-aware input ABI; other input-only borrowed bindings remain gated.
    if borrowed_shape && binding.returned_borrow.is_none() && binding.module!="std:sqlite" {
        return Err(Diagnostic::new("native-return-borrow", "A native result that borrows storage requires a returned-borrow contract.", span));
    }
    project_shapes(records, binding, span)
}

fn project_shapes(records: &mut Vec<Record>, binding: &NativeBinding, span: Span) -> Result<(), Diagnostic> {
    for ty in binding.parameters.iter().map(|parameter| parameter.ty).chain(std::iter::once(binding.result)) {
        let NativeType::Structured(signature) = ty else { continue; };
        prepare_signature(records,signature,&[],span)?;
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn prepare_private_view_test(records: &mut Vec<Record>, binding: &NativeBinding) -> Result<(), Diagnostic> {
    binding.validate_returned_borrow().unwrap();
    project_shapes(records, binding, Span::default())
}

pub(crate) fn resolve(records: &[Record], signature: &Signature) -> Option<Type> {
    resolve_signature(records,signature,&[])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_bindings::{NativeBackend, NativeParameter};
    use crate::native_signatures::NativeTypeId;
    #[test]
    fn instantiated_schema_signatures_keep_element_identity() {
        static ARRAY_SCHEMA:Signature=Signature::Applied(NativeTypeId::Schema,&[Signature::Array(&Signature::Parameter(0))]);
        let mut program=crate::syntax::parse("function main(){}").unwrap();
        for element in [Signature::String,Signature::Int] {
            prepare_signature(&mut program.records,&ARRAY_SCHEMA,&[element],Span::default()).unwrap();
        }
        let count=program.records.len();
        prepare_signature(&mut program.records,&ARRAY_SCHEMA,&[Signature::String],Span::default()).unwrap();
        assert_eq!(program.records.len(),count);
        let checked=crate::analysis::analyze(program).unwrap();
        let string=resolve_signature(&checked.records,&ARRAY_SCHEMA,&[Signature::String]).unwrap();
        let integer=resolve_signature(&checked.records,&ARRAY_SCHEMA,&[Signature::Int]).unwrap();
        assert_ne!(string,integer);
        for ty in [string,integer] {
            let Type::Record(id)=ty else{panic!("schema record")};
            assert_eq!(checked.records[id].native_type,Some(NativeTypeId::Schema));
        }
        let mut records=Vec::new();
        assert!(prepare_signature(&mut records,&ARRAY_SCHEMA,&[],Span::default()).is_err());
        assert!(records.is_empty());
    }

    #[test]
    fn numeric_union_signature_preserves_distinct_checked_variants() {
        static LIMIT: Signature=Signature::Union(&[Signature::Int,Signature::Number]);
        assert_eq!(LIMIT.render(|_|None).unwrap(),"(int | number)");
        assert!(!LIMIT.contains_borrowed_native().unwrap());
        let binding=NativeBinding{module:"std:schema",name:"probe",parameters:&[],result:NativeType::Structured(&LIMIT),returned_borrow:None,backend:NativeBackend::Bundled{symbol:"unused"},features:&[]};
        let mut program=crate::syntax::parse("function main(){}").unwrap();
        prepare(&mut program.records,&binding,Span::default()).unwrap();
        let checked=crate::analysis::analyze(program).unwrap();
        let Type::Union(id)=resolve(&checked.records,&LIMIT).unwrap() else{panic!("numeric alternatives collapsed")};
        assert_eq!(checked.unions[id.index()].members.len(),2);
        assert!(checked.unions[id.index()].members.contains(&Type::Int));
        assert!(checked.unions[id.index()].members.contains(&Type::Number));
        assert!(Signature::Union(&[Signature::Int,Signature::Named(NativeTypeId::ByteView)]).contains_borrowed_native().unwrap());
    }

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
    fn optional_result_view_projects_the_checked_nullable_payload() {
        static OPTIONAL: Signature = Signature::Optional(&Signature::Named(NativeTypeId::ByteView));
        static RESULT: Signature = Signature::Result(&OPTIONAL, &Signature::Named(NativeTypeId::ByteError));
        let mut binding = *crate::native_bindings::lookup("std:bytes", "view").unwrap();
        binding.result = NativeType::Structured(&RESULT);
        binding.returned_borrow.as_mut().unwrap().slot = crate::native_return_borrows::ReturnSlot::ResultOkOptional;
        let mut program = crate::syntax::parse("function main(){}").unwrap();
        prepare(&mut program.records, &binding, Span::default()).unwrap();
        let checked = crate::analysis::analyze(program).unwrap();
        let Type::Union(id) = resolve(&checked.records, &RESULT).unwrap() else { panic!("Result union"); };
        let variants = crate::result_identity::variants(&checked.records, &checked.unions[id.index()].members).unwrap();
        let Type::Union(payload) = crate::result_identity::payload(&checked.records[variants[0]]).unwrap() else { panic!("optional payload union"); };
        let members = &checked.unions[payload.index()].members;
        assert_eq!(members.len(), 2);
        assert!(members.contains(&Type::Undefined));
        assert!(members.iter().any(|ty| matches!(ty, Type::Record(id) if checked.records[*id].native_type == Some(NativeTypeId::ByteView))));
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
