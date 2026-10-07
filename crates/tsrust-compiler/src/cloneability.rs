//! Structural clone eligibility, independent of callable environment origins.
use crate::syntax::{CallTarget, Record, Type, UnionId};

/// These emitters copy receiver elements even when the receiver is temporary.
/// Concat and filter borrow inputs; flat transfers temporary receivers.
pub(crate) fn copies_array_receiver(
    target: Option<CallTarget>,
    stable: bool,
    arguments: usize,
) -> bool {
    if matches!(
        target,
        Some(CallTarget::ArrayReduce | CallTarget::ArrayReduceRight)
    ) {
        return arguments == 1;
    }
    if matches!(target, Some(CallTarget::ArrayFlat(_))) {
        return stable;
    }
    matches!(
        target,
        Some(
            CallTarget::ArraySlice
                | CallTarget::ArrayConcat
                | CallTarget::ArrayFilter
                | CallTarget::ArrayToReversed
                | CallTarget::ArrayWith
                | CallTarget::ArrayToSorted
                | CallTarget::ArrayToSpliced
                | CallTarget::ArrayFill
                | CallTarget::ArrayCopyWithin
        )
    )
}

pub(crate) struct Cloneability {
    records: Vec<bool>,
}

impl Cloneability {
    pub(crate) fn new(records: &[Record], members: impl Fn(UnionId) -> Vec<Type>) -> Self {
        let mut reverse = vec![Vec::new(); records.len()];
        let mut allowed = vec![true; records.len()];
        let mut rejected = Vec::new();
        for (parent, record) in records.iter().enumerate() {
            let mut pending = record
                .fields
                .iter()
                .map(|field| field.ty.expect("checked field"))
                .collect::<Vec<_>>();
            if let Some(map) = &record.map_definition { pending.extend([map.key, map.value]); }
            while let Some(ty) = pending.pop() {
                match ty {
                    Type::Record(child) | Type::Set(child) | Type::Map(child) => reverse[child].push(parent),
                    Type::Array(element) => pending.push(element.ty()),
                    Type::Union(id) | Type::BorrowedUnion(id) => pending.extend(members(id)),
                    Type::String
                    | Type::Enum(_, _) | Type::Int
                    | Type::Number
                    | Type::Boolean
                    | Type::Null
                    | Type::Undefined => (),
                    Type::Closure(_)
                    | Type::Callback(_)
                    | Type::Callable(_)
                    | Type::StoredCallable(_) | Type::BorrowedCallable(_)
                    | Type::Void => {
                        if allowed[parent] {
                            allowed[parent] = false;
                            rejected.push(parent);
                        }
                    }
                }
            }
        }
        while let Some(child) = rejected.pop() {
            for &parent in &reverse[child] {
                if allowed[parent] {
                    allowed[parent] = false;
                    rejected.push(parent);
                }
            }
        }
        Self { records: allowed }
    }

    pub(crate) fn record(&self, id: usize) -> bool {
        self.records[id]
    }

    pub(crate) fn allows(&self, ty: Type, members: impl Fn(UnionId) -> Vec<Type>) -> bool {
        let mut leaf = ty;
        while let Type::Array(element) = leaf {
            leaf = element.ty();
        }
        match leaf {
            Type::Record(id) | Type::Set(id) | Type::Map(id) => return self.record(id),
            Type::String
            | Type::Enum(_, _) | Type::Int
            | Type::Number
            | Type::Boolean
            | Type::Null
            | Type::Undefined => return true,
            Type::Closure(_)
            | Type::Callback(_)
            | Type::Callable(_)
            | Type::StoredCallable(_) | Type::BorrowedCallable(_)
            | Type::Void => return false,
            Type::Union(_) | Type::BorrowedUnion(_) => (),
            Type::Array(_) => unreachable!("array dimensions removed"),
        }
        let mut pending = vec![leaf];
        while let Some(ty) = pending.pop() {
            match ty {
                Type::Record(id) | Type::Set(id) | Type::Map(id) => {
                    if !self.record(id) {
                        return false;
                    }
                }
                Type::Array(element) => pending.push(element.ty()),
                Type::Union(id) | Type::BorrowedUnion(id) => pending.extend(members(id)),
                Type::String
                | Type::Enum(_, _) | Type::Int
                | Type::Number
                | Type::Boolean
                | Type::Null
                | Type::Undefined => (),
                Type::Closure(_)
                | Type::Callback(_)
                | Type::Callable(_)
                | Type::StoredCallable(_) | Type::BorrowedCallable(_)
                | Type::Void => {
                    return false;
                }
            }
        }
        true
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::Span;
    use crate::syntax::{ArrayElement, RecordField, Token, TokenKind};
    use crate::type_syntax::{TypeName, ValueAnnotation, ValueAnnotationKind};

    pub(crate) fn record(types: &[Type]) -> Record {
            let span = Span::new(0, 1);
        Record {
            map_definition: None, set_definition: None,                enum_definition: None,
            name: Token {
                text: "Schema".into(),
                span,
                kind: TokenKind::Name,
            },
            type_parameters: Vec::new(),
            application: None,
            fields: types
                .iter()
                .enumerate()
                .map(|(index, &ty)| RecordField {
                    name: Token {
                        text: format!("field{index}"),
                        span,
                        kind: TokenKind::Name,
                    },
                    annotation: ValueAnnotation {
                        kind: ValueAnnotationKind::Named(TypeName {
                            text: "Schema".into(),
                            span,
                        }),
                        span,
                    },
                    ty: Some(ty),
                    callable_declaration: None, optional: false,
                })
                .collect(),
        }
    }

    #[test]
    fn clone_eligibility_propagates_through_cycles_arrays_and_nullable_payloads() {
        let unions = crate::unions::UnionArena::default();
        let nullable = unions
            .intern(vec![Type::Record(0), Type::Null], Span::new(0, 1))
            .unwrap();
        let records = [
            record(&[Type::Array(ArrayElement::Record(1, 0))]),
            record(&[Type::Array(ArrayElement::Record(0, 0)), Type::Callback(0)]),
            record(&[nullable]),
            record(&[Type::String, Type::Int]),
        ];
        let eligibility = Cloneability::new(&records, |id| unions.members(id));
        assert!(!eligibility.record(0));
        assert!(!eligibility.record(1));
        assert!(!eligibility.record(2));
        assert!(eligibility.record(3));
        assert!(!eligibility.allows(nullable, |id| unions.members(id)));
        let present = unions
            .intern(vec![Type::Record(3), Type::Undefined], Span::new(0, 1))
            .unwrap();
        assert!(eligibility.allows(present, |id| unions.members(id)));
        assert!(
            eligibility.allows(Type::BorrowedUnion(present.union().unwrap()), |id| unions
                .members(id))
        );
        assert!(
            !eligibility.allows(Type::Array(ArrayElement::Record(2, 127)), |id| unions
                .members(id))
        );
        assert!(
            eligibility.allows(Type::Array(ArrayElement::Record(3, 127)), |id| unions
                .members(id))
        );
        for ty in [
            Type::Closure(0),
            Type::Callback(0),
            Type::Callable(0),
            Type::Void,
        ] {
            assert!(!eligibility.allows(ty, |id| unions.members(id)));
        }
    }

    #[test]
    fn large_schema_propagation_uses_no_recursive_worker_frames() {
        let count = 15_000;
        let mut records = (0..count)
            .map(|index| record(&[Type::Array(ArrayElement::Record(index + 1, 0))]))
            .collect::<Vec<_>>();
        records.push(record(&[Type::Closure(0)]));
        let eligibility = Cloneability::new(&records, |_| Vec::new());
        assert!(eligibility.records.iter().all(|value| !value));
    }
}

#[cfg(test)]
#[test]
fn copying_policy_keeps_transfer_and_borrow_operations_separate() {
    for target in [
        CallTarget::ArraySlice,
        CallTarget::ArrayConcat,
        CallTarget::ArrayFilter,
        CallTarget::ArrayToReversed,
        CallTarget::ArrayWith,
        CallTarget::ArrayToSorted,
        CallTarget::ArrayToSpliced,
        CallTarget::ArrayFill,
        CallTarget::ArrayCopyWithin,
    ] {
        assert!(copies_array_receiver(Some(target), false, 0), "{target:?}");
    }
    for target in [
        CallTarget::ArraySort,
        CallTarget::ArrayReverse,
        CallTarget::ArrayPop,
        CallTarget::ArrayShift,
        CallTarget::ArrayRemove,
        CallTarget::ArrayPush,
        CallTarget::ArrayAt,
        CallTarget::ArrayFind,
        CallTarget::ArrayForEach,
    ] {
        assert!(!copies_array_receiver(Some(target), false, 0), "{target:?}");
    }
    assert!(!copies_array_receiver(None, true, 0));
    for levels in [0, 1, 127] {
        assert!(copies_array_receiver(
            Some(CallTarget::ArrayFlat(levels)),
            true,
            0
        ));
        assert!(!copies_array_receiver(
            Some(CallTarget::ArrayFlat(levels)),
            false,
            0
        ));
    }
}

#[cfg(test)]
#[test]
fn reduction_only_copies_an_implicit_seed() {
    for target in [CallTarget::ArrayReduce, CallTarget::ArrayReduceRight] {
        for stable in [false, true] {
            assert!(copies_array_receiver(Some(target), stable, 1));
            assert!(!copies_array_receiver(Some(target), stable, 2));
        }
    }
}

#[cfg(test)]
#[test]
fn stored_callable_is_an_owned_noncloneable_family_not_a_callback_path() {
    let stored = Type::StoredCallable(7);
    assert!(!stored.is_copy());
    assert_eq!(stored.owning(), stored);
    assert_ne!(stored, Type::Closure(7));
    assert_ne!(stored, Type::Callback(7));
    assert_eq!(crate::emit::ty(stored), "StoredCallable7");
    let records = vec![
        tests::record(&[stored]),
        tests::record(&[Type::Array(crate::syntax::ArrayElement::Record(0, 2))]),
    ];
    let eligibility = Cloneability::new(&records, |_| Vec::new());
    assert!(!eligibility.allows(stored, |_| Vec::new()));
    assert!(!eligibility.record(0));
    assert!(!eligibility.record(1));
    let unions = crate::unions::UnionArena::default();
    let span = crate::Span::new(3, 12);
    let nullable = unions.intern([stored, Type::Null], span).unwrap();
    assert!(!nullable.is_copy());
    assert_eq!(unions.payload(nullable), Some(stored));
    assert!(!eligibility.allows(nullable, |id| unions.members(id)));
    let wrappers = vec![tests::record(&[nullable])];
    let wrapper_eligibility = Cloneability::new(&wrappers, |id| unions.members(id));
    assert!(!wrapper_eligibility.record(0));
}

#[cfg(test)]
#[test]
fn stored_callable_array_leaves_keep_owned_vec_layout_and_reject_cloning() {
    let eligibility = Cloneability::new(&[], |_| Vec::new());
    for depth in [0, 1, 127] {
        let ty = Type::Array(crate::syntax::ArrayElement::StoredCallable(3, depth));
        assert!(!eligibility.allows(ty, |_| Vec::new()));
        assert!(!crate::storage::inline_array(ty));
        assert!(!crate::storage::numeric_array(ty));
        assert_eq!(
            crate::emit::ty(ty),
            format!(
                "{}StoredCallable3{}",
                "Vec<".repeat(depth as usize + 1),
                ">".repeat(depth as usize + 1)
            )
        );
    }
}
