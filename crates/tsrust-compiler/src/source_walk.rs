//! Iterative source body traversal shared by template transformation passes.
use crate::syntax::*;
use crate::type_syntax::ValueAnnotation;
use crate::Diagnostic;

enum AnnotationWork<'a> {
    Parameter(&'a mut CallableParameter),
    Callback(&'a mut CallbackAnnotation),
    Result(&'a mut ResultAnnotation),
}

/// Visit callable parameters left to right, then their result. The worklist
/// holds the continuation instead of retaining one Rust frame per signature.
fn annotations(
    root: AnnotationWork<'_>,
    visit: &mut impl FnMut(&mut ValueAnnotation) -> Result<(), Diagnostic>,
) -> Result<(), Diagnostic> {
    let mut pending = vec![root];
    while let Some(next) = pending.pop() {
        match next {
            AnnotationWork::Parameter(parameter) => match &mut parameter.ty {
                ParameterAnnotation::Named(value) => visit(value)?,
                ParameterAnnotation::Callback(callback) => pending.push(AnnotationWork::Callback(callback)),
                ParameterAnnotation::Inferred(_) => {}
            },
            AnnotationWork::Callback(callback) => {
                pending.push(AnnotationWork::Result(&mut callback.result));
                pending.extend(callback.params.iter_mut().rev().map(AnnotationWork::Parameter));
            }
            AnnotationWork::Result(result) => match result {
                ResultAnnotation::Inferred(_) => {}
                ResultAnnotation::Named(value) => visit(value)?,
                ResultAnnotation::Callable { params, result, .. } => {
                    pending.push(AnnotationWork::Result(result));
                    pending.extend(params.iter_mut().rev().map(AnnotationWork::Parameter));
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn visit_parameter(
    value: &mut CallableParameter,
    visit: &mut impl FnMut(&mut ValueAnnotation) -> Result<(), Diagnostic>,
) -> Result<(), Diagnostic> {
    annotations(AnnotationWork::Parameter(value), visit)
}
pub(crate) fn callback_annotation(
    value: &mut CallbackAnnotation,
    visit: &mut impl FnMut(&mut ValueAnnotation) -> Result<(), Diagnostic>,
) -> Result<(), Diagnostic> {
    annotations(AnnotationWork::Callback(value), visit)
}
pub(crate) fn result_annotation(
    value: &mut ResultAnnotation,
    visit: &mut impl FnMut(&mut ValueAnnotation) -> Result<(), Diagnostic>,
) -> Result<(), Diagnostic> {
    annotations(AnnotationWork::Result(value), visit)
}
pub(crate) fn function_annotations(
    function: &mut Function,
    visit: &mut impl FnMut(&mut ValueAnnotation) -> Result<(), Diagnostic>,
) -> Result<(), Diagnostic> {
    for (_, annotation, callback) in &mut function.params {
        visit(annotation)?;
        if let Some(callback) = callback {
            callback_annotation(callback, visit)?;
        }
    }
    if let Some(result) = &mut function.result {
        result_annotation(result, visit)?;
    }
    Ok(())
}
pub(crate) fn body(
    body: &mut [Stmt],
    annotation_visit: &mut impl FnMut(&mut ValueAnnotation) -> Result<(), Diagnostic>,
    expression_visit: &mut impl FnMut(&mut Expr) -> Result<(), Diagnostic>,
) -> Result<(), Diagnostic> {
    walk(body, &mut [], None, annotation_visit, expression_visit)
}
pub(crate) fn function_body(
    function: &mut Function,
    annotation_visit: &mut impl FnMut(&mut ValueAnnotation) -> Result<(), Diagnostic>,
    expression_visit: &mut impl FnMut(&mut Expr) -> Result<(), Diagnostic>,
) -> Result<(), Diagnostic> {
    walk(&mut function.body, &mut function.parameter_initializers, None, annotation_visit, expression_visit)
}
pub(crate) fn expression(
    value:&mut Expr,
    visit:&mut impl FnMut(&mut Expr)->Result<(),Diagnostic>,
)->Result<(),Diagnostic> {
    walk(&mut [],&mut [],Some(value),&mut |_|Ok(()),visit)
}
fn walk(
    body: &mut [Stmt],
    initializers: &mut [ParameterInitializer],
    expression: Option<&mut Expr>,
    annotation_visit: &mut impl FnMut(&mut ValueAnnotation) -> Result<(), Diagnostic>,
    expression_visit: &mut impl FnMut(&mut Expr) -> Result<(), Diagnostic>,
) -> Result<(), Diagnostic> {
    enum Work<'a> {
        Statement(&'a mut Stmt),
        Expression(&'a mut Expr),
    }
    let mut work = body.iter_mut().map(Work::Statement).collect::<Vec<_>>();
    if let Some(value)=expression {work.push(Work::Expression(value));}
    for initializer in initializers {
        if let ParameterInitializer::Default { value, .. } = initializer { work.push(Work::Expression(value)); }
    }
    while let Some(node) = work.pop() {
        match node {
            Work::Statement(statement) => match &mut statement.kind {
                StmtKind::LocalFunction(_) => unreachable!("local declarations must be resolved before checked traversal"),
                    StmtKind::Try(region) => work.extend(region.statements_mut().map(Work::Statement)),
                    StmtKind::Throw(value) => work.push(Work::Expression(value)),
                StmtKind::Let {
                    annotation, value, ..
                } => {
                    if let Some(annotation) = annotation {
                        annotation_visit(annotation)?;
                    }
                    work.push(Work::Expression(value));
                }
                StmtKind::Block(body) => work.extend(body.iter_mut().map(Work::Statement)),
                StmtKind::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    work.push(Work::Expression(condition));
                    work.extend(then_body.iter_mut().chain(else_body).map(Work::Statement));
                }
                StmtKind::While { condition, body } => {
                    work.push(Work::Expression(condition));
                    work.extend(body.iter_mut().map(Work::Statement));
                }
                StmtKind::Loop {
                    condition,
                    body,
                    update,
                    ..
                } => {
                    if let Some(condition) = condition {
                        work.push(Work::Expression(condition));
                    }
                    work.extend(body.iter_mut().chain(update).map(Work::Statement));
                }
                StmtKind::ForOf { iterable, body, .. } => {
                    work.push(Work::Expression(iterable));
                    work.extend(body.iter_mut().map(Work::Statement));
                }
                StmtKind::Assign { target, value, .. } => {
                    work.push(Work::Expression(target));
                    work.push(Work::Expression(value));
                }
                StmtKind::Return(Some(value)) | StmtKind::Expr(value) => {
                    work.push(Work::Expression(value))
                }
                StmtKind::Break | StmtKind::Continue | StmtKind::Return(None) => {}
            },
            Work::Expression(expression) => {
                expression_visit(expression)?;
                match &mut expression.kind {
                    ExprKind::SpreadCall(plan) => work.extend(plan.expressions_mut().map(Work::Expression)),
                    ExprKind::SpreadValue(_) => (),
                    ExprKind::Closure {
                        params,
                        result,
                        body,
                        ..
                    } => {
                        for parameter in params {
                            visit_parameter(parameter, annotation_visit)?;
                            if let ParameterInitializer::Default { value, .. } = &mut parameter.initializer { work.push(Work::Expression(value)); }
                        }
                        result_annotation(result, annotation_visit)?;
                        work.extend(body.iter_mut().map(Work::Statement));
                    }
                    ExprKind::Conditional {
                        predicate,
                        consequent,
                        alternate,
                    } => {
                        work.push(Work::Expression(predicate));
                        work.push(Work::Expression(consequent));
                        work.push(Work::Expression(alternate));
                    }
                    ExprKind::Optional {
                        base: left,
                        body: right,
                        ..
                    }
                    | ExprKind::Binary { left, right, .. } | ExprKind::Index(left, right) => {
                        work.push(Work::Expression(left));
                        work.push(Work::Expression(right));
                    }
                    ExprKind::CompoundAssign { target, value, .. }
                    | ExprKind::Assign { target, value, .. }
                    | ExprKind::CoalesceAssign { target, value, .. }
                    | ExprKind::LogicalAssign { target, value, .. } => {
                        work.push(Work::Expression(target));
                        work.push(Work::Expression(value));
                    }
                    ExprKind::OptionalChain(value)
                    | ExprKind::Unary { value, .. }
                    | ExprKind::Update { target: value, .. }
                    | ExprKind::Field(value, _)
                    | ExprKind::Inject { value, .. }
                    | ExprKind::Project { value, .. } => work.push(Work::Expression(value)),
                    ExprKind::Record(fields) => {
                        work.extend(fields.iter_mut().map(|(_, value)| Work::Expression(value)))
                    }
                    ExprKind::TypeAssertion {value,annotation,..} => {
                        annotation_visit(annotation)?;
                        work.push(Work::Expression(value));
                    }
                    ExprKind::ArrayOf(values, element) | ExprKind::SetConstructor(values, element) | ExprKind::MapConstructor(values, element) | ExprKind::NativeConstructor(values, element) => {
                        if let Some(element) = element { annotation_visit(element)?; }
                        work.extend(values.iter_mut().map(Work::Expression));
                    }
                    ExprKind::Array(values) | ExprKind::Template(values) | ExprKind::Sequence(values) => {
                        work.extend(values.iter_mut().map(Work::Expression))
                    }
                    ExprKind::Call(callee, arguments) => {
                        work.push(Work::Expression(callee));
                        work.extend(arguments.iter_mut().map(Work::Expression));
                    }
                    ExprKind::Selected { .. }
                    | ExprKind::Name(_)
                    | ExprKind::String(_)
                    | ExprKind::Integer(_)
                    | ExprKind::EnumMember { .. } | ExprKind::Number(_)
                    | ExprKind::Boolean(_)
                    | ExprKind::Null
                    | ExprKind::Undefined => {}
                }
            }
        }
    }
    Ok(())
}

/// Last source token owned by a statement subtree; statement diagnostic spans
/// alone can cover only an opening token and are not lexical scope boundaries.
pub(crate) fn body_end(body: &[Stmt]) -> usize {
    let mut end = 0;
    let mut pending = body.iter().collect::<Vec<_>>();
    while let Some(statement) = pending.pop() {
        end = end.max(statement.span.end);
        match &statement.kind {
            StmtKind::LocalFunction(function) => end = end.max(function.scope_span.end),
            StmtKind::Try(region) => pending.extend(region.statements()),
            StmtKind::Throw(value) => end=end.max(value.span.end),
            StmtKind::Let { value, .. } | StmtKind::Expr(value) | StmtKind::Return(Some(value)) => {
                end = end.max(value.span.end)
            }
            StmtKind::Assign { target, value, .. } => {
                end = end.max(target.span.end).max(value.span.end)
            }
            StmtKind::Block(body) => pending.extend(body),
            StmtKind::If {
                condition,
                then_body,
                else_body,
            } => {
                end = end.max(condition.span.end);
                pending.extend(then_body);
                pending.extend(else_body);
            }
            StmtKind::While { condition, body } => {
                end = end.max(condition.span.end);
                pending.extend(body);
            }
            StmtKind::Loop {
                condition,
                body,
                update,
                ..
            } => {
                if let Some(condition) = condition {
                    end = end.max(condition.span.end);
                }
                pending.extend(body);
                pending.extend(update);
            }
            StmtKind::ForOf { iterable, body, .. } => {
                end = end.max(iterable.span.end);
                pending.extend(body);
            }
            StmtKind::Return(None) | StmtKind::Break | StmtKind::Continue => {}
        }
    }
    end
}

#[cfg(test)]
mod annotation_worklist_tests {
    use super::*;
    use crate::Span;

    fn named(name: &str, start: usize) -> ResultAnnotation {
        ResultAnnotation::Named(Box::new(ValueAnnotation {
            kind: crate::type_syntax::ValueAnnotationKind::Named(crate::type_syntax::TypeName {
                text: name.into(), span: Span { start, end: start + 1 },
            }),
            span: Span { start, end: start + 1 },
        }))
    }

    fn parameter(name: &str, ty: ParameterAnnotation) -> CallableParameter {
        CallableParameter {
            rest: false,
            name: Token { kind: TokenKind::Name, text: name.into(), span: Span::default() },
            ty, access: None, initializer: ParameterInitializer::Required,
        }
    }

    #[test]
    fn deep_callable_results_use_worklist_on_normal_worker_stack() {
        let mut result = named("leaf", 7);
        for _ in 0..16_384 {
            result = ResultAnnotation::Callable {
                kind: None, params: Vec::new(), result: Box::new(result), span: Span::default(),
            };
        }
        let mut visited = Vec::new();
        let outcome = result_annotation(&mut result, &mut |value| {
            visited.push(value.span);
            Ok(())
        });
        // Exercise production wrapper disposal after the traversal.
        drop(result);
        outcome.unwrap();
        assert_eq!(visited, vec![Span { start: 7, end: 8 }]);
    }

    #[test]
    fn annotation_order_and_first_error_preserve_parameter_then_result_spans() {
        let ResultAnnotation::Named(ref first) = named("first", 1) else { unreachable!() };
        let ResultAnnotation::Named(ref second) = named("second", 2) else { unreachable!() };
        let mut result = ResultAnnotation::Callable {
            kind: None,
            params: vec![
                parameter("callback", ParameterAnnotation::Callback(Box::new(CallbackAnnotation {
                    kind: CallbackKind::Read,
                    params: vec![parameter("first", ParameterAnnotation::Named(first.clone()))],
                    result: Box::new(named("nested_result", 3)), span: Span::default(),
                }))),
                parameter("second", ParameterAnnotation::Named(second.clone())),
            ],
            result: Box::new(named("outer_result", 4)), span: Span::default(),
        };
        let mut visited = Vec::new();
        result_annotation(&mut result, &mut |value| {
            visited.push(value.span.start);
            Ok(())
        }).unwrap();
        assert_eq!(visited, [1, 3, 2, 4]);
        let mut visited = Vec::new();
        let error = result_annotation(&mut result, &mut |value| {
            visited.push(value.span.start);
            if value.span.start == 3 { return Err(Diagnostic::new("witness", "stop", value.span)); }
            Ok(())
        }).unwrap_err();
        assert_eq!(visited, [1, 3]);
        assert_eq!(error.span, Span { start: 3, end: 4 });
    }
}
