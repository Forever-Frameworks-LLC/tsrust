//! Count by-value environment lifetime slots without recursive type traversal.
use crate::syntax::{Record, Type, UnionId};

pub(crate) fn count(
    root: Type,
    records: &[Record],
    members: impl Fn(UnionId) -> Vec<Type>,
    leaf: impl Fn(Type) -> usize,
) -> usize {
    let recursive = contains_native_view(root, records, &members)
        .then(|| crate::recursive_view_shapes::Index::new(records, &members));
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
                if recursive.as_ref().is_some_and(|index| index.access(id).is_some()) || record.native_type.is_some_and(|id|crate::native_types::declaration(id).borrow_access().is_some()) {
                    total=total.saturating_add(1);
                    if total>128 {return 129;}
                    continue;
                }
                for field in &record.fields {
                    let Some(ty) = field.ty else {
                        return 129;
                    };
                    pending.push((ty, false));
                }
            }
            Type::Array(element) if contains_native_view(element.ty(), records, &members) => pending.push((element.ty(),false)),
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

pub(crate) fn contains_native_view(root:Type,records:&[Record],members:&impl Fn(UnionId)->Vec<Type>)->bool {
    let mut pending=vec![root];
    let mut seen=std::collections::HashSet::new();
    while let Some(ty)=pending.pop() {
        if !seen.insert(ty) {continue;}
        match ty {
            Type::Record(id)=>{
                let Some(record)=records.get(id) else {continue;};
                if record.native_type.is_some_and(|id|crate::native_types::declaration(id).borrow_access().is_some()) {return true;}
                pending.extend(record.fields.iter().filter_map(|field|field.ty));
            }
            Type::Array(element)=>pending.push(element.ty()),
            Type::Union(id)|Type::BorrowedUnion(id)=>pending.extend(members(id)),
            _=>{}
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recursive_native_records_share_one_slot_without_merging_outer_siblings() {
        let mut window = crate::cloneability::tests::record(&[]);
        window.native_type = Some(crate::native_signatures::NativeTypeId::ByteView);
        let array = |id| Type::Array(crate::syntax::ArrayElement::Record(id, 0));
        let records = vec![
            window,
            crate::cloneability::tests::record(&[Type::Record(0), array(1)]),
            crate::cloneability::tests::record(&[Type::Record(1), Type::Record(0), Type::Record(1)]),
            crate::cloneability::tests::record(&[array(4), Type::Record(0)]),
            crate::cloneability::tests::record(&[array(3)]),
        ];
        assert_eq!(count(Type::Record(1), &records, |_| vec![], |_| 0), 1);
        assert_eq!(count(Type::Record(2), &records, |_| vec![], |_| 0), 3);
        assert_eq!(count(Type::Record(3), &records, |_| vec![], |_| 0), 1);
        assert_eq!(count(Type::Record(4), &records, |_| vec![], |_| 0), 1);
    }

    #[test]
    fn native_slice_captures_reserve_inner_lifetime_slots() {
        let mut window=crate::cloneability::tests::record(&[]);
        window.native_type=Some(crate::native_signatures::NativeTypeId::ByteView);
        let pair=crate::cloneability::tests::record(&[Type::Record(0),Type::Record(0)]);
        assert_eq!(count(Type::Record(0),&[window.clone()],|_|vec![],|_|0),1);
        assert_eq!(count(Type::Array(crate::syntax::ArrayElement::Record(0,0)),&[window.clone()],|_|vec![],|_|0),1);
        assert_eq!(count(Type::Record(1),&[window,pair],|_|vec![],|_|0),2);
        let recursive=crate::cloneability::tests::record(&[Type::Array(crate::syntax::ArrayElement::Record(0,0))]);
        assert_eq!(count(Type::Record(0),&[recursive],|_|vec![],|_|0),0);
    }
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
