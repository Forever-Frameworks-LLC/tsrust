//! Branch availability, flow and capture ownership retained across scheduled bodies.
use super::super::*;

pub(super) struct Then {
    before: Vec<(usize, Availability)>,
    before_owners: crate::capture_loan_owners::Owners,
    entry_flow: crate::flow::Facts,
    else_flow: crate::flow::Facts,
}

pub(super) struct Else {
    entry: Then,
    after_then_owners: crate::capture_loan_owners::Owners,
    after_then_flow: crate::flow::Facts,
    after_then: Vec<Availability>,
    then_exits: bool,
}

impl Then {
    pub(super) fn abort(self, context: &mut Context) {
        for (id, state) in self.before { state.restore(&mut context.bindings[id]); }
        context.flow = self.entry_flow;
        context.capture_loan_owners = self.before_owners;
        Analyzer::restore_live_capture_loans(context);
    }
}
impl Else {
    pub(super) fn abort(self, context: &mut Context) { self.entry.abort(context); }
}

impl Analyzer {
    pub(super) fn prepare_if_then(&self, context: &mut Context, condition: &Expr, actual: Type) -> Result<Box<Then>, Diagnostic> {
        self.require(actual, Some(Type::Boolean), condition.span)?;
        // Snapshot only visible bindings. Keep contracts and mutation flags
        // accumulating across both arms; restore only availability.
        let before = context
            .names
            .values()
            .map(|&id| (id, Availability::of(&context.bindings[id])))
            .collect::<Vec<_>>();
        let (then_flow, else_flow) = Self::condition_outcomes(context, condition);
        let before_owners = context.capture_loan_owners.clone();
        let entry_flow = std::mem::replace(&mut context.flow, then_flow);
        Ok(Box::new(Then { before, before_owners, entry_flow, else_flow }))
    }

    pub(super) fn prepare_if_else(context: &mut Context, entry: Box<Then>, then_exits: bool) -> Box<Else> {
        let Then { before, before_owners, else_flow, .. } = &*entry;
        let after_then_owners = context.capture_loan_owners.clone();
        let after_then_flow = context.flow.clone();
        let after_then = before
            .iter()
            .map(|(id, _)| Availability::of(&context.bindings[*id]))
            .collect::<Vec<_>>();
        for (id, state) in before {
            state.clone().restore(&mut context.bindings[*id]);
        }
        context.flow = else_flow.clone();
        context.capture_loan_owners = before_owners.clone();
        Self::restore_live_capture_loans(context);
        Box::new(Else { entry: *entry, after_then_owners, after_then_flow, after_then, then_exits })
    }

    pub(super) fn finish_if(&mut self, context: &mut Context, frame: Box<Else>, then_body: &mut [Stmt], else_body: &mut [Stmt], else_exits: bool, span: Span) -> Result<bool, Diagnostic> {
        let Else { entry, after_then_owners, after_then_flow, after_then, then_exits } = *frame;
        let before = entry.before;
        context.capture_loan_owners = crate::capture_loan_owners::Owners::reaching(
            (!then_exits).then_some(after_then_owners),
            (!else_exits).then(|| context.capture_loan_owners.clone()),
        );
        Self::restore_live_capture_loans(context);
        let after_else_flow = context.flow.clone();
        context.flow = match (then_exits, else_exits) {
            (false, false) => crate::flow::join(&after_then_flow, &after_else_flow),
            (false, true) => after_then_flow,
            (true, false) => after_else_flow,
            (true, true) => HashMap::new(),
        };
        Self::join_if_availability(context, before, after_then, then_exits, else_exits);
        self.record_map_completion_release(context,then_body,else_body,span)?;
        self.record_branch_carriers(context,then_body,else_body)?;
        Ok(then_exits && else_exits)
    }
}
