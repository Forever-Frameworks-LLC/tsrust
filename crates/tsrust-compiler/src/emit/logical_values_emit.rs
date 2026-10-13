//! Native selection returns operand values and evaluates the RHS only when needed.
use super::*;

pub(super) struct LogicalPlan {
    pub(super) left_mode: Mode,
    pub(super) right_mode: Mode,
    pub(super) skip_right: bool,
    lhs: Type,
    rhs: Type,
    result: Type,
    result_view: bool,
    left_read: bool,
    op: BinaryOp,
    can_truthy: bool,
    can_falsy: bool,
}


impl Emitter<'_> {
    fn logical_truthy(&self, value: &str, ty: Type, reference: bool) -> String {
        let scalar = if reference { format!("*({value})") } else { value.to_owned() };
        match ty {
            Type::Null | Type::Undefined => "false".into(),
            Type::Boolean => scalar,
            Type::Int => format!("({scalar}) != 0"),
            Type::Number => format!("({scalar}) != 0.0 && !({value}).is_nan()"),
            Type::String => format!("!({value}).is_empty()"),
            Type::Union(id) | Type::BorrowedUnion(id) => {
                let view = matches!(ty, Type::BorrowedUnion(_)) || (!id.copy && reference);
                let arms = self.program.unions[id.index()].members.iter().enumerate().map(|(index, member)| {
                    let body = self.logical_truthy("logical_truthy_payload", *member, true);
                    format!("{}{}::V{index}(logical_truthy_payload)=>{body}", if view { "ViewUnion" } else { "Union" }, id.index())
                }).collect::<Vec<_>>().join(",");
                format!("match &({value}) {{{arms}}}")
            }
            _ => "true".into(),
        }
    }

    fn logical_selected_value(&self, code: &str, actual: Type, result: Type, source_view: bool, target_view: bool) -> String {
        if let Some(from) = actual.union() {
            let arms = self.program.unions[from.index()].members.iter().enumerate().map(|(index, member)| {
                let payload = if source_view && member.is_copy() { "*logical_payload" } else { "logical_payload" };
                let body = self.logical_selected_value(payload, *member, result, source_view && !member.is_copy(), target_view);
                format!("{}{}::V{index}(logical_payload)=>{body}", if source_view { "ViewUnion" } else { "Union" }, from.index())
            }).collect::<Vec<_>>().join(",");
            return format!("match {code} {{{arms}}}");
        }
        if let Some(to) = result.union() {
            let Some(index) = self.program.unions[to.index()].members.iter().position(|member| *member == actual) else {
                return "unreachable!(\"unselected logical alternative\")".into();
            };
            let payload = if target_view && actual.is_copy() {
                match actual {
                    Type::Null => format!("{{let _={code}; &TsrNull}}"),
                    Type::Undefined => format!("{{let _={code}; &TsrUndefined}}"),
                    _ => format!("&({code})"),
                }
            } else { code.to_owned() };
            return format!("{}{}::V{index}({payload})", if target_view { "ViewUnion" } else { "Union" }, to.index());
        }
        if actual == result { code.to_owned() } else { "unreachable!(\"unselected logical alternative\")".into() }
    }

    /// `!value` for a value that is not a boolean: read it once and negate its truth.
    pub(super) fn logical_not(&self, fun: &CheckedFunction, value: &Expr) -> String {
        let ty = value.ty.expect("checked truthiness operand");
        let source = if ty.is_copy() { self.expr(fun, value, Mode::Copy) } else { self.conditional_arm(fun, value, Mode::Read) };
        self.finish_logical_not(value, source)
    }

    pub(super) fn finish_logical_not(&self, value: &Expr, source: String) -> String {
        let ty = value.ty.expect("checked truthiness operand");
        let view = matches!(ty, Type::BorrowedUnion(_)) || ty.union().is_some_and(|id| !id.copy);
        let test = self.logical_truthy("logical_left", if view { Type::BorrowedUnion(ty.union().expect("union view")) } else { ty }, !ty.is_copy());
        format!("{{let logical_left={source}; !({test})}}")
    }

    pub(super) fn prepare_logical_emission(&self, fun: &CheckedFunction, expr: &Expr, op: BinaryOp, left: &Expr, right: &Expr, mode: Mode) -> LogicalPlan {
        let lhs = left.ty.expect("logical left");
        let rhs = right.ty.expect("logical right");
        let result = expr.ty.expect("logical result");
        let result_view = matches!(result, Type::BorrowedUnion(_)) || mode == Mode::Read
            || (mode == Mode::Copy && !result.is_copy())
            || Self::borrowed_place_expression(fun, expr);
        let alternatives = lhs.union().map(|id| self.program.unions[id.index()].members.clone()).unwrap_or_else(|| vec![lhs]);
        let selected_owned = alternatives.iter().any(|ty| {
            !ty.is_copy() && if op == BinaryOp::Or { !matches!(ty, Type::Null | Type::Undefined) } else { *ty == Type::String }
        });
        let left_read = matches!(lhs, Type::BorrowedUnion(_)) || result_view || !selected_owned;
        let can_truthy = alternatives.iter().any(|ty| !matches!(ty, Type::Null | Type::Undefined));
        let can_falsy = alternatives.iter().any(|ty| matches!(ty, Type::Null | Type::Undefined | Type::Boolean | Type::Int | Type::Number | Type::String));
        LogicalPlan {
            left_mode: if lhs.is_copy() { Mode::Copy } else if left_read { Mode::Read } else { mode },
            right_mode: if rhs.is_copy() { Mode::Copy } else if result_view { Mode::Read } else { mode },
            skip_right: (op == BinaryOp::Or && !can_falsy) || (op == BinaryOp::And && !can_truthy),
            lhs, rhs, result, result_view, left_read, op, can_truthy, can_falsy,
        }
    }

    pub(super) fn finish_logical_emission(&self, plan: LogicalPlan, source: String, fallback: Option<String>) -> String {
        let LogicalPlan { lhs, rhs, result, result_view, left_read, op, can_truthy, can_falsy, skip_right, .. } = plan;
        let source_view = matches!(lhs, Type::BorrowedUnion(_)) || (lhs.union().is_some_and(|id| !id.copy) && left_read);
        let test = self.logical_truthy("logical_left", if source_view { Type::BorrowedUnion(lhs.union().unwrap()) } else { lhs }, !lhs.is_copy() && left_read);
        let selected = self.logical_selected_value("logical_left", lhs, result, source_view, result_view && result.union().is_some_and(|id| !id.copy));
        if skip_right { return format!("{{let logical_left={source}; {selected}}}"); }
        let fallback = fallback.expect("reachable logical fallback");
        let fallback_view = matches!(rhs, Type::BorrowedUnion(_)) || (rhs.union().is_some_and(|id| !id.copy) && result_view);
        let fallback = self.logical_selected_value(&fallback, rhs, result, fallback_view, result_view && result.union().is_some_and(|id| !id.copy));
        if (op == BinaryOp::Or && !can_truthy) || (op == BinaryOp::And && !can_falsy) {
            return format!("{{let logical_left={source}; {fallback}}}");
        }
        let (yes, no) = if op == BinaryOp::Or { (selected, fallback) } else { (fallback, selected) };
        format!("{{let logical_left={source}; if {test} {{ {yes} }} else {{ {no} }} }}")
    }

    pub(super) fn logical_value(&self, fun: &CheckedFunction, expr: &Expr, op: BinaryOp, left: &Expr, right: &Expr, mode: Mode) -> String {
        let plan = self.prepare_logical_emission(fun, expr, op, left, right, mode);
        let source = if left.ty.unwrap().is_copy() { self.expr(fun, left, plan.left_mode) }
            else { self.conditional_arm(fun, left, plan.left_mode) };
        let fallback = if plan.skip_right { None } else { Some(if right.ty.unwrap().is_copy() { self.expr(fun, right, plan.right_mode) }
            else { self.conditional_arm(fun, right, plan.right_mode) }) };
        self.finish_logical_emission(plan, source, fallback)
    }
}
