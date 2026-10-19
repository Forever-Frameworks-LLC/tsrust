//! One construction worklist spans bodies, signatures and default expressions.
use super::*;

#[derive(Clone, Copy)]
enum Source<'a> { Expr(&'a Expr), Stmt(&'a Stmt), Function(&'a Function), Callback(&'a CallbackAnnotation), Result(&'a ResultAnnotation) }
enum Built { Expr(Expr), Stmt(Stmt), Function(Function), Callback(CallbackAnnotation), Result(ResultAnnotation) }
struct Values<'a>(std::vec::Drain<'a, Built>);
impl Values<'_> {
    fn expr(&mut self) -> Expr { let Some(Built::Expr(value)) = self.0.next() else { unreachable!("cloned expression") }; value }
    fn boxed(&mut self) -> Box<Expr> { Box::new(self.expr()) }
    fn exprs(&mut self, count: usize) -> Vec<Expr> { (0..count).map(|_| self.expr()).collect() }
    fn stmts(&mut self, count: usize) -> Vec<Stmt> { (0..count).map(|_| { let Some(Built::Stmt(value)) = self.0.next() else { unreachable!("cloned statement") }; value }).collect() }
    fn callback(&mut self) -> CallbackAnnotation { let Some(Built::Callback(value)) = self.0.next() else { unreachable!("cloned callback") }; value }
    fn result(&mut self) -> ResultAnnotation { let Some(Built::Result(value)) = self.0.next() else { unreachable!("cloned result") }; value }
    fn initializer(&mut self, value: &ParameterInitializer) -> ParameterInitializer {
        match value {
            ParameterInitializer::Required => ParameterInitializer::Required,
            ParameterInitializer::Optional(span) => ParameterInitializer::Optional(*span),
            ParameterInitializer::Default { span, .. } => ParameterInitializer::Default { value: self.boxed(), span: *span },
        }
    }
    fn parameters(&mut self, values: &[CallableParameter]) -> Vec<CallableParameter> {
        values.iter().map(|value| {
            let ty = match &value.ty {
                ParameterAnnotation::Named(value) => ParameterAnnotation::Named(value.clone()),
                ParameterAnnotation::Inferred(value) => ParameterAnnotation::Inferred(value.clone()),
                ParameterAnnotation::Callback(_) => ParameterAnnotation::Callback(Box::new(self.callback())),
            };
            CallableParameter { rest: value.rest, name: value.name.clone(), ty, access: value.access.clone(), initializer: self.initializer(&value.initializer) }
        }).collect()
    }
}
fn initializer<'a>(value: &'a ParameterInitializer, children: &mut Vec<Source<'a>>) {
    if let ParameterInitializer::Default { value, .. } = value { children.push(Source::Expr(value)); }
}
fn parameters<'a>(values: &'a [CallableParameter], children: &mut Vec<Source<'a>>) {
    for value in values {
        if let ParameterAnnotation::Callback(callback) = &value.ty { children.push(Source::Callback(callback)); }
        initializer(&value.initializer, children);
    }
}
fn children(source: Source<'_>) -> Vec<Source<'_>> {
    let mut out = Vec::new();
    match source {
        Source::Expr(value) => match &value.kind {
            ExprKind::SpreadCall(plan) => out.extend(plan.expressions().map(Source::Expr)),
            ExprKind::OptionalChain(value) | ExprKind::Update { target: value, .. }
            | ExprKind::Unary { value, .. } | ExprKind::Inject { value, .. }
            | ExprKind::Project { value, .. } | ExprKind::TypeAssertion { value, .. }
            | ExprKind::Field(value, _) => out.push(Source::Expr(value)),
            ExprKind::Optional { base, body, .. } => out.extend([Source::Expr(base), Source::Expr(body)]),
            ExprKind::Conditional { predicate, consequent, alternate } => out.extend([Source::Expr(predicate), Source::Expr(consequent), Source::Expr(alternate)]),
            ExprKind::CompoundAssign { target, value, .. } | ExprKind::CoalesceAssign { target, value, .. }
            | ExprKind::Assign { target, value, .. } | ExprKind::LogicalAssign { target, value, .. } => out.extend([Source::Expr(target), Source::Expr(value)]),
            ExprKind::Binary { left, right, .. } | ExprKind::Index(left, right) => out.extend([Source::Expr(left), Source::Expr(right)]),
            ExprKind::Match { body, .. } => out.extend(body.iter().map(Source::Stmt)),
            ExprKind::Closure { params, result, body, .. } => {
                parameters(params, &mut out); out.push(Source::Result(result)); out.extend(body.iter().map(Source::Stmt));
            }
            ExprKind::Record(values) => out.extend(values.iter().map(|(_, value)| Source::Expr(value))),
            ExprKind::Array(values) | ExprKind::Template(values) | ExprKind::Sequence(values)
            | ExprKind::ArrayOf(values, _) | ExprKind::SetConstructor(values, _)
            | ExprKind::MapConstructor(values, _) | ExprKind::NativeConstructor(values, _) => out.extend(values.iter().map(Source::Expr)),
            ExprKind::Call(callee, arguments) => { out.push(Source::Expr(callee)); out.extend(arguments.iter().map(Source::Expr)); }
            ExprKind::SpreadValue(_) | ExprKind::Selected { .. } | ExprKind::EnumMember { .. }
            | ExprKind::Name(_) | ExprKind::String(_) | ExprKind::Integer(_) | ExprKind::Number(_)
            | ExprKind::Boolean(_) | ExprKind::Null | ExprKind::Undefined => {},
        },
        Source::Stmt(value) => match &value.kind {
            StmtKind::LocalFunction(value) => out.push(Source::Function(value)),
            StmtKind::Try(region) => out.extend(region.statements().map(Source::Stmt)),
            StmtKind::Throw(value) | StmtKind::Let { value, .. } | StmtKind::Return(Some(value)) | StmtKind::Expr(value) => out.push(Source::Expr(value)),
            StmtKind::Block(body) => out.extend(body.iter().map(Source::Stmt)),
            StmtKind::Loop { condition, body, update, .. } => {
                out.extend(condition.iter().map(Source::Expr)); out.extend(body.iter().chain(update).map(Source::Stmt));
            }
            StmtKind::ForOf { iterable, body, .. } | StmtKind::While { condition: iterable, body } => { out.push(Source::Expr(iterable)); out.extend(body.iter().map(Source::Stmt)); }
            StmtKind::If { condition, then_body, else_body } => { out.push(Source::Expr(condition)); out.extend(then_body.iter().chain(else_body).map(Source::Stmt)); }
            StmtKind::Assign { target, value, .. } => out.extend([Source::Expr(target), Source::Expr(value)]),
            StmtKind::Break | StmtKind::Continue | StmtKind::Return(None) => {},
        },
        Source::Function(value) => {
            for (_, _, callback) in &value.params { if let Some(callback) = callback { out.push(Source::Callback(callback)); } }
            for value in &value.parameter_initializers { initializer(value, &mut out); }
            if let Some(result) = &value.result { out.push(Source::Result(result)); }
            out.extend(value.body.iter().map(Source::Stmt));
        }
        Source::Callback(value) => { parameters(&value.params, &mut out); out.push(Source::Result(&value.result)); }
        Source::Result(value) => if let ResultAnnotation::Callable { params, result, .. } = value { parameters(params, &mut out); out.push(Source::Result(result)); },
    }
    out
}
fn expression_kind(kind: &ExprKind, values: &mut Values<'_>) -> ExprKind {
    match kind {
        ExprKind::Match { id, body } => ExprKind::Match { id: *id, body: values.stmts(body.len()) },
        ExprKind::SpreadCall(plan) => {
            let callee = plan.callee.as_ref().map(|value| SpreadCallee { value: values.boxed(), access: value.access, selection: value.selection });
            let segments = plan.segments.iter().map(|value| SpreadSegment { value: values.boxed(), spread: value.spread, element: value.element, access: value.access, length: value.length }).collect();
            let call = values.boxed();
            let variants = plan.variants.iter().map(|value| SpreadCallVariant { selection: value.selection, count: value.count, call: values.boxed() }).collect();
            ExprKind::SpreadCall(Box::new(CheckedSpreadCall { callee, segments, call, parameters: plan.parameters.clone(), selected_types: plan.selected_types.clone(), selected_unions: plan.selected_unions.clone(), variants, slot: plan.slot, minimum: plan.minimum, maximum: plan.maximum, builtin: plan.builtin }))
        }
        ExprKind::OptionalChain(_) => ExprKind::OptionalChain(values.boxed()),
        ExprKind::Optional { slot, union, access, branch, owned, .. } => ExprKind::Optional { base: values.boxed(), body: values.boxed(), slot: *slot, union: *union, access: *access, branch: *branch, owned: *owned },
        ExprKind::Conditional { .. } => ExprKind::Conditional { predicate: values.boxed(), consequent: values.boxed(), alternate: values.boxed() },
        ExprKind::CompoundAssign { op, operator, .. } => ExprKind::CompoundAssign { target: values.boxed(), value: values.boxed(), op: *op, operator: *operator },
        ExprKind::CoalesceAssign { branch, operator, .. } => ExprKind::CoalesceAssign { target: values.boxed(), value: values.boxed(), branch: *branch, operator: *operator },
        ExprKind::Assign { operator, .. } => ExprKind::Assign { target: values.boxed(), value: values.boxed(), operator: *operator },
        ExprKind::LogicalAssign { and, operator, .. } => ExprKind::LogicalAssign { target: values.boxed(), value: values.boxed(), and: *and, operator: *operator },
        ExprKind::Update { increment, prefix, operator, .. } => ExprKind::Update { target: values.boxed(), increment: *increment, prefix: *prefix, operator: *operator },
        ExprKind::Unary { op, operator, .. } => ExprKind::Unary { value: values.boxed(), op: *op, operator: *operator },
        ExprKind::Binary { op, operator, .. } => ExprKind::Binary { left: values.boxed(), right: values.boxed(), op: *op, operator: *operator },
        ExprKind::Closure { id, params, body, .. } => ExprKind::Closure { id: *id, params: values.parameters(params), result: values.result(), body: values.stmts(body.len()) },
        ExprKind::Inject { union, .. } => ExprKind::Inject { value: values.boxed(), union: *union },
        ExprKind::Project { union, narrowed, .. } => ExprKind::Project { value: values.boxed(), union: *union, narrowed: *narrowed },
        ExprKind::Field(_, name) => ExprKind::Field(values.boxed(), name.clone()),
        ExprKind::Record(fields) => ExprKind::Record(fields.iter().map(|(name, _)| (name.clone(), values.expr())).collect()),
        ExprKind::Array(items) => ExprKind::Array(values.exprs(items.len())),
        ExprKind::Template(items) => ExprKind::Template(values.exprs(items.len())),
        ExprKind::Sequence(items) => ExprKind::Sequence(values.exprs(items.len())),
        ExprKind::ArrayOf(items, annotation) => ExprKind::ArrayOf(values.exprs(items.len()), annotation.clone()),
        ExprKind::SetConstructor(items, annotation) => ExprKind::SetConstructor(values.exprs(items.len()), annotation.clone()),
        ExprKind::MapConstructor(items, annotation) => ExprKind::MapConstructor(values.exprs(items.len()), annotation.clone()),
        ExprKind::NativeConstructor(items, annotation) => ExprKind::NativeConstructor(values.exprs(items.len()), annotation.clone()),
        ExprKind::TypeAssertion { annotation, satisfies, .. } => ExprKind::TypeAssertion { value: values.boxed(), annotation: annotation.clone(), satisfies: *satisfies },
        ExprKind::Index(_, _) => ExprKind::Index(values.boxed(), values.boxed()),
        ExprKind::Call(_, arguments) => ExprKind::Call(values.boxed(), values.exprs(arguments.len())),
        // These variants contain no AST ownership edges.
        ExprKind::SpreadValue(_) | ExprKind::Selected { .. } | ExprKind::EnumMember { .. }
        | ExprKind::Name(_) | ExprKind::String(_) | ExprKind::Integer(_) | ExprKind::Number(_)
        | ExprKind::Boolean(_) | ExprKind::Null | ExprKind::Undefined => kind.clone(),
    }
}
fn statement_kind(kind: &StmtKind, values: &mut Values<'_>) -> StmtKind {
    match kind {
        StmtKind::LocalFunction(_) => { let Some(Built::Function(value)) = values.0.next() else { unreachable!() }; StmtKind::LocalFunction(Box::new(value)) }
        StmtKind::Try(region) => {
            let body = values.stmts(region.body.len());
            let catch = region.catch.as_ref().map(|value| CatchClause { error_type: value.error_type, error_contract: value.error_contract.clone(), name: value.name.clone(), binding: value.binding, body: values.stmts(value.body.len()) });
            let finally = region.finally.as_ref().map(|value| values.stmts(value.len()));
            StmtKind::Try(Box::new(ExceptionRegion { body, catch, finally }))
        }
        StmtKind::Throw(_) => StmtKind::Throw(values.expr()),
        StmtKind::Let { rebindable, name, annotation, binding, .. } => StmtKind::Let { rebindable: *rebindable, name: name.clone(), annotation: annotation.clone(), value: values.expr(), binding: *binding },
        StmtKind::Return(value) => StmtKind::Return(value.as_ref().map(|_| values.expr())),
        StmtKind::Expr(_) => StmtKind::Expr(values.expr()),
        StmtKind::Block(body) => StmtKind::Block(values.stmts(body.len())),
        StmtKind::Loop { condition, body, update, post_test, tail_reachable } => StmtKind::Loop { condition: condition.as_ref().map(|_| values.expr()), body: values.stmts(body.len()), update: values.stmts(update.len()), post_test: *post_test, tail_reachable: *tail_reachable },
        StmtKind::ForOf { name, rebindable, body, binding, .. } => StmtKind::ForOf { name: name.clone(), rebindable: *rebindable, iterable: values.expr(), body: values.stmts(body.len()), binding: *binding },
        StmtKind::While { body, .. } => StmtKind::While { condition: values.expr(), body: values.stmts(body.len()) },
        StmtKind::If { then_body, else_body, .. } => StmtKind::If { condition: values.expr(), then_body: values.stmts(then_body.len()), else_body: values.stmts(else_body.len()) },
        StmtKind::Assign { operator, .. } => StmtKind::Assign { target: values.expr(), value: values.expr(), operator: *operator },
        StmtKind::Break => StmtKind::Break,
        StmtKind::Continue => StmtKind::Continue,
    }
}
fn finish(source: Source<'_>, values: &mut Values<'_>) -> Built {
    match source {
        Source::Expr(value) => Built::Expr(Expr { kind: expression_kind(&value.kind, values), span: value.span, ty: value.ty, binding: value.binding, target: value.target,
            schema_invocations: value.schema_invocations.clone(), json_string_view: value.json_string_view, moved_spread:value.moved_spread, supplied_count: value.supplied_count, traversal_method: value.traversal_method,
            may_fail: value.may_fail, own_may_fail: value.own_may_fail, effects_checked: value.effects_checked, callable_origin: value.callable_origin,
            generic_invocation: value.generic_invocation, capture_slots: value.capture_slots.clone(), depth: value.depth }),
        Source::Stmt(value) => Built::Stmt(Stmt { labels: value.labels.clone(), control_target: value.control_target, kind: statement_kind(&value.kind, values), span: value.span }),
        Source::Function(value) => {
            let params = value.params.iter().map(|(name, annotation, callback)| (name.clone(), annotation.clone(), callback.as_ref().map(|_| values.callback()))).collect();
            let parameter_initializers = value.parameter_initializers.iter().map(|value| values.initializer(value)).collect();
            let result = value.result.as_ref().map(|_| values.result());
            Built::Function(Function { application: value.application.clone(), scope_span: value.scope_span, name: value.name.clone(), type_parameters: value.type_parameters.clone(), params, parameter_initializers,
                parameter_access: value.parameter_access.clone(), rest_parameter: value.rest_parameter, result, body: values.stmts(value.body.len()) })
        }
        Source::Callback(value) => Built::Callback(CallbackAnnotation { kind: value.kind, params: values.parameters(&value.params), result: Box::new(values.result()), span: value.span }),
        Source::Result(value) => Built::Result(match value {
            ResultAnnotation::Inferred(span) => ResultAnnotation::Inferred(*span),
            ResultAnnotation::Named(value) => ResultAnnotation::Named(value.clone()),
            ResultAnnotation::Callable { kind, params, span, .. } => ResultAnnotation::Callable { kind: *kind, params: values.parameters(params), result: Box::new(values.result()), span: *span },
        }),
    }
}
fn clone_tree(source: Source<'_>) -> Built {
    enum Work<'a> { Visit(Source<'a>), Finish(Source<'a>, usize) }
    let mut pending = vec![Work::Visit(source)];
    let mut built = Vec::new();
    while let Some(work) = pending.pop() {
        match work {
            Work::Visit(source) => {
                pending.push(Work::Finish(source, built.len()));
                pending.extend(children(source).into_iter().rev().map(Work::Visit));
            }
            Work::Finish(source, start) => {
                let mut values = Values(built.drain(start..));
                let value = finish(source, &mut values);
                debug_assert!(values.0.next().is_none(), "unused cloned child");
                drop(values);
                built.push(value);
            }
        }
    }
    built.pop().unwrap()
}
impl Clone for Expr { fn clone(&self) -> Self { let Built::Expr(value) = clone_tree(Source::Expr(self)) else { unreachable!() }; value } }
impl Clone for Stmt { fn clone(&self) -> Self { let Built::Stmt(value) = clone_tree(Source::Stmt(self)) else { unreachable!() }; value } }
impl Clone for Function { fn clone(&self) -> Self { let Built::Function(value) = clone_tree(Source::Function(self)) else { unreachable!() }; value } }
impl Clone for CallbackAnnotation { fn clone(&self) -> Self { let Built::Callback(value) = clone_tree(Source::Callback(self)) else { unreachable!() }; value } }
impl Clone for ResultAnnotation { fn clone(&self) -> Self { let Built::Result(value) = clone_tree(Source::Result(self)) else { unreachable!() }; value } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clone_keeps_checked_metadata_and_does_not_recompute_depth() {
        let span = Span::new(8, 17);
        let mut value = Expr::checked(ExprKind::Array(vec![Expr::new(ExprKind::Integer("7".into()), span)]), span, Some(Type::Int));
        value.binding = Some(9);
        value.target = Some(CallTarget::Function(12));
        value.schema_invocations = Some(vec![4, 2].into_boxed_slice());
        value.json_string_view = true;
        value.supplied_count = Some(CallArgumentCount::Static(3));
        value.traversal_method = Some(TraversalMethod { id: 14, access: crate::Mode::Mutate });
        value.may_fail = true; value.own_may_fail = true; value.effects_checked = true;
        value.callable_origin = Some(CallableOrigin::Storage(5));
        value.generic_invocation = Some(20); value.capture_slots = Some(vec![7, 3].into_boxed_slice());
        value.depth = 93;
        let mut copied = value.clone();
        assert_eq!(format!("{value:?}"), format!("{copied:?}"));
        assert_eq!(copied.depth, 93);
        let ExprKind::Array(items) = &mut copied.kind else { unreachable!() };
        items[0].kind = ExprKind::Boolean(false);
        let ExprKind::Array(items) = &value.kind else { unreachable!() };
        assert!(matches!(&items[0].kind, ExprKind::Integer(text) if text == "7"));
    }
    #[test]
    fn bounded_source_and_checked_bodies_preserve_every_debug_field() {
        for source in [
            "function outer(value:int=3):int{function inner(x:int=value):int{return x+1;}try{if(value>0){return inner();}else{throw value;}}catch(error){return 0;}finally{console.log(value);}}function main(){console.log(outer());}",
            "function main(){const values=[1,2,3];let n=0;for(const value of values){n+=value;}while(n>0){n-=1;if(n===2){continue;}if(n===1){break;}}const action=(value:int):int=>{return value+1;};console.log(action(n));}",
            "function add(a:int,b:int):int{return a+b;}function main(){const args=[1,2];console.log(add(...args));}",
        ] {
            let parsed = parse(source).unwrap();
            for function in &parsed.functions { assert_eq!(format!("{function:?}"), format!("{:?}", function.clone())); }
            let checked = crate::check(source).unwrap();
            for function in &checked.program.functions { assert_eq!(format!("{:?}", function.body), format!("{:?}", function.body.clone())); }
        }
    }
    #[test]
    fn deep_mixed_clone_keeps_each_owned_child_and_source_location() {
        let mut value = Expr::new(ExprKind::Name("leaf".into()), Span::new(0, 1));
        for index in 0..16_384 {
            let span = Span::new(index + 1, index + 2);
            value = Expr::new(ExprKind::Closure { id: index, params: vec![CallableParameter {
                rest: false, name: Token { kind: TokenKind::Name, text: "value".into(), span },
                ty: ParameterAnnotation::Inferred(Token { kind: TokenKind::Name, text: "value".into(), span }), access: None,
                initializer: ParameterInitializer::Default { value: Box::new(value), span },
            }], result: ResultAnnotation::Inferred(span), body: Vec::new() }, span);
        }
        let copied = value.clone();
        let mut current = &copied;
        for index in (0..16_384).rev() {
            assert_eq!(current.span, Span::new(index + 1, index + 2));
            let ExprKind::Closure { id, params, result, body } = &current.kind else { unreachable!() };
            assert_eq!(*id, index); assert!(body.is_empty());
            assert!(matches!(result, ResultAnnotation::Inferred(span) if *span == current.span));
            let ParameterInitializer::Default { value, span } = &params[0].initializer else { unreachable!() };
            assert_eq!(*span, current.span); current = value;
        }
        assert!(matches!(&current.kind, ExprKind::Name(name) if name == "leaf"));
        drop(copied); drop(value);
    }
}
