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
    let result = Resolver { records, nodes: 0 }.declaration(id, span, 0);
    if result.is_err() { records.truncate(start); }
    result
}

/// Project recursive signature metadata into structured annotations, interning
/// referenced native declarations in the same arena. Failure is transactional.
pub fn project(records: &mut Vec<Record>, signature: &Signature, span: Span) -> Result<ValueAnnotation, Diagnostic> {
    let start = records.len();
    let result = Resolver { records, nodes: 0 }.signature(signature, span, 0);
    if result.is_err() { records.truncate(start); }
    result
}

struct Resolver<'a> { records: &'a mut Vec<Record>, nodes: usize }
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
            application: None, type_parameters: Vec::new(), fields: Vec::new(),
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
            Signature::String => return Ok(named("string", span)),
            Signature::Boolean => return Ok(named("boolean", span)),
            Signature::Int => return Ok(named("int", span)),
            Signature::Number => return Ok(named("number", span)),
            Signature::Void => return Ok(named("void", span)),
            Signature::Named(id) => { self.declaration(*id, span, depth + 1)?; return Ok(named(&linked_name(*id), span)); }
            Signature::Array(element) => Kind::Array(Box::new(self.signature(element, span, depth + 1)?)),
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
