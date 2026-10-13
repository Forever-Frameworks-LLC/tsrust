//! Value-returning logical expressions; boolean guards retain their existing flow checker.
use super::*;

impl Analyzer {
    pub(super) fn logical_alternatives(&self, ty: Type) -> Vec<Type> {
        ty.union().map(|id| self.unions.members(id)).unwrap_or_else(|| vec![ty])
    }

    pub(super) fn logical_truthiness(ty: Type) -> (bool, bool) {
        match ty {
            Type::Null | Type::Undefined => (false, true),
            Type::Boolean | Type::Int | Type::Number | Type::String => (true, true),
            Type::Void => (false, false),
            _ => (true, false),
        }
    }

    pub(super) fn logical_selection(&self, lhs: Type, op: BinaryOp) -> (Vec<Type>, bool) {
        let members = self.logical_alternatives(lhs);
        let selected = members.iter().copied().filter(|ty| {
            let (truthy, falsy) = Self::logical_truthiness(*ty);
            if op == BinaryOp::Or { truthy } else { falsy }
        }).collect();
        let reaches_right = members.iter().any(|ty| {
            let (truthy, falsy) = Self::logical_truthiness(*ty);
            if op == BinaryOp::Or { falsy } else { truthy }
        });
        (selected, reaches_right)
    }



}
