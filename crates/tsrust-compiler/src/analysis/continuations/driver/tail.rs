//! Tail-loop task shares the checker stack, including unreachable update contexts.
use super::*;

pub(super) struct TailTask {
    pub(super) statement: Box<Stmt>,
    pub(super) context: ContextId,
    pub(super) result_type: Type,
    pub(super) control_boundary: usize,
    pub(super) isolated: Option<ContextId>,
    pub(super) state: State,
}

pub(super) enum State {
    Start,
    Condition(Box<super::super::loops::TailEntry>),
    Body(Box<super::super::loops::Tail>),
    Update(Box<super::super::loops::Tail>),
    Predicate(Box<super::super::loops::Tail>),
}

fn complete<'a>(contexts: &mut Contexts<'_>, mut task: TailTask, result: Result<bool, Diagnostic>) -> Step<'a> {
    if let Some(child) = task.isolated.take() { contexts.take(child); }
    let context = contexts.get(task.context);
    while context.loops.len() > task.control_boundary {
        let frame = context.loops.pop().expect("unfinished tail loop");
        exception_state::consume_target(context, frame.target);
    }
    Step::Complete(Completion::Tail(task.statement, result))
}

fn predicate<'a>(analyzer: &mut Analyzer, contexts: &mut Contexts<'_>, mut task: TailTask, state: State) -> Step<'a> {
    let context_id = task.isolated.unwrap_or(task.context);
    let context = contexts.get(context_id);
    let StmtKind::Loop { condition: Some(condition), .. } = &mut task.statement.kind else { unreachable!("scheduled loop condition") };
    analyzer.truthy_condition(context, condition);
    let expression = std::mem::replace(condition, Expr::new(ExprKind::Undefined, condition.span));
    task.state = state;
    let child = Task::Expression(ExpressionTask::new(Expression::Child(Box::new(expression)), context_id, Mode::Copy,
        ActiveLoans::Shared(std::rc::Rc::new(Loans::new())), Some(Type::Boolean)));
    Step::Push(Task::Tail(task), child)
}

fn body<'a>(analyzer: &mut Analyzer, contexts: &mut Contexts<'_>, mut task: TailTask, entry: Box<super::super::loops::TailEntry>, actual: Option<Type>) -> Step<'a> {
    let StmtKind::Loop { condition, body, post_test, .. } = &mut task.statement.kind else { unreachable!("tail loop") };
    let frame = match analyzer.prepare_tail_body(contexts.get(task.context), condition, *post_test, actual, entry, task.statement.span, &task.statement.labels) {
        Ok(frame) => frame,
        Err(error) => return complete(contexts, task, Err(error)),
    };
    let child = Task::Block(BlockTask { body: Body::Child { body: std::mem::take(body), start: 0 }, context: task.context,
        result_type: task.result_type, keep_names: false, frame: None, pending: None, label_span: None });
    task.state = State::Body(frame);
    Step::Push(Task::Tail(task), child)
}

fn finish<'a>(analyzer: &mut Analyzer, contexts: &mut Contexts<'_>, mut task: TailTask, mut frame: Box<super::super::loops::Tail>) -> Step<'a> {
    let isolated = task.isolated.take();
    if let Some(child) = isolated { contexts.take(child); }
    let context = contexts.get(task.context);
    let StmtKind::Loop { condition, body, update, .. } = &mut task.statement.kind else { unreachable!("tail completion") };
    let result = (|| {
        if isolated.is_none() { Analyzer::finish_tail_back(context, &mut frame, condition.is_some(), task.statement.span)?; }
        analyzer.finish_tail_completion(context, condition, body, update, task.statement.span, frame)
    })();
    complete(contexts, task, result)
}

pub(super) fn step<'a>(analyzer: &mut Analyzer, contexts: &mut Contexts<'_>, mut task: TailTask, completion: Option<Completion<'a>>) -> Step<'a> {
    match std::mem::replace(&mut task.state, State::Start) {
        State::Start => {
            let entry = Analyzer::prepare_tail_entry(contexts.get(task.context));
            if matches!(&task.statement.kind, StmtKind::Loop { condition: Some(_), post_test: false, .. }) {
                predicate(analyzer, contexts, task, State::Condition(entry))
            } else { body(analyzer, contexts, task, entry, None) }
        }
        State::Condition(entry) => {
            let Completion::Expression(expression, result) = completion.expect("initial condition") else { unreachable!("condition completion") };
            let StmtKind::Loop { condition, .. } = &mut task.statement.kind else { unreachable!("condition slot") };
            *condition = Some(*expression.into_child());
            match result {
                Ok(actual) => body(analyzer, contexts, task, entry, Some(actual)),
                Err(error) => complete(contexts, task, Err(error)),
            }
        }
        State::Body(frame) => {
            let Completion::Block(body, result) = completion.expect("tail body") else { unreachable!("body completion") };
            let StmtKind::Loop { body: destination, update, tail_reachable, .. } = &mut task.statement.kind else { unreachable!("body slot") };
            *destination = body.into_child();
            let context = contexts.get(task.context);
            let reachable = match result.map(|exits| Analyzer::prepare_tail_update(context, exits, tail_reachable)) {
                Ok(reachable) => reachable,
                Err(error) => return complete(contexts, task, Err(error)),
            };
            let context_id = if reachable { task.context } else {
                let mut isolated = Box::new(context.clone());
                isolated.effects_only = true;
                let id = contexts.push(isolated);
                task.isolated = Some(id);
                id
            };
            let child = Task::Block(BlockTask { body: Body::Child { body: std::mem::take(update), start: 0 }, context: context_id,
                result_type: task.result_type, keep_names: false, frame: None, pending: None, label_span: None });
            task.state = State::Update(frame);
            Step::Push(Task::Tail(task), child)
        }
        State::Update(frame) => {
            let Completion::Block(body, result) = completion.expect("tail update") else { unreachable!("update completion") };
            let StmtKind::Loop { update, .. } = &mut task.statement.kind else { unreachable!("update slot") };
            *update = body.into_child();
            if let Err(error) = result { return complete(contexts, task, Err(error)); }
            if matches!(&task.statement.kind, StmtKind::Loop { condition: Some(_), post_test: true, .. }) {
                predicate(analyzer, contexts, task, State::Predicate(frame))
            } else { finish(analyzer, contexts, task, frame) }
        }
        State::Predicate(frame) => {
            let Completion::Expression(expression, result) = completion.expect("tail predicate") else { unreachable!("predicate completion") };
            let StmtKind::Loop { condition, .. } = &mut task.statement.kind else { unreachable!("predicate slot") };
            *condition = Some(*expression.into_child());
            let result = result.and_then(|actual| analyzer.require(actual, Some(Type::Boolean), condition.as_ref().unwrap().span));
            if let Err(error) = result { return complete(contexts, task, Err(error)); }
            finish(analyzer, contexts, task, frame)
        }
    }
}
