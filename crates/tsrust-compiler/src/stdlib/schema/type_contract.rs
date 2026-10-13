//! Exact output-shape contracts for explicitly named recursive schemas.
use crate::syntax::{Record,Type,UnionId};
use std::collections::HashSet;

pub(crate) fn equivalent(left:Type,right:Type,records:&[Record],members:&impl Fn(UnionId)->Vec<Type>)->bool {
    fn visit(left:Type,right:Type,records:&[Record],members:&impl Fn(UnionId)->Vec<Type>,active:&mut HashSet<(Type,Type)>)->bool {
        if left==right { return true; }
        if !active.insert((left,right)) { return true; }
        let result=match (left,right) {
            (Type::Array(left),Type::Array(right))=>visit(left.ty(),right.ty(),records,members,active),
            (Type::Record(left),Type::Record(right)) if records[left].native_type.is_none()&&records[right].native_type.is_none()=>{
                let left=&records[left].fields;let right=&records[right].fields;
                left.len()==right.len()&&left.iter().all(|field|right.iter().any(|other|
                    field.name.text==other.name.text&&field.optional==other.optional&&visit(field.ty.unwrap(),other.ty.unwrap(),records,members,active)))
            }
            (Type::Union(left),Type::Union(right))=>{
                let left=members(left);let right=members(right);
                left.len()==right.len()
                    &&left.iter().all(|left|right.iter().any(|right|visit(*left,*right,records,members,active)))
                    &&right.iter().all(|right|left.iter().any(|left|visit(*left,*right,records,members,active)))
            }
            _=>false,
        };
        active.remove(&(left,right));
        result
    }
    visit(left,right,records,members,&mut HashSet::new())
}
