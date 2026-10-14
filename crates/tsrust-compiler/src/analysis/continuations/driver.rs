//! Owned work frames for checker suspension.
//!
//! Blocks, return values and closure bodies share this work stack. Remaining
//! expression and control-flow operations use compatibility dispatch while they
//! are migrated; recursive compatibility entry is not stackless acceptance.
use super::super::*;
mod tail;
mod region;
mod closure_parameters;
mod optional;
mod functions;
mod parameters;
mod methods;
mod transform;
mod reduction;
mod sorting;
mod adapters;
mod deferred;
mod generic;
mod spread;
mod array_spread;
mod object_spread;
mod collection_constructor;
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
    Region(usize),
    Throw(usize),
    Assignment(usize),
    Tail(usize),
    WhileCondition { index: usize, frame: Box<super::loops::Condition> },
    Iterable(usize),
    LoopBody { index: usize, frame: Box<super::loops::BodyLoop> },
    Declaration { index: usize, frame: Box<super::declarations::Declaration>, module_scope: Option<Span> },
    IfCondition(usize),
    IfThen { index: usize, frame: Box<super::branches::Then> },
    IfElse { index: usize, frame: Box<super::branches::Else> },
    Expression { index: usize, removal: Option<Box<crate::capture_loan_owners::Owners>>, precise: bool },
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
    ArraySpread,
    ObjectSpread,
    CollectionConstructor,
    Spread,
    GenericReceiver,
    GenericCall,
    Adapter,
    Deferred { origin: usize, conditional: Option<(super::call::CallCompletion, bool)> },
    Transform(Box<transform::State>),
    Reduction(Box<reduction::State>),
    Sorting(Box<sorting::State>),
    Wrapper(super::wrappers::Frame),
    Start,
    Prelude(super::prelude::Frame),
    Optional,
    Assignment,
    Sequence,
    NullishAssignmentTarget(Box<super::assignment::NullishTarget>),
    NullishAssignmentValue(Box<super::assignment::NullishValue>),
    LogicalLeft(Box<super::logical::Left>),
    LogicalRight { frame: Box<super::logical::Right>, child_context: Option<ContextId> },
    CoalesceLeft(Box<super::coalesce::Left>),
    CoalesceRight { frame: Box<super::coalesce::Right>, child_context: Option<ContextId> },
    ConditionalPredicate { mode: Mode, expected: Option<Type> },
    ConditionalArm { plan: Box<ConditionalPlan>, mode: Mode, left: Option<Type>, stored: Option<super::StoredValueFrame> },
    Unary { frame: Box<super::operators::UnaryFrame>, expected: Option<Type> },
    BinaryLeft(Box<super::operators::BinaryFrame>),
    BinaryRight(Box<super::operators::BinaryFrame>),
    IndexBase(super::access::IndexFrame),
    IndexOperand(super::access::IndexOperand),
    Field { frame: super::access::FieldFrame, mode: Mode, rewrite: Option<String> },
    Literal { frame: Box<super::literals::LiteralFrame>, child: super::literals::Child },
    Closure { frame: Box<super::closure::Frame>, child: ContextId },
    ClosureDefault { frame:Box<super::closure::Parameters>,child:ContextId,work:Box<super::closure::ParameterWork> },
    ClosurePattern { frame:Box<super::closure::Parameters>,child:ContextId,start:usize },
    Method { origin: usize, conditional: Option<(super::call::CallCompletion, bool)> },
    DeclaredFunction { index: usize, resolve_completed: bool },
    Callee { frame: super::call::CalleeFrame, publish_type: bool },
    InvocationCallee { frame: super::call::CallCompletion, invocation: super::call::CallableInvocation, publish_type: bool },
    StoredFieldCallee { params: Vec<ArgumentContract>, result: Type, invocation: Mode, family: usize },
    Argument { call: Box<CallArguments>, index: usize, frame: super::arguments::ArgumentFrame },
}

enum InvocationFinish {
    Method { origin: usize },
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

enum AssignmentState { Start, Target(super::assignment::Target), Value(super::assignment::Value) }

struct AssignmentTask<'a> {
    target: Box<Expr>,
    value: Box<Expr>,
    kind: super::assignment::Kind,
    operator: Span,
    loans: ActiveLoans<'a>,
    context: ContextId,
    state: AssignmentState,
}

struct SequenceTask<'a> {
    expression: Box<Expr>,
    context: ContextId,
    loans: ActiveLoans<'a>,
    mode: Mode,
    expected: Option<Type>,
    discarded: bool,
    precise: bool,
    snapshot_requested: bool,
    snapshot: Option<Option<Box<crate::capture_loan_owners::Owners>>>,
    index: usize,
    pending: bool,
    boundary: Option<super::sequence::Boundary>,
}

impl<'a> SequenceTask<'a> {
    fn new(expression: Box<Expr>, context: ContextId, loans: ActiveLoans<'a>, mode: Mode, expected: Option<Type>, discarded: bool, precise: bool, snapshot_requested: bool) -> Self {
        Self { expression, context, loans, mode, expected, discarded, precise, snapshot_requested,
            snapshot: None, index: 0, pending: false, boundary: None }
    }
}

struct DiscardTask<'a> { expression: Box<Expr>, context: ContextId, loans: ActiveLoans<'a>, pending: bool }

enum StoreState { Start, Target(super::store::FieldTarget), Value(super::store::Value) }
struct StoreTask<'a> { expression: Box<Expr>, context: ContextId, loans: ActiveLoans<'a>, state: StoreState, precise: bool }

struct CallableTask<'a> {
    body: Option<Body<'a>>,
    context: ContextId,
    result_type: Type,
    boundary: Option<super::exceptions::CallableBoundary>,
}

enum Task<'a> { CollectionConstructor(Box<collection_constructor::CollectionConstructorTask<'a>>), ObjectSpread(Box<object_spread::ObjectSpreadTask<'a>>), ArraySpread(Box<array_spread::ArraySpreadTask<'a>>), Spread(Box<spread::SpreadTask<'a>>), Generic(Box<generic::GenericTask>), Deferred(Box<deferred::DeferredTask<'a>>), Adapter(Box<adapters::AdapterTask<'a>>), Method(Box<methods::MethodTask>), Parameters(Box<parameters::ParametersTask>), Function(Box<functions::FunctionTask>), Optional(Box<optional::OptionalTask<'a>>), Callable(Box<CallableTask<'a>>), Region(Box<region::RegionTask<'a>>), Tail(tail::TailTask), Store(StoreTask<'a>), Sequence(SequenceTask<'a>), Discard(DiscardTask<'a>), Block(BlockTask<'a>), Expression(ExpressionTask<'a>), Assignment(AssignmentTask<'a>) }
enum Completion<'a> {
    Spread(Box<Expr>, Result<Type, Diagnostic>),
    Generic(Box<Expr>, Result<(), Diagnostic>),
    Deferred(Box<Expr>, Result<(usize, Type), Diagnostic>),
    Method(Result<(usize, Type), Diagnostic>),
    Parameters(Vec<Stmt>, Result<(), Diagnostic>),
    Function(Result<(), Diagnostic>),
    Region(region::Region<'a>, region::ResultStates),
    Tail(Box<Stmt>, Result<bool, Diagnostic>),
    Sequence(Box<Expr>, Result<Type, Diagnostic>, Option<Option<Box<crate::capture_loan_owners::Owners>>>),
    Assignment(AssignmentTask<'a>, Result<Type, Diagnostic>),
    Block(Body<'a>, Result<bool, Diagnostic>),
    Expression(Expression<'a>, Result<Type, Diagnostic>),
}
enum Step<'a> {
    Push(Task<'a>, Task<'a>),
    Continue(Task<'a>),
    Complete(Completion<'a>),
}

pub(super) fn check_exception_callable(analyzer: &mut Analyzer, context: &mut Context, body: &mut [Stmt], result: Type) -> Result<bool, Diagnostic> {
    let task = Task::Callable(Box::new(CallableTask { body: Some(Body::Root(body)), context: ContextId(0), result_type: result, boundary: None }));
    let Completion::Block(_, result) = drive(analyzer, context, task) else { unreachable!("exception callable result") };
    result
}

fn step_callable<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: Box<CallableTask<'a>>, completion: Option<Completion<'a>>) -> Step<'a> {
    if let Some(boundary) = task.boundary.take() {
        let Completion::Block(body, checked) = completion.expect("exception callable body") else { unreachable!("exception callable child") };
        let result = Analyzer::finish_exception_callable(context, boundary, checked);
        Step::Complete(Completion::Block(body, result))
    } else {
        task.boundary = Some(analyzer.prepare_exception_callable(context));
        let body = task.body.take().expect("callable source");
        let child = Task::Block(BlockTask { body, context: task.context, result_type: task.result_type,
            frame: None, pending: None, keep_names: false, label_span: None });
        Step::Push(Task::Callable(task), child)
    }
}

#[cfg(test)]
pub(super) fn check_region(analyzer: &mut Analyzer, context: &mut Context, region: &mut ExceptionRegion, result: Type, mode: super::exceptions::RegionMode, runtime: Option<Type>, unwind: &mut dyn FnMut(Span, &mut exception_state::ExitState)) -> region::ResultStates {
    let environment = region::environment(analyzer, mode, runtime)?;
    let task = region::RegionTask::new(region::Region::Root(region), ContextId(0), result, environment, Some(unwind));
    let Completion::Region(_, result) = drive(analyzer, context, Task::Region(task)) else { unreachable!("root exception region") };
    result
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

pub(super) fn check_assignment(analyzer: &mut Analyzer, context: &mut Context, target: &mut Expr, value: &mut Expr, kind: super::assignment::Kind, operator: Span, loans: &Loans) -> Result<Type, Diagnostic> {
    let target_span = target.span;
    let value_span = value.span;
    let task = AssignmentTask {
        target: Box::new(std::mem::replace(target, Expr::new(ExprKind::Undefined, target_span))),
        value: Box::new(std::mem::replace(value, Expr::new(ExprKind::Undefined, value_span))),
        kind, operator, loans: ActiveLoans::Borrowed(loans), context: ContextId(0), state: AssignmentState::Start,
    };
    let Completion::Assignment(task, result) = drive(analyzer, context, Task::Assignment(task)) else { unreachable!("assignment result"); };
    *target = *task.target;
    *value = *task.value;
    result
}

fn step_assignment<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: AssignmentTask<'a>, completion: Option<Completion<'a>>) -> Step<'a> {
    match std::mem::replace(&mut task.state, AssignmentState::Start) {
        AssignmentState::Start => {
            let frame = match analyzer.prepare_assignment_target(context, &mut task.target, task.kind) {
                Ok(frame) => frame,
                Err(error) => return Step::Complete(Completion::Assignment(task, Err(error))),
            };
            let span = task.target.span;
            let target = std::mem::replace(&mut task.target, Box::new(Expr::new(ExprKind::Undefined, span)));
            let child = Task::Expression(ExpressionTask::new(Expression::Child(target), task.context, Mode::Mutate, task.loans.clone(), None));
            task.state = AssignmentState::Target(frame);
            Step::Push(Task::Assignment(task), child)
        }
        AssignmentState::Target(frame) => {
            let Completion::Expression(target, checked) = completion.expect("assignment target completed") else { unreachable!("target result"); };
            task.target = target.into_child();
            match analyzer.assignment_selected(context, &mut task.target, &mut task.value, frame, task.operator, &task.loans, checked) {
                Ok(super::assignment::Selected::Ready(ty)) => Step::Complete(Completion::Assignment(task, Ok(ty))),
                Err(error) => Step::Complete(Completion::Assignment(task, Err(error))),
                Ok(super::assignment::Selected::Value { frame, mode, loans, expected }) => {
                    let span = task.value.span;
                    let value = std::mem::replace(&mut task.value, Box::new(Expr::new(ExprKind::Undefined, span)));
                    let child = Task::Expression(ExpressionTask::new(Expression::Child(value), task.context, mode, ActiveLoans::Shared(std::rc::Rc::new(loans)), Some(expected)));
                    task.state = AssignmentState::Value(frame);
                    Step::Push(Task::Assignment(task), child)
                }
            }
        }
        AssignmentState::Value(frame) => {
            let Completion::Expression(value, checked) = completion.expect("assignment value completed") else { unreachable!("value result"); };
            task.value = value.into_child();
            let result = checked.and_then(|actual| analyzer.finish_assignment_value(context, &task.target, &task.value, frame, task.operator, &task.loans, actual));
            Step::Complete(Completion::Assignment(task, result))
        }
    }
}

pub(super) fn check_store(analyzer: &mut Analyzer, context: &mut Context, target: &mut Expr, value: &mut Expr, loans: &Loans, precise: bool) -> Result<(), Diagnostic> {
    let span = target.span;
    let value_span = value.span;
    let expression = Box::new(Expr::new(ExprKind::Assign {
        target: Box::new(std::mem::replace(target, Expr::new(ExprKind::Undefined, span))),
        value: Box::new(std::mem::replace(value, Expr::new(ExprKind::Undefined, value_span))), operator: span,
    }, span));
    let root = Task::Store(StoreTask { expression, context: ContextId(0), loans: ActiveLoans::Borrowed(loans), state: StoreState::Start, precise });
    let Completion::Expression(mut expression, result) = drive(analyzer, context, root) else { unreachable!("store result"); };
    let ExprKind::Assign { target: old_target, value: old_value, .. } = &mut expression.value().kind else { unreachable!("store operands"); };
    std::mem::swap(target, old_target);
    std::mem::swap(value, old_value);
    result.map(|_| ())
}

fn store_value_child<'a>(mut task: StoreTask<'a>, frame: super::store::Value) -> Step<'a> {
    let ExprKind::Assign { value, .. } = &mut task.expression.kind else { unreachable!("store value slot"); };
    let span = value.span;
    let value = std::mem::replace(value, Box::new(Expr::new(ExprKind::Undefined, span)));
    let loans = ActiveLoans::Shared(std::rc::Rc::new(frame.loans.clone()));
    let child = if task.precise {
        Task::Sequence(SequenceTask::new(value, task.context, loans, Mode::Consume, frame.expected, false, true, true))
    } else { Task::Expression(ExpressionTask::new(Expression::Child(value), task.context, Mode::Consume, loans, frame.expected)) };
    task.state = StoreState::Value(frame);
    Step::Push(Task::Store(task), child)
}

fn step_store<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: StoreTask<'a>, completion: Option<Completion<'a>>) -> Step<'a> {
    let result = match std::mem::replace(&mut task.state, StoreState::Start) {
        StoreState::Start => {
            let ExprKind::Assign { target, value, .. } = &mut task.expression.kind else { unreachable!("store operands"); };
            if let Err(error) = analyzer.reject_string_position_write(context, target) { Err(error) }
            else if matches!(target.kind, ExprKind::Name(_)) {
                match analyzer.prepare_named_store_work(context, target, value, &task.loans) {
                    Ok(frame) => return store_value_child(task, frame),
                    Err(error) => Err(error),
                }
            } else {
                match analyzer.prepare_field_store_work(context, target) {
                    Ok(frame) => {
                        let span = target.span;
                        let target = std::mem::replace(target, Box::new(Expr::new(ExprKind::Undefined, span)));
                        let child = Task::Expression(ExpressionTask::new(Expression::Child(target), task.context, Mode::Mutate, task.loans.clone(), None));
                        task.state = StoreState::Target(frame);
                        return Step::Push(Task::Store(task), child);
                    }
                    Err(error) => Err(error),
                }
            }
        }
        StoreState::Target(frame) => {
            let Completion::Expression(target, result) = completion.expect("store target completed") else { unreachable!("target result"); };
            let ExprKind::Assign { target: destination, value, .. } = &mut task.expression.kind else { unreachable!("store target slot"); };
            *destination = target.into_child();
            match analyzer.field_store_selected(context, destination, value, frame, &task.loans, result) {
                Ok(frame) => return store_value_child(task, frame),
                Err(error) => Err(error),
            }
        }
        StoreState::Value(mut frame) => {
            let (value, result) = match completion.expect("store value completed") {
                Completion::Expression(value, result) => (value.into_child(), result),
                Completion::Sequence(value, result, snapshot) => {
                    if let (super::store::Finish::Named { removal, .. }, Some(snapshot)) = (&mut frame.finish, snapshot) { *removal = snapshot; }
                    if result.is_ok() { context.storage_exits.record_bound_sequence(value.span); }
                    (value, result)
                }
                _ => unreachable!("store value result"),
            };
            let ExprKind::Assign { target, value: destination, .. } = &mut task.expression.kind else { unreachable!("store value slot"); };
            *destination = value;
            result.and_then(|actual| analyzer.finish_store_work(context, target, destination, frame, &task.loans, actual))
        }
    };
    if result.is_ok() { task.expression.ty = Some(Type::Void); }
    Step::Complete(Completion::Expression(Expression::Child(task.expression), result.map(|_| Type::Void)))
}

pub(super) fn check_sequence(analyzer: &mut Analyzer, context: &mut Context, expression: &mut Expr, mode: Mode, loans: &Loans, expected: Option<Type>, discarded: bool, precise: bool, final_snapshot: Option<&mut Option<Box<crate::capture_loan_owners::Owners>>>) -> Result<Type, Diagnostic> {
    let span = expression.span;
    let value = Box::new(std::mem::replace(expression, Expr::new(ExprKind::Undefined, span)));
    let task = SequenceTask::new(value, ContextId(0), ActiveLoans::Borrowed(loans), mode, expected, discarded, precise, final_snapshot.is_some());
    let Completion::Sequence(value, result, snapshot) = drive(analyzer, context, Task::Sequence(task)) else { unreachable!("sequence result"); };
    *expression = *value;
    if let (Some(destination), Some(snapshot)) = (final_snapshot, snapshot) { *destination = snapshot; }
    result
}

fn step_sequence<'a>(_analyzer: &mut Analyzer, context: &mut Context, mut task: SequenceTask<'a>, completion: Option<Completion<'a>>) -> Step<'a> {
    let ExprKind::Sequence(values) = &mut task.expression.kind else { unreachable!("sequence operands"); };
    if task.pending {
        let Completion::Expression(value, checked) = completion.expect("sequence operand completed") else { unreachable!("sequence operand result"); };
        values[task.index] = *value.into_child();
        if let Some(boundary) = task.boundary.take() { Analyzer::finish_sequence_operand(context, &values[task.index], boundary, checked.is_ok()); }
        let ty = match checked {
            Ok(ty) => ty,
            Err(error) => return Step::Complete(Completion::Sequence(task.expression, Err(error), task.snapshot)),
        };
        task.index += 1;
        if task.index == values.len() {
            let last = values.last().expect("nonempty sequence");
            let ty = if task.discarded { Type::Void } else { ty };
            task.expression.ty = Some(ty);
            task.expression.binding = if task.discarded { None } else { last.binding };
            task.expression.callable_origin = if task.discarded { None } else { last.callable_origin };
            let outcomes = Analyzer::condition_outcomes(context, last);
            context.condition_flows.insert((task.expression.span.start, task.expression.span.end), outcomes);
            return Step::Complete(Completion::Sequence(task.expression, Ok(ty), task.snapshot));
        }
    }
    let prefix = task.index + 1 < values.len();
    let discard = prefix || task.discarded;
    if discard { task.boundary = Some(Analyzer::prepare_sequence_operand(context, &values[task.index], task.precise, prefix)); }
    else if task.snapshot_requested { task.snapshot = Some(Analyzer::array_removal_snapshot(context, &values[task.index])); }
    let span = values[task.index].span;
    let value = Box::new(std::mem::replace(&mut values[task.index], Expr::new(ExprKind::Undefined, span)));
    let child = if discard { Task::Discard(DiscardTask { expression: value, context: task.context, loans: task.loans.clone(), pending: false }) }
        else { Task::Expression(ExpressionTask::new(Expression::Child(value), task.context, task.mode, task.loans.clone(), task.expected)) };
    task.pending = true;
    Step::Push(Task::Sequence(task), child)
}

fn step_discard<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: DiscardTask<'a>, completion: Option<Completion<'a>>) -> Step<'a> {
    if task.pending {
        return match completion.expect("discarded child completed") {
            Completion::Sequence(expr, result, _) => Step::Complete(Completion::Expression(Expression::Child(expr), result)),
            result @ Completion::Expression(..) => Step::Complete(result),
            Completion::Assignment(child, result) => {
                let ExprKind::CompoundAssign { target, value, .. } = &mut task.expression.kind else { unreachable!("discarded string compound assignment"); };
                *target = child.target; *value = child.value;
                if result.is_ok() { task.expression.ty = Some(Type::Void); }
                Step::Complete(Completion::Expression(Expression::Child(task.expression), result.map(|_| Type::Void)))
            }
            _ => unreachable!("discard completion"),
        };
    }
    if matches!(task.expression.kind, ExprKind::Sequence(_)) {
        let span = task.expression.span;
        let value = std::mem::replace(&mut task.expression, Box::new(Expr::new(ExprKind::Undefined, span)));
        let child = Task::Sequence(SequenceTask::new(value, task.context, task.loans.clone(), Mode::Read, None, true, false, false));
        task.pending = true;
        return Step::Push(Task::Discard(task), child);
    }
    if let ExprKind::Assign { target, .. } = &task.expression.kind
        && analyzer.expression_type_hint(context, target).is_some_and(|ty| !ty.is_copy()) {
        let span = task.expression.span;
        let expression = std::mem::replace(&mut task.expression, Box::new(Expr::new(ExprKind::Undefined, span)));
        let child = Task::Store(StoreTask { expression, context: task.context, loans: task.loans.clone(), state: StoreState::Start, precise: false });
        task.pending = true;
        return Step::Push(Task::Discard(task), child);
    }
    if let ExprKind::CompoundAssign { target, value, op, operator } = &mut task.expression.kind
        && analyzer.expression_type_hint(context, target) == Some(Type::String) {
        let span = target.span;
        let target = std::mem::replace(target, Box::new(Expr::new(ExprKind::Undefined, span)));
        let span = value.span;
        let value = std::mem::replace(value, Box::new(Expr::new(ExprKind::Undefined, span)));
        let child = Task::Assignment(AssignmentTask { target, value, kind: super::assignment::Kind::Compound { op: *op, allow_owned: true }, operator: *operator,
            loans: task.loans.clone(), context: task.context, state: AssignmentState::Start });
        task.pending = true;
        return Step::Push(Task::Discard(task), child);
    }
    let span = task.expression.span;
    let value = std::mem::replace(&mut task.expression, Box::new(Expr::new(ExprKind::Undefined, span)));
    let child = Task::Expression(ExpressionTask::new(Expression::Child(value), task.context, Mode::Read, task.loans.clone(), None));
    task.pending = true;
    Step::Push(Task::Discard(task), child)
}

fn drive<'a>(analyzer: &mut Analyzer, context: &mut Context, root: Task<'a>) -> Completion<'a> {
    let mut contexts = Contexts { root: context, children: Vec::new() };
    let mut tasks = vec![root];
    let mut completion = None;
    loop {
        let Some(task) = tasks.pop() else { return completion.expect("root completion") };
        let step = match task {
            Task::CollectionConstructor(task) => collection_constructor::step(analyzer, contexts.get(task.context), task, completion.take()),
            Task::ObjectSpread(task) => object_spread::step(analyzer, contexts.get(task.context), task, completion.take()),
            Task::ArraySpread(task) => array_spread::step(analyzer, contexts.get(task.context), task, completion.take()),
            Task::Spread(task) => spread::step(analyzer, contexts.get(task.context), task, completion.take()),
            Task::Generic(task) => generic::step(analyzer, task, completion.take()),
            Task::Deferred(task) => deferred::step(analyzer, contexts.get(task.context), task, completion.take()),
            Task::Adapter(task) => adapters::step(analyzer, contexts.get(task.context), task, completion.take()),
            Task::Method(task) => methods::step(analyzer, &mut contexts, task, completion.take()),
            Task::Parameters(task) => parameters::step(analyzer, contexts.get(task.context), task, completion.take()),
            Task::Function(task) => functions::step(analyzer, &mut contexts, task, completion.take()),
            Task::Optional(task)=>optional::step(analyzer,contexts.get(task.context),task,completion.take()),
            Task::Callable(task) => step_callable(analyzer, contexts.get(task.context), task, completion.take()),
            Task::Region(task) => region::step(analyzer, &mut contexts, task, completion.take()),
            Task::Tail(task) => tail::step(analyzer, &mut contexts, task, completion.take()),
            Task::Store(task) => step_store(analyzer, contexts.get(task.context), task, completion.take()),
            Task::Sequence(task) => step_sequence(analyzer, contexts.get(task.context), task, completion.take()),
            Task::Discard(task) => step_discard(analyzer, contexts.get(task.context), task, completion.take()),
            Task::Assignment(task) => step_assignment(analyzer, contexts.get(task.context), task, completion.take()),
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
                (BlockPending::Region(index), Completion::Region(region, result)) => {
                    let statement = &mut task.body.statements()[index];
                    let StmtKind::Try(destination) = &mut statement.kind else { unreachable!("exception region slot") };
                    *destination = region.into_child();
                    let (completed, _) = result?;
                    task.frame.as_mut().expect("region scope").exited = completed.forward_exits(analyzer, context, statement.span)?;
                }
                (BlockPending::Throw(index), Completion::Expression(expression, result)) => {
                    let StmtKind::Throw(value) = &mut task.body.statements()[index].kind else { unreachable!("throw value slot") };
                    *value = *expression.into_child();
                    analyzer.finish_throw_value(context, value, result?)?;
                    task.frame.as_mut().expect("throw scope").exited = true;
                }
                (BlockPending::Assignment(index), completion) => {
                    let (target, value, result) = match completion {
                        Completion::Assignment(assignment, result) => (assignment.target, assignment.value, result),
                        Completion::Expression(expression, result) => {
                            let ExprKind::Assign { target, value, .. } = expression.into_child().into_kind() else { unreachable!("store statement result") };
                            (target, value, result)
                        },
                        _ => unreachable!("assignment statement completion"),
                    };
                    let StmtKind::Assign { target: destination, value: replacement, .. } = &mut task.body.statements()[index].kind else { unreachable!("assignment statement slots") };
                    *destination = *target;
                    *replacement = *value;
                    result?;
                }
                (BlockPending::Tail(index), Completion::Tail(statement, result)) => {
                    task.body.statements()[index] = *statement;
                    task.frame.as_mut().expect("loop scope").exited = result?;
                }
                (BlockPending::WhileCondition { index, frame }, Completion::Expression(expression, result)) => {
                    let stmt = &mut task.body.statements()[index];
                    let StmtKind::While { condition, body } = &mut stmt.kind else { unreachable!("while condition slot") };
                    *condition = *expression.into_child();
                    let frame = analyzer.prepare_while_body(context, condition, result?, frame, stmt.span, &stmt.labels)?;
                    let body = std::mem::take(body);
                    task.pending = Some(BlockPending::LoopBody { index, frame });
                    return Ok(BlockAction::Push(Task::Block(BlockTask {
                        body: Body::Child { body, start: 0 }, context: task.context, frame: None, pending: None,
                        result_type: task.result_type, keep_names: false, label_span: None,
                    })));
                }
                (BlockPending::Iterable(index), Completion::Expression(expression, result)) => {
                    let stmt = &mut task.body.statements()[index];
                    let StmtKind::ForOf { name, rebindable, binding, iterable, body } = &mut stmt.kind else { unreachable!("iterable slot") };
                    *iterable = *expression.into_child();
                    let frame = analyzer.prepare_iteration_body(context, (name, *rebindable, binding), iterable, result?, stmt.span, &stmt.labels)?;
                    let body = std::mem::take(body);
                    task.pending = Some(BlockPending::LoopBody { index, frame });
                    return Ok(BlockAction::Push(Task::Block(BlockTask {
                        body: Body::Child { body, start: 0 }, context: task.context, frame: None, pending: None,
                        result_type: task.result_type, keep_names: false, label_span: None,
                    })));
                }
                (BlockPending::LoopBody { index, frame }, Completion::Block(completed, result)) => {
                    let stmt = &mut task.body.statements()[index];
                    let body = match &mut stmt.kind {
                        StmtKind::While { body, .. } | StmtKind::ForOf { body, .. } => body,
                        _ => unreachable!("loop body slot"),
                    };
                    *body = completed.into_child();
                    let breaks = analyzer.finish_scheduled_loop(context, body, stmt.span, frame, result)?;
                    if !breaks && let StmtKind::While { condition, .. } = &stmt.kind
                        && let Some((_, no)) = analyzer.pure_guard_outcomes(context, condition) { context.flow = no; }
                }
                (BlockPending::Declaration { index, mut frame, module_scope }, completion) => {
                    let (expression, result) = match completion {
                        Completion::Expression(expression, result) => (expression.into_child(), result),
                        Completion::Sequence(expression, result, snapshot) => {
                            if let Some(snapshot) = snapshot { frame.removal = snapshot; }
                            (expression, result)
                        },
                        _ => unreachable!("declaration initializer result"),
                    };
                    let StmtKind::Let { rebindable, name, value, binding, .. } = &mut task.body.statements()[index].kind else { unreachable!("declaration initializer slot") };
                    *value = *expression;
                    let result = result.and_then(|actual| analyzer.finish_declaration(context, *rebindable, name, value, binding, frame, actual));
                    analyzer.module_literal_scope = module_scope;
                    result?;
                }
                (BlockPending::IfCondition(index), Completion::Expression(expression, result)) => {
                    let StmtKind::If { condition, then_body, .. } = &mut task.body.statements()[index].kind else { unreachable!("if condition slot") };
                    *condition = *expression.into_child();
                    let branch = analyzer.prepare_if_then(context, condition, result?)?;
                    let body = std::mem::take(then_body);
                    task.pending = Some(BlockPending::IfThen { index, frame: branch });
                    return Ok(BlockAction::Push(Task::Block(BlockTask {
                        body: Body::Child { body, start: 0 }, context: task.context, frame: None, pending: None,
                        result_type: task.result_type, keep_names: false, label_span: None,
                    })));
                }
                (BlockPending::IfThen { index, frame }, Completion::Block(body, result)) => {
                    let StmtKind::If { then_body, else_body, .. } = &mut task.body.statements()[index].kind else { unreachable!("then body slot") };
                    *then_body = body.into_child();
                    let exited = match result { Ok(exited) => exited, Err(error) => { frame.abort(context); return Err(error); } };
                    let branch = Analyzer::prepare_if_else(context, frame, exited);
                    let body = std::mem::take(else_body);
                    task.pending = Some(BlockPending::IfElse { index, frame: branch });
                    return Ok(BlockAction::Push(Task::Block(BlockTask {
                        body: Body::Child { body, start: 0 }, context: task.context, frame: None, pending: None,
                        result_type: task.result_type, keep_names: false, label_span: None,
                    })));
                }
                (BlockPending::IfElse { index, frame }, Completion::Block(body, result)) => {
                    let stmt = &mut task.body.statements()[index];
                    let StmtKind::If { then_body, else_body, .. } = &mut stmt.kind else { unreachable!("else body slot") };
                    *else_body = body.into_child();
                    let exited = match result { Ok(exited) => exited, Err(error) => { frame.abort(context); return Err(error); } };
                    task.frame.as_mut().expect("if scope").exited = analyzer.finish_if(context, frame, then_body, else_body, exited, stmt.span)?;
                }
                (BlockPending::Expression { index, removal, precise }, completion) => {
                    let (expression, result) = match completion {
                        Completion::Expression(expression, result) => (expression.into_child(), result),
                        Completion::Sequence(expression, result, _) => (expression, result),
                        _ => unreachable!("statement expression result"),
                    };
                    let StmtKind::Expr(value) = &mut task.body.statements()[index].kind else { unreachable!("statement expression slot") };
                    *value = *expression;
                    let ty = result?;
                    if precise { context.storage_exits.record_discarded_sequence(value.span); }
                    if matches!(ty, Type::Closure(_)) && !matches!(value.kind, ExprKind::Call(..)) {
                        return Err(Diagnostic::new("unsupported", "Bind a closure before using it.", value.span));
                    }
                    Analyzer::finish_discarded_array_removal(context, value, removal);
                }
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
            let exited = if let Some(span) = task.label_span.take() {
                analyzer.finish_labeled_block(context, task.body.statements(), exited, span)?
            } else { exited };
            return Ok(BlockAction::Done(exited));
        };
        let stmt = &mut task.body.statements()[index];
        match &mut stmt.kind {
            StmtKind::Try(region) if context.exception_environment.is_some() && context.completion_frame.is_some() => {
                let active = context.exception_environment.as_ref().expect("exception environment");
                let mode = if active.partial_contracts { super::exceptions::RegionMode::Partial }
                    else if active.refresh_contracts { super::exceptions::RegionMode::Current }
                    else { super::exceptions::RegionMode::Fixed(active.effects.clone()) };
                let environment = region::environment(analyzer, mode, active.runtime)?;
                let region = std::mem::replace(region, Box::new(ExceptionRegion { body: Vec::new(), catch: None, finally: None }));
                let child = region::RegionTask::new(region::Region::Child(region), task.context, task.result_type, environment, None);
                task.pending = Some(BlockPending::Region(index));
                Ok(BlockAction::Push(Task::Region(child)))
            }
            StmtKind::Throw(value) if context.completion_frame.is_some() => {
                let expected = analyzer.prepare_throw_value(context, value)?;
                let value = Box::new(std::mem::replace(value, Expr::new(ExprKind::Undefined, value.span)));
                task.pending = Some(BlockPending::Throw(index));
                Ok(BlockAction::Push(Task::Expression(ExpressionTask::new(Expression::Child(value), task.context,
                    Mode::Consume, ActiveLoans::Shared(std::rc::Rc::new(Loans::new())), expected))))
            }
            StmtKind::Assign { target, value, operator } => {
                let precise = operator.is_none() && matches!(target.kind, ExprKind::Name(_))
                    && analyzer.expression_type_hint(context, target).is_some_and(|ty| analyzer.aggregate_callable_facts(ty).scoped || analyzer.nullable_scoped_record_declaration(ty).is_some())
                    && analyzer.prepare_bound_removal_sequence(context, value);
                analyzer.reject_string_position_write(context, target)?;
                let kind = if let Some((op, _)) = operator { Some(super::assignment::Kind::Compound { op: *op, allow_owned: true }) }
                    else if !precise && !matches!(crate::flow::place(target, &context.names).and_then(|place| analyzer.place_type(context, &place)), Some(Type::Union(_)))
                        && matches!(analyzer.expression_type_hint(context, target), Some(Type::Int | Type::Number | Type::Boolean)) { Some(super::assignment::Kind::Scalar) }
                    else { None };
                let operator = operator.map_or(target.span, |(_, span)| span);
                let target = Box::new(std::mem::replace(target, Expr::new(ExprKind::Undefined, target.span)));
                let value = Box::new(std::mem::replace(value, Expr::new(ExprKind::Undefined, value.span)));
                let loans = ActiveLoans::Shared(std::rc::Rc::new(Loans::new()));
                let child = if let Some(kind) = kind {
                    Task::Assignment(AssignmentTask { target, value, kind, operator, loans, context: task.context, state: AssignmentState::Start })
                } else {
                    let expression = Box::new(Expr::new(ExprKind::Assign { target, value, operator }, stmt.span));
                    Task::Store(StoreTask { expression, context: task.context, loans, state: StoreState::Start, precise })
                };
                task.pending = Some(BlockPending::Assignment(index));
                Ok(BlockAction::Push(child))
            }
            StmtKind::Loop { .. } => {
                let statement = Box::new(std::mem::replace(stmt, Stmt { kind: StmtKind::Break, span: stmt.span, labels: Vec::new(), control_target: None }));
                task.pending = Some(BlockPending::Tail(index));
                Ok(BlockAction::Push(Task::Tail(tail::TailTask { statement, context: task.context, result_type: task.result_type,
                    control_boundary: context.loops.len(), isolated: None, state: tail::State::Start })))
            }
            StmtKind::While { condition, .. } => {
                let frame = Analyzer::prepare_while_condition(context);
                analyzer.truthy_condition(context, condition);
                let value = std::mem::replace(condition, Expr::new(ExprKind::Undefined, condition.span));
                task.pending = Some(BlockPending::WhileCondition { index, frame });
                Ok(BlockAction::Push(Task::Expression(ExpressionTask::new(Expression::Child(Box::new(value)), task.context,
                    Mode::Copy, ActiveLoans::Shared(std::rc::Rc::new(Loans::new())), Some(Type::Boolean)))))
            }
            StmtKind::ForOf { iterable, .. } => {
                let value = std::mem::replace(iterable, Expr::new(ExprKind::Undefined, iterable.span));
                task.pending = Some(BlockPending::Iterable(index));
                Ok(BlockAction::Push(Task::Expression(ExpressionTask::new(Expression::Child(Box::new(value)), task.context,
                    Mode::Read, ActiveLoans::Shared(std::rc::Rc::new(Loans::new())), None))))
            }
            StmtKind::Let { name, annotation, value, binding, .. } => {
                let module_scope = analyzer.module_literal_scope;
                analyzer.module_literal_scope = analyzer.program.module_owner_bindings.contains(&name.span).then_some(value.span);
                let declaration = match analyzer.prepare_declaration(context, name, annotation, value, binding) {
                    Ok(Some(declaration)) => declaration,
                    result => { analyzer.module_literal_scope = module_scope; result?; return Ok(BlockAction::Continue); },
                };
                let value = Box::new(std::mem::replace(value, Expr::new(ExprKind::Undefined, value.span)));
                let loans = ActiveLoans::Shared(std::rc::Rc::new(Loans::new()));
                let child = if declaration.retained_sequence {
                    Task::Sequence(SequenceTask::new(value, task.context, loans, declaration.mode, declaration.expected, false, true, true))
                } else {
                    Task::Expression(ExpressionTask::new(Expression::Child(value), task.context, declaration.mode, loans, declaration.expected))
                };
                task.pending = Some(BlockPending::Declaration { index, frame: declaration, module_scope });
                Ok(BlockAction::Push(child))
            }
            StmtKind::If { condition, .. } => {
                analyzer.truthy_condition(context, condition);
                let value = std::mem::replace(condition, Expr::new(ExprKind::Undefined, condition.span));
                task.pending = Some(BlockPending::IfCondition(index));
                Ok(BlockAction::Push(Task::Expression(ExpressionTask::new(
                    Expression::Child(Box::new(value)), task.context, Mode::Read,
                    ActiveLoans::Shared(std::rc::Rc::new(Loans::new())), Some(Type::Boolean),
                ))))
            }
            StmtKind::Expr(value) => {
                analyzer.truthy_logical_assignment(context, value);
                let removal = Analyzer::array_removal_snapshot(context, value);
                let view = analyzer.expression_type_hint(context, value).is_some_and(|ty| matches!(ty, Type::BorrowedUnion(_)));
                let place_view = analyzer.borrowed_place_hint(context, value);
                let sequence = matches!(value.kind, ExprKind::Sequence(_));
                let precise = sequence && analyzer.prepare_removal_sequence(context, value);
                let value = Box::new(std::mem::replace(value, Expr::new(ExprKind::Undefined, value.span)));
                let loans = ActiveLoans::Shared(std::rc::Rc::new(Loans::new()));
                let child = if sequence {
                    Task::Sequence(SequenceTask::new(value, task.context, loans, Mode::Read, None, true, precise, false))
                } else {
                    Task::Expression(ExpressionTask::new(Expression::Child(value), task.context,
                        if view || place_view { Mode::Read } else { Mode::Consume }, loans, None))
                };
                task.pending = Some(BlockPending::Expression { index, removal, precise });
                Ok(BlockAction::Push(child))
            }
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
        Err(error) => {
            if let Some(frame) = task.frame.take() { analyzer.leave_block(context, frame, true); }
            if task.label_span.take().is_some() {
                let frame = context.loops.pop().expect("failed labeled block frame");
                exception_state::consume_target(context, frame.target);
            }
            Step::Complete(Completion::Block(task.body, Err(error)))
        },
    }
}

fn schedule_conditional_arm<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, plan: Box<ConditionalPlan>, mode: Mode, left: Option<Type>) -> Step<'a> {
    let expr = task.expression.value();
    let ExprKind::Conditional { consequent, alternate, .. } = &mut expr.kind else { unreachable!("conditional arm parent"); };
    let expected = if left.is_some() { plan.right_context } else { plan.left_context };
    let value = if left.is_some() { alternate } else { consequent };
    let stored = if let Some(Type::StoredCallable(family)) = expected {
        match analyzer.prepare_stored_value(context, value, family) {
            Ok(frame) => Some(frame),
            Err(error) => return finish_expression(analyzer, context, task, Err(error)),
        }
    } else { None };
    let child_mode = if stored.is_some() { Mode::Consume } else { mode };
    let child_expected = stored.as_ref().map_or(expected, |frame| frame.expected);
    let span = value.span;
    let value = std::mem::replace(value, Box::new(Expr::new(ExprKind::Undefined, span)));
    let child = Task::Expression(ExpressionTask::new(Expression::Child(value), task.context, child_mode, task.loans.clone(), child_expected));
    task.state = ExpressionState::ConditionalArm { plan, mode, left, stored };
    Step::Push(Task::Expression(task), child)
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
        ExpressionState::ArraySpread | ExpressionState::ObjectSpread | ExpressionState::CollectionConstructor => {
            let Completion::Spread(value,result)=completion.expect("array spread completion") else{unreachable!()};
            *task.expression.value()=*value;
            finish_expression(analyzer,contexts.get(task.context),task,result)
        }
        ExpressionState::Spread => {
            let Completion::Spread(value,result)=completion.expect("spread call completion") else{unreachable!()};
            *task.expression.value()=*value;
            let context=contexts.get(task.context);
            let result=result.map(|ty|{analyzer.resolve_completed_capture_invocation(context,task.expression.value(),ty);ty});
            finish_expression(analyzer,context,task,result)
        }
        ExpressionState::GenericReceiver => {
            let Completion::Generic(value, result) = completion.expect("receiver generic preparation") else { unreachable!() };
            *Analyzer::call_preparation_receiver(task.expression.value()) = *value;
            let context = contexts.get(task.context);
            if let Err(error) = result { return finish_expression(analyzer, context, task, Err(error)); }
            start_invocation(analyzer, context, task, false, true)
        }
        ExpressionState::GenericCall => {
            let Completion::Generic(value, result) = completion.expect("call generic preparation") else { unreachable!() };
            *task.expression.value() = *value;
            let context = contexts.get(task.context);
            if let Err(error) = result { return finish_expression(analyzer, context, task, Err(error)); }
            start_invocation(analyzer, context, task, true, false)
        }

        ExpressionState::Deferred { origin, conditional } => {
            let Completion::Deferred(value, result) = completion.expect("deferred invocation") else { unreachable!() };
            *task.expression.value() = *value;
            let context = contexts.get(task.context);
            let expr = task.expression.value();
            let result = result.and_then(|(method, result)| {
                if let Some((frame, publish_type)) = conditional {
                    complete_conditional_call(analyzer, context, expr, frame, (CallTarget::ClosureMethod(method), result), publish_type)
                } else {
                    analyzer.remember_native_closure_returns(context, origin, Some(method), expr.span)?;
                    if analyzer.native_borrow_types { expr.callable_origin = Some(CallableOrigin::Closure(origin)); }
                    expr.target = Some(CallTarget::ClosureMethod(method));
                    expr.ty = Some(result);
                    analyzer.resolve_completed_capture_invocation(context, expr, result);
                    Ok(result)
                }
            });
            finish_expression(analyzer, context, task, result)
        }
        ExpressionState::Adapter => {
            let Completion::Expression(value, result) = completion.expect("generic adapter") else { unreachable!("adapter result") };
            *task.expression.value() = *value.into_child();
            finish_expression(analyzer, contexts.get(task.context), task, result)
        }
        ExpressionState::Reduction(state) => reduction::resume(analyzer, contexts.get(task.context), task, state, completion.expect("reduction child")),
        ExpressionState::Sorting(state) => sorting::resume(analyzer, contexts.get(task.context), task, state, completion.expect("sorting child")),
        ExpressionState::Transform(state) => transform::resume(analyzer, contexts.get(task.context), task, state, completion.expect("transform child")),
        ExpressionState::Wrapper(frame) => {
            let Completion::Expression(value, result) = completion.expect("wrapper operand completed") else { unreachable!("wrapper result") };
            let context = contexts.get(task.context);
            let expr = task.expression.value();
            *frame.child(expr) = value.into_child();
            let result = analyzer.finish_wrapper_work(expr, frame, result);
            finish_expression(analyzer, context, task, result)
        },
        ExpressionState::Optional=>{
            let Completion::Expression(value,result)=completion.expect("optional chain completed") else{unreachable!("optional result")};
            *task.expression.value()=*value.into_child();
            let context=contexts.get(task.context);
            if result.is_ok() && task.checked.is_none(){
                let expr=task.expression.value();expr.json_string_view=analyzer.json_string_proof(context,expr).is_some() || analyzer.is_string_view(context,expr);
            }
            finish_expression(analyzer,context,task,result)
        },
        ExpressionState::Prelude(frame)=>{
            let Completion::Expression(value,result)=completion.expect("contextual child completed") else{unreachable!("contextual child result")};
            let context=contexts.get(task.context);
            let expr=task.expression.value();*expr=*value.into_child();
            let result=analyzer.finish_prelude_work(context,expr,frame,result);
            finish_expression(analyzer,context,task,result)
        },
        ExpressionState::ClosureDefault{frame,child,work}=>closure_parameters::resume_default(analyzer,contexts,task,frame,child,work,completion.expect("default completed")),
        ExpressionState::ClosurePattern{frame,child,start}=>closure_parameters::resume_pattern(analyzer,contexts,task,frame,child,start,completion.expect("pattern completed")),
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
        ExpressionState::Method { origin, conditional } => {
            let Completion::Method(result) = completion.expect("method completed") else { unreachable!("method completion") };
            let context = contexts.get(task.context);
            let expr = task.expression.value();
            let result = result.and_then(|(method, result)| {
                let ExprKind::Call(_, args) = &mut expr.kind else { unreachable!("method invocation") };
                let result = analyzer.closure_method_result(context, origin, method, result, args, expr.span);
                if let Some((frame, publish_type)) = conditional {
                    complete_conditional_call(analyzer, context, expr, frame, (CallTarget::ClosureMethod(method), result), publish_type)
                } else {
                    analyzer.remember_native_closure_returns(context, origin, Some(method), expr.span)?;
                    if analyzer.native_borrow_types { expr.callable_origin = Some(CallableOrigin::Closure(origin)); }
                    expr.target = Some(CallTarget::ClosureMethod(method));
                    expr.ty = Some(result);
                    analyzer.resolve_completed_capture_invocation(context, expr, result);
                    Ok(result)
                }
            });
            finish_expression(analyzer, context, task, result)
        }
        ExpressionState::DeclaredFunction { index, resolve_completed } => {
            let Completion::Function(result) = completion.expect("declared function completed") else { unreachable!("declared function result") };
            let context = contexts.get(task.context);
            match result {
                Ok(()) => resume_declared_call(analyzer, context, task, index, resolve_completed),
                Err(error) => finish_expression(analyzer, context, task, Err(error)),
            }
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
        ExpressionState::NullishAssignmentTarget(frame) => {
            let Completion::Expression(value, checked) = completion.expect("nullish target completed") else { unreachable!("nullish target result"); };
            let context = contexts.get(task.context);
            let expr = task.expression.value();
            let ExprKind::CoalesceAssign { target, .. } = &mut expr.kind else { unreachable!("nullish target slot"); };
            *target = value.into_child();
            match analyzer.nullish_assignment_selected(context, expr, *frame, &task.loans, checked) {
                Err(error) => finish_expression(analyzer, context, task, Err(error)),
                Ok((frame, loans, expected)) => {
                    let ExprKind::CoalesceAssign { value, .. } = &mut expr.kind else { unreachable!("nullish value slot"); };
                    let span = value.span;
                    let value = std::mem::replace(value, Box::new(Expr::new(ExprKind::Undefined, span)));
                    let child = Task::Expression(ExpressionTask::new(Expression::Child(value), task.context, Mode::Consume, ActiveLoans::Shared(std::rc::Rc::new(loans)), Some(expected)));
                    task.state = ExpressionState::NullishAssignmentValue(Box::new(frame));
                    Step::Push(Task::Expression(task), child)
                }
            }
        }
        ExpressionState::NullishAssignmentValue(frame) => {
            let Completion::Expression(value, checked) = completion.expect("nullish value completed") else { unreachable!("nullish value result"); };
            let context = contexts.get(task.context);
            let expr = task.expression.value();
            let ExprKind::CoalesceAssign { value: destination, .. } = &mut expr.kind else { unreachable!("nullish value slot"); };
            *destination = value.into_child();
            let result = checked.and_then(|actual| analyzer.finish_nullish_assignment_work(context, expr, *frame, &task.loans, actual));
            finish_expression(analyzer, context, task, result)
        }
        ExpressionState::Sequence => {
            let Completion::Sequence(value, result, _) = completion.expect("expression sequence completed") else { unreachable!("sequence result"); };
            *task.expression.value() = *value;
            finish_expression(analyzer, contexts.get(task.context), task, result)
        }
        ExpressionState::Assignment => {
            let Completion::Assignment(child, result) = completion.expect("assignment completed") else { unreachable!("assignment completion"); };
            let expr = task.expression.value();
            match &mut expr.kind {
                ExprKind::Assign { target, value, .. } | ExprKind::CompoundAssign { target, value, .. }
                | ExprKind::LogicalAssign { target, value, .. } => { *target = child.target; *value = child.value; }
                ExprKind::Update { target, .. } => *target = child.target,
                _ => unreachable!("assignment slots"),
            }
            finish_expression(analyzer, contexts.get(task.context), task, result)
        }
        ExpressionState::LogicalLeft(frame) => {
            let Completion::Expression(value, checked) = completion.expect("logical left completed") else { unreachable!("logical result"); };
            let context = contexts.get(task.context);
            let expr = task.expression.value();
            let ExprKind::Binary { left, .. } = &mut expr.kind else { unreachable!("logical slots"); };
            *left = value.into_child();
            let mut frame = match checked.and_then(|lhs| analyzer.logical_right_work(context, expr, *frame, lhs, &task.loans)) {
                Ok(frame) => Box::new(frame),
                Err(error) => return finish_expression(analyzer, context, task, Err(error)),
            };
            let child_context = frame.context.take().map(|context| contexts.push(context));
            let ExprKind::Binary { right, .. } = &mut expr.kind else { unreachable!("logical slots"); };
            let span = right.span;
            let value = std::mem::replace(right, Box::new(Expr::new(ExprKind::Undefined, span)));
            let loans = ActiveLoans::Shared(std::rc::Rc::new(std::mem::take(&mut frame.loans)));
            let child = Task::Expression(ExpressionTask::new(Expression::Child(value), child_context.unwrap_or(task.context), frame.access, loans, frame.expected));
            task.state = ExpressionState::LogicalRight { frame, child_context };
            Step::Push(Task::Expression(task), child)
        }
        ExpressionState::LogicalRight { frame, child_context } => {
            let Completion::Expression(value, checked) = completion.expect("logical fallback completed") else { unreachable!("logical result"); };
            if let Some(child) = child_context { drop(contexts.take(child)); }
            let context = contexts.get(task.context);
            let expr = task.expression.value();
            let ExprKind::Binary { right, .. } = &mut expr.kind else { unreachable!("logical slots"); };
            *right = value.into_child();
            let result = checked.and_then(|rhs| analyzer.finish_logical_work(context, expr, *frame, rhs));
            finish_expression(analyzer, context, task, result)
        }
        ExpressionState::CoalesceLeft(frame) => {
            let Completion::Expression(value, checked) = completion.expect("coalescing left completed") else { unreachable!("coalescing result"); };
            let context = contexts.get(task.context);
            let expr = task.expression.value();
            let ExprKind::Binary { left, .. } = &mut expr.kind else { unreachable!("coalescing slots"); };
            *left = value.into_child();
            let mut frame = match checked.and_then(|lhs| analyzer.coalesce_right_work(context, expr, *frame, lhs, &task.loans)) {
                Ok(frame) => Box::new(frame),
                Err(error) => return finish_expression(analyzer, context, task, Err(error)),
            };
            let child_context = frame.context.take().map(|context| contexts.push(context));
            let ExprKind::Binary { right, .. } = &mut expr.kind else { unreachable!("coalescing slots"); };
            let span = right.span;
            let value = std::mem::replace(right, Box::new(Expr::new(ExprKind::Undefined, span)));
            let loans = ActiveLoans::Shared(std::rc::Rc::new(std::mem::take(&mut frame.loans)));
            let child = Task::Expression(ExpressionTask::new(Expression::Child(value), child_context.unwrap_or(task.context), frame.access, loans, frame.expected));
            task.state = ExpressionState::CoalesceRight { frame, child_context };
            Step::Push(Task::Expression(task), child)
        }
        ExpressionState::CoalesceRight { frame, child_context } => {
            let Completion::Expression(value, checked) = completion.expect("coalescing fallback completed") else { unreachable!("coalescing result"); };
            if let Some(child) = child_context { drop(contexts.take(child)); }
            let context = contexts.get(task.context);
            let expr = task.expression.value();
            let ExprKind::Binary { right, .. } = &mut expr.kind else { unreachable!("coalescing slots"); };
            *right = value.into_child();
            let result = checked.and_then(|rhs| analyzer.finish_coalesce_work(context, expr, *frame, rhs));
            finish_expression(analyzer, context, task, result)
        }
        ExpressionState::ConditionalPredicate { mode, expected } => {
            let Completion::Expression(value, checked) = completion.expect("conditional predicate completed") else { unreachable!("predicate result"); };
            let context = contexts.get(task.context);
            let expr = task.expression.value();
            let ExprKind::Conditional { predicate, consequent, alternate } = &mut expr.kind else { unreachable!("conditional slots"); };
            *predicate = value.into_child();
            let plan = checked.and_then(|ty| analyzer.require(ty, Some(Type::Boolean), predicate.span))
                .and_then(|()| analyzer.conditional_plan(context, predicate, consequent, alternate, mode, expected));
            match plan {
                Ok(plan) => schedule_conditional_arm(analyzer, context, task, plan, mode, None),
                Err(error) => finish_expression(analyzer, context, task, Err(error)),
            }
        }
        ExpressionState::ConditionalArm { mut plan, mode, left, stored } => {
            let Completion::Expression(value, checked) = completion.expect("conditional arm completed") else { unreachable!("conditional result"); };
            let context = contexts.get(task.context);
            let expr = task.expression.value();
            let ExprKind::Conditional { consequent, alternate, .. } = &mut expr.kind else { unreachable!("conditional slots"); };
            let destination = if left.is_some() { &mut *alternate } else { &mut *consequent };
            *destination = value.into_child();
            let checked = checked.and_then(|actual| {
                if let Some(stored) = stored { analyzer.finish_stored_value(context, destination, stored, actual) } else { Ok(actual) }
            });
            if let Some(left) = left {
                let result = analyzer.conditional_join(context, consequent, alternate, plan, left, checked);
                finish_expression(analyzer, context, task, result)
            } else {
                match analyzer.conditional_between(context, consequent, alternate, &mut plan, checked) {
                    Ok(left) => schedule_conditional_arm(analyzer, context, task, plan, mode, Some(left)),
                    Err(error) => finish_expression(analyzer, context, task, Err(error)),
                }
            }
        }
        ExpressionState::Unary { frame, expected } => {
            let Completion::Expression(value, checked) = completion.expect("unary operand completed") else { unreachable!("unary operand result"); };
            let context = contexts.get(task.context);
            let expr = task.expression.value();
            let ExprKind::Unary { value: destination, .. } = &mut expr.kind else { unreachable!("unary operand slot"); };
            *destination = value.into_child();
            let result = analyzer.finish_unary_work(context, expr, *frame, checked, expected);
            finish_expression(analyzer, context, task, result)
        }
        ExpressionState::BinaryLeft(mut frame) => {
            let Completion::Expression(value, checked) = completion.expect("left operand completed") else { unreachable!("binary operand result"); };
            let context = contexts.get(task.context);
            let expr = task.expression.value();
            let ExprKind::Binary { left, .. } = &mut expr.kind else { unreachable!("left operand slot"); };
            *left = value.into_child();
            let operand = match analyzer.finish_binary_left_work(context, expr, &mut frame, checked, &task.loans) {
                Ok(operand) => operand,
                Err(error) => return finish_expression(analyzer, context, task, Err(error)),
            };
            let ExprKind::Binary { right, .. } = &mut expr.kind else { unreachable!("right operand slot"); };
            let span = right.span;
            let value = std::mem::replace(right, Box::new(Expr::new(ExprKind::Undefined, span)));
            let loans = operand.loans.map_or_else(|| task.loans.clone(), |loans| ActiveLoans::Shared(std::rc::Rc::new(loans)));
            let child = Task::Expression(ExpressionTask::new(Expression::Child(value), task.context, operand.mode, loans, operand.expected));
            task.state = ExpressionState::BinaryRight(frame);
            Step::Push(Task::Expression(task), child)
        }
        ExpressionState::BinaryRight(frame) => {
            let Completion::Expression(value, checked) = completion.expect("right operand completed") else { unreachable!("binary operand result"); };
            let context = contexts.get(task.context);
            let expr = task.expression.value();
            let ExprKind::Binary { right, .. } = &mut expr.kind else { unreachable!("right operand slot"); };
            *right = value.into_child();
            let result = analyzer.finish_binary_right_work(context, expr, *frame, checked);
            finish_expression(analyzer, context, task, result)
        }
        ExpressionState::Start => {
            debug_assert!(completion.is_none());
            let context = contexts.get(task.context);
            let expr = task.expression.value();
            let prepared = analyzer.prepare_expression_work(context, expr, task.mode, &task.loans, task.expected, &mut task.completion);
            let frame = match prepared {
                Ok(PreparedExpression::Adapter(request)) => return adapters::schedule(task, *request),
                Ok(PreparedExpression::Evaluate(frame)) => frame,
                Ok(PreparedExpression::Wrapper { frame, mode, expected }) => {
                    let destination = frame.child(expr);
                    let span = destination.span;
                    let value = std::mem::replace(destination, Box::new(Expr::new(ExprKind::Undefined, span)));
                    let child = Task::Expression(ExpressionTask::new(Expression::Child(value), task.context, mode, task.loans.clone(), expected));
                    task.state = ExpressionState::Wrapper(frame);
                    return Step::Push(Task::Expression(task), child);
                },
                Ok(PreparedExpression::Literal(frame)) => return schedule_literal(analyzer, context, task, Box::new(frame)),
                Ok(PreparedExpression::Optional{mode,expected})=>return schedule_optional(task,mode,expected),
                Ok(PreparedExpression::Prelude{mode,expected,frame})=>{
                    let span=expr.span;
                    let value=Box::new(std::mem::replace(expr,Expr::new(ExprKind::Undefined,span)));
                    let child=Task::Expression(ExpressionTask::new(Expression::Child(value),task.context,mode,task.loans.clone(),expected));
                    task.state=ExpressionState::Prelude(frame);
                    return Step::Push(Task::Expression(task),child);
                },
                Ok(PreparedExpression::Sequence { mode, expected }) => {
                    let span = expr.span;
                    let value = Box::new(std::mem::replace(expr, Expr::new(ExprKind::Undefined, span)));
                    let child = Task::Sequence(SequenceTask::new(value, task.context, task.loans.clone(), mode, expected, false, false, false));
                    task.state = ExpressionState::Sequence;
                    return Step::Push(Task::Expression(task), child);
                }
                Ok(PreparedExpression::Ready(ty)) => return finish_expression(analyzer, context, task, Ok(ty)),
                Err(error) => return finish_expression(analyzer, context, task, Err(error)),
            };
            let invocation = frame.invocation;
            let expected = frame.expected;
            let mode = frame.mode;
            task.checked = Some(frame);
            match analyzer.prepare_named_adapter(context, expr, mode, expected) {
                Ok(Some(request)) => return adapters::schedule(task, request),
                Ok(None) => {},
                Err(error) => return finish_expression(analyzer, context, task, Err(error)),
            }
            if matches!(&expr.kind,ExprKind::ArrayOf(values,_)|ExprKind::Array(values) if values.iter().any(|value|matches!(value.kind,ExprKind::Unary{op:UnaryOp::Spread,..}))){
                let span=expr.span;let value=Box::new(std::mem::replace(expr,Expr::new(ExprKind::Undefined,span)));
                let child=Task::ArraySpread(Box::new(array_spread::ArraySpreadTask::new(value,task.context,task.loans.clone(),expected)));
                task.state=ExpressionState::ArraySpread;return Step::Push(Task::Expression(task),child);
            }
            if matches!(&expr.kind,ExprKind::Record(fields) if fields.iter().any(|(key,_)|key.text=="record#spread")){
                let span=expr.span;let value=Box::new(std::mem::replace(expr,Expr::new(ExprKind::Undefined,span)));
                let child=Task::ObjectSpread(Box::new(object_spread::ObjectSpreadTask::new(value,task.context,task.loans.clone(),expected)));
                task.state=ExpressionState::ObjectSpread;return Step::Push(Task::Expression(task),child);
            }
            if matches!(expr.kind,ExprKind::SetConstructor(_,_)|ExprKind::MapConstructor(_,_)){
                let span=expr.span;let value=Box::new(std::mem::replace(expr,Expr::new(ExprKind::Undefined,span)));
                let child=Task::CollectionConstructor(Box::new(collection_constructor::CollectionConstructorTask::new(value,task.context,task.loans.clone(),expected)));
                task.state=ExpressionState::CollectionConstructor;return Step::Push(Task::Expression(task),child);
            }
            if matches!(expr.kind,ExprKind::OptionalChain(_)){return schedule_optional(task,mode,expected);}
            if matches!(expr.kind, ExprKind::CoalesceAssign { .. }) {
                let frame = match analyzer.prepare_nullish_assignment_work(context, expr, mode, expected) {
                    Ok(frame) => frame,
                    Err(error) => return finish_expression(analyzer, context, task, Err(error)),
                };
                let ExprKind::CoalesceAssign { target, .. } = &mut expr.kind else { unreachable!("nullish target slot"); };
                let span = target.span;
                let target = std::mem::replace(target, Box::new(Expr::new(ExprKind::Undefined, span)));
                let child = Task::Expression(ExpressionTask::new(Expression::Child(target), task.context, Mode::Mutate, task.loans.clone(), None));
                task.state = ExpressionState::NullishAssignmentTarget(Box::new(frame));
                return Step::Push(Task::Expression(task), child);
            }
            let assignment = match &mut expr.kind {
                ExprKind::Assign { target, value, operator } => Some((target, Some(value), super::assignment::Kind::Scalar, *operator)),
                ExprKind::LogicalAssign { target, value, operator, .. } => Some((target, Some(value), super::assignment::Kind::Logical, *operator)),
                ExprKind::CompoundAssign { target, value, op, operator } => Some((target, Some(value), super::assignment::Kind::Compound { op: *op, allow_owned: false }, *operator)),
                ExprKind::Update { target, operator, .. } => Some((target, None, super::assignment::Kind::Update, *operator)),
                _ => None,
            };
            if let Some((target, value, kind, operator)) = assignment {
                let span = target.span;
                let target = std::mem::replace(target, Box::new(Expr::new(ExprKind::Undefined, span)));
                let value = if let Some(value) = value {
                    let span = value.span;
                    std::mem::replace(value, Box::new(Expr::new(ExprKind::Undefined, span)))
                } else { Box::new(Expr::new(ExprKind::Undefined, operator)) };
                let child = Task::Assignment(AssignmentTask { target, value, kind, operator, loans: task.loans.clone(), context: task.context, state: AssignmentState::Start });
                task.state = ExpressionState::Assignment;
                return Step::Push(Task::Expression(task), child);
            }
            if matches!(expr.kind, ExprKind::Binary { op: BinaryOp::And | BinaryOp::Or, .. }) {
                let frame = Box::new(analyzer.prepare_logical_work(context, expr, mode, expected));
                let ExprKind::Binary { left, .. } = &mut expr.kind else { unreachable!("logical slots"); };
                let span = left.span;
                let value = std::mem::replace(left, Box::new(Expr::new(ExprKind::Undefined, span)));
                let child = Task::Expression(ExpressionTask::new(Expression::Child(value), task.context, frame.access, task.loans.clone(), frame.operand_expected));
                task.state = ExpressionState::LogicalLeft(frame);
                return Step::Push(Task::Expression(task), child);
            }
            if matches!(expr.kind, ExprKind::Binary { op: BinaryOp::Coalesce | BinaryOp::PatternDefault, .. }) {
                let frame = Box::new(analyzer.prepare_coalesce_work(context, expr, mode, expected));
                let ExprKind::Binary { left, .. } = &mut expr.kind else { unreachable!("coalescing slots"); };
                let span = left.span;
                let value = std::mem::replace(left, Box::new(Expr::new(ExprKind::Undefined, span)));
                let child = Task::Expression(ExpressionTask::new(Expression::Child(value), task.context, frame.access, task.loans.clone(), frame.operand_expected));
                task.state = ExpressionState::CoalesceLeft(frame);
                return Step::Push(Task::Expression(task), child);
            }
            if let ExprKind::Conditional { predicate, .. } = &mut expr.kind {
                analyzer.truthy_condition(context, predicate);
                let span = predicate.span;
                let predicate = std::mem::replace(predicate, Box::new(Expr::new(ExprKind::Undefined, span)));
                let child = Task::Expression(ExpressionTask::new(Expression::Child(predicate), task.context, Mode::Copy, task.loans.clone(), Some(Type::Boolean)));
                task.state = ExpressionState::ConditionalPredicate { mode, expected };
                return Step::Push(Task::Expression(task), child);
            }
            match analyzer.prepare_unary_work(context, expr, mode, expected) {
                Ok(Some(super::operators::UnaryWork::Ready(ty))) => return finish_expression(analyzer, context, task, Ok(ty)),
                Ok(Some(super::operators::UnaryWork::Child(frame, operand))) => {
                    let ExprKind::Unary { value, .. } = &mut expr.kind else { unreachable!("unary first operand"); };
                    let span = value.span;
                    let value = std::mem::replace(value, Box::new(Expr::new(ExprKind::Undefined, span)));
                    let child = Task::Expression(ExpressionTask::new(Expression::Child(value), task.context, operand.mode, task.loans.clone(), operand.expected));
                    task.state = ExpressionState::Unary { frame: Box::new(frame), expected };
                    return Step::Push(Task::Expression(task), child);
                }
                Ok(None) => (),
                Err(error) => return finish_expression(analyzer, context, task, Err(error)),
            }
            match analyzer.prepare_binary_work(context, expr, expected) {
                Ok(Some((frame, operand))) => {
                    let ExprKind::Binary { left, .. } = &mut expr.kind else { unreachable!("binary first operand"); };
                    let span = left.span;
                    let value = std::mem::replace(left, Box::new(Expr::new(ExprKind::Undefined, span)));
                    let child = Task::Expression(ExpressionTask::new(Expression::Child(value), task.context, operand.mode, task.loans.clone(), operand.expected));
                    task.state = ExpressionState::BinaryLeft(Box::new(frame));
                    return Step::Push(Task::Expression(task), child);
                }
                Ok(None) => (),
                Err(error) => return finish_expression(analyzer, context, task, Err(error)),
            }
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
                let prepared=analyzer.start_closure(context,(*id,params,result),expected,body,expr.span,&task.loans);
                match prepared{
                    Ok(super::closure::Started::Ready(ty))=>{expr.ty=Some(ty);finish_expression(analyzer,context,task,Ok(ty))}
                    Ok(super::closure::Started::Parameters(frame,child_context))=>{
                        let child=contexts.push(child_context);
                        closure_parameters::next(analyzer,contexts,task,frame,child,false)
                    }
                    Err(error)=>finish_expression(analyzer,context,task,Err(error)),
                }
            } else {
                if invocation {
                    start_invocation(analyzer, context, task, false, false)
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
        Ok(super::call::PreparedInvocation::Deferred(request)) => return schedule_deferred(task, request, Some((frame, publish_type))),
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

fn schedule_deferred<'a>(mut task: ExpressionTask<'a>, request: crate::analysis::deferred_generic_call_work::Request, conditional: Option<(super::call::CallCompletion, bool)>) -> Step<'a> {
    let expr = task.expression.value();
    let span = expr.span;
    let value = Box::new(std::mem::replace(expr, Expr::new(ExprKind::Undefined, span)));
    let origin = request.origin;
    let child = Task::Deferred(Box::new(deferred::DeferredTask::new(value, request, task.context, task.loans.clone())));
    task.state = ExpressionState::Deferred { origin, conditional };
    Step::Push(Task::Expression(task), child)
}

fn start_declared_call<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, resolve_completed: bool) -> Step<'a> {
    let prepared = analyzer.prepare_declared_call(context, task.expression.value())
        .and_then(|index| analyzer.function_request_ready(index).map(|ready| (index, ready)));
    match prepared {
        Ok((index, true)) => resume_declared_call(analyzer, context, task, index, resolve_completed),
        Ok((index, false)) => {
            task.state = ExpressionState::DeclaredFunction { index, resolve_completed };
            Step::Push(Task::Expression(task), Task::Function(Box::new(functions::FunctionTask::new(index))))
        }
        Err(error) => finish_expression(analyzer, context, task, Err(error)),
    }
}

fn schedule_method<'a>(mut task: ExpressionTask<'a>, origin: usize, types: Vec<Type>, conditional: Option<(super::call::CallCompletion, bool)>) -> Step<'a> {
    let span = task.expression.value().span;
    task.state = ExpressionState::Method { origin, conditional };
    Step::Push(Task::Expression(task), Task::Method(Box::new(methods::MethodTask::Start { origin, types, span })))
}

pub(in crate::analysis) fn check_function_parameters(analyzer: &mut Analyzer, context: &mut Context, body: &mut Vec<Stmt>, parameters: crate::analysis::function_parameters::FunctionParameters) -> Result<(), Diagnostic> {
    let task = Task::Parameters(Box::new(parameters::ParametersTask::new(parameters, ContextId(0), std::mem::take(body))));
    let Completion::Parameters(checked, result) = drive(analyzer, context, task) else { unreachable!("named parameter result") };
    *body = checked;
    result
}

pub(in crate::analysis) fn check_function(analyzer: &mut Analyzer, index: usize) -> Result<(), Diagnostic> {
    let mut root = Context::new(String::new());
    let task = Task::Function(Box::new(functions::FunctionTask::new(index)));
    let Completion::Function(result) = drive(analyzer, &mut root, task) else { unreachable!("function schedule result") };
    result
}

fn resume_declared_call<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, index: usize, resolve_completed: bool) -> Step<'a> {
    let expr = task.expression.value();
    let prepared = (|| {
        let (index, captures, params) = analyzer.completed_declared_call_contract(context, expr, index)?;
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
        InvocationFinish::Method { origin } => {
            let types = analyzer.closure_method_argument_types(origin, args);
            return schedule_method(task, origin, types, None);
        }
        InvocationFinish::Conditional { frame, invocation, publish_type } => {
            match analyzer.finish_callable_invocation(context, args, expr.span, invocation) {
                Ok(super::call::FinishedInvocation::Ready(target, result)) => complete_conditional_call(analyzer, context, expr, frame, (target, result), publish_type),
                Ok(super::call::FinishedInvocation::Method(origin, types)) => return schedule_method(task, origin, types, Some((frame, publish_type))),
                Err(error) => Err(error),
            }
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
    fn scheduled_exception_failures_restore_region_frames_and_catch_names() {
        for source in [
            "function main(){try{throw missing;}catch(error){}}",
            "function main(){try{throw 1;}catch(error){missing;}}",
            "function main(){try{throw 1;}finally{missing;}}",
            "function main(){try{throw 1;}catch(error){try{throw error;}finally{missing;}}}",
            "function main(){try{for(;;){break;}}finally{missing;}}",
            "function main(){try{const run=():int=>{try{return 1;}finally{missing;}};run();}catch(error){}}",
        ] {
            let source = formatted(source);
            let mut parsed = crate::syntax::parse(&source).unwrap();
            let mut analyzer = Analyzer::new(parsed.clone()).unwrap();
            let mut context = Context::new("main".into());
            let outer = Analyzer::bind(&mut context, &Token { text: "error".into(), kind: TokenKind::Name, span: Span::new(0, 1) }, Type::String, false).unwrap();
            let StmtKind::Try(region) = &mut parsed.functions[0].body[0].kind else { panic!("exception region"); };
            let dead_key = region.catch.as_ref().and_then(|handler| handler.body.first()).map(|statement| statement.span.start);
            if let Some(key) = dead_key {
                analyzer.dead_handlers.insert(key, Box::new((context.clone(), region.catch.as_ref().unwrap().clone(), Type::Void)));
            }
            let error = analyzer.check_typed_exception_region(&mut context, region, Type::Void, &crate::error_effects::ErrorEffects::default(), None, &mut |_,_|{}).err().expect("invalid child");
            assert_eq!(&source[error.span.start..error.span.end], "missing");
            if let Some(key) = dead_key { assert!(analyzer.dead_handlers.contains_key(&key), "failed catch must not publish successful handler bookkeeping"); }
            assert_eq!(context.names.len(), 1);
            assert_eq!(context.names["error"], outer);
            assert!(context.lexical_scopes.is_empty());
            assert!(context.loops.is_empty());
            assert!(context.completion_frame.is_none());
            assert!(context.error_frame.is_none());
            assert!(context.region_entry_loans.is_none());
            assert!(context.region_loop_depth.is_none());
            assert!(context.exception_environment.is_none());
            assert_eq!(analyzer.depth, 0);
            assert!(format!("{region:?}").contains("missing"));
        }
    }

    #[test]
    fn scheduled_loop_failures_restore_iterators_tails_and_assignment_children() {
        for source in [
            "function main(){outer:{for(const item of [1]){missing;}}}",
            "function main(){outer:{for(const item of [1]){for(const nested of [2]){missing;}}}}",
            "function main(){outer:{for(;true;missing){break;}}}",
            "function main(){outer:{for(;true;missing){continue;}}}",
            "function main(){outer:{do{break;}while(missing);}}",
            "function main(){outer:{do{continue;}while(missing);}}",
            "function main(){let value=0;value=(()=>{for(const item of [1]){missing;}return 1;})();}",
            "function main(){let value=0;value+=(()=>{do{missing;}while(false);return 1;})();}",
            r#"function main(){let value="old";value=(()=>{while(true){missing;}return "new";})();}"#,
        ] {
            let source = formatted(source);
            let mut parsed = crate::syntax::parse(&source).unwrap();
            let mut analyzer = Analyzer::new(parsed.clone()).unwrap();
            let mut context = Context::new("main".into());
            let error = check_block(&mut analyzer, &mut context, &mut parsed.functions[0].body, Type::Void, false).unwrap_err();
            assert_eq!(&source[error.span.start..error.span.end], "missing");
            assert!(context.names.is_empty());
            assert!(context.lexical_scopes.is_empty());
            assert!(context.loops.is_empty());
            assert!(context.iteration_loans.is_empty());
            assert!(context.capture_loans.is_empty());
            assert!(context.mutation_place.is_none());
            assert!(context.replacement.is_none());
            assert_eq!(analyzer.depth,0);
            assert!(format!("{:?}",parsed.functions[0].body).contains("missing"));
        }
    }

    #[test]
    fn failed_statement_children_restore_scopes_labels_and_module_context() {
        for source in [
            "function main(){outer: {let value=1;if(true){let value=2;missing;}else{}}}",
            "function main(){outer: {let value=1;if(false){}else{let value=2;missing;}}}",
            "function main(){outer: {const f=()=>{if(true){missing;}else{}};}}",
            "function main(){outer: {while(true){missing;}}}",
        ] {
            let source = formatted(source);
            let mut parsed = crate::syntax::parse(&source).unwrap();
            let mut analyzer = Analyzer::new(parsed.clone()).unwrap();
            let mut context = Context::new("main".into());
            let outer = Analyzer::bind(&mut context, &Token { text: "value".into(), kind: TokenKind::Name, span: Span::new(0,5) }, Type::Int, false).unwrap();
            let module_scope = Some(Span::new(10000,10001));
            analyzer.module_literal_scope = module_scope;
            let error = check_block(&mut analyzer, &mut context, &mut parsed.functions[0].body, Type::Void, false).unwrap_err();
            assert_eq!(&source[error.span.start..error.span.end], "missing");
            assert_eq!(context.names.len(),1);
            assert_eq!(context.names["value"],outer);
            assert!(context.lexical_scopes.is_empty());
            assert!(context.loops.is_empty());
            assert!(context.capture_loans.is_empty());
            assert_eq!(analyzer.depth,0);
            assert_eq!(analyzer.module_literal_scope,module_scope);
            // Error unwinding must restore child AST slots as well as scopes.
            let rendered = format!("{:?}", parsed.functions[0].body);
            assert!(rendered.contains("missing"));
        }
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
    fn sequence_operands_share_the_checker_stack() {
        let source = formatted("function main():int{return (0,987654321);}");
        let mut parsed = crate::syntax::parse(&source).unwrap();
        let mut analyzer = Analyzer::new(parsed.clone()).unwrap();
        let mut context = Context::new("main".into());
        let StmtKind::Return(Some(template)) = &mut parsed.functions[0].body[0].kind else { panic!("return"); };
        let ExprKind::Sequence(values) = &template.kind else { panic!("sequence"); };
        let mut nested = values.last().unwrap().clone();
        for _ in 0..4096 {
            let mut parent = template.clone();
            let ExprKind::Sequence(values) = &mut parent.kind else { unreachable!() };
            *values.last_mut().unwrap() = nested;
            nested = parent;
        }
        assert_eq!(check_expression(&mut analyzer, &mut context, &mut nested, Mode::Consume, &Loans::new(), Some(Type::Int)).unwrap(), Type::Int);
        assert_eq!(analyzer.depth, 0);
        assert!(context.projection.is_none());
        assert!(context.capture_loans.is_empty());
    }

    #[test]
    fn failed_sequence_operands_restore_owned_store_and_prefix_state() {
        for source in [
            "function main():int{let text=\"old\";return (text=(0,missing),7);}",
            "function main():int{const rows=[[1]];return (rows[true]=[2],7);}",
            "function main():int{let text=\"old\";return (text+=(0,missing),7);}",
        ] {
            let source = formatted(source);
            let mut parsed = crate::syntax::parse(&source).unwrap();
            let mut analyzer = Analyzer::new(parsed.clone()).unwrap();
            let mut context = Context::new("main".into());
            let body = &mut parsed.functions[0].body;
            let error = check_block(&mut analyzer, &mut context, body, Type::Int, false).unwrap_err();
            assert_eq!(&source[error.span.start..error.span.end], if source.contains("missing") { "missing" } else { "true" });
            assert!(context.projection.is_none());
            assert!(context.mutation_place.is_none());
            assert!(context.replacement.is_none());
            assert!(context.keep_flow.is_empty());
            assert!(context.suppressed_flow.is_empty());
            assert!(context.capture_loans.is_empty());
            let StmtKind::Return(Some(value)) = &body[1].kind else { panic!("restored return"); };
            let ExprKind::Sequence(values) = &value.kind else { panic!("restored sequence"); };
            assert!(matches!(values[0].kind, ExprKind::Assign { .. } | ExprKind::CompoundAssign { .. }));
        }
    }

    #[test]
    fn failed_assignment_operands_restore_mutation_scope_and_trees() {
        for expression in [
            "values[true]=1", "values[true]+=1", "values[true]&&=true", "values[true]??=1", "++values[true]",
            "values[0]=missing", "values[0]+=missing", "values[0]??=missing",
        ] {
            let source = formatted(&format!("function main():int{{const values=[1];return {expression};}}"));
            let mut parsed = crate::syntax::parse(&source).unwrap();
            let mut analyzer = Analyzer::new(parsed.clone()).unwrap();
            let mut context = Context::new("main".into());
            let body = &mut parsed.functions[0].body;
            let error = check_block(&mut analyzer, &mut context, body, Type::Int, false).unwrap_err();
            let expected = if expression.contains("missing") { "missing" } else { "true" };
            assert_eq!(&source[error.span.start..error.span.end], expected);
            assert!(context.mutation_place.is_none());
            assert!(context.replacement.is_none());
            assert!(context.keep_flow.is_empty());
            assert!(context.suppressed_flow.is_empty());
            let StmtKind::Return(Some(value)) = &body[1].kind else { panic!("return restored"); };
            let target = match &value.kind {
                ExprKind::Assign { target, .. } | ExprKind::CompoundAssign { target, .. }
                | ExprKind::LogicalAssign { target, .. } | ExprKind::CoalesceAssign { target, .. }
                | ExprKind::Update { target, .. } => target,
                _ => panic!("assignment restored"),
            };
            assert!(matches!(target.kind, ExprKind::Index(..)));
        }
    }

    #[test]
    fn failed_logical_operands_restore_boolean_value_and_unreachable_children() {
        for expression in ["true&&missing", "0||missing", "[1]||missing"] {
            let source = formatted(&format!("function main(){{return {expression};}}"));
            let mut parsed = crate::syntax::parse(&source).unwrap();
            let mut analyzer = Analyzer::new(parsed.clone()).unwrap();
            let mut context = Context::new("main".into());
            let StmtKind::Return(Some(value)) = &mut parsed.functions[0].body[0].kind else { panic!("return"); };
            let error = check_expression(&mut analyzer, &mut context, value, Mode::Consume, &Loans::new(), None).unwrap_err();
            assert_eq!(error.code, "name");
            assert_eq!(&source[error.span.start..error.span.end], "missing");
            let ExprKind::Binary { left, right, .. } = &value.kind else { panic!("restored logical operands"); };
            assert!(left.ty.is_some());
            assert!(matches!(&right.kind, ExprKind::Name(name) if name == "missing"));
            assert_eq!(analyzer.depth, 0);
        }
    }

    #[test]
    fn failed_coalescing_fallback_restores_reachable_and_unreachable_children() {
        for expression in ["null??missing", "7??missing"] {
            let source = formatted(&format!("function main():int{{return {expression};}}"));
            let mut parsed = crate::syntax::parse(&source).unwrap();
            let mut analyzer = Analyzer::new(parsed.clone()).unwrap();
            let mut context = Context::new("main".into());
            let StmtKind::Return(Some(value)) = &mut parsed.functions[0].body[0].kind else { panic!("return"); };
            let error = check_expression(&mut analyzer, &mut context, value, Mode::Consume, &Loans::new(), Some(Type::Int)).unwrap_err();
            assert_eq!(error.code, "name");
            assert_eq!(&source[error.span.start..error.span.end], "missing");
            let ExprKind::Binary { left, right, .. } = &value.kind else { panic!("restored coalescing operands"); };
            assert!(left.ty.is_some());
            assert!(matches!(&right.kind, ExprKind::Name(name) if name == "missing"));
            assert_eq!(analyzer.depth, 0);
        }
    }

    #[test]
    fn failed_conditional_arm_restores_both_children_and_diagnostic_location() {
        let source = formatted("function main():int{return true?7:false;}");
        let mut parsed = crate::syntax::parse(&source).unwrap();
        let mut analyzer = Analyzer::new(parsed.clone()).unwrap();
        let mut context = Context::new("main".into());
        let StmtKind::Return(Some(value)) = &mut parsed.functions[0].body[0].kind else { panic!("return"); };
        let error = check_expression(&mut analyzer, &mut context, value, Mode::Consume, &Loans::new(), Some(Type::Int)).unwrap_err();
        assert_eq!(error.code, "type");
        assert_eq!(&source[error.span.start..error.span.end], "false");
        let ExprKind::Conditional { predicate, consequent, alternate } = &value.kind else { panic!("restored conditional"); };
        assert_eq!(predicate.ty, Some(Type::Boolean));
        assert_eq!(consequent.ty, Some(Type::Int));
        assert!(matches!(alternate.kind, ExprKind::Boolean(false)));
        assert_eq!(analyzer.depth, 0);
    }

    #[test]
    fn failed_operator_operands_restore_observation_and_partial_trees() {
        for (result, expression) in [
            ("string", "typeof (1 + true)"),
            ("boolean", "null === (1 + true)"),
            ("boolean", "undefined !== (1 + true)"),
        ] {
            let source = formatted(&format!("function main():{result}{{return {expression};}}"));
            let mut parsed = crate::syntax::parse(&source).unwrap();
            let mut analyzer = Analyzer::new(parsed.clone()).unwrap();
            let mut context = Context::new("main".into());
            let body = &mut parsed.functions[0].body;
            let ty = if result == "string" { Type::String } else { Type::Boolean };
            let StmtKind::Return(Some(value)) = &mut body[0].kind else { panic!("return expression"); };
            // Isolate expression cleanup from the block's separate error policy.
            let error = check_expression(&mut analyzer, &mut context, value, Mode::Consume, &Loans::new(), Some(ty)).unwrap_err();
            assert_eq!(error.code, "type");
            assert_eq!(&source[error.span.start..error.span.end], "+");
            assert!(!analyzer.observing_values);
            assert!(context.observer_places.is_empty());
            assert!(context.flow.keys().all(|place| place.fields.iter().all(|field| !field.starts_with("\0comparison:"))));
            assert_eq!(analyzer.depth, 0);
            assert!(matches!(body[0].kind, StmtKind::Return(Some(_))));
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

fn schedule_optional<'a>(mut task:ExpressionTask<'a>,mode:Mode,expected:Option<Type>)->Step<'a>{
    let expr=task.expression.value();let span=expr.span;
    let value=Box::new(std::mem::replace(expr,Expr::checked(ExprKind::Null,span,None)));
    let child=optional::OptionalTask::new(Expression::Child(value),task.context,mode,task.loans.clone(),expected);
    task.state=ExpressionState::Optional;
    Step::Push(Task::Expression(task),Task::Optional(Box::new(child)))
}
pub(in crate::analysis) fn check_optional(analyzer:&mut Analyzer,context:&mut Context,expression:&mut Expr,mode:Mode,loans:&Loans,expected:Option<Type>)->Result<Type,Diagnostic>{
    let task=optional::OptionalTask::new(Expression::Root(expression),ContextId(0),mode,ActiveLoans::Borrowed(loans),expected);
    let Completion::Expression(_,result)=drive(analyzer,context,Task::Optional(Box::new(task))) else{unreachable!("optional root")};result
}

pub(in crate::analysis) fn check_generic_preparation(analyzer: &mut Analyzer, context: &Context, expr: &mut Expr, expected: Option<Type>) -> Result<(), Diagnostic> {
    let span = expr.span;
    let value = Box::new(std::mem::replace(expr, Expr::new(ExprKind::Undefined, span)));
    let task = generic::GenericTask::new(value, std::rc::Rc::new(context.clone()), expected);
    let mut root = Context::new(context.function_name.clone());
    let Completion::Generic(value, result) = drive(analyzer, &mut root, Task::Generic(Box::new(task))) else { unreachable!("generic root") };
    *expr = *value;
    result
}

fn start_invocation<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: ExpressionTask<'a>, prepared: bool, receiver_prepared: bool) -> Step<'a> {
    let expected = task.checked.as_ref().expect("checked invocation").expected;
    let expr = task.expression.value();
    let needs_preparation = !prepared && matches!(&expr.kind, ExprKind::Call(callee, _)
        if matches!(&callee.kind, ExprKind::Name(name) if !context.names.contains_key(name) && !context.pending_names.contains(name)
            && (analyzer.generic_functions.template(name).is_some()
                || !analyzer.discovering_results && analyzer.function_names.get(name).is_some_and(|&index|
                    analyzer.signatures[index].inferred_result && analyzer.checked[index].is_none()
                    && !analyzer.visiting[index] && !analyzer.active_recursive_functions.contains(&index)))));
    if needs_preparation {
        let span = expr.span;
        let value = Box::new(std::mem::replace(expr, Expr::new(ExprKind::Undefined, span)));
        let child = Task::Generic(Box::new(generic::GenericTask::new(value, std::rc::Rc::new(context.clone()), expected)));
        task.state = ExpressionState::GenericCall;
        return Step::Push(Task::Expression(task), child);
    }
                    let dispatch = analyzer.prepare_call_work(context, expr, &task.loans, expected, receiver_prepared);
                    match dispatch {
                        Ok(CallDispatch::Spread) => {
                            let span=expr.span;
                            let value=Box::new(std::mem::replace(expr,Expr::new(ExprKind::Undefined,span)));
                            let child=Task::Spread(Box::new(spread::SpreadTask::new(value,task.context,task.loans.clone(),expected)));
                            task.state=ExpressionState::Spread;
                            Step::Push(Task::Expression(task),child)
                        }
                        Ok(CallDispatch::PrepareReceiver) => {
                            let receiver = Analyzer::call_preparation_receiver(expr);
                            let span = receiver.span;
                            let value = Box::new(std::mem::replace(receiver, Expr::new(ExprKind::Undefined, span)));
                            let child = Task::Generic(Box::new(generic::GenericTask::new(value, std::rc::Rc::new(context.clone()), None)));
                            task.state = ExpressionState::GenericReceiver;
                            Step::Push(Task::Expression(task), child)
                        }
                        Ok(CallDispatch::Reduction) => reduction::start(analyzer, context, task),
                        Ok(CallDispatch::Sorting { copying, expected }) => sorting::start(analyzer, context, task, copying, expected),
                        Ok(CallDispatch::Transform(request)) => transform::start(analyzer, context, task, request),
                        Ok(CallDispatch::Ready(ty)) => finish_expression(analyzer, context, task, Ok(ty)),
                        Ok(CallDispatch::Declared { resolve_completed }) => start_declared_call(analyzer, context, task, resolve_completed),
                        Ok(CallDispatch::Deferred(request)) => schedule_deferred(task, request, None),
                        Ok(CallDispatch::Method { binding, origin, params, invocation }) => {
                            let ExprKind::Call(callee, _) = &expr.kind else { unreachable!("method call") };
                            let mut active = (*task.loans).clone();
                            active.entry(binding).or_default().push((invocation, callee.span));
                            let arguments = super::arguments::Arguments::new(params, active, context, true, false, true);
                            schedule_argument(analyzer, context, task, Box::new(CallArguments { arguments, finish: InvocationFinish::Method { origin } }))
                        }
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
}
