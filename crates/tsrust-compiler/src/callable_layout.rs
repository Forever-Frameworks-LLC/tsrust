//! Native capture fields, isolated from callable dispatch and expression emission.
use crate::Mode;
use crate::analysis::{CheckedClosure, CheckedProgram};
use crate::syntax::Type;

/// Empty storage families still need a native type when concrete declarations use them.
pub(crate) fn declared_storage_families(
    program: &CheckedProgram,
    generic_field: impl Fn(usize,usize)->bool,
) -> std::collections::HashSet<usize> {
    let family = |ty| match ty {
        Type::StoredCallable(id)
        | Type::BorrowedCallable(id)
        | Type::Array(crate::syntax::ArrayElement::StoredCallable(id, _)) => Some(id),
        _ => None,
    };
    let mut families=std::collections::HashSet::new();
    let mut pending=Vec::new();
    for (record,info) in program.records.iter().enumerate() {
        for (position,field) in info.fields.iter().enumerate() {
            if !generic_field(record,position) && let Some(ty)=field.ty {pending.push(ty);}
        }
    }
    let functions = program
        .functions
        .iter()
        .chain(program.closures.iter().map(|closure| &closure.function))
        .chain(
            program
                .closures
                .iter()
                .flat_map(|closure| &closure.methods)
                .map(|method| &method.function),
        );
    for function in functions.filter(|function| !function.template) {
        pending.extend(function.bindings.iter().flat_map(|binding|std::iter::once(binding.ty).chain(binding.input_ty)));
        pending.push(function.result);
    }
    // Symbolic unions at generic record fields do not require a concrete
    // owning family. Follow only unions reached from concrete storage roots.
    let mut seen=std::collections::HashSet::new();
    while let Some(ty)=pending.pop() {
        if !seen.insert(ty) {continue;}
        if let Some(id)=family(ty) {families.insert(id);}
        match ty {
            Type::Union(id)|Type::BorrowedUnion(id)=>pending.extend(program.unions[id.index()].members.iter().copied()),
            Type::Array(element)=>pending.push(element.ty()),
            _=>{},
        }
    }
    families
}

#[cfg(test)]
pub(crate) fn capture_layout(
    program: &CheckedProgram,
    closure: &CheckedClosure,
    data_type: impl Fn(Type) -> String,
) -> (Vec<String>, Vec<String>) {
    capture_layout_with_environments(
        program,
        closure,
        data_type,
        |_, _| None,
        |binding| closure.function.bindings[binding].ty,
        |ty| {
            crate::environment_lifetimes::count(
                ty,
                &program.records,
                |id| program.unions[id.index()].members.clone(),
                |ty| match ty {
                    Type::Closure(id) => program.closures[id].lifetime_count,
                    Type::BorrowedCallable(id) => program
                        .stored_contracts
                        .origins(id)
                        .unwrap()
                        .iter()
                        .map(|&origin| program.closures[origin].lifetime_count)
                        .sum(),
                    Type::StoredCallable(id) => program.stored_contracts.scoped_native_family(id).map_or(0, crate::native_view_callable_storage::Family::lifetimes),
                    _ => 0,
                },
            )
        },
    )
}

pub(crate) fn capture_layout_with_environments(
    program: &CheckedProgram,
    closure: &CheckedClosure,
    data_type: impl Fn(Type) -> String,
    environment_type: impl Fn(Type, &[String]) -> Option<String>,
    storage_type: impl Fn(usize) -> Type,
    lifetime_count: impl Fn(Type) -> usize,
) -> (Vec<String>, Vec<String>) {
    let total = closure
        .captures
        .iter()
        .map(|capture| {
            usize::from(matches!(capture.storage, Mode::Read | Mode::Mutate))
                + lifetime_count(storage_type(capture.inner))
        })
        .sum::<usize>();
    let lifetimes = (0..total).map(|id| format!("'c{id}")).collect::<Vec<_>>();
    let mut next = 0;
    let fields = closure
        .captures
        .iter()
        .map(|capture| {
            let captured = storage_type(capture.inner);
            if let Some(id)=captured.union()
                && crate::environment_lifetimes::contains_native_view(captured,&program.records,&|id|program.unions[id.index()].members.clone()) {
                let count=lifetime_count(captured);
                let borrowed=matches!(capture.storage,Mode::Read|Mode::Mutate);
                let start=next;next+=usize::from(borrowed)+count;
                let (ty,begin,reference)=match capture.storage {
                    Mode::Read=>(Type::BorrowedUnion(id),start,String::new()),
                    Mode::Mutate=>(captured,start+1,format!("&{} mut ",lifetimes[start])),
                    _=>(captured,start,String::new()),
                };
                let field=environment_type(ty,&lifetimes[begin..next]).unwrap_or_else(||data_type(ty));
                return format!("{reference}{field}");
            }
            let payload = captured
                .union()
                .and_then(|id| {
                    program.unions[id.index()]
                        .members
                        .iter()
                        .copied()
                        .find(|member| !matches!(member, Type::Null | Type::Undefined))
                })
                .unwrap_or(captured);
            let view =
                capture.storage == Mode::Read && captured.union().is_some_and(|union| !union.copy);
            let view_lifetime = if view {
                let lifetime = lifetimes[next].clone();
                next += 1;
                Some(lifetime)
            } else {
                None
            };
            let reference = if view {
                String::new()
            } else {
                match capture.storage {
                    Mode::Read | Mode::Mutate => {
                        let name = &lifetimes[next];
                        next += 1;
                        format!(
                            "&{name} {}",
                            if capture.storage == Mode::Mutate {
                                "mut "
                            } else {
                                ""
                            }
                        )
                    }
                    _ => String::new(),
                }
            };
            let field_type = match payload {
                Type::Closure(id) => {
                    let count = lifetime_count(Type::Closure(id));
                    let generics = if count == 0 {
                        String::new()
                    } else {
                        format!("<{}>", lifetimes[next..next + count].join(", "))
                    };
                    next += count;
                    format!("Closure{id}{generics}")
                }
                other => {
                    let count = lifetime_count(other);
                    if count != 0 {
                        let rendered = environment_type(other, &lifetimes[next..next + count]);
                        next += count;
                        rendered.unwrap_or_else(|| data_type(other))
                    } else {
                        data_type(other)
                    }
                }
            };
            if let Some(id) = captured.union() {
                if let Some(lifetime) = view_lifetime {
                    return format!(
                        "ViewUnion{}<{lifetime}{}>",
                        id.index(),
                        if payload == captured
                            || !matches!(
                                payload,
                                Type::Closure(_)
                                    | Type::Callback(_)
                                    | Type::StoredCallable(_)
                                    | Type::BorrowedCallable(_)
                            )
                        {
                            String::new()
                        } else {
                            format!(", {field_type}")
                        }
                    );
                }
                if matches!(
                    payload,
                    Type::Closure(_)
                        | Type::Callback(_)
                        | Type::StoredCallable(_)
                        | Type::BorrowedCallable(_)
                ) {
                    return format!("{reference}Union{}<{field_type}>", id.index());
                }
                // A data payload keeps its tagged slot; only callable payloads
                // parameterize the union.
                return format!("{reference}{}", data_type(captured));
            }
            format!("{reference}{field_type}")
        })
        .collect();
    (lifetimes, fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_capture_modes_and_nested_lifetimes_have_distinct_native_fields() {
        let source = "type User = { name: string; }; function main() { const text = \"Pat\"; const read = (): void => { console.log(text); }; const wrapper = (): void => { read(); }; wrapper(); const user: User = { name: \"old\" }; const change = (): void => { user.name = \"Sam\"; }; change(); }";
        let checked = crate::check(source).unwrap_or_else(|error| panic!("{error:?}"));
        let program = &checked.program;
        let fields = program
            .closures
            .iter()
            .map(|closure| {
                capture_layout(program, closure, |ty| match ty {
                    Type::String => "Text".into(),
                    Type::Record(id) => format!("Record{id}"),
                    _ => panic!("unexpected capture {ty:?}"),
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(fields[0], (vec!["'c0".into()], vec!["&'c0 Text".into()]));
        assert_eq!(
            fields[1],
            (
                vec!["'c0".into(), "'c1".into()],
                vec!["&'c0 Closure0<'c1>".into()]
            )
        );
        assert_eq!(
            fields[2],
            (vec!["'c0".into()], vec!["&'c0 mut Record0".into()])
        );
    }

    #[test]
    fn owned_capture_layout_retains_values_without_borrowed_lifetimes() {
        let checked = crate::check("function main() { const text = \"Pat\"; const action: Once<Function<string>> = (): string => { return text; }; console.log(action()); }").unwrap_or_else(|error| panic!("{error:?}"));
        let program = &checked.program;
        let (lifetimes, fields) = capture_layout(program, &program.closures[0], |_| "Text".into());
        assert!(lifetimes.is_empty());
        assert_eq!(fields, vec!["Text"]);
    }
}

/// Validate by-value edges only. Arrays and captured references provide native
/// indirection; boxing/iterative cleanup for callable cycles is a later gate.
pub(crate) fn validate_storage_layout(program: &CheckedProgram) -> Result<(), crate::Diagnostic> {
    let families = program.stored_contracts.families().collect::<Vec<_>>();
    if !families.iter().any(|(_, _, origins)| !origins.is_empty())
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
    let records = program.records.len();
    let closures = program.closures.len();
    let family_offset = records + closures;
    let family_count = families.iter().map(|(id, _, _)| id + 1).max().unwrap_or(0);
    let union_offset = family_offset + family_count;
    let mut graph = vec![Vec::new(); union_offset + program.unions.len()];
    let node = |ty: Type| -> Result<Option<usize>, crate::Diagnostic> {
        let candidate = match ty {
            Type::Record(id) if id < records => Some(id),
            Type::Closure(id) if id < closures => Some(records + id),
            Type::StoredCallable(id) if id < family_count => Some(family_offset + id),
            Type::Union(id) if id.index() < program.unions.len() => Some(union_offset + id.index()),
            Type::Record(_) | Type::Closure(_) | Type::StoredCallable(_) | Type::Union(_) => {
                return Err(crate::Diagnostic::new(
                    "callable-layout",
                    "Unknown native callable layout dependency.",
                    crate::Span::default(),
                ));
            }
            _ => None,
        };
        Ok(candidate)
    };
    for (id, record) in program.records.iter().enumerate() {
        graph[id].extend(
            record
                .fields
                .iter()
                .map(|field| field.ty.expect("checked field")),
        );
    }
    for (id, closure) in program.closures.iter().enumerate() {
        graph[records + id].extend(
            closure
                .captures
                .iter()
                .filter(|capture| !matches!(capture.storage, Mode::Read | Mode::Mutate))
                .map(|capture| closure.function.bindings[capture.inner].ty),
        );
    }
    for (id, _, origins) in families {
        graph[family_offset + id].extend(origins.iter().copied().map(Type::Closure));
    }
    for (id, union) in program.unions.iter().enumerate() {
        graph[union_offset + id].extend(union.members.iter().copied());
    }
    let mut colors = vec![0u8; graph.len()];
    // Unreferenced symbolic unions are not emitted storage roots.
    for root in 0..union_offset {
        if colors[root] != 0 {
            continue;
        }
        colors[root] = 1;
        let mut stack = vec![(root, 0usize)];
        while let Some((current, edge)) = stack.last_mut() {
            if *edge == graph[*current].len() {
                colors[*current] = 2;
                stack.pop();
                continue;
            }
            let dependency = graph[*current][*edge];
            *edge += 1;
            let Some(next) = node(dependency)? else {
                continue;
            };
            if next >= graph.len() {
                return Err(crate::Diagnostic::new(
                    "callable-layout",
                    "Unknown native callable layout dependency.",
                    crate::Span::default(),
                ));
            }
            if colors[next] == 1 {
                let span = if *current >= records && *current < family_offset {
                    program.closures[*current - records].span
                } else if *current < records {
                    program.records[*current].name.span
                } else {
                    crate::Span::default()
                };
                return Err(crate::Diagnostic::new(
                    "callable-layout",
                    "Recursive callable storage needs an explicit native indirection.",
                    span,
                ));
            }
            if colors[next] == 0 {
                colors[next] = 1;
                stack.push((next, 0));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[test]
fn unused_symbolic_nullable_layouts_do_not_alias_live_native_nodes() {
    let mut checked = crate::check("type Handler={run:()=>int;};function main(){const holder:Handler={run:():int=>{return 7;}};console.log(holder.run());}").unwrap();
    let program = &mut checked.program;
    assert!(program.stored_contracts.populated().next().is_some());
    let index = program.unions.len();
    program.unions.push(crate::unions::UnionType {
        members: vec![Type::Closure(program.closures.len()), Type::Null],
    });
    let family_count = program
        .stored_contracts
        .families()
        .map(|(id, _, _)| id + 1)
        .max()
        .unwrap();
    let second_index = program.unions.len();
    program.unions.push(crate::unions::UnionType {
        // The old untyped graph would mistake this removed origin for the
        // union node itself and reject an unused symbolic type as a cycle.
        members: vec![
            Type::Closure(program.closures.len() + family_count + second_index),
            Type::Undefined,
        ],
    });
    // An unused discovery member is not a native storage dependency.
    validate_storage_layout(program).unwrap();
    let nullable = Type::Union(crate::syntax::UnionId {
        index: u32::try_from(index).unwrap(),
        copy: false,
    });
    program.records[0].fields[0].ty = Some(nullable);
    // A reachable invalid closure must not alias the following family node.
    let error = validate_storage_layout(program).unwrap_err();
    assert_eq!(error.code, "callable-layout");
    assert!(
        error
            .message
            .contains("Unknown native callable layout dependency")
    );
}

#[cfg(test)]
#[test]
fn recursive_callable_layout_is_rejected_while_array_indirection_remains_finite() {
    let mut checked = crate::check("type State = { value: int; }; function make(state: State): () => int { return (): int => { return state.value; }; } function sink(action: Function<int>): int { return action(); } function main() {}").unwrap_or_else(|error| panic!("{error:?}"));
    let program = &mut checked.program;
    let Type::Closure(origin) = program.functions[0].result else {
        panic!("factory origin")
    };
    program
        .stored_contracts
        .register_origin(
            0,
            origin,
            &program.callback_contracts,
            &program.callable_shapes,
            &program.closures,
            crate::Span::default(),
        )
        .unwrap();
    validate_storage_layout(program).unwrap();
    program.records[0].fields[0].ty = Some(Type::StoredCallable(0));
    assert_eq!(
        validate_storage_layout(program).unwrap_err().code,
        "callable-layout"
    );
    let capture = program.closures[origin].captures[0].inner;
    program.closures[origin].function.bindings[capture].ty =
        Type::Array(crate::syntax::ArrayElement::Record(0, 0));
    validate_storage_layout(program).unwrap();
    program.closures[origin].function.bindings[capture].ty = Type::Record(0);
    program.closures[origin].captures[0].storage = Mode::Read;
    validate_storage_layout(program).unwrap();

    // Checked schema depth must not become native test-worker stack depth.
    // Connect a long record chain back through the stored environment.
    program.closures[origin].captures[0].storage = Mode::Consume;
    const DEPTH: usize = 15_000;
    program.records = (0..DEPTH)
        .map(|id| {
            crate::cloneability::tests::record(&[if id + 1 == DEPTH {
                Type::Int
            } else {
                Type::Record(id + 1)
            }])
        })
        .collect();
    validate_storage_layout(program).unwrap();
    program.records[DEPTH - 1].fields[0].ty = Some(Type::StoredCallable(0));
    assert_eq!(
        validate_storage_layout(program).unwrap_err().code,
        "callable-layout"
    );
}
