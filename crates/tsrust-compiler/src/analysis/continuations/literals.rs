//! Literal children retain contextual conversion and completion state while suspended.
use super::super::*;

pub(in crate::analysis) struct StoredValueFrame {
    pub(in crate::analysis) family: usize,
    pub(in crate::analysis) expected: Option<Type>,
    pub(in crate::analysis) convert: bool,
}

pub(super) enum LiteralFrame {
    Record {
        frame: Box<super::records::RecordFrame>,
        next: usize,
    },
    Array {
        element: Option<ArrayElement>,
        next: usize,
    },
    Template {
        next: usize,
    },
}

pub(super) struct Child {
    pub(super) index: usize,
    pub(super) mode: Mode,
    pub(super) expected: Option<Type>,
    pub(super) stored: Option<StoredValueFrame>,
    pub(super) field: Option<super::records::FieldFrame>,
}

pub(super) enum Advance {
    Child(Child),
    Done(Type),
}

impl LiteralFrame {
    pub(super) fn child<'a>(&self, expr: &'a mut Expr, index: usize) -> &'a mut Expr {
        match &mut expr.kind {
            ExprKind::Array(values) | ExprKind::Template(values) => &mut values[index],
            ExprKind::Record(fields) => &mut fields[index].1,
            _ => unreachable!("literal child slot"),
        }
    }
}

impl Analyzer {
    pub(super) fn prepare_literal_work(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        expected: Option<Type>,
    ) -> Result<Option<LiteralFrame>, Diagnostic> {
        match &mut expr.kind {
            ExprKind::Record(fields)
                if !fields.iter().any(|(key, _)| key.text == "record#spread") =>
            {
                let frame = self.prepare_record_work(fields, expected, expr.span)?;
                Ok(Some(LiteralFrame::Record {
                    frame: Box::new(frame),
                    next: 0,
                }))
            }
            ExprKind::Array(values)
                if !values.iter().any(|value| {
                    matches!(
                        value.kind,
                        ExprKind::Unary {
                            op: UnaryOp::Spread,
                            ..
                        }
                    )
                }) =>
            {
                let element =
                    self.array_literal_element_hint(context, values, expected, expr.span)?;
                Ok(Some(LiteralFrame::Array { element, next: 0 }))
            }
            ExprKind::Template(_) => Ok(Some(LiteralFrame::Template { next: 0 })),
            _ => Ok(None),
        }
    }

    pub(super) fn advance_literal_work(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        frame: &mut LiteralFrame,
    ) -> Result<Advance, Diagnostic> {
        match frame {
            LiteralFrame::Record { frame, next } => {
                let ExprKind::Record(fields) = &mut expr.kind else {
                    unreachable!("record frame");
                };
                let Some((key, value)) = fields.get_mut(*next) else {
                    return self
                        .finish_record_work(context, fields, frame, expr.span)
                        .map(Advance::Done);
                };
                let (expected, field) = self.prepare_record_child(context, key, value, frame)?;
                Ok(Advance::Child(Child {
                    index: *next,
                    mode: Mode::Consume,
                    expected,
                    stored: None,
                    field: Some(field),
                }))
            }
            LiteralFrame::Array { element, next } => {
                let ExprKind::Array(values) = &mut expr.kind else {
                    unreachable!("array frame");
                };
                let Some(value) = values.get_mut(*next) else {
                    self.join_native_view_literal(values, *element);
                    return self
                        .finish_array_literal(values, *element, expr.span)
                        .map(Advance::Done);
                };
                if element.is_none()
                    && let ExprKind::Closure { id, .. } = value.kind
                {
                    self.escaping.insert(id);
                }
                let stored = if let Some(ArrayElement::StoredCallable(family, 0)) = *element {
                    let family = self.native_view_literal_family(family, value);
                    Some(self.prepare_stored_value(context, value, family)?)
                } else {
                    None
                };
                let expected = stored
                    .as_ref()
                    .map_or_else(|| element.map(ArrayElement::ty), |frame| frame.expected);
                Ok(Advance::Child(Child {
                    index: *next,
                    mode: Mode::Consume,
                    expected,
                    stored,
                    field: None,
                }))
            }
            LiteralFrame::Template { next } => {
                let ExprKind::Template(parts) = &expr.kind else {
                    unreachable!("template frame");
                };
                if *next == parts.len() {
                    return Ok(Advance::Done(Type::String));
                }
                Ok(Advance::Child(Child {
                    index: *next,
                    mode: Mode::Read,
                    expected: None,
                    stored: None,
                    field: None,
                }))
            }
        }
    }

    pub(super) fn finish_literal_child(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        frame: &mut LiteralFrame,
        child: Child,
        actual: Type,
    ) -> Result<(), Diagnostic> {
        if let LiteralFrame::Record { frame, next } = frame {
            let ExprKind::Record(fields) = &mut expr.kind else {
                unreachable!("record frame");
            };
            let (key, value) = &mut fields[child.index];
            self.finish_record_child(
                context,
                key,
                value,
                frame,
                child.field.expect("record field completion"),
                actual,
            )?;
            *next += 1;
            return Ok(());
        }
        let value = frame.child(expr, child.index);
        let actual = if let Some(stored) = child.stored {
            self.finish_stored_value(context, value, stored, actual)?
        } else {
            actual
        };
        match frame {
            LiteralFrame::Record { .. } => unreachable!("record completion handled above"),
            LiteralFrame::Array { element, next } => {
                self.finish_array_literal_element(context, value, actual, element)?;
                *next += 1;
            }
            LiteralFrame::Template { next } => {
                self.finish_template_part(value, actual)?;
                *next += 1;
            }
        }
        Ok(())
    }
}
