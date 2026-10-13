//! Coalescing operand modes and branch output without recursive emission.
use super::*;

pub(super) struct Plan {
    pub(super) absent_only: bool,
    pub(super) undefined_only: bool,
    pub(super) left_mode: Mode,
    pub(super) right_mode: Mode,
    logical_union: Option<UnionId>,
    union: Option<UnionId>,
    result: Type,
    borrowed: bool,
    target_view: bool,
}

impl Emitter<'_> {
    pub(super) fn prepare_coalesce_emission<'a>(&self, expr: &Expr, left: &'a Expr, mode: Mode) -> (Plan, &'a Expr) {
        let undefined_only=matches!(expr.kind,ExprKind::Binary{op:BinaryOp::PatternDefault,..});
        let absent_only = left.ty.is_some_and(|ty| {
            (ty==Type::Undefined || (!undefined_only && ty==Type::Null))
                || ty.union().is_some_and(|id| {
                    self.program.unions[id.index()]
                        .members
                        .iter()
                        .all(|ty| (*ty==Type::Undefined || (!undefined_only && *ty==Type::Null)) || matches!(ty,Type::Callback(id) if self.program.absent_callback_payloads.contains(id)))
                })
        });
        if absent_only {
            return (Plan { absent_only, undefined_only, left_mode: Mode::Copy, right_mode: mode,
                logical_union: None, union: None, result: expr.ty.unwrap(), borrowed: false, target_view: false }, left);
        }
        let logical_union = left.ty.and_then(Type::union).expect("dynamic nullable lhs");
        // Match the physical tag directly: retagging an owning narrowed value
        // first would obscure the absent arm's ability to borrow its source.
        let (left, union) = if let ExprKind::Project { union, value, .. } = &left.kind {
            (value.as_ref(), *union)
        } else {
            (left, logical_union)
        };
        let result = expr.ty.expect("coalescing result");
        let borrowed =
            matches!(left.ty, Some(Type::BorrowedUnion(_))) || (!union.copy && mode == Mode::Read);
        let access = if borrowed {
            Mode::Read
        } else if union.copy {
            Mode::Copy
        } else {
            mode
        };
        let target_view = matches!(result, Type::BorrowedUnion(_))
            || (result.union().is_some_and(|id| !id.copy) && mode == Mode::Read);
        (Plan { absent_only, undefined_only, left_mode: access, right_mode: if target_view { Mode::Read } else { mode },
            logical_union: Some(logical_union), union: Some(union), result, borrowed, target_view }, left)
    }

    pub(super) fn finish_coalesce_emission(&self, plan: Plan, right: &Expr, source: String, fallback: String) -> String {
        if plan.absent_only { return format!("{{ let _nullish_left={source}; {fallback} }}"); }
        let Plan { undefined_only, left_mode: access, logical_union, union, result, borrowed, target_view, .. } = plan;
        let logical_union = logical_union.expect("dynamic logical union");
        let union = union.expect("dynamic physical union");
        let fallback = if let Some(target) = result.union() {
            let actual = right.ty.expect("coalescing fallback");
            let prefix = if target_view { "ViewUnion" } else { "Union" };
            if let Some(from) = actual.union() {
                if from == target {
                    fallback
                } else {
                    let source_view =
                        matches!(actual, Type::BorrowedUnion(_)) || (!from.copy && target_view);
                    let arms = self.program.unions[from.index()]
                        .members
                        .iter()
                        .enumerate()
                        .map(|(index, member)| {
                            let to = self.program.unions[target.index()]
                                .members
                                .iter()
                                .position(|ty| {
                                    ty == member
                                        || matches!(ty, Type::Callback(_))
                                            && matches!(
                                                member,
                                                Type::Closure(_) | Type::Callback(_)
                                            )
                                })
                                .expect("fallback subset");
                            format!(
                                "{}{}::V{index}(payload)=>{prefix}{}::V{to}({})",
                                if source_view { "ViewUnion" } else { "Union" },
                                from.index(),target.index(),if !target_view&&!source_view{self.union_transfer_payload(from,target,*member,"payload")}else{"payload".into()}
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(",");
                    format!("match {fallback} {{{arms}}}")
                }
            } else {
                let index = self.program.unions[target.index()]
                    .members
                    .iter()
                    .position(|ty| *ty == actual)
                    .expect("fallback member");
                let code = if target_view && actual.is_copy() {
                    format!("&({fallback})")
                } else {
                    fallback
                };
                format!("{prefix}{}::V{index}({})", target.index(),if target_view{code}else{self.union_store_payload(target,actual,&code)})
            }
        } else {
            fallback
        };
        let arms = self.program.unions[union.index()]
            .members
            .iter()
            .enumerate()
            .map(|(index, member)| {
                let selected = if !self.program.unions[logical_union.index()]
                    .members
                    .contains(member)
                {
                    "unreachable!(\"checked nullable guard\")".to_owned()
                } else if *member==Type::Undefined || (!undefined_only && *member==Type::Null) {
                    fallback.clone()
                } else if let Some(target) = result.union() {
                    let to = self.program.unions[target.index()]
                        .members
                        .iter()
                        .position(|ty| {
                            ty == member
                                || matches!(ty, Type::Callback(_))
                                    && matches!(member, Type::Closure(_) | Type::Callback(_))
                        })
                        .expect("retained payload variant");
                    let payload = if target_view { self.data_view_payload(*member,"payload",borrowed) } else if borrowed && target.copy { "*payload".to_owned() } else { self.union_transfer_payload(union,target,*member,"payload") };
                    format!("{}{}::V{to}({payload})",if target_view { "ViewUnion" } else { "Union" },target.index())
                } else if !borrowed && access==Mode::Consume {
                    self.union_move_payload(union,*member,"payload")
                } else if access==Mode::Mutate && self.boxed_union_member(union,*member) {
                    "&mut **payload".to_owned()
                } else { "payload".to_owned() };
                format!(
                    "{}{}::V{index}(payload)=>{selected}",
                    if borrowed { "ViewUnion" } else { "Union" },
                    union.index()
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!("match {source} {{{arms}}}")
    }

    pub(super) fn prepare_pattern_fallback_work(&self, right: &Expr, mode: Mode) -> (Mode, Option<usize>) {
        if mode == Mode::Read && Self::temporary_storage(right) && !right.ty.unwrap().is_copy()
            && !matches!(right.ty, Some(Type::BorrowedUnion(_))) {
            let slot = self.conditional_next.get(); self.conditional_next.set(slot + 1);
            let annotation = if right.ty.unwrap().union().is_some() { format!(": {}", self.native_type(right.ty.unwrap())) } else { String::new() };
            self.conditional_temporaries.borrow_mut().push(format!("let pattern_default_owner_{slot}{annotation};"));
            (Mode::Consume, Some(slot))
        } else { (mode, None) }
    }

    pub(super) fn finish_pattern_fallback_work(&self, right: &Expr, slot: Option<usize>, owner: String) -> String {
        let Some(slot) = slot else { return owner; };
        let view = if right.ty.unwrap().union().is_some() {
            format!("pattern_default_owner_{slot}.{}()", if self.callback_payload(right.ty.unwrap()).is_some() { "tsr_view" } else { "tsr_data_view" })
        } else { format!("&pattern_default_owner_{slot}") };
        format!("{{pattern_default_owner_{slot}={owner};{view}}}")
    }
}
