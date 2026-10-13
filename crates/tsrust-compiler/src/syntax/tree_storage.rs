//! Release expression and statement ownership edges without recursive destruction.
use super::*;

enum Owned { Expression(Expr), Statement(Stmt), Function(Function), Callback(CallbackAnnotation), Result(ResultAnnotation) }
fn initializer(value: ParameterInitializer, pending: &mut Vec<Owned>) {
    if let ParameterInitializer::Default { value, .. } = value { pending.push(Owned::Expression(*value)); }
}
fn parameters(values: Vec<CallableParameter>, pending: &mut Vec<Owned>) {
    for value in values {
        initializer(value.initializer, pending);
        if let ParameterAnnotation::Callback(callback) = value.ty { pending.push(Owned::Callback(*callback)); }
    }
}
fn result(value: &mut ResultAnnotation, pending: &mut Vec<Owned>) {
    if let ResultAnnotation::Callable { params, result, .. } = value {
        parameters(std::mem::take(params), pending);
        pending.push(Owned::Result(std::mem::replace(result.as_mut(), ResultAnnotation::Inferred(Span::default()))));
    }
}
fn expression(kind: ExprKind, pending: &mut Vec<Owned>) {
    match kind {
        ExprKind::SpreadCall(plan) => {
            if let Some(callee) = plan.callee { pending.push(Owned::Expression(*callee.value)); }
            pending.extend(plan.segments.into_iter().map(|value| Owned::Expression(*value.value)));
            pending.push(Owned::Expression(*plan.call));
            pending.extend(plan.variants.into_iter().map(|value| Owned::Expression(*value.call)));
        }
        ExprKind::OptionalChain(value) | ExprKind::Update { target: value, .. }
        | ExprKind::Unary { value, .. } | ExprKind::Inject { value, .. }
        | ExprKind::Project { value, .. } | ExprKind::TypeAssertion { value, .. }
        | ExprKind::Field(value, _) => pending.push(Owned::Expression(*value)),
        ExprKind::Optional { base, body, .. } => {
            pending.push(Owned::Expression(*base)); pending.push(Owned::Expression(*body));
        }
        ExprKind::Conditional { predicate, consequent, alternate } => {
            pending.push(Owned::Expression(*predicate)); pending.push(Owned::Expression(*consequent)); pending.push(Owned::Expression(*alternate));
        }
        ExprKind::CompoundAssign { target, value, .. } | ExprKind::CoalesceAssign { target, value, .. }
        | ExprKind::Assign { target, value, .. } | ExprKind::LogicalAssign { target, value, .. } => {
            pending.push(Owned::Expression(*target)); pending.push(Owned::Expression(*value));
        }
        ExprKind::Binary { left, right, .. } | ExprKind::Index(left, right) => {
            pending.push(Owned::Expression(*left)); pending.push(Owned::Expression(*right));
        }
        ExprKind::Closure { params, result, body, .. } => {
            parameters(params, pending); pending.push(Owned::Result(result));
            pending.extend(body.into_iter().map(Owned::Statement));
        }
        ExprKind::Record(values) => pending.extend(values.into_iter().map(|(_, value)| Owned::Expression(value))),
        ExprKind::Array(values) | ExprKind::Template(values) | ExprKind::Sequence(values)
        | ExprKind::ArrayOf(values, _) | ExprKind::SetConstructor(values, _)
        | ExprKind::MapConstructor(values, _) | ExprKind::NativeConstructor(values, _) => pending.extend(values.into_iter().map(Owned::Expression)),
        ExprKind::Call(callee, arguments) => {
            pending.push(Owned::Expression(*callee)); pending.extend(arguments.into_iter().map(Owned::Expression));
        }
        ExprKind::SpreadValue(_) | ExprKind::Selected { .. } | ExprKind::EnumMember { .. }
        | ExprKind::Name(_) | ExprKind::String(_) | ExprKind::Integer(_) | ExprKind::Number(_)
        | ExprKind::Boolean(_) | ExprKind::Null | ExprKind::Undefined => {},
    }
}
fn statement(kind: StmtKind, pending: &mut Vec<Owned>) {
    match kind {
        StmtKind::LocalFunction(value) => pending.push(Owned::Function(*value)),
        StmtKind::Try(region) => {
            pending.extend(region.body.into_iter().map(Owned::Statement));
            if let Some(catch) = region.catch { pending.extend(catch.body.into_iter().map(Owned::Statement)); }
            if let Some(finally) = region.finally { pending.extend(finally.into_iter().map(Owned::Statement)); }
        }
        StmtKind::Throw(value) | StmtKind::Let { value, .. } | StmtKind::Return(Some(value)) | StmtKind::Expr(value) => pending.push(Owned::Expression(value)),
        StmtKind::Block(body) => pending.extend(body.into_iter().map(Owned::Statement)),
        StmtKind::Loop { condition, body, update, .. } => {
            pending.extend(condition.into_iter().map(Owned::Expression));
            pending.extend(body.into_iter().chain(update).map(Owned::Statement));
        }
        StmtKind::ForOf { iterable, body, .. } | StmtKind::While { condition: iterable, body } => {
            pending.push(Owned::Expression(iterable)); pending.extend(body.into_iter().map(Owned::Statement));
        }
        StmtKind::If { condition, then_body, else_body } => {
            pending.push(Owned::Expression(condition)); pending.extend(then_body.into_iter().chain(else_body).map(Owned::Statement));
        }
        StmtKind::Assign { target, value, .. } => {
            pending.push(Owned::Expression(target)); pending.push(Owned::Expression(value));
        }
        StmtKind::Break | StmtKind::Continue | StmtKind::Return(None) => {},
    }
}
fn release(mut pending: Vec<Owned>) {
    while let Some(value) = pending.pop() {
        match value {
            Owned::Expression(mut value) => expression(std::mem::replace(&mut value.kind, ExprKind::Undefined), &mut pending),
            Owned::Statement(mut value) => statement(std::mem::replace(&mut value.kind, StmtKind::Break), &mut pending),
            Owned::Function(value) => {
                pending.extend(value.body.into_iter().map(Owned::Statement));
                for value in value.parameter_initializers { initializer(value, &mut pending); }
                for (_, _, callback) in value.params { if let Some(callback) = callback { pending.push(Owned::Callback(callback)); } }
                if let Some(result) = value.result { pending.push(Owned::Result(result)); }
            }
            Owned::Callback(mut value) => {
                parameters(std::mem::take(&mut value.params), &mut pending);
                result(&mut value.result, &mut pending);
            }
            Owned::Result(mut value) => result(&mut value, &mut pending),
        }
    }
}
impl Expr {
    pub(crate) fn take_kind(&mut self) -> ExprKind { std::mem::replace(&mut self.kind, ExprKind::Undefined) }
    pub(crate) fn into_kind(mut self) -> ExprKind { std::mem::replace(&mut self.kind, ExprKind::Undefined) }
}
impl Stmt {
    pub(crate) fn into_kind(mut self) -> StmtKind { std::mem::replace(&mut self.kind, StmtKind::Break) }
}
impl Drop for Expr {
    fn drop(&mut self) {
        if matches!(self.kind, ExprKind::Undefined) { return; }
        let mut pending = Vec::new();
        expression(std::mem::replace(&mut self.kind, ExprKind::Undefined), &mut pending);
        release(pending);
    }
}
impl Drop for Stmt {
    fn drop(&mut self) {
        if matches!(self.kind, StmtKind::Break) { return; }
        let mut pending = Vec::new();
        statement(std::mem::replace(&mut self.kind, StmtKind::Break), &mut pending);
        release(pending);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn leaf() -> Expr { Expr::new(ExprKind::Integer("7".into()), Span::new(1, 2)) }
    fn statement(kind: StmtKind) -> Stmt { Stmt { kind, labels: Vec::new(), control_target: None, span: Span::new(1, 2) } }
    fn parameter(ty: ParameterAnnotation, initializer: ParameterInitializer) -> CallableParameter {
        CallableParameter { rest: false, name: Token { kind: TokenKind::Name, text: "value".into(), span: Span::new(1, 2) }, ty, access: None, initializer }
    }
    #[test]
    fn deep_expression_closure_and_annotation_defaults_release_on_normal_stack() {
        let span = Span::new(1, 2);
        let mut value = leaf();
        for index in 0..16_384 {
            value = if index % 3 == 0 {
                Expr::new(ExprKind::Index(Box::new(value), Box::new(leaf())), span)
            } else if index % 3 == 1 {
                Expr::new(ExprKind::Closure { id: index, params: Vec::new(), result: ResultAnnotation::Inferred(span),
                    body: vec![statement(StmtKind::Try(Box::new(ExceptionRegion { body: vec![statement(StmtKind::Return(Some(value)))], catch: None, finally: Some(vec![statement(StmtKind::Expr(leaf()))]) })))] }, span)
            } else {
                let callback = CallbackAnnotation { kind: CallbackKind::Owned, span,
                    params: vec![parameter(ParameterAnnotation::Named(Box::new(crate::type_syntax::parse("int").unwrap())), ParameterInitializer::Default { value: Box::new(value), span })],
                    result: Box::new(ResultAnnotation::Inferred(span)) };
                Expr::new(ExprKind::Closure { id: index, params: vec![parameter(ParameterAnnotation::Callback(Box::new(callback)), ParameterInitializer::Required)], result: ResultAnnotation::Inferred(span), body: Vec::new() }, span)
            };
        }
        drop(value);
    }
    #[test]
    fn deep_blocks_and_local_function_signatures_release_on_normal_stack() {
        let span = Span::new(1, 2);
        let template = parse("function main(){}").unwrap().functions.remove(0);
        let mut value = statement(StmtKind::Expr(leaf()));
        for index in 0..16_384 {
            if index % 2 == 0 { value = statement(StmtKind::Block(vec![value])); }
            else {
                let mut function = template.clone();
                function.parameter_initializers = vec![ParameterInitializer::Default { value: Box::new(Expr::new(ExprKind::Closure {
                    id: index, params: Vec::new(), result: ResultAnnotation::Inferred(span), body: vec![value],
                }, span)), span }];
                value = statement(StmtKind::LocalFunction(Box::new(function)));
            }
        }
        drop(value);
    }
    #[test]
    fn extraction_preserves_live_children_and_remaining_metadata() {
        let span = Span::new(8, 13);
        let mut value = Expr::checked(ExprKind::Array(vec![leaf()]), span, Some(Type::Int));
        value.binding = Some(19);
        let ExprKind::Array(children) = value.take_kind() else { unreachable!() };
        assert!(matches!(value.kind, ExprKind::Undefined));
        assert_eq!(value.span, span);
        assert_eq!(value.ty, Some(Type::Int));
        assert_eq!(value.binding, Some(19));
        assert!(matches!(&children[0].kind, ExprKind::Integer(text) if text == "7"));
        let StmtKind::Expr(value) = statement(StmtKind::Expr(children.into_iter().next().unwrap())).into_kind() else { unreachable!() };
        assert!(matches!(value.into_kind(), ExprKind::Integer(text) if text == "7"));
    }
}
