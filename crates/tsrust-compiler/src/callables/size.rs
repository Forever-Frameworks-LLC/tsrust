//! Conservative by-value storage budgets after environment origins are known.
use crate::analysis::CheckedProgram;
use crate::syntax::Type;
use crate::{Diagnostic, Mode, Span};
use std::collections::{HashMap, HashSet};

// rustc rejects types larger than isize::MAX bytes; inline callables match it.
const MAX_INLINE_BYTES: usize = isize::MAX as usize;

pub(crate) fn validate(program: &CheckedProgram) -> Result<(), Diagnostic> {
    if program.stored_contracts.populated().next().is_none()
        && !program
            .records
            .iter()
            .flat_map(|record| &record.fields)
            .any(|field| {
                field.ty.is_some_and(|ty| {
                    matches!(
                        program
                            .unions
                            .get(ty.union().map(|id| id.index()).unwrap_or(usize::MAX))
                            .and_then(|union| union
                                .members
                                .iter()
                                .copied()
                                .find(|member| !matches!(member, Type::Null | Type::Undefined)))
                            .unwrap_or(ty),
                        Type::StoredCallable(_)
                    )
                })
            })
    {
        return Ok(());
    }
    let links = crate::recursive_links::Index::new(&program.records, |id| {
        program.unions[id.index()].members.clone()
    });
    let mut sizes = HashMap::new();
    let mut active = HashSet::new();
    let roots = (0..program.records.len()).map(Type::Record).chain(
        program
            .stored_contracts
            .populated()
            .map(|(id, _, _)| Type::StoredCallable(id)),
    );
    for root in roots {
        let mut pending = vec![(root, false)];
        while let Some((ty, finish)) = pending.pop() {
            if sizes.contains_key(&ty) {
                continue;
            }
            let span = match ty {
                Type::Record(id) => program.records[id].name.span,
                Type::Closure(id) => program.closures[id].span,
                _ => Span::default(),
            };
            let failure = || {
                Diagnostic::new(
                    "callable-layout",
                    "Inline callable storage exceeds Rust's maximum object size (isize::MAX bytes).",
                    span,
                )
            };
            let dependencies = dependencies(program, &links, ty, span)?;
            if !finish {
                if !active.insert(ty) {
                    return Err(Diagnostic::new(
                        "callable-layout",
                        "Recursive callable storage needs an explicit native indirection.",
                        span,
                    ));
                }
                pending.push((ty, true));
                pending.extend(dependencies.into_iter().rev().map(|child| (child, false)));
                continue;
            }
            let alignment = std::mem::align_of::<String>().max(std::mem::align_of::<i64>());
            let padded = |bytes: usize| {
                bytes
                    .checked_add(alignment - 1)
                    .map(|bytes| bytes & !(alignment - 1))
                    .ok_or_else(failure)
            };
            let size = match ty.scalar_base() {
                Type::Record(_) | Type::Closure(_) => {
                    let mut total = 0usize;
                    for child in dependencies {
                        total = total
                            .checked_add(padded(sizes[&child])?)
                            .ok_or_else(failure)?;
                    }
                    if let Type::Closure(id) = ty {
                        let references = program.closures[id]
                            .captures
                            .iter()
                            .filter(|capture| matches!(capture.storage, Mode::Read | Mode::Mutate))
                            .count();
                        // Two words also cover fat references. Stored environments
                        // normally have no borrowed captures; this is conservative.
                        total = total
                            .checked_add(
                                references
                                    .checked_mul(2 * std::mem::size_of::<usize>())
                                    .ok_or_else(failure)?,
                            )
                            .ok_or_else(failure)?;
                    }
                    total
                }
                Type::StoredCallable(_) | Type::BorrowedCallable(_) | Type::Union(_) => {
                    let mut largest = dependencies
                        .iter()
                        .map(|child| sizes[child])
                        .max()
                        .unwrap_or(0);
                    let alternatives = if let Type::Union(id) = ty {
                        let members = &program.unions[id.index()].members;
                        if members.iter().any(|member| matches!(member, Type::Record(record) if links.boxed(id, *record))) {
                            // The payload is an owned link pointer, not its inline record.
                            largest = largest.max(std::mem::size_of::<usize>());
                        }
                        members.len()
                    } else {
                        dependencies.len()
                    };
                    let tag = if alternatives > 1 { alignment } else { 0 };
                    padded(largest)?.checked_add(tag).ok_or_else(failure)?
                }
                Type::Enum(_, _) => std::mem::size_of::<u32>(),
                Type::Int => std::mem::size_of::<i64>(),
                Type::Number => std::mem::size_of::<f64>(),
                Type::Boolean => std::mem::size_of::<bool>(),
                Type::String => crate::storage::string_storage_bound(),
                Type::Array(element) => crate::storage::array_storage_bound(element),
                // Collection entries live on the heap. Only their native headers
                // contribute to the containing record or callable environment.
                Type::Map(_) | Type::Set(_) => {
                    std::mem::size_of::<Vec<Option<usize>>>()
                        + std::mem::size_of::<HashMap<u64, Vec<usize>>>()
                        + std::mem::size_of::<usize>()
                }
                Type::Callback(id) if program.absent_callback_payloads.contains(&id) => 0,
                Type::Null | Type::Undefined | Type::Void => 0,
                _ => {
                    return Err(Diagnostic::new(
                        "callable-layout",
                        "Callable capture storage requires a concrete native value type.",
                        span,
                    ));
                }
            };
            if size > MAX_INLINE_BYTES {
                return Err(failure());
            }
            active.remove(&ty);
            sizes.insert(ty, size);
        }
    }
    Ok(())
}

fn dependencies(
    program: &CheckedProgram,
    links: &crate::recursive_links::Index,
    ty: Type,
    span: Span,
) -> Result<Vec<Type>, Diagnostic> {
    Ok(match ty {
        Type::Record(id) => program.records[id]
            .fields
            .iter()
            .map(|field| field.ty.expect("checked field"))
            // Declaration-only scoped fields are generic native parameters. A
            // constructed record replaces them with concrete environment types.
            .filter(|ty| {
                let payload = ty.union().and_then(|id| program.unions[id.index()].members.iter().copied().find(|member| !matches!(member, Type::Null | Type::Undefined))).unwrap_or(*ty);
                !matches!(payload, Type::Callback(id) if !program.absent_callback_payloads.contains(&id))
            })
            .collect(),
        Type::Closure(id) => program.closures[id]
            .captures
            .iter()
            .filter(|capture| !matches!(capture.storage, Mode::Read | Mode::Mutate))
            .map(|capture| program.closures[id].function.bindings[capture.inner].ty)
            .collect(),
        Type::StoredCallable(id) | Type::BorrowedCallable(id) => program
            .stored_contracts
            .origins(id)
            .ok_or_else(|| {
                Diagnostic::new(
                    "callable-layout",
                    "Unknown callable storage signature.",
                    span,
                )
            })?
            .iter()
            .copied()
            .map(Type::Closure)
            .collect(),
        Type::Union(id) => program.unions[id.index()].members.iter().copied().filter(|member| {
            !matches!(member, Type::Record(record) if links.boxed(id, *record))
        }).collect(),
        _ => Vec::new(),
    })
}

#[cfg(test)]
#[test]
fn duplicated_capture_layout_rejects_large_inline_storage_but_arrays_are_bounded() {
    let mut checked = crate::check("type State={value:int;};function make(state:State):()=>int{return ():int=>{return state.value;};}function sink(action:Function<int>):int{return action();}function other(action:Once<Function<int>>):Once<Function<int>>{return action;}function main(){}").unwrap();
    let program = &mut checked.program;
    let Type::Closure(origin) = program.functions[0].result else {
        panic!("factory origin");
    };
    program
        .stored_contracts
        .register_origin(
            0,
            origin,
            &program.callback_contracts,
            &program.callable_shapes,
            &program.closures,
            Span::default(),
        )
        .unwrap();
    validate(program).unwrap();
    program
        .records
        .push(crate::cloneability::tests::record(&[Type::StoredCallable(
            1,
        )]));
    crate::callable_layout::validate_storage_layout(program).unwrap();
    validate(program).unwrap();
    // Doubling past isize::MAX bytes exceeds Rust's maximum object size.
    program.records = (0..62)
        .map(|id| {
            crate::cloneability::tests::record(&[
                if id == 61 {
                    Type::Int
                } else {
                    Type::Record(id + 1)
                },
                if id == 61 {
                    Type::Int
                } else {
                    Type::Record(id + 1)
                },
            ])
        })
        .collect();
    assert_eq!(validate(program).unwrap_err().code, "callable-layout");
    program.records.truncate(1);
    program.records[0].fields.truncate(1);
    program.records[0].fields[0].ty = Some(Type::Array(crate::syntax::ArrayElement::Record(0, 0)));
    validate(program).unwrap();
}
