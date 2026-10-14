//! Heap continuations retain regional failure contracts without a source-depth cutoff.
use super::*;

pub(super) fn expression_effect(root: &mut Expr, effects: &ErrorEffects) -> FailureSet {
    enum Work<'a> {
        Visit(&'a mut Expr),
        Finish { may_fail: &'a mut bool, own: FailureSet, start: usize, reverse: bool },
    }
    let mut pending = vec![Work::Visit(root)];
    let mut results: Vec<FailureSet> = Vec::new();
    while let Some(work) = pending.pop() {
        match work {
            Work::Finish { may_fail, mut own, start, reverse } => {
                // List folds historically prepend each child's alternatives. Preserve
                // that payload order as well as left-to-right point annotation.
                if reverse { for child in results.drain(start..).rev() { own |= child; } }
                else { for child in results.drain(start..) { own |= child; } }
                *may_fail = own.may_fail();
                results.push(own);
            }
            Work::Visit(value) => {
                let own = if let ExprKind::Match { body, .. } = &mut value.kind { body_effect(body, effects) } else { own_expression_effect(value, effects) };
                value.own_may_fail = own.may_fail();
                value.effects_checked = true;
                let reverse = matches!(&value.kind, ExprKind::SpreadCall(_) | ExprKind::Array(_) | ExprKind::ArrayOf(..) | ExprKind::SetConstructor(..) | ExprKind::MapConstructor(..) | ExprKind::NativeConstructor(..) | ExprKind::Template(_) | ExprKind::Sequence(_) | ExprKind::Record(_));
                pending.push(Work::Finish { may_fail: &mut value.may_fail, own, start: results.len(), reverse });
                let children = pending.len();
                match &mut value.kind {
                    ExprKind::SpreadCall(plan) => pending.extend(plan.expressions_mut().map(Work::Visit)),
                    ExprKind::TypeAssertion { value, .. } | ExprKind::OptionalChain(value) | ExprKind::Unary { value, .. } | ExprKind::Inject { value, .. } | ExprKind::Project { value, .. } | ExprKind::Update { target: value, .. } | ExprKind::Field(value, _) => pending.push(Work::Visit(value)),
                    ExprKind::Optional { base, body, branch, .. } => {
                        pending.push(Work::Visit(base));
                        if *branch != NullishBranch::Absent { pending.push(Work::Visit(body)); }
                    }
                    ExprKind::Index(base, index) | ExprKind::Binary { left: base, right: index, .. } => pending.extend([Work::Visit(base), Work::Visit(index)]),
                    ExprKind::Assign { target, value, .. } | ExprKind::CompoundAssign { target, value, .. } | ExprKind::CoalesceAssign { target, value, .. } | ExprKind::LogicalAssign { target, value, .. } => pending.extend([Work::Visit(target), Work::Visit(value)]),
                    ExprKind::Conditional { predicate, consequent, alternate } => pending.extend([Work::Visit(predicate), Work::Visit(consequent), Work::Visit(alternate)]),
                    ExprKind::Call(callee, args) => { pending.push(Work::Visit(callee)); pending.extend(args.iter_mut().map(Work::Visit)); }
                    ExprKind::Array(values) | ExprKind::ArrayOf(values, _) | ExprKind::SetConstructor(values, _) | ExprKind::MapConstructor(values, _) | ExprKind::NativeConstructor(values, _) | ExprKind::Template(values) | ExprKind::Sequence(values) => pending.extend(values.iter_mut().map(Work::Visit)),
                    ExprKind::Record(fields) => pending.extend(fields.iter_mut().map(|(_, value)| Work::Visit(value))),
                    // Creating a closure does not execute its source body.
                    ExprKind::Match { .. } | ExprKind::Closure { .. } | ExprKind::SpreadValue(_) | ExprKind::Selected { .. } | ExprKind::Name(_) | ExprKind::String(_) | ExprKind::Integer(_) | ExprKind::EnumMember { .. } | ExprKind::Number(_) | ExprKind::Boolean(_) | ExprKind::Null | ExprKind::Undefined => {}
                }
                pending[children..].reverse();
            }
        }
    }
    results.pop().expect("root expression effect")
}

pub(super) fn body_effect(root: &mut [Stmt], effects: &ErrorEffects) -> FailureSet {
    enum Work<'a> {
        Body(&'a mut [Stmt]),
        Statement(&'a mut Stmt),
        Expression(&'a mut Expr),
        Join(usize),
        AfterTry(Option<&'a mut CatchClause>, Option<&'a mut Vec<Stmt>>),
        AfterCatch(bool, Option<&'a mut Vec<Stmt>>),
        Cleanup(Option<&'a mut Vec<Stmt>>),
        AfterCleanup(FailureSet, bool),
    }
    let mut pending = vec![Work::Body(root)];
    let mut results: Vec<FailureSet> = Vec::new();
    while let Some(work) = pending.pop() {
        match work {
            Work::Body(body) => {
                pending.push(Work::Join(body.len()));
                pending.extend(body.iter_mut().rev().map(Work::Statement));
            }
            Work::Expression(value) => results.push(expression_effect(value, effects)),
            Work::Join(count) => {
                let mut found = FailureSet::default();
                for child in results.drain(results.len() - count..) { found |= child; }
                results.push(found);
            }
            Work::AfterTry(handler, cleanup) => {
                if let Some(handler) = handler {
                    let incoming = results.pop().expect("try body effect");
                    let reachable = incoming.may_fail();
                    handler.error_contract = Some(Box::new(incoming));
                    pending.push(Work::AfterCatch(reachable, cleanup));
                    pending.push(Work::Body(&mut handler.body));
                } else { pending.push(Work::Cleanup(cleanup)); }
            }
            Work::AfterCatch(reachable, cleanup) => {
                if !reachable { *results.last_mut().expect("catch body effect") = FailureSet::default(); }
                pending.push(Work::Cleanup(cleanup));
            }
            Work::Cleanup(cleanup) => {
                if let Some(body) = cleanup {
                    let found = results.pop().expect("outgoing region effect");
                    let abrupt = crate::completion::definitely_abrupt(body);
                    pending.push(Work::AfterCleanup(found, abrupt));
                    pending.push(Work::Body(body));
                }
            }
            Work::AfterCleanup(found, abrupt) => {
                let cleanup = results.pop().expect("cleanup effect");
                results.push(if abrupt { cleanup } else { found | cleanup });
            }
            Work::Statement(statement) => {
                let operation_failure = statement_operation_effect(statement).is_some_and(|(_, fails)| fails);
                match &mut statement.kind {
                    StmtKind::LocalFunction(_) => unreachable!("local declarations must be resolved before checking"),
                    StmtKind::Throw(value) => results.push(expression_effect(value, effects) | FailureSet::thrown(value.ty)),
                    StmtKind::Try(region) => {
                        pending.push(Work::AfterTry(region.catch.as_mut(), region.finally.as_mut()));
                        pending.push(Work::Body(&mut region.body));
                    }
                    StmtKind::Let { value, .. } | StmtKind::Expr(value) => pending.push(Work::Expression(value)),
                    StmtKind::Return(value) => results.push(value.as_mut().map(|value| expression_effect(value, effects)).unwrap_or_default()),
                    StmtKind::Assign { target, value, .. } => results.push((if operation_failure { FailureSet::runtime() } else { FailureSet::default() }) | expression_effect(target, effects) | expression_effect(value, effects)),
                    StmtKind::Block(body) => pending.push(Work::Body(body)),
                    StmtKind::If { condition, then_body, else_body } => {
                        pending.extend([Work::Join(3), Work::Body(else_body), Work::Body(then_body), Work::Expression(condition)]);
                    }
                    StmtKind::While { condition, body } | StmtKind::ForOf { iterable: condition, body, .. } => {
                        pending.extend([Work::Join(2), Work::Body(body), Work::Expression(condition)]);
                    }
                    StmtKind::Loop { condition, body, update, .. } => {
                        pending.extend([Work::Join(2 + usize::from(condition.is_some())), Work::Body(update), Work::Body(body)]);
                        if let Some(condition) = condition { pending.push(Work::Expression(condition)); }
                    }
                    StmtKind::Break | StmtKind::Continue => results.push(FailureSet::default()),
                }
            }
        }
    }
    results.pop().expect("root body effect")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn statement(kind: StmtKind) -> Stmt {
        Stmt { kind, span: crate::Span::new(11, 19), labels: vec![], control_target: None }
    }

    #[test]
    fn deep_expression_effects_keep_known_payloads_and_every_point() {
        let span = crate::Span::new(11, 19);
        let mut value = Expr::checked(ExprKind::Boolean(true), span, Some(Type::Boolean));
        value.target = Some(CallTarget::Function(0));
        let effects = ErrorEffects { function_errors: vec![FailureSet::thrown(Some(Type::String))], ..Default::default() };
        for _ in 0..16_384 {
            value = Expr::checked(ExprKind::Unary { op: UnaryOp::Not, operator: span, value: Box::new(value) }, span, Some(Type::Boolean));
        }
        assert_eq!(expression_effect(&mut value, &effects), FailureSet::thrown(Some(Type::String)));
        let mut count = 0;
        let mut current = &value;
        loop {
            assert!(current.effects_checked && current.may_fail);
            assert_eq!(current.span, span);
            if let ExprKind::Unary { value: child, .. } = &current.kind {
                assert!(!current.own_may_fail);
                current = child;
                count += 1;
            } else { assert!(current.own_may_fail); break; }
        }
        assert_eq!(count, 16_384);
    }

    #[test]
    fn deep_regions_preserve_catch_payload_and_handler_reachability() {
        let thrown = Expr::checked(ExprKind::Integer("7".into()), crate::Span::new(11, 19), Some(Type::Int));
        let mut body = vec![statement(StmtKind::Throw(thrown))];
        for depth in 0..16_384 {
            body = vec![statement(if depth % 2 == 0 { StmtKind::Block(body) } else {
                StmtKind::Try(Box::new(ExceptionRegion { body, catch: None, finally: None }))
            })];
        }
        let handler = CatchClause { name: None, binding: None, error_type: None, error_contract: None, body: vec![] };
        let mut root = vec![statement(StmtKind::Try(Box::new(ExceptionRegion { body, catch: Some(handler), finally: None })))];
        assert_eq!(body_effect(&mut root, &ErrorEffects::default()), FailureSet::default());
        let StmtKind::Try(region) = &root[0].kind else { unreachable!() };
        assert_eq!(region.catch.as_ref().unwrap().error_contract.as_deref(), Some(&FailureSet::thrown(Some(Type::Int))));
        drop(root);
    }

    #[test]
    fn deep_abrupt_cleanup_replaces_pending_failure() {
        let mut cleanup = vec![statement(StmtKind::Return(None))];
        for _ in 0..16_384 { cleanup = vec![statement(StmtKind::Block(cleanup))]; }
        assert!(crate::completion::definitely_abrupt(&cleanup));
        let thrown = Expr::checked(ExprKind::Integer("7".into()), crate::Span::new(11, 19), Some(Type::Int));
        let mut root = vec![statement(StmtKind::Try(Box::new(ExceptionRegion {
            body: vec![statement(StmtKind::Throw(thrown))], catch: None, finally: Some(cleanup),
        })))];
        assert_eq!(body_effect(&mut root, &ErrorEffects::default()), FailureSet::default());
        drop(root);
    }

    #[test]
    fn list_payload_order_and_absent_optional_points_are_preserved() {
        let span = crate::Span::new(11, 19);
        let leaf = |id| { let mut value = Expr::checked(ExprKind::Integer("7".into()), span, Some(Type::Int)); value.target = Some(CallTarget::Function(id)); value };
        let effects = ErrorEffects { function_errors: vec![FailureSet::thrown(Some(Type::Int)), FailureSet::thrown(Some(Type::String))], ..Default::default() };
        let mut list = Expr::checked(ExprKind::Sequence(vec![leaf(0), leaf(1)]), span, Some(Type::Int));
        assert_eq!(expression_effect(&mut list, &effects).explicit, vec![Type::String, Type::Int]);
        // Parse only the shallow optional wrapper, retaining its real metadata.
        let mut checked = crate::check("function main(){const value:int[]|null=null;value?.length;}").unwrap();
        let main = checked.program.functions.iter_mut().find(|function|function.name=="main").unwrap();
        let StmtKind::Expr(value) = &mut main.body[1].kind else { panic!("optional expression") };
        let ExprKind::Optional { body, branch, .. } = &mut value.kind else { panic!("checked optional") };
        *branch = NullishBranch::Absent;
        **body = leaf(0);
        assert_eq!(expression_effect(value, &effects), FailureSet::default());
        let ExprKind::Optional { body, .. } = &value.kind else { unreachable!() };
        assert!(!body.effects_checked, "an absent branch must remain unvisited");
    }
}
