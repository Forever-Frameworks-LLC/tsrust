//! Count by-value environment lifetime slots without recursive type traversal.
use crate::syntax::{Record, Type, UnionId};

pub(crate) fn count(
    root: Type,
    records: &[Record],
    members: impl Fn(UnionId) -> Vec<Type>,
    leaf: impl Fn(Type) -> usize,
) -> usize {
    let mut total = 0usize;
    let mut pending = vec![(root, false)];
    let mut active = std::collections::HashSet::new();
    while let Some((ty, finished)) = pending.pop() {
        if finished {
            active.remove(&ty);
            continue;
        }
        if matches!(
            ty,
            Type::Record(_) | Type::Union(_) | Type::BorrowedUnion(_)
        ) {
            if !active.insert(ty) {
                return 129;
            }
            pending.push((ty, true));
        }
        match ty {
            Type::Record(id) => {
                let Some(record) = records.get(id) else {
                    return 129;
                };
                for field in &record.fields {
                    let Some(ty) = field.ty else {
                        return 129;
                    };
                    pending.push((ty, false));
                }
            }
            Type::Union(id) | Type::BorrowedUnion(id) => {
                pending.extend(members(id).into_iter().map(|ty| (ty, false)))
            }
            _ => total = total.saturating_add(leaf(ty)),
        }
        if total > 128 {
            return 129;
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_fields_have_distinct_slots_and_cycles_remain_bounded() {
        let mut records = vec![
            crate::cloneability::tests::record(&[Type::Record(1), Type::Record(1)]),
            crate::cloneability::tests::record(&[Type::Closure(0)]),
        ];
        assert_eq!(
            count(
                Type::Record(0),
                &records,
                |_| vec![],
                |ty| usize::from(matches!(ty, Type::Closure(_)))
            ),
            2
        );
        records[1].fields[0].ty = Some(Type::Record(0));
        assert_eq!(count(Type::Record(0), &records, |_| vec![], |_| 0), 129);
    }

    #[test]
    fn complexity_limit_and_unknown_fields_are_conservative() {
        let mut record = crate::cloneability::tests::record(&vec![Type::Closure(0); 129]);
        assert_eq!(
            count(Type::Record(0), &[record.clone()], |_| vec![], |_| 1),
            129
        );
        record.fields.truncate(1);
        record.fields[0].ty = None;
        assert_eq!(count(Type::Record(0), &[record], |_| vec![], |_| 0), 129);
    }
}
