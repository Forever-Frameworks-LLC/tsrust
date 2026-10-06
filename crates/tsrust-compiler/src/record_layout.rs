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
        if let Some((ty, declaration)) = callables
            .resolver(record_names, unions)
            .storage_declaration(annotation)?
        {
            return Ok((ty, Some(declaration)));
        }
        callables
            .resolver(record_names, unions)
            .resolve_value_annotation(annotation, false)
            .map(|ty| (ty, None))
    })?;
    validate_record_layouts(records, unions)?;
    Ok(())
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

fn validate_record_layouts(
    records: &[Record],
    unions: &crate::unions::UnionArena,
) -> Result<(), Diagnostic> {
    validate_inline_layout(records, unions)
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

fn validate_inline_layout(
    records: &[Record],
    unions: &crate::unions::UnionArena,
) -> Result<(), Diagnostic> {
    let alternatives = |ty: Type| if let Type::Union(id) = ty { unions.members(id) } else { vec![ty] };
    let edges = records.iter().map(|record| record.fields.iter().flat_map(|field| {
        alternatives(field.ty.expect("resolved record field")).into_iter().filter_map(|ty| {
            if let Type::Record(child) = ty { Some((child,field.annotation.span)) } else { None }
        }).collect::<Vec<_>>()
    }).collect::<Vec<_>>()).collect::<Vec<_>>();
    let mut states = vec![0u8; records.len()];
    let mut depths = vec![0usize; records.len()];
    let mut sizes = vec![0usize; records.len()];
    let alignment = std::mem::align_of::<String>().max(std::mem::align_of::<i64>());
    const MAX_INLINE_BYTES: usize = 1 << 30;
    for root in 0..records.len() {
        if states[root] != 0 {
            continue;
        }
        states[root] = 1;
        let mut stack = vec![(root, 0)];
        while let Some(&(id, next)) = stack.last() {
            if let Some(&(child, span)) = edges[id].get(next) {
                stack.last_mut().expect("layout frame").1 += 1;
                match states[child] {
                    0 => { states[child] = 1; stack.push((child, 0)); }
                    1 => return Err(Diagnostic::new("recursive-layout", "Direct record fields form an infinite layout. Use array indirection to break the cycle.", span).related(records[child].name.span, "The layout returns to this record type.")),
                    _ => (),
                }
                continue;
            }
            let mut depth = 1;
            let mut size = 0usize;
            for field in &records[id].fields {
                let field_type = field.ty.expect("resolved record field");
                let tag = if matches!(field_type, Type::Union(_)) {
                    alignment
                } else {
                    0
                };
                let payload_bytes = alternatives(field_type).into_iter().map(|ty| match ty {
                    Type::Int => std::mem::size_of::<i64>(),
                    Type::Number => std::mem::size_of::<f64>(),
                    Type::Boolean => std::mem::size_of::<bool>(),
                    Type::Enum(enumeration, _) => if records[enumeration].enum_definition.as_ref().expect("enum definition").members.is_empty() { 0 } else { std::mem::size_of::<u32>() },
                    Type::Null | Type::Undefined => 0,
                    Type::String => crate::storage::string_storage_bound(),
                    Type::Set(_) | Type::Map(_) => std::mem::size_of::<Vec<()>>() + std::mem::size_of::<std::collections::HashMap<u64, Vec<usize>>>() + std::mem::size_of::<usize>(),
                    Type::Array(element) => crate::storage::array_storage_bound(element),
                    Type::Record(child) => { depth = depth.max(depths[child] + 1); sizes[child] }
                    Type::StoredCallable(_) => 0,
                    _ => unreachable!("record field type"),
                }).max().unwrap_or(0);
                let bytes = tag + payload_bytes;
                // Rounding every field to the largest primitive alignment is a
                // conservative upper bound independent of native field ordering.
                let padded = (bytes + alignment - 1) & !(alignment - 1);
                size = size
                    .checked_add(padded)
                    .filter(|&n| n <= MAX_INLINE_BYTES)
                    .ok_or_else(|| {
                        Diagnostic::new(
                            "unsupported",
                            "Inline record storage exceeds the prototype limit of 1 GiB.",
                            field.annotation.span,
                        )
                    })?;
                if depth > 128 {
                    return Err(Diagnostic::new(
                        "unsupported",
                        "Direct record layout nesting exceeds the prototype limit of 128.",
                        field.annotation.span,
                    ));
                }
            }
            depths[id] = depth;
            sizes[id] = size;
            states[id] = 2;
            stack.pop();
        }
    }
    Ok(())
}
