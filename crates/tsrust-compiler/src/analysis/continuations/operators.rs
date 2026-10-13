//! Operator continuations retain observation and guard state across operands.
use super::super::*;

#[derive(Clone, Copy)]
pub(super) enum Comparison { Ordinary, Typeof, Nullish, Union }

pub(super) struct BinaryFrame {
    comparison: Comparison,
    expected: Option<Type>,
    equality: bool,
    ordering: bool,
    string_addition: bool,
    left_type: Option<Type>,
    observing: Option<bool>,
    captures: Option<crate::capture_loan_identity::Boundary>,
    observed_place: Option<crate::flow::Place>,
    marker: Option<crate::flow::Place>,
}

pub(super) struct Operand {
    pub(super) mode: Mode,
    pub(super) expected: Option<Type>,
    pub(super) loans: Option<Loans>,
}

impl Analyzer {
    pub(super) fn prepare_binary_work(
        &mut self, context: &mut Context, expr: &mut Expr, expected: Option<Type>,
    ) -> Result<Option<(BinaryFrame, Operand)>, Diagnostic> {
        let ExprKind::Binary { op, left, right, .. } = &mut expr.kind else { return Ok(None); };
        if matches!(op, BinaryOp::Coalesce | BinaryOp::PatternDefault | BinaryOp::And | BinaryOp::Or) {
            return Ok(None);
        }
        let equality = matches!(op, BinaryOp::Equal | BinaryOp::NotEqual);
        let ordering = matches!(op, BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual);
        let hints = [self.expression_type_hint(context, left), self.expression_type_hint(context, right)];
        let comparison = if equality && Self::is_typeof_guard(context, left, right) {
            Comparison::Typeof
        } else if equality && hints.iter().any(|ty| matches!(ty, Some(Type::Union(_) | Type::BorrowedUnion(_))))
            && !hints.iter().any(|ty| matches!(ty, Some(Type::Null | Type::Undefined))) {
            Comparison::Union
        } else if equality && hints.iter().any(|ty| matches!(ty, Some(Type::Null | Type::Undefined))) {
            Comparison::Nullish
        } else { Comparison::Ordinary };
        let mut frame = BinaryFrame {
            comparison, expected: None, equality, ordering, string_addition: false,
            left_type: None, observing: None, captures: None, observed_place: None, marker: None,
        };
        let operand = match comparison {
            Comparison::Typeof => Operand { mode: Mode::Read, expected: Some(Type::String), loans: None },
            Comparison::Nullish | Comparison::Union => {
                frame.captures = Some(crate::capture_loan_identity::Boundary::capture(&context.capture_loans));
                frame.observing = Some(self.observing_values);
                self.observing_values = true;
                frame.observed_place = Self::enter_operand_observation(context, left);
                Operand { mode: Mode::Read, expected: None, loans: None }
            }
            Comparison::Ordinary => {
                if *op == BinaryOp::Add && hints.iter().any(|ty| ty.map(Type::scalar_base) == Some(Type::String)) {
                    self.string_concatenation_operand(context, left)?;
                    self.string_concatenation_operand(context, right)?;
                }
                frame.string_addition = *op == BinaryOp::Add && (expected == Some(Type::String)
                    || self.expression_type_hint(context, left).map(Type::scalar_base) == Some(Type::String)
                    || self.expression_type_hint(context, right).map(Type::scalar_base) == Some(Type::String));
                let number = expected != Some(Type::Int)
                    && ((!equality && !ordering && expected == Some(Type::Number))
                        || self.expression_type_hint(context, left).map(Type::scalar_base) == Some(Type::Number)
                        || self.expression_type_hint(context, right).map(Type::scalar_base) == Some(Type::Number));
                frame.expected = if op.bitwise_or_power() {
                    Some(self.bitwise_or_power_type(context, left, Some(right), expected))
                } else if frame.string_addition { Some(Type::String) }
                else if number { Some(Type::Number) }
                else if equality || ordering { None }
                else { Some(Type::Int) };
                Operand { mode: if equality || ordering || frame.string_addition { Mode::Read } else { Mode::Copy }, expected: frame.expected, loans: None }
            }
        };
        Ok(Some((frame, operand)))
    }

    fn enter_operand_observation(context: &mut Context, expr: &Expr) -> Option<crate::flow::Place> {
        let place = crate::flow::place(expr, &context.names);
        if let Some(place) = &place { context.observer_places.insert(place.clone()); }
        place
    }

    fn finish_operand_observation(context: &mut Context, frame: &mut BinaryFrame) {
        if let Some(place) = frame.observed_place.take() { context.observer_places.remove(&place); }
    }

    fn finish_binary_observation(&mut self, context: &mut Context, frame: &mut BinaryFrame) {
        if let Some(observing) = frame.observing.take() { self.observing_values = observing; }
        if let Some(captures) = frame.captures.take() { captures.unwind(&mut context.capture_loans); }
    }

    pub(super) fn finish_binary_left_work(
        &mut self, context: &mut Context, expr: &mut Expr, frame: &mut BinaryFrame,
        checked: Result<Type, Diagnostic>, loans: &Loans,
    ) -> Result<Operand, Diagnostic> {
        Self::finish_operand_observation(context, frame);
        let result = (|| {
            let ExprKind::Binary { operator, left, right, .. } = &mut expr.kind else { unreachable!("binary work"); };
            let actual = checked?;
            let left_type = if matches!(frame.comparison, Comparison::Ordinary) { actual.scalar_base() } else { actual };
            match frame.comparison {
                Comparison::Ordinary => {
                    if (frame.equality && !matches!(left_type, Type::Int | Type::Number | Type::Boolean | Type::String | Type::Enum(_, _)))
                        || (frame.ordering && !matches!(left_type, Type::Int | Type::Number | Type::String | Type::Enum(_, _))) {
                        return Err(Diagnostic::new("type", if frame.equality {
                            "Strict equality requires two int, number, boolean, or string values."
                        } else { "Ordering requires two int values, two number values, or two string values." }, *operator));
                    }
                    self.require(left_type, frame.expected, *operator)?;
                }
                Comparison::Typeof => self.require(left_type, Some(Type::String), left.span)?,
                Comparison::Nullish if left_type == Type::Void => return Err(Diagnostic::new("type", "A void result is not a value for equality.", left.span)),
                _ => (),
            }
            frame.left_type = Some(left_type);
            let retain = match frame.comparison {
                Comparison::Ordinary => left_type == Type::String,
                Comparison::Typeof => true,
                Comparison::Nullish | Comparison::Union => !left_type.is_copy(),
            };
            let active = if retain {
                let mut active = loans.clone();
                for root in Self::loan_roots(left, Mode::Read) { active.entry(root).or_default().push((Mode::Read, left.span)); }
                Some(active)
            } else { None };
            if !matches!(frame.comparison, Comparison::Ordinary) {
                frame.marker = Self::guard_snapshot(context, left, expr.span);
            }
            if matches!(frame.comparison, Comparison::Nullish | Comparison::Union) {
                frame.observed_place = Self::enter_operand_observation(context, right);
            }
            Ok(Operand {
                mode: if !matches!(frame.comparison, Comparison::Ordinary) || left_type == Type::String { Mode::Read } else { Mode::Copy },
                expected: match frame.comparison { Comparison::Ordinary => Some(left_type), Comparison::Typeof => Some(Type::String), _ => None },
                loans: active,
            })
        })();
        if result.is_err() { self.finish_binary_observation(context, frame); }
        result
    }

    pub(super) fn finish_binary_right_work(
        &mut self, context: &mut Context, expr: &mut Expr, mut frame: BinaryFrame,
        checked: Result<Type, Diagnostic>,
    ) -> Result<Type, Diagnostic> {
        Self::finish_operand_observation(context, &mut frame);
        let stable = if matches!(frame.comparison, Comparison::Ordinary) { true }
            else { Self::finish_guard_snapshot(context, frame.marker.take()) };
        self.finish_binary_observation(context, &mut frame);
        let right_type = checked?;
        let left_type = frame.left_type.expect("checked left operand");
        let ExprKind::Binary { op, operator, left, right } = &expr.kind else { unreachable!("binary completion"); };
        match frame.comparison {
            Comparison::Ordinary => {
                self.require(right_type, Some(left_type), *operator)?;
                if frame.equality && (crate::flow::guard_pair(left, right, &context.names).is_some() || self.record_discriminator_guard(context, left, right).is_some()) {
                    self.remember_guard(context, expr.span, *op, left, right);
                }
                Ok(match op {
                    op if op.bitwise_or_power() => left_type,
                    BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Remainder => left_type,
                    _ => Type::Boolean,
                })
            }
            Comparison::Typeof => {
                self.require(right_type, Some(Type::String), right.span)?;
                if stable { self.remember_guard(context, expr.span, *op, left, right); }
                Ok(Type::Boolean)
            }
            Comparison::Nullish => {
                if right_type == Type::Void { return Err(Diagnostic::new("type", "A void result is not a value for equality.", right.span)); }
                if stable { self.remember_guard(context, expr.span, *op, left, right); }
                Ok(Type::Boolean)
            }
            Comparison::Union => self.finish_union_equality_work(context, (expr.span, *operator), *op, left, right, left_type, right_type, stable),
        }
    }
}

pub(super) enum UnaryFrame {
    Scalar(Type),
    Truth { narrowed: bool, reject_void: bool },
    Variant(Type),
    Inspection {
        result: Type,
        observing: bool,
        captures: crate::capture_loan_identity::Boundary,
        place: Option<crate::flow::Place>,
    },
    NonNull { mode: Mode },
}

pub(super) enum UnaryWork {
    Ready(Type),
    Child(UnaryFrame, Operand),
}

impl Analyzer {
    pub(super) fn prepare_unary_work(
        &mut self, context: &mut Context, expr: &mut Expr, mode: Mode, expected: Option<Type>,
    ) -> Result<Option<UnaryWork>, Diagnostic> {
        let ExprKind::Unary { op, value, .. } = &mut expr.kind else { return Ok(None); };
        let (frame, mode, expected) = match *op {
            UnaryOp::NonNull => (UnaryFrame::NonNull { mode }, mode, None),
            UnaryOp::VariantIs(variant) => (UnaryFrame::Variant(variant), Mode::Read, None),
            UnaryOp::Typeof | UnaryOp::Void => {
                if *op == UnaryOp::Typeof && let ExprKind::Name(name) = &value.kind
                    && !context.names.contains_key(name)
                    && context.lexical_scopes.iter().any(|scope| scope.contains(name)) {
                    return Err(Diagnostic::new("name", format!("Cannot inspect {name:?} with typeof before its declaration."), value.span));
                }
                let unresolved = *op == UnaryOp::Typeof
                    && matches!(&value.kind, ExprKind::Name(name) if !context.names.contains_key(name) && !context.pending_names.contains(name) && !self.function_names.contains_key(name) && crate::syntax::global_number_member(name, false).is_none() && crate::syntax::global_number_member(name, true).is_none());
                let result = if *op == UnaryOp::Typeof { Type::String } else { Type::Undefined };
                if unresolved {
                    value.ty = Some(Type::Undefined);
                    expr.ty = Some(result);
                    return Ok(Some(UnaryWork::Ready(result)));
                }
                let captures = crate::capture_loan_identity::Boundary::capture(&context.capture_loans);
                let observing = self.observing_values;
                self.observing_values = true;
                let place = Self::enter_operand_observation(context, value);
                (UnaryFrame::Inspection { result, observing, captures, place }, Mode::Read, None)
            }
            UnaryOp::Not if self.expression_type_hint(context, value).is_some_and(|ty| !matches!(ty, Type::Boolean | Type::Void)) => {
                let ty = self.expression_type_hint(context, value).expect("truthiness hint");
                if matches!(ty, Type::Null | Type::Undefined) {
                    (UnaryFrame::Truth { narrowed: false, reject_void: false }, Mode::Copy, None)
                } else if self.nullable_place(ty, value) {
                    self.truthy_condition(context, value);
                    (UnaryFrame::Truth { narrowed: true, reject_void: false }, Mode::Copy, Some(Type::Boolean))
                } else {
                    (UnaryFrame::Truth { narrowed: false, reject_void: true }, if ty.is_copy() { Mode::Copy } else { Mode::Read }, None)
                }
            }
            UnaryOp::Not | UnaryOp::Positive | UnaryOp::Negate | UnaryOp::BitNot => {
                let operand = if *op == UnaryOp::Not { Type::Boolean }
                    else if *op == UnaryOp::BitNot { self.bitwise_or_power_type(context, value, None, expected) }
                    else if expected == Some(Type::Int) { Type::Int }
                    else if expected == Some(Type::Number) || self.expression_type_hint(context, value) == Some(Type::Number) { Type::Number }
                    else { Type::Int };
                (UnaryFrame::Scalar(operand), Mode::Copy, Some(operand))
            }
            _ => return Ok(None),
        };
        Ok(Some(UnaryWork::Child(frame, Operand { mode, expected, loans: None })))
    }

    pub(super) fn finish_unary_work(
        &mut self, context: &mut Context, expr: &mut Expr, frame: UnaryFrame,
        checked: Result<Type, Diagnostic>, expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        let ExprKind::Unary { op, operator, value } = &mut expr.kind else { unreachable!("unary work"); };
        match frame {
            UnaryFrame::Inspection { result, observing, captures, place } => {
                if let Some(place) = place { context.observer_places.remove(&place); }
                self.observing_values = observing;
                checked?;
                captures.unwind(&mut context.capture_loans);
                Ok(result)
            }
            UnaryFrame::Scalar(operand) => {
                self.require(checked?, Some(operand), *operator)?;
                if *op == UnaryOp::Not {
                    let (yes, no) = Self::condition_outcomes(context, value);
                    context.condition_flows.insert((expr.span.start, expr.span.end), (no, yes));
                }
                Ok(operand)
            }
            UnaryFrame::Truth { narrowed, reject_void } => {
                let actual = checked?;
                if narrowed {
                    self.require(actual, Some(Type::Boolean), *operator)?;
                    let (yes, no) = Self::condition_outcomes(context, value);
                    context.condition_flows.insert((expr.span.start, expr.span.end), (no, yes));
                } else if reject_void && actual == Type::Void {
                    return Err(Diagnostic::new("type", "A function call without a return value has no truth value.", value.span));
                }
                Ok(Type::Boolean)
            }
            UnaryFrame::Variant(variant) => {
                let ty = checked?;
                if !ty.union().is_some_and(|union| self.unions.members(union).contains(&variant)) {
                    return Err(Diagnostic::new("match-pattern", "The selected variant is not part of this value's type.", *operator));
                }
                self.require(Type::Boolean, expected, expr.span)?;
                Ok(Type::Boolean)
            }
            UnaryFrame::NonNull { mode } => self.finish_non_null_work(expr, mode, expected, checked?),
        }
    }
}
