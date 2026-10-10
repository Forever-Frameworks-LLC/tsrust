//! Owner footprints and holder paths are distinct sides of a retained loan.
use super::native_view_loans::{Loan, Segment};
use super::*;

pub(super) fn access_fields(context: &Context, root: usize) -> &[String] {
    if let Some(projection) = &context.projection
        && projection.root == context.bindings[root].name
    {
        return &projection.path;
    }
    context
        .mutation_place
        .as_ref()
        .filter(|place| place.root == root)
        .map_or(&[], |place| place.fields.as_slice())
}

pub(super) fn overlaps(left: &[String], right: &[String]) -> bool {
    left.starts_with(right) || right.starts_with(left)
}

pub(super) fn under(loan: &Loan, fields: &[String]) -> bool {
    loan.path.len() >= fields.len()
        && loan
            .path
            .iter()
            .zip(fields)
            .all(|(segment, field)| matches!(segment,Segment::Field(name) if name == field))
}

pub(super) fn consume(loans: &mut [Loan], fields: &[String]) {
    for loan in loans {
        if under(loan, fields) {
            loan.retained = false;
        }
    }
}

pub(super) fn sources(
    context: &Context,
    source: &Expr,
) -> Result<Vec<crate::flow::Place>, Diagnostic> {
    let mut pending = vec![source];
    let mut places = Vec::new();
    while let Some(value) = pending.pop() {
        match &value.kind {
            ExprKind::Conditional {
                consequent,
                alternate,
                ..
            } => {
                pending.push(consequent);
                pending.push(alternate);
            }
            ExprKind::Sequence(values) => {
                if let Some(last) = values.last() {
                    pending.push(last);
                }
            }
            ExprKind::TypeAssertion { value, .. } | ExprKind::Project { value, .. } => {
                pending.push(value)
            }
            _ if Analyzer::named_storage(value) => {
                let footprint = crate::flow::mutation_footprint(value, &context.names);
                let roots = Analyzer::view_sources(context, value);
                if roots.is_empty() {
                    return Err(source_error(value.span));
                }
                for root in roots {
                    let fields = footprint
                        .as_ref()
                        .filter(|place| place.root == root)
                        .map_or_else(Vec::new, |place| place.fields.clone());
                    places.push(crate::flow::Place { root, fields });
                }
            }
            _ => return Err(source_error(value.span)),
        }
    }
    if places.is_empty() {
        return Err(source_error(source.span));
    }
    places.sort_by(|left, right| (left.root, &left.fields).cmp(&(right.root, &right.fields)));
    places.dedup();
    Ok(places)
}
fn source_error(span: Span) -> Diagnostic {
    Diagnostic::new(
        "native-view-source",
        "Every source alternative of a stored native view requires named storage that outlives the view.",
        span,
    )
}

impl Analyzer {
    pub(super) fn replace_native_view_field(
        context: &mut Context,
        target: &Expr,
        value: &Expr,
    ) -> Result<(), Diagnostic> {
        if !context.native_view_calls {
            return Ok(());
        }
        let Some(place) = crate::flow::place(target, &context.names) else {
            return Ok(());
        };
        if place.fields.is_empty() {
            return Ok(());
        }
        let mut incoming = Self::native_view_value_loans(context, value)?;
        for loan in &mut incoming {
            for field in place.fields.iter().rev() {
                native_view_projections::wrap(loan, &Segment::Field(field.clone()));
            }
        }
        let mut retained = context.bindings[place.root]
            .native_view_loans
            .as_deref()
            .cloned()
            .unwrap_or_default();
        retained.retain(|loan| !under(loan, &place.fields));
        retained.extend(incoming);
        native_view_projections::normalize(&mut retained);
        Self::set_native_view_loans(context, place.root, retained);
        Ok(())
    }
}
