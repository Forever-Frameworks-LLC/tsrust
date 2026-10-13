//! Exception-region continuations, including canonical and per-exit cleanup replay.
use super::*;
use super::super::exceptions::{ExitBoundary, Exits};
use exception_state::{CheckEnvironment, CompletionStates, ExitState};
use crate::error_effects::{ErrorEffects, FailureSet};
use crate::completion::CompletionKind;

pub(super) type ResultStates = Result<(CompletionStates, FailureSet), Diagnostic>;

pub(super) enum Region<'a> { #[allow(dead_code)] Root(&'a mut ExceptionRegion), Child(Box<ExceptionRegion>) }
impl Region<'_> {
    fn value(&mut self) -> &mut ExceptionRegion { match self { Self::Root(value) => value, Self::Child(value) => value } }
    pub(super) fn into_child(self) -> Box<ExceptionRegion> { let Self::Child(value) = self else { unreachable!("root region") }; value }
}

use super::super::exceptions::RegionMode as Environment;

pub(super) fn environment(analyzer: &mut Analyzer, mode: Environment, runtime: Option<Type>) -> Result<std::rc::Rc<CheckEnvironment>, Diagnostic> {
    let (effects, refresh_contracts, partial_contracts) = match mode {
        Environment::Fixed(effects) => (effects, false, false),
        Environment::Current => (analyzer.checked_error_effects()?, true, false),
        Environment::Partial => (analyzer.discovery_error_effects(), true, true),
    };
    Ok(std::rc::Rc::new(CheckEnvironment { effects, runtime, refresh_contracts, partial_contracts }))
}

enum Phase { Start, Body, Catch, Canonical, Replay }
struct Frame {
    outer: usize,
    effects: ErrorEffects,
    runtime: Option<Type>,
    refresh: bool,
    pending: CompletionStates,
    incoming: FailureSet,
    outgoing: FailureSet,
    map_boundary: Option<usize>,
    cleanup: Option<Box<Cleanup>>,
}
struct Cleanup {
    entry: Box<Context>,
    source: Vec<Stmt>,
    canonical: Option<(Vec<Stmt>, Vec<Binding>)>,
    paths: std::vec::IntoIter<(Option<CompletionKind>, ExitState)>,
    output: CompletionStates,
    contract: FailureSet,
    destination: Option<CompletionKind>,
}

pub(super) struct RegionTask<'a> {
    region: Region<'a>,
    context: ContextId,
    result: Type,
    environment: std::rc::Rc<CheckEnvironment>,
    previous_environment: Option<Option<std::rc::Rc<CheckEnvironment>>>,
    unwind: Option<&'a mut dyn FnMut(Span, &mut ExitState)>,
    phase: Phase,
    boundary: Option<ExitBoundary>,
    catch_scope: Option<(String, Option<usize>)>,
    isolated: Option<ContextId>,
    frame: Option<Box<Frame>>,
}
impl<'a> RegionTask<'a> {
    pub(super) fn new(region: Region<'a>, context: ContextId, result: Type, environment: std::rc::Rc<CheckEnvironment>, unwind: Option<&'a mut dyn FnMut(Span, &mut ExitState)>) -> Box<Self> {
        Box::new(Self { region, context, result, environment, unwind, previous_environment: None, phase: Phase::Start,
            boundary: None, catch_scope: None, isolated: None, frame: None })
    }

    fn finish_boundary(&mut self, contexts: &mut Contexts<'_>, result: Result<bool, Diagnostic>) -> Result<Exits, Diagnostic> {
        let context = contexts.get(self.isolated.unwrap_or(self.context));
        if let Some((name, previous)) = self.catch_scope.take() {
            context.names.remove(&name);
            if let Some(id) = previous { context.names.insert(name, id); }
        }
        Analyzer::finish_exit_frames(context, self.boundary.take().expect("region boundary"), result)
    }

    fn states(&mut self, exits: Exits) -> CompletionStates {
        let (normal, frame, errors) = exits;
        let mut unwind = |span, state: &mut ExitState| { if let Some(callback) = &mut self.unwind { callback(span, state); } };
        let mut states = CompletionStates::from_completions(normal, frame, &mut unwind);
        states.record_failures(errors, &mut unwind);
        states
    }
}

fn complete<'a>(contexts: &mut Contexts<'_>, mut task: Box<RegionTask<'a>>, mut result: ResultStates) -> Step<'a> {
    if task.boundary.is_some() {
        let error = result.as_ref().err().expect("unfinished boundary only on failure").clone();
        if let Err(error) = task.finish_boundary(contexts, Err(error)) { result = Err(error); }
    }
    if let Some(child) = task.isolated.take() { contexts.take(child); }
    if let Some(previous) = task.previous_environment.take() { contexts.get(task.context).exception_environment = previous; }
    Step::Complete(Completion::Region(task.region, result))
}

fn body_child<'a>(contexts: &mut Contexts<'_>, mut task: Box<RegionTask<'a>>, body: Vec<Stmt>, phase: Phase) -> Step<'a> {
    let id = task.isolated.unwrap_or(task.context);
    task.boundary = Some(Analyzer::prepare_exit_frames(contexts.get(id)));
    block_child(task, body, phase, id)
}

fn block_child<'a>(mut task: Box<RegionTask<'a>>, body: Vec<Stmt>, phase: Phase, id: ContextId) -> Step<'a> {
    task.phase = phase;
    let child = Task::Block(BlockTask { body: Body::Child { body, start: 0 }, context: id, result_type: task.result,
        frame: None, pending: None, keep_names: false, label_span: None });
    Step::Push(Task::Region(task), child)
}

fn catch_child<'a>(analyzer: &mut Analyzer, contexts: &mut Contexts<'_>, mut task: Box<RegionTask<'a>>) -> Step<'a> {
    let frame = task.frame.as_mut().expect("checked region body");
    let Some(handler) = &mut task.region.value().catch else { return cleanup_start(analyzer, contexts, task); };
    handler.binding = None;
    handler.error_type = None;
    handler.error_contract = Some(Box::new(frame.incoming.clone()));
    frame.outgoing = FailureSet::default();
    let state = frame.pending.take_throw();
    if let Some(key) = handler.body.first().map(|statement| statement.span.start) {
        if state.is_none() && !analyzer.effects_only && !analyzer.discovering_results {
            analyzer.dead_handlers.insert(key, Box::new((contexts.get(task.context).clone(), handler.clone(), task.result)));
        }
    }
    let Some(state) = state else { return cleanup_start(analyzer, contexts, task); };
    let context = contexts.get(task.context);
    state.restore(context);
    let prepared = (|| {
        handler.error_type = if let Some(name) = &handler.name { frame.incoming.catch_type(&analyzer.unions, frame.runtime, name.span)? } else { None };
        task.boundary = Some(Analyzer::prepare_exit_frames(context));
        if let Some(name) = &handler.name {
            let ty = handler.error_type.ok_or_else(|| Diagnostic::new("error-effect", "Catch payload type must be resolved before checking its binding.", name.span))?;
            task.catch_scope = Some((name.text.clone(), context.names.remove(&name.text)));
            let binding = Analyzer::bind(context, name, ty, false)?;
            handler.binding = Some(binding);
            context.bindings[binding].declaration_span = Some(name.span);
            context.bindings[binding].rebindable = true;
        }
        Ok::<_, Diagnostic>(std::mem::take(&mut handler.body))
    })();
    match prepared {
        Ok(body) => { let id = task.context; block_child(task, body, Phase::Catch, id) },
        Err(error) => complete(contexts, task, Err(error)),
    }
}

fn cleanup_start<'a>(analyzer: &mut Analyzer, contexts: &mut Contexts<'_>, mut task: Box<RegionTask<'a>>) -> Step<'a> {
    let frame = task.frame.as_mut().expect("handled region");
    let context = contexts.get(task.context);
    if let Some(boundary) = frame.map_boundary { frame.pending.record_completion_storage(&mut context.storage_exits, boundary, map_completion_payloads::Phase::Handled); }
    let Some(source) = task.region.value().finally.as_ref() else { return finish(analyzer, contexts, task); };
    let canonical_entry = frame.pending.joined_entry();
    let pending = std::mem::take(&mut frame.pending);
    frame.cleanup = Some(Box::new(Cleanup { entry: Box::new(context.clone()), source: source.clone(), canonical: None,
        paths: pending.into_paths().into_iter(), output: CompletionStates::default(), contract: FailureSet::default(), destination: None }));
    if let Some(state) = canonical_entry {
        let mut isolated = Box::new(context.clone());
        state.restore(&mut isolated);
        task.isolated = Some(contexts.push(isolated));
        let body = source.clone();
        body_child(contexts, task, body, Phase::Canonical)
    } else { cleanup_next(analyzer, contexts, task) }
}

fn cleanup_next<'a>(analyzer: &mut Analyzer, contexts: &mut Contexts<'_>, mut task: Box<RegionTask<'a>>) -> Step<'a> {
    let frame = task.frame.as_mut().expect("cleanup region");
    let cleanup = frame.cleanup.as_mut().expect("cleanup replay");
    if let Some((destination, state)) = cleanup.paths.next() {
        cleanup.destination = destination;
        let context = contexts.get(task.context);
        *context = (*cleanup.entry).clone();
        state.restore(context);
        let body = cleanup.source.clone();
        return body_child(contexts, task, body, Phase::Replay);
    }
    let cleanup = frame.cleanup.take().expect("completed cleanup");
    if let Some((canonical, bindings)) = cleanup.canonical {
        *task.region.value().finally.as_mut().expect("finally body") = canonical;
        let context = contexts.get(task.context);
        context.bindings.truncate(cleanup.entry.bindings.len());
        context.bindings.extend(bindings.into_iter().skip(cleanup.entry.bindings.len()));
    }
    if crate::completion::definitely_abrupt(&cleanup.source) { frame.outgoing = cleanup.contract; }
    else { frame.outgoing |= cleanup.contract; }
    frame.pending = cleanup.output;
    finish(analyzer, contexts, task)
}

fn finish<'a>(analyzer: &mut Analyzer, contexts: &mut Contexts<'_>, mut task: Box<RegionTask<'a>>) -> Step<'a> {
    let frame = task.frame.take().expect("completed region");
    let context = contexts.get(task.context);
    if let Some(boundary) = frame.map_boundary {
        frame.pending.record_completion_storage(&mut context.storage_exits, boundary, map_completion_payloads::Phase::Cleaned);
        if let Err(error) = analyzer.plan_map_completion_layouts(context, boundary) { return complete(contexts, task, Err(error)); }
    }
    frame.pending.restore_normal(context);
    complete(contexts, task, Ok((frame.pending, frame.outgoing)))
}

pub(super) fn step<'a>(analyzer: &mut Analyzer, contexts: &mut Contexts<'_>, mut task: Box<RegionTask<'a>>, completion: Option<Completion<'a>>) -> Step<'a> {
    let phase = std::mem::replace(&mut task.phase, Phase::Start);
    if matches!(phase, Phase::Start) {
        task.previous_environment = Some(contexts.get(task.context).exception_environment.replace(task.environment.clone()));
        let body = std::mem::take(&mut task.region.value().body);
        return body_child(contexts, task, body, Phase::Body);
    }
    let Completion::Block(body, checked) = completion.expect("region child") else { unreachable!("region body completion") };
    let mut body = body.into_child();
    // Restore borrowed/owned source slots even when the child failed.
    match phase {
        Phase::Body => task.region.value().body = std::mem::take(&mut body),
        Phase::Catch => task.region.value().catch.as_mut().expect("catch").body = std::mem::take(&mut body),
        Phase::Replay => *task.region.value().finally.as_mut().expect("finally") = std::mem::take(&mut body),
        Phase::Canonical => {},
        Phase::Start => unreachable!(),
    }
    let exits = match task.finish_boundary(contexts, checked) { Ok(exits) => exits, Err(error) => return complete(contexts, task, Err(error)) };
    let processed = (|| -> Result<(), Diagnostic> {
        match phase {
            Phase::Body => {
                let context = contexts.get(task.context);
                let result_type = task.result;
                let refresh = task.environment.refresh_contracts;
                let mut effects = task.environment.effects.clone();
                let partial = task.environment.partial_contracts;
                let initial_runtime = task.environment.runtime;
                let region = task.region.value();
                if refresh {
                    analyzer.check_pending_region_functions(&region.body, result_type)?;
                    effects = if partial { analyzer.region_caller_error_effects(context, &region.body, result_type) } else { analyzer.region_error_effects(context)? };
                }
                let incoming = crate::error_effects::annotate_region_body(&mut region.body, &effects);
                let points = crate::error_effects::OperationEffects::for_body(&region.body);
                let (normal, frame, mut errors) = exits;
                errors.refine(&mut |span| points.get(span));
                let runtime = if initial_runtime.is_none() && incoming.runtime && !incoming.opaque && region.catch.as_ref().is_some_and(|handler|handler.name.is_some()) {
                    Some(analyzer.materialize_discovery_runtime_error())
                } else { initial_runtime };
                let outer = frame.outer;
                let map_boundary = region.body.first().or_else(|| region.finally.as_ref().and_then(|body|body.first())).map(|statement|statement.span.start);
                let pending = task.states((normal, frame, errors));
                if let Some(boundary) = map_boundary { pending.record_completion_storage(&mut context.storage_exits, boundary, map_completion_payloads::Phase::Body); }
                task.frame = Some(Box::new(Frame { outer, effects, runtime, refresh, pending, outgoing: incoming.clone(), incoming, map_boundary, cleanup: None }));
            }
            Phase::Catch | Phase::Replay => {
                let context = contexts.get(task.context);
                let frame = task.frame.as_ref().expect("region phases");
                let effects = if frame.refresh { analyzer.region_error_effects(context)? } else { frame.effects.clone() };
                let outer = frame.outer;
                let source = match phase { Phase::Catch => &mut task.region.value().catch.as_mut().expect("catch").body,
                    Phase::Replay => task.region.value().finally.as_mut().expect("finally"), _ => unreachable!() };
                let contract = crate::error_effects::annotate_region_body(source, &effects);
                let points = crate::error_effects::OperationEffects::for_body(source);
                let (normal, frame, mut errors) = exits;
                errors.refine(&mut |span| points.get(span));
                let mut states = task.states((normal, frame, errors));
                states.project_outer(outer);
                let frame = task.frame.as_mut().expect("region phases");
                match phase {
                    Phase::Catch => {
                        frame.outgoing |= contract;
                        frame.pending.merge_cleanup(None, states);
                        if let Some(key) = task.region.value().catch.as_ref().and_then(|handler| handler.body.first()).map(|statement| statement.span.start) {
                            analyzer.dead_handlers.remove(&key);
                        }
                    },
                    Phase::Replay => { let cleanup = frame.cleanup.as_mut().expect("cleanup"); cleanup.contract |= contract; cleanup.output.merge_cleanup(cleanup.destination, states); },
                    _ => unreachable!(),
                }
            }
            Phase::Canonical => {
                let context = contexts.get(task.isolated.expect("canonical context"));
                let frame = task.frame.as_ref().expect("cleanup region");
                let effects = if frame.refresh { analyzer.region_error_effects(context)? } else { frame.effects.clone() };
                crate::error_effects::annotate_region_body(&mut body, &effects);
                let context = contexts.take(task.isolated.take().expect("canonical context"));
                task.frame.as_mut().unwrap().cleanup.as_mut().unwrap().canonical = Some((body, context.bindings));
            }
            Phase::Start => unreachable!(),
        }
        Ok(())
    })();
    if let Err(error) = processed { return complete(contexts, task, Err(error)); }
    match phase {
        Phase::Body => catch_child(analyzer, contexts, task),
        Phase::Catch => cleanup_start(analyzer, contexts, task),
        Phase::Canonical | Phase::Replay => cleanup_next(analyzer, contexts, task),
        Phase::Start => unreachable!(),
    }
}
