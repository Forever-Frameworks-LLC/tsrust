//! Stores keep captured input loans alive independently of their installer value.
use super::*;
use crate::capture_loan_owners::Place;

impl Analyzer {
    pub(super) fn retain_captured_record_borrows(
        &mut self,
        context: &mut Context,
        closure: usize,
        receiver: usize,
        args: &[Expr],
        span: Span,
        effects: Vec<record_parameter_borrows::CapturedBorrow>,
    ) -> Result<(), Diagnostic> {
        for effect in effects {
            let Some(argument) = args.get(effect.target.binding) else {
                continue;
            };
            let Some(binding) = Self::root_binding(argument) else {
                return Err(Diagnostic::new(
                    "callback-escape",
                    "A retained callback capture requires caller-owned named record storage.",
                    argument.span,
                ));
            };
            let Some((_, mut fields)) = Self::direct_fields(argument) else {
                return Err(Diagnostic::new(
                    "callback-escape",
                    "A retained callback capture requires a stable record field.",
                    argument.span,
                ));
            };
            fields.extend(effect.target.fields);
            let target = Place { binding, fields };
            self.retain_record_invocation_contract(context, &target, effect.origin, span)?;
            let capture = &self.closures[closure].captures[effect.source_capture];
            let loans = if matches!(capture.storage, Mode::Consume | Mode::Mutate) {
                vec![(receiver, effect.mode, span)]
            } else {
                let specific = context.capture_loan_owners.loans_under(&Place {
                    binding: receiver,
                    fields: vec![format!("$capture{}", capture.inner)],
                });
                let inherited = if specific.is_empty() {
                    context.capture_loan_owners.loans_under(&Place {
                        binding: receiver,
                        fields: Vec::new(),
                    })
                } else {
                    specific
                };
                let loans = inherited
                    .into_iter()
                    .filter_map(|id| context.capture_loan_ids.loan(id))
                    .collect::<Vec<_>>();
                if loans.is_empty() {
                    vec![(receiver, effect.mode, span)]
                } else {
                    loans
                }
            };
            for loan in loans {
                let id = context.capture_loan_ids.intern(loan);
                context.capture_loan_owners.attach(target.clone(), [id]);
                if !context.capture_loans.contains(&loan) {
                    context.capture_loans.push(loan);
                }
            }
        }
        Ok(())
    }
}
