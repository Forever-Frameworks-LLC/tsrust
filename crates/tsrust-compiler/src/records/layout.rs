//! Record field resolution and bounded native inline layout validation.
use crate::Diagnostic;
use crate::syntax::{Record, Type};
use std::collections::HashMap;

/// Only direct record fields contribute edges to an inline layout cycle.
/// Iterative traversal keeps hostile type graphs off the compiler call stack.
pub(crate) fn append_checked_record_fields(
    records: &mut [Record],
    record_names: &HashMap<String, usize>,
    unions: &crate::unions::UnionArena,
    fields: &mut Vec<HashMap<String, Type>>,
    callables: &mut crate::callable_resolution::Tables,
) -> Result<(), Diagnostic> {
    resolve_record_fields(records, record_names, fields, |annotation| {
        match crate::scoped_record_fields::resolve(
            &mut callables.resolver(record_names, unions), annotation,
        )? {
            Some(crate::scoped_record_fields::FieldContract::Owned { ty, declaration }) => {
                return Ok((ty, Some(declaration)));
            }
            Some(crate::scoped_record_fields::FieldContract::Scoped { ty, declaration }) => {
                return Ok((ty, Some(declaration)));
            }
            None => {}
        }
        callables
            .resolver(record_names, unions)
            .resolve_value_annotation(annotation, false)
            .map(|ty| (ty, None))
    })?;
    let mut validated = std::mem::take(&mut callables.layouts);
    let result = validated.extend(records, unions);
    callables.layouts = validated;
    result
}

/// Inline layout facts for a validated prefix of the record table.
///
/// A resolved record only names records that existed when it was resolved, so
/// appending records never changes the layout of earlier ones. Validation
/// therefore covers only new records, reusing the sizes of earlier ones. Binding
/// a reserved recursive union can add edges to earlier records, so any change to
/// the members of an existing union restarts validation from the first record. A failed validation keeps the old prefix
/// and reports the same error again on the next append.
#[derive(Default)]
pub(crate) struct Validated {
    records: usize,
    bindings: u64,
    depths: Vec<usize>,
    sizes: Vec<usize>,
}
impl Validated {
    fn extend(&mut self, records: &[Record], unions: &crate::unions::UnionArena) -> Result<(), Diagnostic> {
        let bindings = unions.member_rewrites();
        if bindings != self.bindings || self.records > records.len() {
            *self = Self { bindings, ..Self::default() };
        }
        if self.records == records.len() {
            return Ok(());
        }
        let links = crate::recursive_links::Index::new_from(records, self.records, |id| unions.members(id));
        let (depths, sizes) = inline_layout(records, self.records, &self.depths, &self.sizes, unions, &|id, child| links.boxed(id, child))?;
        self.depths = depths;
        self.sizes = sizes;
        self.records = records.len();
        Ok(())
    }
}

/// Resolve newly materialized declarations with the caller's shared type tables.
/// Native layout acceptance remains separate from source contract resolution.
pub(crate) fn resolve_record_fields(
    records: &mut [Record],
    record_names: &HashMap<String, usize>,
    fields: &mut Vec<HashMap<String, Type>>,
    mut resolve: impl FnMut(
        &crate::type_syntax::ValueAnnotation,
    ) -> Result<(Type, Option<usize>), Diagnostic>,
) -> Result<(), Diagnostic> {
    for record in records.iter_mut().skip(fields.len()) {
        let mut names = HashMap::new();
        for field in &mut record.fields {
            let mut annotation = &field.annotation;
            while let crate::type_syntax::ValueAnnotationKind::Group(inner) = &annotation.kind {
                annotation = inner;
            }
            if matches!(&annotation.kind, crate::type_syntax::ValueAnnotationKind::Named(name)
                if !matches!(name.text.as_str(), "string" | "int" | "number" | "boolean" | "null" | "undefined")
                    && !record_names.contains_key(&name.text))
            {
                return Err(Diagnostic::new(
                    "unsupported",
                    "Record fields support string, int, boolean, named record, or array types.",
                    field.annotation.span,
                ));
            }
            if names.contains_key(&field.name.text) {
                return Err(Diagnostic::new(
                    "duplicate",
                    format!("Field {:?} is already defined.", field.name.text),
                    field.name.span,
                ));
            }
            let (ty, declaration) = resolve(&field.annotation)?;
            field.ty = Some(ty);
            field.callable_declaration = declaration;
            names.insert(field.name.text.clone(), ty);
        }
        fields.push(names);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_field_resolution_reuses_shared_families_after_late_materialization() {
        let names = HashMap::from([("Item".into(), 0)]);
        let unions = crate::unions::UnionArena::default();
        let mut tables = crate::callable_resolution::Tables::default();
        let mut fields = Vec::new();
        let mut records = Vec::new();
        for source in [
            "Function<int, [item: ReadOnly<Item>]>",
            "((Function<int, [other: ReadOnly<Item>]>))",
            "Once<Function<int>>",
        ] {
            let mut record = crate::cloneability::tests::record(&[Type::Int]);
            record.fields[0].annotation = crate::type_syntax::parse(source).unwrap();
            records.push(record);
            resolve_record_fields(&mut records, &names, &mut fields, |annotation| {
                tables
                    .resolver(&names, &unions)
                    .storage_declaration(annotation)?
                    .map(|(ty, id)| (ty, Some(id)))
                    .ok_or_else(|| {
                        Diagnostic::new("type", "Expected a callable field.", annotation.span)
                    })
            })
            .unwrap();
        }
        assert_eq!(records[0].fields[0].ty, Some(Type::StoredCallable(0)));
        assert_eq!(records[1].fields[0].ty, Some(Type::StoredCallable(0)));
        assert_eq!(records[2].fields[0].ty, Some(Type::StoredCallable(1)));
        assert_eq!(fields.len(), 3);
        for (index, record) in records.iter().enumerate() {
            assert_eq!(record.fields[0].callable_declaration, Some(index));
        }
        let labels = tables
            .callbacks
            .iter()
            .map(|contract| contract.parameter_names.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            labels,
            [
                vec!["item".to_owned()],
                vec!["other".to_owned()],
                Vec::new()
            ]
        );
        assert!(tables.storage.populated().next().is_none());
    }

    #[test]
    fn borrowed_field_failure_does_not_publish_a_record_field_table() {
        let names = HashMap::new();
        let unions = crate::unions::UnionArena::default();
        let mut tables = crate::callable_resolution::Tables::default();
        let mut record = crate::cloneability::tests::record(&[Type::Int]);
        record.fields[0].annotation = crate::type_syntax::parse("((Mutable<Function<int>>))").unwrap();
        let span = record.fields[0].annotation.span;
        let mut fields = Vec::new();
        let error = resolve_record_fields(&mut [record], &names, &mut fields, |annotation| {
            tables
                .resolver(&names, &unions)
                .storage_declaration(annotation)
                .map(|ty| {
                    let (ty, id) = ty.unwrap();
                    (ty, Some(id))
                })
        })
        .unwrap_err();
        assert_eq!(error.code, "callback-escape");
        assert_eq!(error.span, span);
        assert!(fields.is_empty());
        assert!(tables.callbacks.is_empty());
    }

    #[test]
    fn duplicate_fields_reject_before_resolving_the_duplicate_contract() {
        let names = HashMap::from([("Schema".into(), 0)]);
        let mut record = crate::cloneability::tests::record(&[Type::Int, Type::Int]);
        record.fields[1].name = record.fields[0].name.clone();
        let span = record.fields[1].name.span;
        let mut fields = Vec::new();
        let mut resolved = 0;
        let error = resolve_record_fields(&mut [record], &names, &mut fields, |_| {
            resolved += 1;
            Ok((Type::Int, None))
        })
        .unwrap_err();
        assert_eq!(error.code, "duplicate");
        assert_eq!(error.span, span);
        assert_eq!(resolved, 1);
        assert!(fields.is_empty());
    }
}

#[cfg(test)]
fn validate_inline_layout(
    records: &[Record],
    unions: &crate::unions::UnionArena,
) -> Result<(), Diagnostic> {
    Validated::default().extend(records, unions)
}

#[cfg(test)]
fn validate_inline_layout_with_indirection(
    records: &[Record],
    unions: &crate::unions::UnionArena,
    indirect: &impl Fn(crate::syntax::UnionId, usize) -> bool,
) -> Result<(), Diagnostic> {
    inline_layout(records, 0, &[], &[], unions, indirect).map(drop)
}

/// Validate the inline layout of `records[from..]`, given the depths and sizes of
/// the already validated `records[..from]`. Returns depths and sizes of all.
fn inline_layout(
    records: &[Record],
    from: usize,
    known_depths: &[usize],
    known_sizes: &[usize],
    unions: &crate::unions::UnionArena,
    indirect: &impl Fn(crate::syntax::UnionId, usize) -> bool,
) -> Result<(Vec<usize>, Vec<usize>), Diagnostic> {
    let alternatives = |ty: Type| crate::recursive_links::inline_payloads(ty,|id|unions.members(id));
    let edges = records[from..].iter().map(|record| record.fields.iter().flat_map(|field| {
        alternatives(field.ty.expect("resolved record field")).into_iter().filter_map(|payload| {
            if let Type::Record(child) = payload.ty {
                if payload.union.is_some_and(|id| indirect(id, child)) { None }
                else { Some((child,field.annotation.span)) }
            } else { None }
        }).collect::<Vec<_>>()
    }).collect::<Vec<_>>()).collect::<Vec<_>>();
    let mut states = vec![2u8; from];
    states.resize(records.len(), 0);
    let mut depths = known_depths.to_vec();
    depths.resize(records.len(), 0);
    let mut sizes = known_sizes.to_vec();
    sizes.resize(records.len(), 0);
    let alignment = std::mem::align_of::<String>().max(std::mem::align_of::<i64>());
    // rustc rejects types larger than isize::MAX bytes; inline records match it.
    const MAX_INLINE_BYTES: usize = isize::MAX as usize;
    for root in from..records.len() {
        if states[root] != 0 {
            continue;
        }
        states[root] = 1;
        let mut stack = vec![(root, 0)];
        while let Some(&(id, next)) = stack.last() {
            if let Some(&(child, span)) = edges[id - from].get(next) {
                stack.last_mut().expect("layout frame").1 += 1;
                match states[child] {
                    0 => { states[child] = 1; stack.push((child, 0)); }
                    1 => return Err(Diagnostic::new("recursive-layout", "Direct record fields form an infinite layout. Use a nullable link or array indirection to break the cycle.", span).related(records[child].name.span, "The layout returns to this record type.")),
                    _ => (),
                }
                continue;
            }
            let mut depth = 1;
            let mut size = 0usize;
            for field in &records[id].fields {
                let field_type = field.ty.expect("resolved record field");
                let bytes = alternatives(field_type).into_iter().map(|payload| payload.tags * alignment + match payload.ty.scalar_base() {
                    Type::Int => std::mem::size_of::<i64>(),
                    Type::Number => std::mem::size_of::<f64>(),
                    Type::Boolean => std::mem::size_of::<bool>(),
                    Type::Enum(enumeration, _) => if records[enumeration].enum_definition.as_ref().expect("enum definition").members.is_empty() { 0 } else { std::mem::size_of::<u32>() },
                    Type::Null | Type::Undefined => 0,
                    Type::String => crate::storage::string_storage_bound(),
                    Type::Set(_) | Type::Map(_) => std::mem::size_of::<Vec<()>>() + std::mem::size_of::<std::collections::HashMap<u64, Vec<usize>>>() + std::mem::size_of::<usize>(),
                    Type::Array(element) => crate::storage::array_storage_bound(element),
                    Type::Record(child) if payload.union.is_some_and(|id| indirect(id, child)) => std::mem::size_of::<usize>(),
                    Type::Record(child) => { depth = depth.max(depths[child] + 1); sizes[child] }
                    Type::StoredCallable(_) | Type::BorrowedCallable(_) | Type::Callback(_) | Type::Closure(_) => 0,
                    _ => unreachable!("record field type"),
                }).max().unwrap_or(0);
                // Rounding every field to the largest primitive alignment is a
                // conservative upper bound independent of native field ordering.
                let padded = (bytes + alignment - 1) & !(alignment - 1);
                size = size
                    .checked_add(padded)
                    .filter(|&n| n <= MAX_INLINE_BYTES)
                    .ok_or_else(|| {
                        Diagnostic::new(
                            "unsupported",
                            "Inline record storage exceeds Rust's maximum object size (isize::MAX bytes).",
                            field.annotation.span,
                        )
                    })?;
            }
            depths[id] = depth;
            sizes[id] = size;
            states[id] = 2;
            stack.pop();
        }
    }
    Ok((depths, sizes))
}


#[cfg(test)]
#[test]
fn nullable_link_layout_terminates_only_at_planned_indirection() {
    use crate::{cloneability::tests::record, syntax::ArrayElement, Span};
    let unions=crate::unions::UnionArena::default();
    let next=unions.intern([Type::Record(0),Type::Undefined],Span::new(0,0)).unwrap();
    for records in [
        vec![record(&[next,Type::String])],
        vec![record(&[Type::Record(1)]),record(&[next])],
        vec![record(&[Type::Array(ArrayElement::Record(0,0)),next])],
    ] {
        let links=crate::recursive_links::Index::new(&records,|id|unions.members(id));
        validate_inline_layout_with_indirection(&records,&unions,&|id,child|links.boxed(id,child)).unwrap();
        validate_inline_layout(&records,&unions).unwrap();
    }
    let records=[record(&[Type::Record(0),next])];
    let links=crate::recursive_links::Index::new(&records,|id|unions.members(id));
    assert_eq!(validate_inline_layout_with_indirection(&records,&unions,&|id,child|links.boxed(id,child)).unwrap_err().code,"recursive-layout");
    let records=[record(&[Type::Int]),record(&[next])];
    let links=crate::recursive_links::Index::new(&records,|id|unions.members(id));
    assert!(!links.boxed(next.union().unwrap(),0));
    validate_inline_layout_with_indirection(&records,&unions,&|id,child|links.boxed(id,child)).unwrap();
}

#[cfg(test)]
#[test]
fn nested_recursive_union_layout_keeps_nullable_indirection() {
    use crate::{cloneability::tests::record,syntax::ArrayElement,Span};
    let unions=crate::unions::UnionArena::default();
    let span=Span::new(0,1);
    let knot=unions.reserve_recursive(span).unwrap();
    unions.bind_recursive(knot.union().unwrap(),vec![Type::Record(0)],span).unwrap();
    let nullable=unions.intern([knot,Type::Null],span).unwrap();
    let records=[record(&[nullable])];
    let links=crate::recursive_links::Index::new(&records,|id|unions.members(id));
    assert!(links.boxed(knot.union().unwrap(),0));
    validate_inline_layout(&records,&unions).unwrap();
    assert_eq!(validate_inline_layout(&[record(&[knot])],&unions).unwrap_err().code,"recursive-layout");
    let arrays=[record(&[Type::Array(ArrayElement::Union(nullable.union().unwrap(),0))])];
    let links=crate::recursive_links::Index::new(&arrays,|id|unions.members(id));
    assert!(!links.boxed(knot.union().unwrap(),0));
    validate_inline_layout(&arrays,&unions).unwrap();
}

#[cfg(test)]
#[test]
fn appended_layout_validation_reuses_the_prefix_and_restarts_after_recursive_binding() {
    use crate::{cloneability::tests::record, Span};
    let unions = crate::unions::UnionArena::default();
    let mut validated = Validated::default();
    let mut records = vec![record(&[Type::Int])];
    validated.extend(&records, &unions).unwrap();
    // A new record pair forms a direct cycle; the failure keeps the old prefix
    // and is reported again by the next append.
    records.push(record(&[Type::Record(2)]));
    records.push(record(&[Type::Record(1)]));
    assert_eq!(validated.extend(&records, &unions).unwrap_err().code, "recursive-layout");
    assert_eq!(validated.records, 1);
    records.push(record(&[Type::Record(0)]));
    assert_eq!(validated.extend(&records, &unions).unwrap_err().code, "recursive-layout");
    // A record that names a reserved slot is valid until the slot binds back to it.
    let mut records = vec![record(&[Type::Int])];
    let knot = unions.reserve_recursive(Span::new(0, 1)).unwrap();
    records.push(record(&[knot]));
    let mut validated = Validated::default();
    validated.extend(&records, &unions).unwrap();
    unions.bind_recursive(knot.union().unwrap(), vec![Type::Record(1)], Span::new(0, 1)).unwrap();
    records.push(record(&[Type::Int]));
    assert_eq!(validated.extend(&records, &unions).unwrap_err().code, "recursive-layout");
}
