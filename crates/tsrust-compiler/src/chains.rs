//! Operator chains that code generation emits flat, such as `a + b + c` or
//! `p && q && r`. Their length is not nesting: the checker and emitter walk them
//! without recursion, so only operands nested inside count toward the limit.
use crate::syntax::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Family {
    /// String concatenation, built in one buffer.
    Text,
    /// Int, number or boolean operators applied to one running value.
    Scalar(Type),
}

/// The flat chain family a checked binary node belongs to, if any.
pub(crate) fn link(expr: &Expr) -> Option<Family> {
    let ExprKind::Binary { op, left, right, .. } = &expr.kind else {
        return None;
    };
    if *op == BinaryOp::Add && left.ty.map(Type::scalar_base) == Some(Type::String) {
        return Some(Family::Text);
    }
    let ty = expr.ty?;
    let fits = match op {
        BinaryOp::And | BinaryOp::Or => ty == Type::Boolean,
        BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Remainder => {
            matches!(ty, Type::Int | Type::Number)
        }
        _ => false,
    };
    let scalar = |operand: &Expr| operand.ty.map(Type::scalar_base) == Some(ty);
    (fits && scalar(left) && scalar(right)).then_some(Family::Scalar(ty))
}

/// Before types are known, the parser admits a left operand that may continue
/// a chain; the checker confirms it with `checked_depth`.
pub(crate) fn may_continue(op: BinaryOp, left: &Expr) -> bool {
    let arithmetic = |op: BinaryOp| matches!(op, BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Remainder);
    let logical = |op: BinaryOp| matches!(op, BinaryOp::And | BinaryOp::Or);
    matches!(&left.kind, ExprKind::Binary { op: inner, .. } if (arithmetic(op) && arithmetic(*inner)) || (logical(op) && logical(*inner)))
}

/// Nesting depth of a checked binary node as emitted. A left operand in the same
/// flat chain adds no level; every other operand adds one.
pub(crate) fn checked_depth(expr: &Expr) -> usize {
    let ExprKind::Binary { left, right, .. } = &expr.kind else {
        return expr.depth;
    };
    match link(expr) {
        Some(family) => {
            let left = if link(left) == Some(family) { left.depth } else { left.depth + 1 };
            left.max(right.depth + 1)
        }
        None => 1 + left.depth.max(right.depth),
    }
}
