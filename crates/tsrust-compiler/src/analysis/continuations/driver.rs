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
    IndexBase(super::access::IndexFrame),
    IndexOperand(super::access::IndexOperand),
    Field { frame: super::access::FieldFrame, mode: Mode, rewrite: Option<String> },
    Literal { frame: Box<super::literals::LiteralFrame>, child: super::literals::Child },
    Closure { frame: Box<super::closure::Frame>, child: ContextId },
    Callee { frame: super::call::CalleeFrame, publish_type: bool },
    InvocationCallee { frame: super::call::CallCompletion, invocation: super::call::CallableInvocation, publish_type: bool },
    StoredFieldCallee { params: Vec<ArgumentContract>, result: Type, invocation: Mode, family: usize },
    Argument { call: Box<CallArguments>, index: usize, frame: super::arguments::ArgumentFrame },
}

enum InvocationFinish {
    Conditional { frame: super::call::CallCompletion, invocation: super::call::CallableInvocation, publish_type: bool },
    Declared { index: usize, captures: Vec<(usize, Mode)>, resolve_completed: bool },
    Bound { binding: usize, result: Type, target: CallTarget },
    StoredField { family: usize, result: Type },
}

struct CallArguments {
    arguments: super::arguments::Arguments,
    finish: InvocationFinish,
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
            match result {
                Ok(ty) => start_call(analyzer, context, task, frame, Some(ty), publish_type),
                Err(error) => finish_expression(analyzer, context, task, Err(error)),
            }
        }
        ExpressionState::InvocationCallee { frame, mut invocation, publish_type } => {
            let Completion::Expression(callee, result) = completion.expect("invocation callee completed") else { unreachable!("callee result") };
            let context = contexts.get(task.context);
            let ExprKind::Call(destination, _) = &mut task.expression.value().kind else { unreachable!("invocation callee slot") };
            *destination = callee.into_child();
            let prepared = result.and_then(|actual| analyzer.finish_invocation_callee(context, destination, &mut invocation, actual, &task.loans));
            match prepared {
                Ok(arguments) => schedule_argument(analyzer, context, task, Box::new(CallArguments {
                    arguments, finish: InvocationFinish::Conditional { frame, invocation, publish_type },
                })),
                Err(error) => finish_expression(analyzer, context, task, Err(error)),
            }
        }
        ExpressionState::StoredFieldCallee { params, result, invocation, family } => {
            let Completion::Expression(callee, checked) = completion.expect("stored field callee completed") else { unreachable!("stored field result") };
            let context = contexts.get(task.context);
            let ExprKind::Call(destination, _) = &mut task.expression.value().kind else { unreachable!("stored field callee slot") };
            *destination = callee.into_child();
            let actual = match checked {
                Ok(actual) => actual,
                Err(error) => return finish_expression(analyzer, context, task, Err(error)),
            };
            // A temporary factory may specialize its returned field family.
            let family = match actual { Type::StoredCallable(family) | Type::BorrowedCallable(family) => family, _ => family };
            let mut active = (*task.loans).clone();
            for root in Analyzer::loan_roots(destination, invocation) { active.entry(root).or_default().push((invocation, destination.span)); }
            let arguments = super::arguments::Arguments::new(params, active, context, false, false, false);
            schedule_argument(analyzer, context, task, Box::new(CallArguments { arguments, finish: InvocationFinish::StoredField { family, result } }))
        }
        ExpressionState::Argument { mut call, index, frame } => {
            let Completion::Expression(argument, result) = completion.expect("argument completed") else { unreachable!("argument result") };
            let context = contexts.get(task.context);
            let ExprKind::Call(_, args) = &mut task.expression.value().kind else { unreachable!("argument parent") };
            args[index] = *argument.into_child();
            let result = result.and_then(|actual| analyzer.finish_argument(&mut args[index], &mut call.arguments, frame, actual));
            match result {
                Ok(()) => schedule_argument(analyzer, context, task, call),
                Err(error) => finish_expression(analyzer, context, task, Err(error)),
            }
        }
        ExpressionState::IndexBase(frame) => {
            let Completion::Expression(value, checked) = completion.expect("index receiver completed") else { unreachable!("index receiver result"); };
            let context = contexts.get(task.context);
            let expr = task.expression.value();
            let ExprKind::Index(base, index) = &mut expr.kind else { unreachable!("index receiver slot"); };
            *base = value.into_child();
            let next = checked.and_then(|ty| analyzer.finish_index_base_work(context, base, index, expr.span, frame, ty, &task.loans));
            match next {
                Err(error) => finish_expression(analyzer, context, task, Err(error)),
                Ok(super::access::IndexAdvance::Done(ty)) => {
                    let result = analyzer.finish_index_work(expr, ty);
                    finish_expression(analyzer, context, task, result)
                }
                Ok(super::access::IndexAdvance::Operand { frame, loans }) => {
                    let span = index.span;
                    let index = std::mem::replace(index, Box::new(Expr::new(ExprKind::Undefined, span)));
                    let request = Task::Expression(ExpressionTask::new(Expression::Child(index), task.context, Mode::Copy, ActiveLoans::Shared(std::rc::Rc::new(loans)), Some(Type::Int)));
                    task.state = ExpressionState::IndexOperand(frame);
                    Step::Push(Task::Expression(task), request)
                }
            }
        }
        ExpressionState::IndexOperand(frame) => {
            let Completion::Expression(value, checked) = completion.expect("index operand completed") else { unreachable!("index operand result"); };
            let context = contexts.get(task.context);
            let expr = task.expression.value();
            let ExprKind::Index(_, index) = &mut expr.kind else { unreachable!("index operand slot"); };
            *index = value.into_child();
            let result = checked.and_then(|actual| analyzer.finish_index_operand_work(context, expr, frame, actual, &task.loans));
            finish_expression(analyzer, context, task, result)
        }
        ExpressionState::Field { frame, mode, rewrite } => {
            let Completion::Expression(value, checked) = completion.expect("field receiver completed") else { unreachable!("field receiver result"); };
            let context = contexts.get(task.context);
            let expr = task.expression.value();
            let (base, name) = match &mut expr.kind {
                ExprKind::Field(base, name) => (base, name.as_str()),
                ExprKind::Index(base, _) => (base, rewrite.as_deref().expect("indexed field name")),
                _ => unreachable!("field receiver slot"),
            };
            *base = value.into_child();
            let result = analyzer.finish_field_work(context, base, name, mode, &task.loans, expr.span, frame, checked);
            if result.is_ok() && let Some(name) = rewrite {
                let span = base.span;
                let base = std::mem::replace(base, Box::new(Expr::new(ExprKind::Undefined, span)));
                expr.kind = ExprKind::Field(base, name);
            }
            finish_expression(analyzer, context, task, result)
        }
        ExpressionState::Literal { mut frame, child } => {
            let Completion::Expression(value, checked) = completion.expect("literal child completed") else { unreachable!("literal child result"); };
            let context = contexts.get(task.context);
            *frame.child(task.expression.value(), child.index) = *value.into_child();
            let checked = checked.and_then(|actual| analyzer.finish_literal_child(context, task.expression.value(), &mut frame, child, actual));
            match checked {
                Ok(()) => schedule_literal(analyzer, context, task, frame),
                Err(error) => finish_expression(analyzer, context, task, Err(error)),
            }
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
            match analyzer.prepare_literal_work(context, expr, expected) {
                Ok(Some(frame)) => return schedule_literal(analyzer, context, task, Box::new(frame)),
                Ok(None) => {},
                Err(error) => return finish_expression(analyzer, context, task, Err(error)),
            }
            let indexed_field = if let ExprKind::Index(base, index) = &expr.kind {
                if let ExprKind::String(name) = &index.kind
                    && matches!(analyzer.expression_type_hint(context, base), Some(Type::Record(record)) if !analyzer.program.records[record].is_tuple()) {
                    Some(name.clone())
                } else { None }
            } else { None };
            if matches!(expr.kind, ExprKind::Field(_, _)) || indexed_field.is_some() {
                return schedule_field(analyzer, context, task, mode, indexed_field);
            }
            if let ExprKind::Index(base, _) = &mut expr.kind {
                let (frame, base_mode, expected) = match analyzer.prepare_index_work(context, base, mode, expected) {
                    Ok(prepared) => prepared,
                    Err(error) => return finish_expression(analyzer, context, task, Err(error)),
                };
                let span = base.span;
                let base = std::mem::replace(base, Box::new(Expr::new(ExprKind::Undefined, span)));
                let child = Task::Expression(ExpressionTask::new(Expression::Child(base), task.context, base_mode, task.loans.clone(), expected));
                task.state = ExpressionState::IndexBase(frame);
                return Step::Push(Task::Expression(task), child);
            }
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
                        Ok(CallDispatch::Declared { resolve_completed }) => start_declared_call(analyzer, context, task, resolve_completed),
                        Ok(CallDispatch::Bound { binding, params, result, invocation, target }) => {
                            let ExprKind::Call(callee, _) = &expr.kind else { unreachable!("bound call") };
                            let mut active = (*task.loans).clone();
                            active.entry(binding).or_default().push((invocation, callee.span));
                            let arguments = super::arguments::Arguments::new(params, active, context, false, false, false);
                            schedule_argument(analyzer, context, task, Box::new(CallArguments { arguments, finish: InvocationFinish::Bound { binding, result, target } }))
                        }
                        Ok(CallDispatch::StoredField { field_type, params, result, invocation, family }) => {
                            let ExprKind::Call(callee, _) = &mut expr.kind else { unreachable!("stored field call") };
                            let span = callee.span;
                            let callee = std::mem::replace(callee, Box::new(Expr::new(ExprKind::Undefined, span)));
                            let child = Task::Expression(ExpressionTask::new(Expression::Child(callee), task.context, invocation, task.loans.clone(), Some(field_type)));
                            task.state = ExpressionState::StoredFieldCallee { params, result, invocation, family };
                            Step::Push(Task::Expression(task), child)
                        }
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
                                match hint {
                                    Ok(hint) => start_call(analyzer, context, task, frame, hint, publish_type),
                                    Err(error) => finish_expression(analyzer, context, task, Err(error)),
                                }
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

fn schedule_field<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, mode: Mode, rewrite: Option<String>) -> Step<'a> {
    let expr = task.expression.value();
    let (base, name) = match &mut expr.kind {
        ExprKind::Field(base, name) => (base, name.as_str()),
        ExprKind::Index(base, _) => (base, rewrite.as_deref().expect("indexed field name")),
        _ => unreachable!("field receiver slot"),
    };
    match analyzer.prepare_field_work(context, base, name, mode, &task.loans, expr.span) {
        Ok(super::access::FieldDispatch::Ready(ty)) => {
            if let Some(name) = rewrite {
                let span = base.span;
                let base = std::mem::replace(base, Box::new(Expr::new(ExprKind::Undefined, span)));
                expr.kind = ExprKind::Field(base, name);
            }
            finish_expression(analyzer, context, task, Ok(ty))
        }
        Err(error) => finish_expression(analyzer, context, task, Err(error)),
        Ok(super::access::FieldDispatch::Receiver { mode: base_mode, frame }) => {
            let span = base.span;
            let base = std::mem::replace(base, Box::new(Expr::new(ExprKind::Undefined, span)));
            let child = Task::Expression(ExpressionTask::new(Expression::Child(base), task.context, base_mode, task.loans.clone(), None));
            task.state = ExpressionState::Field { frame, mode, rewrite };
            Step::Push(Task::Expression(task), child)
        }
    }
}

fn schedule_literal<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, mut frame: Box<super::literals::LiteralFrame>) -> Step<'a> {
    match analyzer.advance_literal_work(context, task.expression.value(), &mut frame) {
        Ok(super::literals::Advance::Done(ty)) => finish_expression(analyzer, context, task, Ok(ty)),
        Err(error) => finish_expression(analyzer, context, task, Err(error)),
        Ok(super::literals::Advance::Child(child)) => {
            let destination = frame.child(task.expression.value(), child.index);
            let span = destination.span;
            let value = std::mem::replace(destination, Expr::new(ExprKind::Undefined, span));
            let request = Task::Expression(ExpressionTask::new(Expression::Child(Box::new(value)), task.context, child.mode, task.loans.clone(), child.expected));
            task.state = ExpressionState::Literal { frame, child };
            Step::Push(Task::Expression(task), request)
        }
    }
}

fn complete_conditional_call(analyzer: &mut Analyzer, context: &mut Context, expr: &mut Expr, frame: super::call::CallCompletion, result: (CallTarget, Type), publish_type: bool) -> Result<Type, Diagnostic> {
    let ExprKind::Call(callee, _) = &mut expr.kind else { unreachable!("call completion") };
    let (target, ty) = analyzer.finish_conditional_call(context, callee, frame, result)?;
    expr.target = Some(target);
    if publish_type { expr.ty = Some(ty); }
    analyzer.resolve_completed_capture_invocation(context, expr, ty);
    Ok(ty)
}

fn start_call<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, frame: super::call::CalleeFrame, hint: Option<Type>, publish_type: bool) -> Step<'a> {
    let expr = task.expression.value();
    let ExprKind::Call(callee, args) = &mut expr.kind else { unreachable!("call preparation") };
    let frame = match analyzer.finish_conditional_callee(callee, args.len(), expr.span, frame, hint) {
        Ok(frame) => frame,
        Err(error) => return finish_expression(analyzer, context, task, Err(error)),
    };
    let expected = task.checked.as_ref().expect("checked call frame").expected;
    let prepared = analyzer.prepare_callable_invocation(context, callee, args, &task.loans, expr.span, frame.hint, frame.checked_callee, expected);
    let mut invocation = match prepared {
        Ok(super::call::PreparedInvocation::Ready(result)) => {
            let result = complete_conditional_call(analyzer, context, expr, frame, result, publish_type);
            return finish_expression(analyzer, context, task, result);
        }
        Ok(super::call::PreparedInvocation::Check(invocation)) => invocation,
        Err(error) => return finish_expression(analyzer, context, task, Err(error)),
    };
    if !frame.checked_callee {
        let span = callee.span;
        let callee = std::mem::replace(callee, Box::new(Expr::new(ExprKind::Undefined, span)));
        let child = Task::Expression(ExpressionTask::new(
            Expression::Child(callee), task.context, invocation.invocation,
            task.loans.clone(), Some(frame.hint),
        ));
        task.state = ExpressionState::InvocationCallee { frame, invocation, publish_type };
        return Step::Push(Task::Expression(task), child);
    }
    let actual = callee.ty.expect("checked expression callee");
    match analyzer.finish_invocation_callee(context, callee, &mut invocation, actual, &task.loans) {
        Ok(arguments) => schedule_argument(analyzer, context, task, Box::new(CallArguments {
            arguments, finish: InvocationFinish::Conditional { frame, invocation, publish_type },
        })),
        Err(error) => finish_expression(analyzer, context, task, Err(error)),
    }
}

fn start_declared_call<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, resolve_completed: bool) -> Step<'a> {
    let expr = task.expression.value();
    let prepared = (|| {
        let (index, captures, params) = analyzer.declared_call_contract(context, expr)?;
        let ExprKind::Call(_, args) = &mut expr.kind else { unreachable!("declared call") };
        Analyzer::prepare_arguments(args, &params, expr.span)?;
        Ok::<_, Diagnostic>((index, captures, params))
    })();
    match prepared {
        Ok((index, captures, params)) => {
            let arguments = super::arguments::Arguments::new(params, (*task.loans).clone(), context, true, true, true);
            schedule_argument(analyzer, context, task, Box::new(CallArguments {
                arguments, finish: InvocationFinish::Declared { index, captures, resolve_completed },
            }))
        }
        Err(error) => finish_expression(analyzer, context, task, Err(error)),
    }
}

fn schedule_argument<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, mut call: Box<CallArguments>) -> Step<'a> {
    let expr = task.expression.value();
    let ExprKind::Call(_, args) = &mut expr.kind else { unreachable!("argument scheduling") };
    if let Some((index, frame)) = analyzer.next_argument(context, args, &mut call.arguments) {
        let span = args[index].span;
        let argument = std::mem::replace(&mut args[index], Expr::new(ExprKind::Undefined, span));
        let child = Task::Expression(ExpressionTask::new(
            Expression::Child(Box::new(argument)), task.context, frame.mode,
            ActiveLoans::Shared(call.arguments.active.clone()), frame.expected,
        ));
        task.state = ExpressionState::Argument { call, index, frame };
        return Step::Push(Task::Expression(task), child);
    }
    call.arguments.finish(context);
    let result = match call.finish {
        InvocationFinish::Conditional { frame, invocation, publish_type } => {
            analyzer.finish_callable_invocation(context, args, expr.span, invocation)
                .and_then(|result| complete_conditional_call(analyzer, context, expr, frame, result, publish_type))
        }
        InvocationFinish::Bound { binding, result, target } => {
            analyzer.finish_bound_call(context, expr, binding, result, target)
                .map(|ty| { analyzer.resolve_completed_capture_invocation(context, expr, ty); ty })
        }
        InvocationFinish::StoredField { family, result } => {
            analyzer.remember_native_family_returns(context, family, expr.span).map(|()| {
                expr.target = Some(CallTarget::StoredCallableField(family));
                expr.ty = Some(result);
                analyzer.resolve_completed_capture_invocation(context, expr, result);
                result
            })
        }
        InvocationFinish::Declared { index, captures, resolve_completed } => {
            analyzer.finish_declared_call(context, expr, index, &captures, &call.arguments.params, &task.loans)
                .map(|ty| { if resolve_completed { analyzer.resolve_completed_capture_invocation(context, expr, ty); } ty })
        }
    };
    finish_expression(analyzer, context, task, result)
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
    fn failed_literal_and_projection_children_restore_the_tree_and_access_scope() {
        for source in [
            "function main(){const values:int[]=[1,true];}",
            "type Row={values:int[];};function main(){const row:Row={values:[1]};row.values[true];}",
            "type Row={value:int;};function main(){const row:Row={value:1};row.missing.length;}",
        ] {
            let source = formatted(source);
            let mut parsed = crate::syntax::parse(&source).unwrap();
            let mut analyzer = Analyzer::new(parsed.clone()).unwrap();
            let mut context = Context::new("main".into());
            let body = &mut parsed.functions[0].body;
            let error = check_block(&mut analyzer, &mut context, body, Type::Void, false).unwrap_err();
            assert_eq!(error.code, "type");
            assert!(context.projection.is_none(), "failed receiver must release projection context");
            if source.contains("missing") {
                let StmtKind::Expr(expr) = &body[1].kind else { panic!("restored field expression"); };
                let ExprKind::Field(base, name) = &expr.kind else { panic!("outer field"); };
                assert_eq!(name, "length");
                assert!(matches!(&base.kind, ExprKind::Field(_, name) if name == "missing"));
            } else {
                assert_eq!(&source[error.span.start..error.span.end], "true");
                if let StmtKind::Let { value, .. } = &body[0].kind
                    && let ExprKind::Array(values) = &value.kind {
                    assert_eq!(values[0].ty, Some(Type::Int));
                    assert!(matches!(values[1].kind, ExprKind::Boolean(true)));
                } else {
                    let StmtKind::Expr(expr) = &body[1].kind else { panic!("restored index expression"); };
                    let ExprKind::Index(base, index) = &expr.kind else { panic!("index"); };
                    assert!(matches!(base.kind, ExprKind::Field(_, _)));
                    assert!(matches!(index.kind, ExprKind::Boolean(true)));
                }
            }
        }
    }

    #[test]
    fn failed_argument_restores_callee_and_previously_checked_arguments() {
        let source = formatted("function pair(left: int, right: int): int { return left; } function main(): int { return pair(1, true); }");
        let mut parsed = crate::syntax::parse(&source).unwrap();
        let mut analyzer = Analyzer::new(parsed.clone()).unwrap();
        let mut context = Context::new("main".into());
        let body = &mut parsed.functions[1].body;
        let error = check_block(&mut analyzer, &mut context, body, Type::Int, false).unwrap_err();
        assert_eq!(error.code, "type");
        assert_eq!(&source[error.span.start..error.span.end], "true");
        let StmtKind::Return(Some(value)) = &body[0].kind else { panic!("restored return") };
        let ExprKind::Call(callee, arguments) = &value.kind else { panic!("restored call") };
        assert!(matches!(&callee.kind, ExprKind::Name(name) if name == "pair"));
        assert_eq!(arguments.len(), 2);
        assert_eq!(arguments[0].ty, Some(Type::Int));
        assert!(matches!(arguments[1].kind, ExprKind::Boolean(true)));
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
