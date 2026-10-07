//! Bounded aggregate traversal for callable retention and scoped escape checks.
use crate::syntax::{Record, Type, UnionId};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Facts {
    pub retains: bool,
    pub scoped: bool,
    pub unknown: bool,
}

pub(crate) fn inspect(
    root: Type,
    records: &[Record],
    members: impl Fn(UnionId) -> Vec<Type>,
    callable_is_scoped: impl Fn(Type) -> bool,
) -> Facts {
    let mut facts = Facts::default();
    let mut pending = vec![root];
    let mut seen = std::collections::HashSet::new();
    while let Some(ty) = pending.pop() {
        if !seen.insert(ty) {
            continue;
        }
        match ty {
            Type::Closure(_)
            | Type::Callback(_)
            | Type::Callable(_)
            | Type::StoredCallable(_)
            | Type::BorrowedCallable(_) => {
                facts.retains = true;
                facts.scoped |= callable_is_scoped(ty);
            }
            Type::Array(element) => pending.push(element.ty()),
            Type::Record(id) | Type::Set(id) | Type::Map(id) => {
                let Some(record) = records.get(id) else {
                    facts.unknown = true;
                    continue;
                };
                for field in &record.fields {
                    if let Some(ty) = field.ty {
                        pending.push(ty);
                    } else {
                        facts.unknown = true;
                    }
                }
                if let Some(map) = &record.map_definition {
                    pending.extend([map.key, map.value]);
                }
                if let Some(set) = &record.set_definition {
                    pending.push(set.key);
                }
            }
            Type::Union(id) | Type::BorrowedUnion(id) => pending.extend(members(id)),
            _ => {}
        }
    }
    facts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::{ArrayElement, MapDefinition, SetDefinition};

    #[test]
    fn recursive_nullable_aggregate_cannot_hide_scoped_callable() {
        let unions = crate::unions::UnionArena::default();
        let nullable = unions
            .intern(
                [Type::BorrowedCallable(7), Type::Null],
                crate::Span::new(0, 1),
            )
            .unwrap();
        let mut records = vec![
            crate::cloneability::tests::record(&[Type::Record(1)]),
            crate::cloneability::tests::record(&[
                Type::Array(ArrayElement::Record(0, 0)),
                nullable,
            ]),
        ];
        for root in [Type::Record(0), Type::Record(1), nullable] {
            assert_eq!(
                inspect(
                    root,
                    &records,
                    |id| unions.members(id),
                    |ty| matches!(ty, Type::BorrowedCallable(_))
                ),
                Facts {
                    retains: true,
                    scoped: true,
                    unknown: false
                }
            );
        }
        records[1].fields.pop();
        assert_eq!(
            inspect(
                Type::Record(0),
                &records,
                |id| unions.members(id),
                |_| false
            ),
            Facts::default()
        );
    }

    #[test]
    fn collections_and_owned_callbacks_keep_distinct_lifetime_facts() {
        let mut map = crate::cloneability::tests::record(&[]);
        map.map_definition = Some(MapDefinition {
            key: Type::String,
            value: Type::BorrowedCallable(1),
            entry: None,
        });
        let mut set = crate::cloneability::tests::record(&[]);
        set.set_definition = Some(SetDefinition {
            key: Type::Record(0),
        });
        let records = vec![map, set];
        assert_eq!(
            inspect(
                Type::Set(1),
                &records,
                |_| vec![],
                |ty| matches!(ty, Type::BorrowedCallable(_))
            ),
            Facts {
                retains: true,
                scoped: true,
                unknown: false
            }
        );
        assert_eq!(
            inspect(Type::StoredCallable(3), &records, |_| vec![], |_| false),
            Facts {
                retains: true,
                scoped: false,
                unknown: false
            }
        );
    }

    #[test]
    fn unresolved_fields_and_missing_layouts_remain_unknown() {
        let mut record = crate::cloneability::tests::record(&[Type::Int]);
        record.fields[0].ty = None;
        for root in [Type::Record(0), Type::Record(9)] {
            assert!(inspect(root, &[record.clone()], |_| vec![], |_| false).unknown);
        }
    }

    #[test]
    fn deep_aggregate_traversal_uses_no_recursive_stack() {
        let mut records = (0..10000)
            .map(|id| crate::cloneability::tests::record(&[Type::Record(id + 1)]))
            .collect::<Vec<_>>();
        records.push(crate::cloneability::tests::record(&[
            Type::BorrowedCallable(0),
        ]));
        assert!(inspect(Type::Record(0), &records, |_| vec![], |_| true).scoped);
    }
}
