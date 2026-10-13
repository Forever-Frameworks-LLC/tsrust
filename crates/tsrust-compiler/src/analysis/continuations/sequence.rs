//! Per-operand cleanup for comma expressions, including discarded prefixes.
use super::super::*;

pub(super) struct Boundary {
    captures: crate::capture_loan_identity::Boundary,
    removal: Option<Box<crate::capture_loan_owners::Owners>>,
    projection: Option<Option<Projection>>,
}

impl Analyzer {
    pub(in crate::analysis) fn sequence_work(&mut self, context: &mut Context, expression: &mut Expr, mode: Mode, loans: &Loans, expected: Option<Type>, discarded: bool, precise: bool, final_snapshot: Option<&mut Option<Box<crate::capture_loan_owners::Owners>>>) -> Result<Type, Diagnostic> {
        super::driver::check_sequence(self, context, expression, mode, loans, expected, discarded, precise, final_snapshot)
    }

    pub(super) fn prepare_sequence_operand(context: &mut Context, expr: &Expr, precise: bool, prefix: bool) -> Boundary {
        let captures = crate::capture_loan_identity::Boundary::capture(&context.capture_loans);
        let removal = precise.then(|| Self::array_removal_snapshot(context, expr)).flatten();
        let projection = if prefix { Some(context.projection.take()) } else { None };
        Boundary { captures, removal, projection }
    }

    pub(super) fn finish_sequence_operand(context: &mut Context, expr: &Expr, frame: Boundary, success: bool) {
        if success { Self::finish_discarded_array_removal(context, expr, frame.removal); }
        if let Some(projection) = frame.projection { context.projection = projection; }
        frame.captures.unwind(&mut context.capture_loans);
        Self::restore_live_capture_loans(context);
    }
}
