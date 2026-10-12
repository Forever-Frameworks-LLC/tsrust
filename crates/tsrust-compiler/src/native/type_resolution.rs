//! Intern native declaration identities in the ordinary nominal record arena.
//! Native storage remains explicit metadata, not a source-visible implementation field.
use crate::{Diagnostic, Span, native_signatures::{NativeTypeId, Signature}, native_types,
    syntax::{Record, RecordField, Token, TokenKind},
    type_syntax::{TypeName, ValueAnnotation, ValueAnnotationKind as Kind}};

pub fn linked_name(id: NativeTypeId) -> String {
    let item = native_types::declaration(id);
    format!("{}native#{}#{}", crate::type_aliases::LINKED_PREFIX, item.module, item.name)
}

/// Repeated imports and aliases share one arena identity. A failed projection
/// removes only declarations created by this operation.
pub fn intern(records: &mut Vec<Record>, id: NativeTypeId, span: Span) -> Result<usize, Diagnostic> {
    let start = records.len();
    let result = Resolver { records, nodes: 0, parameters: &[] }.declaration(id, span, 0);
    if result.is_err() { records.truncate(start); }
    result
}

/// Project recursive signature metadata into structured annotations, interning
/// referenced native declarations in the same arena. Failure is transactional.
pub fn project(records: &mut Vec<Record>, signature: &Signature, span: Span) -> Result<ValueAnnotation, Diagnostic> {
    project_with_parameters(records, signature, &[], span)
}

pub fn project_with_parameters(records: &mut Vec<Record>, signature: &Signature, parameters: &[Signature], span: Span) -> Result<ValueAnnotation, Diagnostic> {
    let start = records.len();
    let result = Resolver { records, nodes: 0, parameters }.signature(signature, span, 0);
    if result.is_err() { records.truncate(start); }
    result
}

struct Resolver<'a> { records: &'a mut Vec<Record>, nodes: usize, parameters: &'a [Signature] }
impl Resolver<'_> {
    fn bound(&mut self, span: Span, depth: usize) -> Result<(), Diagnostic> {
        self.nodes += 1;
        if depth > 128 || self.nodes > 4096 {
            return Err(Diagnostic::new("native-type-limit", "Native type projection exceeds its depth or size limit.", span));
        }
        Ok(())
    }
    fn declaration(&mut self, id: NativeTypeId, span: Span, depth: usize) -> Result<usize, Diagnostic> {
        self.bound(span, depth)?;
        if let Some(index) = self.records.iter().position(|record| record.native_type == Some(id)) { return Ok(index); }
        let item = native_types::declaration(id);
        let index = self.records.len();
        self.records.push(Record {
            native_type: Some(id), name: Token { text: linked_name(id), kind: TokenKind::Symbol, span },
            application: None, type_parameters: item.type_parameters.iter().map(|name| TypeName { text: (*name).into(), span }).collect(), fields: Vec::new(),
            map_definition: None, set_definition: None, enum_definition: None,
        });
        let mut fields = Vec::new();
        for field in item.fields {
            let mut annotation = self.signature(&field.signature, span, depth + 1)?;
            if field.optional {
                annotation = ValueAnnotation { kind: Kind::Union(vec![annotation, named("undefined", span)]), span };
            }
            fields.push(RecordField {
                name: Token { text: field.name.into(), kind: TokenKind::Name, span },
                annotation, optional: field.optional, ty: None, callable_declaration: None,
            });
        }
        self.records[index].fields = fields;
        Ok(index)
    }
    fn signature(&mut self, signature: &Signature, span: Span, depth: usize) -> Result<ValueAnnotation, Diagnostic> {
        self.bound(span, depth)?;
        let kind = match signature {
            Signature::Parameter(index) => {
                let argument = *self.parameters.get(*index).ok_or_else(|| Diagnostic::new("native-type-argument", format!("Missing native type argument at index {index}."), span))?;
                return self.signature(&argument, span, depth + 1);
            }
            Signature::String => return Ok(named("string", span)),
            Signature::Boolean => return Ok(named("boolean", span)),
            Signature::Int => return Ok(named("int", span)),
            Signature::Number => return Ok(named("number", span)),
            Signature::Void => return Ok(named("void", span)),
            Signature::Named(id) => {
                if !native_types::declaration(*id).type_parameters.is_empty() {
                    return Err(Diagnostic::new("native-type-arity", "Generic native types require explicit type arguments.", span));
                }
                self.declaration(*id, span, depth + 1)?; return Ok(named(&linked_name(*id), span));
            }
            Signature::Applied(id, arguments) => {
                if native_types::declaration(*id).type_parameters.len() != arguments.len() {
                    return Err(Diagnostic::new("native-type-arity", "Native type argument count does not match its declaration.", span));
                }
                self.declaration(*id, span, depth + 1)?;
                Kind::Applied {
                    name: TypeName { text: linked_name(*id), span },
                    arguments: arguments.iter().map(|argument| self.signature(argument, span, depth + 1)).collect::<Result<_, _>>()?,
                }
            }
            Signature::Array(element) => Kind::Array(Box::new(self.signature(element, span, depth + 1)?)),
            Signature::Optional(element) => Kind::Union(vec![self.signature(element, span, depth + 1)?, named("undefined", span)]),
            Signature::Result(value, error) => Kind::Applied {
                name: TypeName { text: "Result".into(), span },
                arguments: vec![self.signature(value, span, depth + 1)?, self.signature(error, span, depth + 1)?],
            },
            Signature::Intrinsic(_) => return Err(Diagnostic::new("native-type", "Private callable intrinsics cannot be native record fields.", span)),
        };
        Ok(ValueAnnotation { kind, span })
    }
}
fn named(text: &str, span: Span) -> ValueAnnotation {
    ValueAnnotation { kind: Kind::Named(TypeName { text: text.into(), span }), span }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordered_collection_metadata_validates_keys_and_retains_payload_identity() {
        for (identity, arguments) in [
            (NativeTypeId::OrderedMap, vec![Signature::String, Signature::Named(NativeTypeId::ByteView)]),
            (NativeTypeId::OrderedSet, vec![Signature::Number]),
            (NativeTypeId::Heap, vec![Signature::Int]),
        ] {
            let mut program=crate::syntax::parse("function main(){}").unwrap();
            let annotation=project_with_parameters(&mut program.records, &Signature::Applied(identity, if identity==NativeTypeId::OrderedMap { &[Signature::Parameter(0),Signature::Parameter(1)] } else { &[Signature::Parameter(0)] }), &arguments, Span::default()).unwrap();
            program.type_argument_checks.push(annotation);
            crate::generic_records::specialize_in(&mut program,&crate::unions::UnionArena::default(),&mut crate::callable_resolution::Tables::default()).unwrap();
            let instance=program.records.iter().find(|record|record.native_type==Some(identity)).unwrap();
            assert!(instance.fields.is_empty());
            let resolved=&instance.application.as_ref().unwrap().resolved_arguments;
            assert_eq!(resolved.len(),arguments.len());
            match identity {
                NativeTypeId::OrderedMap => {
                    assert_eq!(resolved[0],crate::syntax::Type::String);
                    let crate::syntax::Type::Record(value)=resolved[1] else {panic!("native value identity lost")};
                    assert_eq!(program.records[value].native_type,Some(NativeTypeId::ByteView));
                    let map=instance.map_definition.as_ref().expect("ordered map uses shared map analysis");
                    assert_eq!(map.key,resolved[0]);assert_eq!(map.value,resolved[1]);
                    let arena=crate::unions::UnionArena::from_snapshot_with_records(Vec::new(),&program.records);
                    let index=program.records.iter().position(|record|std::ptr::eq(record,instance)).unwrap();
                    assert_eq!(arena.named_type(index),crate::syntax::Type::Map(index));
                }
                NativeTypeId::OrderedSet => assert_eq!(resolved[0],crate::syntax::Type::Number),
                NativeTypeId::Heap => assert_eq!(resolved[0],crate::syntax::Type::Int),
                _ => unreachable!(),
            }
            let before=program.records.len();
            assert_eq!(project(&mut program.records,&Signature::Applied(identity,&[]),Span::default()).unwrap_err().code,"native-type-arity");
            assert_eq!(program.records.len(),before);
            assert!(crate::native_exports::lookup("std:collections",native_types::declaration(identity).name).is_ok());
            for key in [Signature::Array(&Signature::Int),Signature::Named(NativeTypeId::Bytes),Signature::Optional(&Signature::Int)] {
                let mut program=crate::syntax::parse("function main(){}").unwrap();
                let parameters=if identity==NativeTypeId::OrderedMap {vec![key,Signature::Int]}else{vec![key]};
                let signature=Signature::Applied(identity, if identity==NativeTypeId::OrderedMap { &[Signature::Parameter(0),Signature::Parameter(1)] } else { &[Signature::Parameter(0)] });
                let annotation=project_with_parameters(&mut program.records,&signature,&parameters,Span::default()).unwrap();
                program.type_argument_checks.push(annotation);
                let error=crate::generic_records::specialize_in(&mut program,&crate::unions::UnionArena::default(),&mut crate::callable_resolution::Tables::default()).err().unwrap();
                assert_eq!(error.code,"ordered-key");
            }
        }
    }

    #[test]
    fn deque_specializations_keep_element_identity_without_exposing_fields() {
        let span = Span::default();
        let mut program = crate::syntax::parse("function main(){}").unwrap();
        intern(&mut program.records, NativeTypeId::Deque, span).unwrap();
        let signature = Signature::Applied(NativeTypeId::Deque, &[Signature::Parameter(0)]);
        for argument in [Signature::Int, Signature::String, Signature::Int] {
            let annotation = project_with_parameters(&mut program.records, &signature, &[argument], span).unwrap();
            program.type_argument_checks.push(annotation);
        }
        let count = program.records.len();
        let wrong_arity = project(&mut program.records, &Signature::Applied(NativeTypeId::Deque, &[]), span).unwrap_err();
        assert_eq!(wrong_arity.code, "native-type-arity");
        assert_eq!(program.records.len(), count);
        let unions = crate::unions::UnionArena::default();
        crate::generic_records::specialize_in(&mut program, &unions, &mut crate::callable_resolution::Tables::default()).unwrap();
        let instances: Vec<_> = program.records.iter().filter(|record| record.native_type == Some(NativeTypeId::Deque)).collect();
        assert_eq!(instances.len(), 2);
        assert!(instances.iter().all(|record| record.fields.is_empty()));
        assert_eq!(instances[0].application.as_ref().unwrap().resolved_arguments, vec![crate::syntax::Type::Int]);
        assert_eq!(instances[1].application.as_ref().unwrap().resolved_arguments, vec![crate::syntax::Type::String]);
        assert_ne!(instances[0].name.text, instances[1].name.text);
        assert!(matches!(crate::native_exports::lookup("std:collections", "Deque"), Ok(crate::native_exports::Export::Type(_))));
    }

    #[test]
    fn generic_projection_preserves_native_identity_and_rolls_back_missing_arguments() {
        let span = Span::default();
        let mut records = Vec::new();
        let original = intern(&mut records, NativeTypeId::Bytes, span).unwrap();
        let signature = Signature::Result(&Signature::Parameter(0), &Signature::Parameter(1));
        let arguments = [Signature::Named(NativeTypeId::ByteView), Signature::Int];
        let projected = project_with_parameters(&mut records, &signature, &arguments, span).unwrap();
        let Kind::Applied { name, arguments } = projected.kind else { panic!("expected Result"); };
        assert_eq!(name.text, "Result");
        let Kind::Named(value) = &arguments[0].kind else { panic!("expected view identity"); };
        assert_eq!(value.text, linked_name(NativeTypeId::ByteView));
        assert_eq!(records[original].native_type, Some(NativeTypeId::Bytes));
        let count = records.len();
        let failed = project_with_parameters(&mut records, &signature, &[Signature::Named(NativeTypeId::File)], span).unwrap_err();
        assert_eq!(failed.code, "native-type-argument");
        assert_eq!(records.len(), count);
        let optional = project_with_parameters(&mut records, &Signature::Optional(&Signature::Parameter(0)), &[Signature::Named(NativeTypeId::ByteView)], span).unwrap();
        let Kind::Union(alternatives) = optional.kind else { panic!("expected optional union"); };
        assert_eq!(alternatives.len(), 2);
        let Kind::Named(absent) = &alternatives[1].kind else { panic!("expected undefined"); };
        assert_eq!(absent.text, "undefined");
        assert_eq!(records.len(), count);
        assert!(!records.iter().any(|record| record.native_type == Some(NativeTypeId::File)));
        let cyclic = project_with_parameters(&mut records, &Signature::Parameter(0), &[Signature::Parameter(0)], span).unwrap_err();
        assert_eq!(cyclic.code, "native-type-limit");
        assert_eq!(records.len(), count);
    }
}
