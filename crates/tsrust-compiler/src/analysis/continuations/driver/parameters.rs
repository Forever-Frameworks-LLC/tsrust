//! Named defaults and destructuring share expression and block continuations.
use super::*;
use crate::analysis::function_parameters::FunctionParameters;
use super::super::defaults::DefaultFrame;

enum Pending {
    Default { frame: DefaultFrame, mode: Mode },
    Pattern { start: usize },
}

pub(super) struct ParametersTask {
    frame: FunctionParameters,
    pub(super) context: ContextId,
    body: Vec<Stmt>,
    pending: Option<Pending>,
    pattern_pending: bool,
}

impl ParametersTask {
    pub(super) fn new(frame: FunctionParameters, context: ContextId, body: Vec<Stmt>) -> Self {
        Self { frame, context, body, pending: None, pattern_pending: false }
    }
}

pub(super) fn step<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: Box<ParametersTask>, completion: Option<Completion<'a>>) -> Step<'a> {
    let next = (|| {
        if let Some(pending) = task.pending.take() {
            match (pending, completion.expect("named parameter child")) {
                (Pending::Default { frame, mode }, Completion::Expression(value, actual)) => {
                    let mut value = value.into_child();
                    analyzer.finish_default_work(context, &mut value, frame, actual?)?;
                    analyzer.finish_named_parameter(context, &mut task.frame, Some(value), mode)?;
                    task.pattern_pending = true;
                }
                (Pending::Pattern { start }, Completion::Block(body, result)) => {
                    task.body.splice(start..start, body.into_child());
                    result?;
                }
                _ => unreachable!("named parameter completion"),
            }
        }
        loop {
            if std::mem::take(&mut task.pattern_pending) {
                let position = task.frame.next - 1;
                let parameter = &task.frame.parameters[position].0;
                let owned = matches!(analyzer.program.functions[task.frame.index].parameter_access[position], Some((Mode::Consume, _)));
                if let Some(range) = analyzer.prepare_parameter_pattern(context, parameter, &mut task.body, owned) {
                    let start = range.start;
                    let body = task.body.drain(range).collect();
                    task.pending = Some(Pending::Pattern { start });
                    return Ok(Some(Task::Block(BlockTask { body: Body::Child { body, start: 0 }, context: task.context,
                        frame: None, pending: None, result_type: Type::Void, keep_names: true, label_span: None })));
                }
            }
            if task.frame.next == task.frame.parameters.len() { return Ok(None); }
            let mode = analyzer.named_default_mode(context, &task.frame)?;
            if let ParameterInitializer::Default { value, .. } = &task.frame.initializers[task.frame.next] {
                let value = value.clone();
                let frame = analyzer.prepare_default_work(context, &value, context.bindings[task.frame.next].ty, mode);
                let child = Task::Expression(ExpressionTask::new(Expression::Child(value), task.context, frame.mode,
                    ActiveLoans::Shared(std::rc::Rc::new(Loans::new())), frame.expected));
                task.pending = Some(Pending::Default { frame, mode });
                return Ok(Some(child));
            }
            analyzer.finish_named_parameter(context, &mut task.frame, None, mode)?;
            task.pattern_pending = true;
        }
    })();
    match next {
        Ok(Some(child)) => Step::Push(Task::Parameters(task), child),
        result => Step::Complete(Completion::Parameters(task.body, result.map(|_| ()))),
    }
}
