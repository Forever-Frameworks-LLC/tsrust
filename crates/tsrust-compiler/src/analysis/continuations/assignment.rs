//! Assignment frames preserve mutation selection and RHS access boundaries.
use super::super::*;

#[derive(Clone, Copy)]
pub(in crate::analysis) enum Kind { Scalar, Logical, Compound { op: BinaryOp, allow_owned: bool }, Update }

pub(in crate::analysis) struct Mutation {
    place: Option<crate::flow::Place>,
    prior: Option<crate::flow::Place>,
    first_keep: usize,
    suppress_target: bool,
}

pub(super) struct Target {
    kind: Kind,
    named: bool,
    mutation: Mutation,
    replacement: Option<Option<Span>>,
}

pub(super) struct Value {
    kind: Kind,
    named: bool,
    ty: Type,
    skipped: Vec<(usize, Availability)>,
}

pub(super) enum Selected {
    Ready(Type),
    Value { frame: Value, mode: Mode, loans: Loans, expected: Type },
}

impl Analyzer {
    pub(in crate::analysis) fn prepare_mutation_work(&mut self, context: &mut Context, target: &Expr, suppress_target: bool) -> Mutation {
        let place = crate::flow::place(target, &context.names);
        let prior = context.mutation_place.clone();
        context.mutation_place = crate::flow::mutation_footprint(target, &context.names);
        let first_keep = context.keep_flow.len();
        // Selecting a stable replacement target has no write effect. Its old
        // proofs remain usable in the RHS; commit invalidates them afterward.
        if suppress_target && let Some(place) = &place {
            context.keep_flow.extend(
                context
                    .flow
                    .keys()
                    .filter(|prior| {
                        prior.root == place.root
                            && (prior.fields.starts_with(&place.fields)
                                || place.fields.starts_with(&prior.fields))
                    })
                    .cloned(),
            );
        }
        if suppress_target && let Some(place) = &place {
            context.suppressed_flow.insert(place.clone());
        }
        Mutation { place, prior, first_keep, suppress_target }
    }

    pub(in crate::analysis) fn finish_mutation_work(&mut self, context: &mut Context, frame: Mutation) {
        let Mutation { place, prior, first_keep, suppress_target } = frame;
        if suppress_target && let Some(place) = &place {
            context.suppressed_flow.remove(place);
        }
        context.mutation_place = prior;
        context.keep_flow.truncate(first_keep);
    }

    pub(in crate::analysis) fn assignment_work(&mut self, context: &mut Context, target: &mut Expr, value: &mut Expr, kind: Kind, operator: Span, loans: &Loans) -> Result<Type, Diagnostic> {
        super::driver::check_assignment(self, context, target, value, kind, operator, loans)
    }

    pub(super) fn prepare_assignment_target(&mut self, context: &mut Context, target: &mut Expr, kind: Kind) -> Result<Target, Diagnostic> {
        if !matches!(kind, Kind::Scalar) { self.reject_string_position_write(context, target)?; }
        let named = matches!(target.kind, ExprKind::Name(_));
        if named {
            self.rebind_target(context, target)?;
        }
        else if !Self::named_storage(target) || !matches!(target.kind, ExprKind::Field(_, _) | ExprKind::Index(_, _)) {
            let message = match kind {
                Kind::Scalar => "Assignment values require a let binding, named record field, or array element.",
                Kind::Logical => "Logical assignment requires a let binding, named record field, or array element.",
                Kind::Compound { .. } => "Compound assignment requires a let binding, named record field, or array element.",
                Kind::Update => "An update requires a let binding, named record field, or array element.",
            };
            return Err(Diagnostic::new("unsupported", message, target.span));
        }
        let replacement = if matches!(kind, Kind::Scalar) { Some(context.replacement.replace(target.span)) } else { None };
        let mutation = self.prepare_mutation_work(context, target, matches!(kind, Kind::Scalar));
        Ok(Target { kind, named, mutation, replacement })
    }

    pub(super) fn assignment_selected(&mut self, context: &mut Context, target: &mut Expr, value: &mut Expr, frame: Target, operator: Span, loans: &Loans, selected: Result<Type, Diagnostic>) -> Result<Selected, Diagnostic> {
        let Target { kind, named, mutation, replacement } = frame;
        self.finish_mutation_work(context, mutation);
        if let Some(prior) = replacement { context.replacement = prior; }
        let ty = selected?;
        let mut skipped = Vec::new();
        match kind {
            Kind::Update => {
                if !matches!(ty, Type::Int | Type::Number) {
                    return Err(Diagnostic::new("type", "An update requires an int or number target without coercion.", operator));
                }
                if named {
                    let id = target.binding.expect("checked update name");
                    if context.is_capture(id) { context.bindings[id].mode = context.bindings[id].mode.max(Mode::Mutate); }
                }
                if ty == Type::Int { exception_state::record_point(context, operator); }
                return Ok(Selected::Ready(ty));
            }
            Kind::Scalar if !ty.is_copy() => return Err(Diagnostic::new("unsupported",
                "The value of an assignment to an array element or a computed place is supported only for int, number and boolean. Assign in its own statement, then read the place.", operator)),
            // Non-boolean `||=`/`&&=` is rewritten earlier for fixed paths only.
            Kind::Logical if self.require(ty, Some(Type::Boolean), operator).is_err() => return Err(Diagnostic::new("unsupported",
                "Logical assignment to a non-boolean value needs a variable or record field target. Read the array element or computed place into a variable, update it, then store it.", operator)),
            Kind::Logical => {},
            Kind::Compound { op, allow_owned } => {
                if !matches!(ty, Type::Int | Type::Number | Type::String) || (ty == Type::String && op != BinaryOp::Add) {
                    return Err(Diagnostic::new("type", "Compound assignment requires the matching implemented int, number, or string operator.", operator));
                }
                if ty == Type::String && !allow_owned {
                    return Err(Diagnostic::new("unsupported", "Compound assignment values support int and number; standalone string += remains supported.", operator));
                }
            }
            Kind::Scalar => {}
        }
        let selected_parent = match &target.kind {
            ExprKind::Field(base, _) if matches!(kind, Kind::Scalar) || matches!(kind, Kind::Compound { .. }) && ty != Type::String => match &base.kind {
                ExprKind::Index(parent, _) => parent.as_ref(),
                _ => base.as_ref(),
            },
            ExprKind::Field(base, _) | ExprKind::Index(base, _) => base.as_ref(),
            _ => target,
        };
        let mut active = if matches!(kind, Kind::Logical) && named { loans.clone() } else { Self::selection_loans(selected_parent, loans) };
        if matches!(kind, Kind::Compound { .. }) && ty == Type::String {
            let root = Self::root_binding(target).expect("checked compound target");
            active.entry(root).or_default().push((Mode::Read, target.span));
            self.string_concatenation_operand(context, value)?;
        }
        if matches!(kind, Kind::Logical) {
            skipped = context.names.values().map(|&id| (id, Availability::of(&context.bindings[id]))).collect();
        }
        let mode = if matches!(kind, Kind::Compound { .. }) && ty == Type::String { Mode::Read } else { Mode::Copy };
        Ok(Selected::Value { frame: Value { kind, named, ty, skipped }, mode, loans: active, expected: ty })
    }

    pub(super) fn finish_assignment_value(&mut self, context: &mut Context, target: &Expr, value: &Expr, frame: Value, operator: Span, loans: &Loans, actual: Type) -> Result<Type, Diagnostic> {
        let Value { kind, named, ty, skipped } = frame;
        self.require(actual, Some(ty), if matches!(kind, Kind::Scalar) { value.span } else { operator })?;
        Self::storage_access(context, target, Mode::Mutate, target.span, loans)?;
        for (id, before) in skipped { before.join(Availability::of(&context.bindings[id])).restore(&mut context.bindings[id]); }
        if let Kind::Compound { op, .. } = kind && ty == Type::Int && crate::error_effects::integer_operation(op) {
            exception_state::record_point(context, operator);
        }
        if named {
            let id = target.binding.expect("checked assignment target");
            if context.is_capture(id) {
                if matches!(kind, Kind::Scalar) { context.bindings[id].used.get_or_insert(target.span); }
                context.bindings[id].mode = context.bindings[id].mode.max(Mode::Mutate);
            }
            if matches!(kind, Kind::Scalar) { context.bindings[id].mutable = true; }
        }
        Ok(ty)
    }
}

pub(super) struct NullishTarget {
    named: bool,
    place: Option<crate::flow::Place>,
    prior_fact: Option<crate::flow::Fact>,
    mutation: Mutation,
    mode: Mode,
    expected: Option<Type>,
}

pub(super) struct NullishValue {
    named: bool,
    place: Option<crate::flow::Place>,
    mode: Mode,
    expected: Option<Type>,
    ty: Type,
    payload: Option<Type>,
    absent: bool,
    possible: Vec<Type>,
    before: Box<Context>,
}

impl Analyzer {
    pub(super) fn prepare_nullish_assignment_work(&mut self, context: &mut Context, expr: &mut Expr, mode: Mode, expected: Option<Type>) -> Result<NullishTarget, Diagnostic> {
        let ExprKind::CoalesceAssign { target, .. } = &mut expr.kind else { unreachable!("nullish assignment"); };
        let named = matches!(target.kind, ExprKind::Name(_));
        if named {
            self.rebind_target(context, target)?;
        } else if !Self::named_storage(target)
            || !matches!(target.kind, ExprKind::Field(..) | ExprKind::Index(..))
        {
            return Err(Diagnostic::new(
                "unsupported",
                "Nullish assignment requires a mutable let binding, named field, or indexed storage.",
                target.span,
            ));
        }
        let place = crate::flow::place(target, &context.names);
        let prior_fact = place.as_ref().and_then(|p| context.flow.get(p)).cloned();
        let mutation = self.prepare_mutation_work(context, target, true);
        Ok(NullishTarget { named, place, prior_fact, mutation, mode, expected })
    }

    pub(super) fn nullish_assignment_selected(&mut self, context: &mut Context, expr: &mut Expr, frame: NullishTarget, loans: &Loans, selected: Result<Type, Diagnostic>) -> Result<(NullishValue, Loans, Type), Diagnostic> {
        let NullishTarget { named, place, prior_fact, mutation, mode, expected } = frame;
        self.finish_mutation_work(context, mutation);
        let ty = selected?;
        let ExprKind::CoalesceAssign { target, value, branch, .. } = &mut expr.kind else { unreachable!("nullish assignment"); };
        let members = ty
            .union()
            .map(|id| self.unions.members(id))
            .unwrap_or_else(|| vec![ty]);
        let possible = prior_fact
            .map(|fact| fact.members)
            .unwrap_or_else(|| members.clone());
        let absent = possible
            .iter()
            .any(|ty| matches!(ty, Type::Null | Type::Undefined));
        *branch = if !absent {
            crate::syntax::NullishBranch::Present
        } else if possible
            .iter()
            .all(|ty| matches!(ty, Type::Null | Type::Undefined))
        {
            crate::syntax::NullishBranch::Absent
        } else {
            crate::syntax::NullishBranch::Dynamic
        };
        let payload = self.unions.present(ty, expr.span)?;
        let before = Box::new(context.clone());
        if let Some(place) = &place {
            let nil = possible
                .iter()
                .copied()
                .filter(|ty| matches!(ty, Type::Null | Type::Undefined))
                .collect::<Vec<_>>();
            if !nil.is_empty() {
                context.flow.insert(
                    place.clone(),
                    crate::flow::Fact {
                        members: nil,
                        origin: Some((expr.span.start, expr.span.end)),
                    },
                );
            }
        }
        let hint = self.expression_type_hint(context, value);
        let mut rhs_members = vec![payload.unwrap_or(ty)];
        if let Some(hint) = hint {
            for member in hint
                .union()
                .map(|id| self.unions.members(id))
                .unwrap_or_else(|| vec![hint])
            {
                if matches!(member, Type::Null | Type::Undefined) {
                    rhs_members.push(member);
                }
            }
        }
        // A sentinel-only destination cannot acquire a different payload.
        if payload.is_none() {
            rhs_members = members.clone();
        }
        let rhs_context = self.unions.intern(rhs_members, value.span)?;
        let selected_parent = match &target.kind {
            ExprKind::Index(base, _) => base.as_ref(),
            ExprKind::Field(base, _) => match &base.kind {
                ExprKind::Index(parent, _) => parent.as_ref(),
                _ => base.as_ref(),
            },
            _ => target.as_ref(),
        };
        let active = Self::selection_loans(selected_parent, loans);
        Ok((NullishValue { named, place, mode, expected, ty, payload, absent, possible, before }, active, rhs_context))
    }

    pub(super) fn finish_nullish_assignment_work(&mut self, context: &mut Context, expr: &mut Expr, frame: NullishValue, loans: &Loans, actual: Type) -> Result<Type, Diagnostic> {
        let NullishValue { named, place, mode, expected, ty, payload, absent, possible, before } = frame;
        let ExprKind::CoalesceAssign { target, value, operator, .. } = &mut expr.kind else { unreachable!("nullish assignment"); };
        self.require(actual, Some(ty), value.span)?;
        if matches!(ty, Type::Closure(_) | Type::Callback(_)) && actual != ty {
            return Err(Diagnostic::new(
                "callable-origin",
                "Nullish assignment must preserve the callable's concrete origin.",
                value.span,
            ));
        }
        let actual_members = actual
            .union()
            .map(|id| self.unions.members(id))
            .unwrap_or_else(|| vec![actual]);
        let actual_members = if let Some(Type::Callback(wanted)) = payload {
            let actual_payload = self.unions.payload(actual).unwrap_or(actual);
            if let Type::Callback(found) = actual_payload
                && wanted != found
            {
                return Err(Diagnostic::new(
                    "callable-origin",
                    "Nullish assignment must preserve the callback parameter origin; matching signatures alone are insufficient.",
                    value.span,
                ));
            }
            actual_members
                .into_iter()
                .map(|member| {
                    if matches!(member, Type::Closure(_) | Type::Callback(_)) {
                        Type::Callback(wanted)
                    } else {
                        member
                    }
                })
                .collect::<Vec<_>>()
        } else {
            actual_members
        };
        let result_members = if absent {
            possible
                .iter()
                .copied()
                .filter(|ty| !matches!(ty, Type::Null | Type::Undefined))
                .chain(actual_members)
                .collect::<Vec<_>>()
        } else {
            possible.clone()
        };
        let logical = self.unions.intern(result_members, expr.span)?;
        if absent {
            Self::storage_access(context, target, Mode::Mutate, target.span, loans)?;
            if possible
                .iter()
                .any(|ty| !matches!(ty, Type::Null | Type::Undefined))
            {
                for id in before.names.values().copied() {
                    Availability::of(&before.bindings[id])
                        .join(Availability::of(&context.bindings[id]))
                        .restore(&mut context.bindings[id]);
                }
                context.flow = crate::flow::join(&before.flow, &context.flow);
            }
        } else {
            // Source-check an unreachable RHS, but retain no access or ownership effects.
            *context = *before;
        }
        if let Some(place) = place {
            crate::flow::invalidate_place(&mut context.flow, &place);
            context.flow.insert(
                place,
                crate::flow::Fact {
                    members: logical
                        .union()
                        .map(|id| self.unions.members(id))
                        .unwrap_or_else(|| vec![logical]),
                    origin: Some((expr.span.start, expr.span.end)),
                },
            );
        }
        if named {
            let id = target.binding.expect("checked target");
            context.bindings[id].mutable = true;
        }
        if !logical.is_copy() && matches!(mode, Mode::Consume | Mode::Mutate) {
            return Err(Diagnostic::new(
                "borrowed-place",
                "An owned assignment result borrows its destination; use a readonly consumer or clone() for independent ownership.",
                *operator,
            ));
        }
        self.require(logical, expected, expr.span)?;
        Ok(if !logical.is_copy() {
            logical.union().map(Type::BorrowedUnion).unwrap_or(logical)
        } else {
            logical
        })
    }
}
