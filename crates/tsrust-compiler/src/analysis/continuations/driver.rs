//! Owned work frames for checker suspension.
//!
//! Blocks, return values and closure bodies share this work stack. Remaining
//! expression and control-flow operations use compatibility dispatch while they
//! are migrated; recursive compatibility entry is not stackless acceptance.
use super::super::*;
use super::block::BlockFrame;
use super::checked_expression::{CheckedExpressionFrame, PreparedExpression};
use super::returns::ReturnFrame;

#[derive(Clone, Copy)]
struct ContextId(usize);

struct Contexts<'a> {
    root: &'a mut Context,
    children: Vec<Box<Context>>,
}

impl Contexts<'_> {
    fn get(&mut self, id: ContextId) -> &mut Context {
        if id.0 == 0 { self.root } else { &mut self.children[id.0 - 1] }
    }

    fn push(&mut self, context: Box<Context>) -> ContextId {
        self.children.push(context);
        ContextId(self.children.len())
    }

    fn take(&mut self, id: ContextId) -> Box<Context> {
        assert_eq!(id.0, self.children.len(), "callable contexts finish in nesting order");
        self.children.pop().expect("completed child context")
    }
}

enum Body<'a> {
    Root(&'a mut [Stmt]),
    Child { body: Vec<Stmt>, start: usize },
}

impl Body<'_> {
    fn statements(&mut self) -> &mut [Stmt] {
        match self {
            Self::Root(body) => body,
            Self::Child { body, start } => &mut body[*start..],
        }
    }

    fn into_child(self) -> Vec<Stmt> {
        let Self::Child { body, .. } = self else { unreachable!("root body cannot be returned to a parent") };
        body
    }
}

enum Expression<'a> {
    Root(&'a mut Expr),
    Child(Box<Expr>),
}

impl Expression<'_> {
    fn value(&mut self) -> &mut Expr {
        match self { Self::Root(value) => value, Self::Child(value) => value }
    }

    fn into_child(self) -> Box<Expr> {
        let Self::Child(value) = self else { unreachable!("root expression cannot be returned to a parent") };
        value
    }
}

enum BlockPending {
    Block(usize),
    Return { index: usize, frame: ReturnFrame },
}

struct BlockTask<'a> {
    body: Body<'a>,
    context: ContextId,
    frame: Option<Box<BlockFrame>>,
    pending: Option<BlockPending>,
    result_type: Type,
    keep_names: bool,
    label_span: Option<Span>,
}

enum ExpressionState {
    Start,
    Closure { frame: Box<super::closure::Frame>, child: ContextId },
    Callee { frame: super::call::CalleeFrame, publish_type: bool },
}

#[derive(Clone)]
enum ActiveLoans<'a> {
    Borrowed(&'a Loans),
    Shared(std::rc::Rc<Loans>),
}

impl std::ops::Deref for ActiveLoans<'_> {
    type Target = Loans;
    fn deref(&self) -> &Loans {
        match self { Self::Borrowed(loans) => loans, Self::Shared(loans) => loans }
    }
}

struct ExpressionTask<'a> {
    expression: Expression<'a>,
    context: ContextId,
    mode: Mode,
    loans: ActiveLoans<'a>,
    expected: Option<Type>,
    completion: Vec<Type>,
    checked: Option<CheckedExpressionFrame>,
    state: ExpressionState,
}

impl<'a> ExpressionTask<'a> {
    fn new(expression: Expression<'a>, context: ContextId, mode: Mode, loans: ActiveLoans<'a>, expected: Option<Type>) -> Self {
        Self { expression, context, mode, loans, expected, completion: Vec::new(), checked: None, state: ExpressionState::Start }
    }
}

enum Task<'a> { Block(BlockTask<'a>), Expression(ExpressionTask<'a>) }
enum Completion<'a> {
    Block(Body<'a>, Result<bool, Diagnostic>),
    Expression(Expression<'a>, Result<Type, Diagnostic>),
}
enum Step<'a> {
    Push(Task<'a>, Task<'a>),
    Continue(Task<'a>),
    Complete(Completion<'a>),
}

pub(super) fn check_block(analyzer: &mut Analyzer, context: &mut Context, body: &mut [Stmt], result_type: Type, keep_names: bool) -> Result<bool, Diagnostic> {
    let task = Task::Block(BlockTask {
        body: Body::Root(body), context: ContextId(0), frame: None, pending: None,
        result_type, keep_names, label_span: None,
    });
    let Completion::Block(_, result) = drive(analyzer, context, task) else { unreachable!("block result") };
    result
}

pub(super) fn check_expression(analyzer: &mut Analyzer, context: &mut Context, expression: &mut Expr, mode: Mode, loans: &Loans, expected: Option<Type>) -> Result<Type, Diagnostic> {
    let task = Task::Expression(ExpressionTask::new(Expression::Root(expression), ContextId(0), mode, ActiveLoans::Borrowed(loans), expected));
    let Completion::Expression(_, result) = drive(analyzer, context, task) else { unreachable!("expression result") };
    result
}

fn drive<'a>(analyzer: &mut Analyzer, context: &mut Context, root: Task<'a>) -> Completion<'a> {
    let mut contexts = Contexts { root: context, children: Vec::new() };
    let mut tasks = vec![root];
    let mut completion = None;
    loop {
        let Some(task) = tasks.pop() else { return completion.expect("root completion") };
        let step = match task {
            Task::Block(task) => step_block(analyzer, contexts.get(task.context), task, completion.take()),
            Task::Expression(task) => step_expression(analyzer, &mut contexts, task, completion.take()),
        };
        match step {
            Step::Push(parent, child) => { tasks.push(parent); tasks.push(child); }
            Step::Continue(task) => tasks.push(task),
            Step::Complete(value) => completion = Some(value),
        }
    }
}

enum BlockAction<'a> { Push(Task<'a>), Continue, Done(bool) }

fn step_block<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: BlockTask<'a>, completion: Option<Completion<'a>>) -> Step<'a> {
    let action = (|| -> Result<BlockAction<'a>, Diagnostic> {
        if let Some(pending) = task.pending.take() {
            match (pending, completion.expect("block child completed")) {
                (BlockPending::Block(index), Completion::Block(body, result)) => {
                    let StmtKind::Block(destination) = &mut task.body.statements()[index].kind else { unreachable!("block child slot") };
                    *destination = body.into_child();
                    task.frame.as_mut().expect("suspended scope").exited = result?;
                }
                (BlockPending::Return { index, frame }, Completion::Expression(expression, result)) => {
                    let stmt = &mut task.body.statements()[index];
                    let StmtKind::Return(value) = &mut stmt.kind else { unreachable!("return child slot") };
                    *value = Some(*expression.into_child());
                    analyzer.finish_return(context, value, frame, result?)?;
                    context.storage_exits.record_expression(stmt.span, &context.bindings);
                    capture_restoration::record(context);
                    task.frame.as_mut().expect("return scope").exited = true;
                }
                _ => unreachable!("block child completion kind"),
            }
        } else { debug_assert!(completion.is_none()); }
        if task.frame.is_none() {
            task.frame = Some(analyzer.enter_block(context, task.body.statements(), task.result_type, task.keep_names)?);
        }
        let frame = task.frame.as_mut().expect("entered block");
        let Some(index) = analyzer.next_block_statement(context, task.body.statements(), frame)? else {
            let frame = task.frame.take().expect("completed block");
            let exited = analyzer.finish_block(context, task.body.statements(), frame)?;
            let exited = if let Some(span) = task.label_span {
                analyzer.finish_labeled_block(context, task.body.statements(), exited, span)?
            } else { exited };
            return Ok(BlockAction::Done(exited));
        };
        let stmt = &mut task.body.statements()[index];
        match &mut stmt.kind {
            StmtKind::Block(body) => {
                let label_span = if stmt.labels.is_empty() { None } else {
                    analyzer.enter_labeled_block(context, stmt.span, &stmt.labels)?;
                    Some(stmt.span)
                };
                let body = std::mem::take(body);
                let result_type = frame.result_type;
                task.pending = Some(BlockPending::Block(index));
                Ok(BlockAction::Push(Task::Block(BlockTask {
                    body: Body::Child { body, start: 0 }, context: task.context, frame: None, pending: None,
                    result_type, keep_names: false, label_span,
                })))
            }
            StmtKind::Return(value) => {
                let returning = analyzer.prepare_return(context, value, frame.result_type, stmt.span);
                if let Some(value) = value.take() {
                    let expected = returning.expected;
                    task.pending = Some(BlockPending::Return { index, frame: returning });
                    Ok(BlockAction::Push(Task::Expression(ExpressionTask::new(
                        Expression::Child(Box::new(value)), task.context, Mode::Consume,
                        ActiveLoans::Shared(std::rc::Rc::new(Loans::new())), expected,
                    ))))
                } else {
                    analyzer.finish_return(context, value, returning, Type::Void)?;
                    context.storage_exits.record_expression(stmt.span, &context.bindings);
                    capture_restoration::record(context);
                    frame.exited = true;
                    Ok(BlockAction::Continue)
                }
            }
            _ => { analyzer.check_block_statement(context, stmt, frame)?; Ok(BlockAction::Continue) }
        }
    })();
    match action {
        Ok(BlockAction::Push(child)) => Step::Push(Task::Block(task), child),
        Ok(BlockAction::Continue) => Step::Continue(Task::Block(task)),
        Ok(BlockAction::Done(exited)) => Step::Complete(Completion::Block(task.body, Ok(exited))),
        Err(error) => Step::Complete(Completion::Block(task.body, Err(error))),
    }
}

fn finish_expression<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, result: Result<Type, Diagnostic>) -> Step<'a> {
    let expr = task.expression.value();
    let result = if let Some(frame) = task.checked.take() { analyzer.finish_checked_expression(context, expr, frame, result) } else { result };
    let result = analyzer.finish_expression(context, expr, result).map(|mut ty| {
        for wrapper in task.completion.into_iter().rev() {
            if let Type::Union(union) = wrapper { expr.inject(union); }
            expr.ty = Some(wrapper);
            ty = wrapper;
        }
        ty
    });
    Step::Complete(Completion::Expression(task.expression, result))
}

fn step_expression<'a>(analyzer: &mut Analyzer, contexts: &mut Contexts<'_>, mut task: ExpressionTask<'a>, completion: Option<Completion<'a>>) -> Step<'a> {
    match std::mem::replace(&mut task.state, ExpressionState::Start) {
        ExpressionState::Closure { frame, child } => {
            let Completion::Block(body, returned) = completion.expect("closure body completed") else { unreachable!("closure body result") };
            let mut child_context = contexts.take(child);
            let returned = returned.and_then(|returned| { analyzer.validate_native_returns(&mut child_context)?; Ok(returned) });
            let expr = task.expression.value();
            let ExprKind::Closure { body: destination, .. } = &mut expr.kind else { unreachable!("closure continuation") };
            *destination = body.into_child();
            let result = analyzer.finish_closure(contexts.get(task.context), frame, child_context, destination, &task.loans, returned);
            if let Ok(ty) = result { expr.ty = Some(ty); }
            finish_expression(analyzer, contexts.get(task.context), task, result)
        }
        ExpressionState::Callee { frame, publish_type } => {
            let Completion::Expression(callee, result) = completion.expect("callee completed") else { unreachable!("callee result") };
            let context = contexts.get(task.context);
            let expr = task.expression.value();
            let ExprKind::Call(destination, _) = &mut expr.kind else { unreachable!("call continuation") };
            *destination = callee.into_child();
            let expected = task.checked.as_ref().expect("checked call frame").expected;
            let result = result.and_then(|ty| finish_call(analyzer, context, expr, &task.loans, expected, frame, Some(ty), publish_type));
            finish_expression(analyzer, context, task, result)
        }
        ExpressionState::Start => {
            debug_assert!(completion.is_none());
            let context = contexts.get(task.context);
            let expr = task.expression.value();
            let prepared = analyzer.prepare_expression_work(context, expr, task.mode, &task.loans, task.expected, &mut task.completion);
            let frame = match prepared {
                Ok(PreparedExpression::Evaluate(frame)) => frame,
                Ok(PreparedExpression::Ready(ty)) => return finish_expression(analyzer, context, task, Ok(ty)),
                Err(error) => return finish_expression(analyzer, context, task, Err(error)),
            };
            let invocation = frame.invocation;
            let expected = frame.expected;
            let mode = frame.mode;
            task.checked = Some(frame);
            if let ExprKind::Closure { id, params, result, body } = &mut expr.kind {
                let prepared = analyzer.prepare_closure(context, (*id, params, result), expected, body, expr.span, &task.loans);
                let (frame, mut child_context) = match prepared {
                    Ok(super::closure::Prepared::Ready(ty)) => {
                        expr.ty = Some(ty);
                        return finish_expression(analyzer, context, task, Ok(ty));
                    }
                    Ok(super::closure::Prepared::Body(frame, context)) => (frame, context),
                    Err(error) => return finish_expression(analyzer, context, task, Err(error)),
                };
                let entered = analyzer.enter_source_callable_body(&mut child_context);
                if entered.is_err() || has_exception_syntax(Analyzer::uninitialized_body(body)) {
                    // Exception environment/exit frames still use compatibility
                    // dispatch until their continuations join this same driver.
                    let returned = entered.and_then(|()| analyzer.check_exception_callable_body(&mut child_context, Analyzer::uninitialized_body(body), frame.result))
                        .and_then(|returned| { analyzer.validate_native_returns(&mut child_context)?; Ok(returned) });
                    let result = analyzer.finish_closure(context, frame, child_context, body, &task.loans, returned);
                    if let Ok(ty) = result { expr.ty = Some(ty); }
                    return finish_expression(analyzer, context, task, result);
                }
                let start = body.len() - Analyzer::uninitialized_body(body).len();
                let body = std::mem::take(body);
                let result_type = frame.result;
                let child = contexts.push(child_context);
                task.state = ExpressionState::Closure { frame, child };
                let child = Task::Block(BlockTask {
                    body: Body::Child { body, start }, context: child, frame: None, pending: None,
                    result_type, keep_names: false, label_span: None,
                });
                Step::Push(Task::Expression(task), child)
            } else {
                if invocation {
                    let dispatch = analyzer.prepare_call_work(context, expr, &task.loans, expected);
                    match dispatch {
                        Ok(CallDispatch::Ready(ty)) => finish_expression(analyzer, context, task, Ok(ty)),
                        Err(error) => finish_expression(analyzer, context, task, Err(error)),
                        Ok(CallDispatch::Conditional { publish_type }) => {
                            let ExprKind::Call(callee, args) = &mut expr.kind else { unreachable!("conditional call") };
                            let frame = match analyzer.prepare_conditional_callee(context, callee, args) {
                                Ok(frame) => frame,
                                Err(error) => return finish_expression(analyzer, context, task, Err(error)),
                            };
                            if frame.checked_callee {
                                let span = callee.span;
                                let callee = std::mem::replace(callee, Box::new(Expr::new(ExprKind::Undefined, span)));
                                let child = Task::Expression(ExpressionTask::new(
                                    Expression::Child(callee), task.context, Mode::Consume,
                                    task.loans.clone(), None,
                                ));
                                task.state = ExpressionState::Callee { frame, publish_type };
                                Step::Push(Task::Expression(task), child)
                            } else {
                                let hint = analyzer.callback_value_hint(context, callee);
                                let result = hint.and_then(|hint| finish_call(analyzer, context, expr, &task.loans, expected, frame, hint, publish_type));
                                finish_expression(analyzer, context, task, result)
                            }
                        }
                    }
                } else {
                    let result = analyzer.expression_kind(context, expr, mode, &task.loans, expected);
                    finish_expression(analyzer, context, task, result)
                }
            }
        }
    }
}

/// Argument and dynamic method requests remain synchronous during migration.
/// Callee evaluation and its closure body already suspend on the shared stack.
fn finish_call(analyzer: &mut Analyzer, context: &mut Context, expr: &mut Expr, loans: &Loans, expected: Option<Type>, frame: super::call::CalleeFrame, hint: Option<Type>, publish_type: bool) -> Result<Type, Diagnostic> {
    let ExprKind::Call(callee, args) = &mut expr.kind else { unreachable!("call completion") };
    let frame = analyzer.finish_conditional_callee(callee, args.len(), expr.span, frame, hint)?;
    let result = analyzer.checked_callable_expression_call(context, callee, args, loans, expr.span, frame.hint, frame.checked_callee, expected)?;
    let (target, ty) = analyzer.finish_conditional_call(context, callee, frame, result)?;
    expr.target = Some(target);
    if publish_type { expr.ty = Some(ty); }
    analyzer.resolve_completed_capture_invocation(context, expr, ty);
    Ok(ty)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn formatted(source: &str) -> String {
        let formatted = crate::format(source).unwrap();
        assert_eq!(crate::format(&formatted).unwrap(), formatted);
        formatted
    }

    fn analyzer() -> Analyzer {
        Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap()
    }

    #[test]
    fn suspended_scopes_restore_shadowed_bindings_and_keep_typed_children() {
        let mut analyzer = analyzer();
        let mut context = Context::new("main".into());
        let outer = Analyzer::bind(&mut context, &Token {
            text: "value".into(), kind: TokenKind::Name, span: Span::new(0, 5),
        }, Type::Int, false).unwrap();
        let source = formatted(
            "function main(){{{let value: string = \"inside\";}} let after: int = value;}"
        );
        let mut parsed = crate::syntax::parse(&source).unwrap();
        let body = &mut parsed.functions[0].body;
        assert!(!check_block(&mut analyzer, &mut context, body, Type::Void, false).unwrap());
        assert_eq!(context.names["value"], outer);
        assert!(!context.names.contains_key("after"));
        assert!(context.lexical_scopes.is_empty());
        assert_eq!(analyzer.depth, 0);
        let StmtKind::Let { value, .. } = &body[1].kind else { panic!("outer declaration") };
        assert_eq!(value.binding, Some(outer));
        assert_eq!(value.ty, Some(Type::Int));
        let StmtKind::Block(first) = &body[0].kind else { panic!("first scope") };
        let StmtKind::Block(second) = &first[0].kind else { panic!("second scope") };
        let StmtKind::Let { value, .. } = &second[0].kind else { panic!("inner declaration") };
        assert_eq!(value.ty, Some(Type::String));
    }

    #[test]
    fn failed_child_restores_every_suspended_body_and_original_error_span() {
        let mut analyzer = analyzer();
        let mut context = Context::new("main".into());
        let source = formatted("function main(){{{let before: int = 7; let bad: int = true;}}}");
        let mut parsed = crate::syntax::parse(&source).unwrap();
        let body = &mut parsed.functions[0].body;
        let error = check_block(&mut analyzer, &mut context, body, Type::Void, false).unwrap_err();
        assert_eq!(error.code, "type");
        assert_eq!(&source[error.span.start..error.span.end], "true");
        let StmtKind::Block(first) = &body[0].kind else { panic!("first scope") };
        let StmtKind::Block(second) = &first[0].kind else { panic!("second scope") };
        assert_eq!(second.len(), 2);
        let StmtKind::Let { value, .. } = &second[0].kind else { panic!("checked prefix") };
        assert_eq!(value.ty, Some(Type::Int));
        let StmtKind::Let { value, .. } = &second[1].kind else { panic!("failed declaration") };
        assert!(matches!(value.kind, ExprKind::Boolean(true)));
    }

    #[test]
    fn nested_return_marks_parent_unreachable_at_the_following_statement() {
        let mut analyzer = analyzer();
        let mut context = Context::new("main".into());
        let source = formatted("function main(): int {{{return 7;}} let after: int = 9;}");
        let mut parsed = crate::syntax::parse(&source).unwrap();
        let body = &mut parsed.functions[0].body;
        let following = body[1].span;
        let error = check_block(&mut analyzer, &mut context, body, Type::Int, false).unwrap_err();
        assert_eq!(error.span, following);
        assert!(error.message.contains("unreachable"));
        let StmtKind::Block(first) = &body[0].kind else { panic!("first scope") };
        let StmtKind::Block(second) = &first[0].kind else { panic!("second scope") };
        let StmtKind::Return(Some(value)) = &second[0].kind else { panic!("return") };
        assert_eq!(value.ty, Some(Type::Int));
    }

    #[test]
    fn labeled_child_break_resumes_parent_and_consumes_its_control_target() {
        let mut analyzer = analyzer();
        let mut context = Context::new("main".into());
        let source = formatted(
            "function main(){outer: {inner: {break outer;}} let after: int = 9;}"
        );
        let mut parsed = crate::syntax::parse(&source).unwrap();
        let body = &mut parsed.functions[0].body;
        assert!(!check_block(&mut analyzer, &mut context, body, Type::Void, false).unwrap());
        assert!(context.loops.is_empty());
        assert!(context.lexical_scopes.is_empty());
        let StmtKind::Let { value, .. } = &body[1].kind else { panic!("resumed parent") };
        assert_eq!(value.ty, Some(Type::Int));
        let target = ControlTarget::Block(body[0].span.start);
        let StmtKind::Block(first) = &body[0].kind else { panic!("outer label") };
        let StmtKind::Block(second) = &first[0].kind else { panic!("inner label") };
        assert_eq!(second[0].control_target, Some(target));
    }
}
