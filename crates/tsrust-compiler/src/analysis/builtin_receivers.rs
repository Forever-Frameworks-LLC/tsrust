//! Builtin method chains check a call receiver as a child task before dispatch.
//! Each listed method reads its receiver with `Mode::Read` and no expected type;
//! the dispatcher then takes the stored result instead of starting a nested driver.
use super::*;

pub(super) fn receiver(expr: &mut Expr) -> &mut Expr {
    let ExprKind::Call(callee, _) = &mut expr.kind else { unreachable!("builtin call") };
    let ExprKind::Field(base, _) = &mut callee.kind else { unreachable!("builtin method") };
    base
}

pub(super) fn receiver_address(expr: &Expr) -> usize {
    match &expr.kind {
        ExprKind::Call(callee, _) => match &callee.kind { ExprKind::Field(base, _) => base.as_ref() as *const Expr as usize, _ => 0 },
        _ => 0,
    }
}

impl Analyzer {
    /// Returns whether the receiver needs builtin literal context when it is prechecked.
    pub(super) fn builtin_receiver_precheck(&self, context: &Context, expr: &Expr) -> Option<bool> {
        let ExprKind::Call(callee, _) = &expr.kind else { return None };
        let ExprKind::Field(base, name) = &callee.kind else { return None };
        // Only nested calls recurse; names, literals and fields stay on the ordinary path.
        if !matches!(base.kind, ExprKind::Call(..)) || base.ty.is_some() { return None; }
        let literals = match name.as_str() {
            "toString" | "join" => true,
            "valueOf" | "isWellFormed" | "toWellFormed" | "trim" | "trimStart" | "trimEnd" | "trimLeft" | "trimRight" | "toLowerCase" | "toUpperCase"
            | "slice" | "charAt" | "substring" | "substr" | "repeat" | "replace" | "replaceAll" | "split" | "padStart" | "padEnd" => false,
            _ => return None,
        };
        // Records may declare methods with these names; their callee checks differ.
        match self.expression_type_hint(context, base)? {
            Type::String | Type::Int | Type::Number | Type::Boolean | Type::Array(_) => Some(literals),
            _ => None,
        }
    }
}
