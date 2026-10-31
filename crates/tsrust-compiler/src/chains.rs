//! Operator chains that code generation emits flat, such as `a + b + c` or
//! `p && q && r`: one buffer or one running value instead of nested Rust, so
//! a long chain compiles like a list of statements.
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
