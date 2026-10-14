//! Loop boundaries retained while conditions, iterables and bodies are scheduled.
use super::super::*;

pub(super) struct Condition {
    entry_flow: crate::flow::Facts,
    entry: Vec<(usize, Availability)>,
}

pub(super) struct BodyLoop {
    entry_owners: crate::capture_loan_owners::Owners,
    baseline: Vec<Availability>,
    zero_flow: crate::flow::Facts,
    control_boundary: usize,
    iterator: Option<(String, usize)>,
}

impl Analyzer {
    pub(super) fn prepare_while_condition(context: &Context) -> Box<Condition> {
        let entry_flow = context.flow.clone();
        let mut entry = context.names.values().copied().collect::<Vec<_>>();
        entry.sort_unstable();
        let entry = entry
            .into_iter()
            .map(|id| (id, Availability::of(&context.bindings[id])))
            .collect::<Vec<_>>();
        Box::new(Condition { entry_flow, entry })
    }

    pub(super) fn prepare_while_body(&mut self, context: &mut Context, condition: &Expr, actual: Type, frame: Box<Condition>, span: Span, labels: &[Token]) -> Result<Box<BodyLoop>, Diagnostic> {
        let Condition { entry_flow, entry } = *frame;
        self.require(actual, Some(Type::Boolean), condition.span)?;
        // The condition runs even when the body never does. Return paths do not
        // reach the join after the loop; break paths do. Back edges must preserve
        // entry availability. A transferred let binding must be reinitialized
        // before a continuing edge; const bindings cannot be reinitialized.
        let after_condition = entry
            .iter()
            .map(|(id, _)| Availability::of(&context.bindings[*id]))
            .collect::<Vec<_>>();
        let (yes, no) = Self::condition_outcomes(context, condition);
        let mut places = Vec::new();
        crate::flow::projected_places(condition, &context.names, &mut places);
        let required = places
            .into_iter()
            .filter_map(|place| entry_flow.get(&place).cloned().map(|fact| (place, fact)))
            .collect();
        context.flow = yes;
        self.prepare_loop_body(context, span, labels, (entry, after_condition), (entry_flow, required, no))
    }

    fn prepare_loop_body(&mut self, context: &mut Context, span: Span, labels: &[Token], availability: (Vec<(usize, Availability)>, Vec<Availability>), flows: (crate::flow::Facts, crate::flow::Facts, crate::flow::Facts)) -> Result<Box<BodyLoop>, Diagnostic> {
        self.thread_scope_exit(context, 0, span)?;
        let control_boundary = context.loops.len();
        let (entry, baseline) = availability;
        let (flow_entry, flow_required, zero_flow) = flows;
        let entry_owners = context.capture_loan_owners.clone();
        context.storage_exits.record_map_loop_edge(span,storage_exit_availability::MapLoopEdge::Entry,&context.capture_loan_owners);
        Self::validate_control_labels(context,labels)?;
        context.loops.push(LoopFrame {
            labels: labels.to_vec(),
            target: ControlTarget::Loop(span.start),
            flow_entry,
            flow_required,
            break_flow: None,
            continue_flow: None,
            breaks: vec![Availability::default(); entry.len()],
            break_seen: false, break_owners:None, continue_owners:None, scope_start: context.bindings.len(),
            defer_continue: false,
            continues: None,
            entry,
            span,
        });
        Ok(Box::new(BodyLoop { entry_owners, baseline, zero_flow, control_boundary, iterator: None }))
    }

    pub(super) fn prepare_iteration_body(&mut self, context: &mut Context, variable: (&Token, bool, &mut Option<usize>), iterable: &Expr, actual: Type, span: Span, labels: &[Token]) -> Result<Box<BodyLoop>, Diagnostic> {
        let (name, rebindable, binding) = variable;
        let first_loan = context.iteration_loans.len();
        let previous = context.names.get(&name.text).copied();
        let result = (|| {
        let element = self.iterable_element(actual).ok_or_else(|| Diagnostic::new("iteration-type", "for...of requires an array, string, Set, Map, Deque, Heap or OrderedSet.", iterable.span))?;
        // Iterable evaluation happens once, even for an empty array. Its moves
        // therefore belong to both the zero-iteration and repeated paths.
        let mut visible = context.names.values().copied().collect::<Vec<_>>();
        visible.sort_unstable();
        let entry = visible
            .into_iter()
            .map(|id| (id, Availability::of(&context.bindings[id])))
            .collect::<Vec<_>>();
        let baseline = entry.iter().map(|(_, state)| state.clone()).collect();
        for id in Self::loan_roots(iterable, Mode::Read) {
            context.iteration_loans.push((id, iterable.span));
        }
        let id = Self::bind(context, name, element.ty(), false)?;
        context.bindings[id].rebindable = rebindable;
        let mut native_loans = Self::native_view_value_loans(context, iterable)?;
        native_loans.retain_mut(|loan| native_view_projections::select(loan, &native_view_loans::Segment::Element));
        if let Type::Map(_) = actual {
            let Type::Record(pair) = element.ty() else { unreachable!("checked Map entry") };
            let field = self.program.records[pair].fields[1].name.text.clone();
            for loan in &mut native_loans { native_view_projections::wrap(loan, &native_view_loans::Segment::Field(field.clone())); }
        }
        Self::set_native_view_loans(context, id, native_loans);
        if !element.ty().is_copy() {
            context.bindings[id].borrowed_element = Some(iterable.span);
        }
        *binding = Some(id);
        let entry_flow = context.flow.clone();
            let mut frame = self.prepare_loop_body(context, span, labels, (entry, baseline), (entry_flow.clone(), HashMap::new(), entry_flow))?;
            frame.iterator = Some((name.text.clone(), first_loan));
            Ok(frame)
        })();
        if result.is_err() {
            context.iteration_loans.truncate(first_loan);
            context.names.remove(&name.text);
            if let Some(previous) = previous { context.names.insert(name.text.clone(), previous); }
        }
        result
    }

    pub(super) fn finish_scheduled_loop(&mut self, context: &mut Context, body: &mut [Stmt], span: Span, frame: Box<BodyLoop>, checked: Result<bool, Diagnostic>) -> Result<bool, Diagnostic> {
        let BodyLoop { entry_owners, baseline, zero_flow, control_boundary, iterator } = *frame;
        let result = checked.and_then(|exits| {
            if !exits { Self::check_loop_back(context)?; }
            self.finish_loop_body(context, body, span, entry_owners, baseline, zero_flow, exits)
        });
        while context.loops.len() > control_boundary {
            let abandoned = context.loops.pop().expect("unfinished loop frame");
            exception_state::consume_target(context, abandoned.target);
        }
        if let Some((name, first_loan)) = iterator {
            context.iteration_loans.truncate(first_loan);
            context.names.remove(&name);
        }
        result
    }
}

pub(super) struct TailEntry {
    initial_flow: crate::flow::Facts,
    entry: Vec<(usize, Availability)>,
}

pub(super) struct Tail {
    zero_flow: Option<crate::flow::Facts>,
    exit_state: Option<Vec<Availability>>,
    exit_owners: Option<crate::capture_loan_owners::Owners>,
}

impl Analyzer {
    pub(super) fn prepare_tail_entry(context: &Context) -> Box<TailEntry> {
        let initial_flow = context.flow.clone();
        let mut visible = context.names.values().copied().collect::<Vec<_>>();
        visible.sort_unstable();
        let entry = visible
            .into_iter()
            .map(|id| (id, Availability::of(&context.bindings[id])))
            .collect::<Vec<_>>();
        Box::new(TailEntry { initial_flow, entry })
    }

    pub(super) fn prepare_tail_body(&mut self, context: &mut Context, condition: &Option<Expr>, post_test: bool, actual: Option<Type>, entry: Box<TailEntry>, span: Span, labels: &[Token]) -> Result<Box<Tail>, Diagnostic> {
        self.thread_scope_exit(context, 0, span)?;
        let TailEntry { initial_flow, entry } = *entry;
        let mut required = HashMap::new();
        let mut zero_flow = None;
        if !post_test && let Some(condition) = condition {
            let actual = actual.expect("scheduled pre-test condition");
            self.require(actual, Some(Type::Boolean), condition.span)?;
            let (yes, no) = Self::condition_outcomes(context, condition);
            let mut places = Vec::new();
            crate::flow::projected_places(condition, &context.names, &mut places);
            required = places
                .into_iter()
                .filter_map(|place| initial_flow.get(&place).cloned().map(|fact| (place, fact)))
                .collect();
            context.flow = yes;
            zero_flow = Some(no);
        }
        // Only a pre-test condition supplies a zero-body exit. do...while
        // inherits its first body effects on every path that reaches its exit.
        let exit_state = (!post_test && condition.is_some()).then(|| {
            entry
                .iter()
                .map(|(id, _)| Availability::of(&context.bindings[*id]))
                .collect::<Vec<_>>()
        });
        let exit_owners = exit_state.as_ref().map(|_|context.capture_loan_owners.clone());
        context.storage_exits.record_map_loop_edge(span,storage_exit_availability::MapLoopEdge::Entry,&context.capture_loan_owners);
        Self::validate_control_labels(context,labels)?;
        context.loops.push(LoopFrame {
            labels: labels.to_vec(),
            target: ControlTarget::Tail(span.start),
            flow_entry: initial_flow,
            flow_required: required,
            break_flow: None,
            continue_flow: None,
            breaks: vec![Availability::default(); entry.len()],
            break_seen: false, break_owners:None, continue_owners:None, scope_start: context.bindings.len(),
            defer_continue: true,
            continues: None,
            entry,
            span,
        });
        Ok(Box::new(Tail { zero_flow, exit_state, exit_owners }))
    }

    pub(super) fn prepare_tail_update(context: &mut Context, body_exits: bool, tail_reachable: &mut bool) -> bool {
        let frame = context.loops.last_mut().expect("new loop frame");
        let mut continuing = frame.continues.take();
        let continuing_owners = crate::capture_loan_owners::Owners::reaching(
            (!body_exits).then(||context.capture_loan_owners.clone()),frame.continue_owners.take());
        if !body_exits {
            let states = frame
                .entry
                .iter()
                .map(|(id, _)| Availability::of(&context.bindings[*id]))
                .collect::<Vec<_>>();
            continuing = Some(match continuing {
                Some(previous) => previous
                    .into_iter()
                    .zip(states)
                    .map(|(a, b)| a.join(b))
                    .collect(),
                None => states,
            });
        }
        *tail_reachable = continuing.is_some();
        if let Some(states) = continuing {
            context.capture_loan_owners = continuing_owners;
            Self::restore_live_capture_loans(context);
            if let Some(continues) = context
                .loops
                .last()
                .and_then(|frame| frame.continue_flow.clone())
            {
                context.flow = if body_exits {
                    continues
                } else {
                    crate::flow::join(&context.flow, &continues)
                };
            }
            for ((id, _), state) in context
                .loops
                .last()
                .expect("new loop frame")
                .entry
                .iter()
                .zip(states)
            {
                state.restore(&mut context.bindings[*id]);
            }
        }
        *tail_reachable
    }

    pub(super) fn finish_tail_back(context: &mut Context, frame: &mut Tail, has_condition: bool, span: Span) -> Result<(), Diagnostic> {
        let Tail { exit_state, exit_owners, .. } = frame;
            Self::check_loop_back(context)?;
            context.storage_exits.record_map_loop_edge(span,storage_exit_availability::MapLoopEdge::Back,&context.capture_loan_owners);
            if has_condition {
                *exit_owners = Some(crate::capture_loan_owners::Owners::reaching(
                    exit_owners.take(),Some(context.capture_loan_owners.clone())));
                let states = context
                    .loops
                    .last()
                    .expect("new loop frame")
                    .entry
                    .iter()
                    .map(|(id, _)| Availability::of(&context.bindings[*id]))
                    .collect::<Vec<_>>();
                *exit_state = Some(match exit_state.take() {
                    Some(previous) => previous
                        .into_iter()
                        .zip(states)
                        .map(|(a, b)| a.join(b))
                        .collect(),
                    None => states,
                });
            }
        Ok(())
    }

    pub(super) fn finish_tail_completion(&mut self, context: &mut Context, condition: &Option<Expr>, body: &mut [Stmt], update: &mut [Stmt], span: Span, frame: Box<Tail>) -> Result<bool, Diagnostic> {
        let Tail { zero_flow, mut exit_state, exit_owners } = *frame;
        let frame = context.loops.pop().expect("matching new loop frame");
        if let Some(owners)=&frame.break_owners {context.storage_exits.record_map_loop_edge(span,storage_exit_availability::MapLoopEdge::Break,owners);}
        context.capture_loan_owners = crate::capture_loan_owners::Owners::reaching(exit_owners,frame.break_owners);
        Self::restore_live_capture_loans(context);
        exception_state::consume_target(context,frame.target);
        if let Some(zero) = zero_flow {
            context.flow = crate::flow::join(&zero, &context.flow);
        }
        if let Some(breaks) = &frame.break_flow {
            context.flow = crate::flow::join(&context.flow, breaks);
        }
        if !frame.break_seen
            && let Some(condition) = condition
            && let Some((_, no)) = self.pure_guard_outcomes(context, condition)
        {
            context.flow = no;
        }
        if frame.break_seen {
            exit_state = Some(match exit_state {
                Some(previous) => previous
                    .into_iter()
                    .zip(frame.breaks)
                    .map(|(a, b)| a.join(b))
                    .collect(),
                None => frame.breaks,
            });
        }
        let exits = exit_state.is_none();
        if let Some(states) = exit_state {
            for ((id, _), state) in frame.entry.into_iter().zip(states) {
                state.restore(&mut context.bindings[id]);
            }
        }
        self.record_map_completion_release(context,body,update,span)?;
        Ok(exits)
    }
}
