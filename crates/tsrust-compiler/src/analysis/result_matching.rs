//! Checked operations inserted only by match lowering; source cannot name them.
use super::*;
use crate::match_coverage::{Coverage, CoverageError};

impl Analyzer {
    pub(super) fn check_mutable_match_owner(context: &Context, index: usize, span: Span) -> Result<(), Diagnostic> {
        if !context.effects_only {
            if let Some(holder) = context.mutable_match_borrows.iter().copied().filter(|id| context.bindings[*id].view_roots.contains(&index) && context.names.get(&context.bindings[*id].name) == Some(id)).min() {
                let holder = &context.bindings[holder];
                return Err(Diagnostic::new("view-conflict", "Cannot access the Result or a parent payload while a match borrows it exclusively. Access the selected match payload instead.", span).related(holder.used.unwrap_or(span), "Exclusive match borrow starts here."));
            }
        }
        Ok(())
    }

    pub(super) fn mutable_match_source(context: &Context, value: &Expr) -> bool {
        match &value.kind {
            ExprKind::Name(name) => context.names.get(name).is_some_and(|id| context.mutable_parameters.contains(id) || context.bindings[*id].mutable_place),
            ExprKind::Field(base, _) | ExprKind::Project { value: base, .. } => Self::mutable_match_source(context, base),
            ExprKind::Call(callee, arguments) if matches!(&callee.kind, ExprKind::Name(name) if name == "@match:payload") => arguments.first().is_some_and(|value| Self::mutable_match_source(context, value)),
            _ => false,
        }
    }

    pub(super) fn mutable_match_initializer(context: &Context, name: &Token, value: &Expr) -> bool {
        let generated = name.text.starts_with("match#")
            || matches!(&value.kind, ExprKind::Call(callee, _) if matches!(&callee.kind, ExprKind::Name(name) if name == "@match:payload"))
            || matches!(&value.kind, ExprKind::Project { .. }) && name.text.starts_with("pattern#")
            || matches!(&value.kind, ExprKind::Field(base, _) if matches!(&base.kind, ExprKind::Name(name) if name.starts_with("pattern#")));
        generated && Self::mutable_match_source(context, value)
    }
    pub(super) fn result_variant_records(&self, ty: Type) -> Option<(UnionId, [usize; 2])> {
        let union = ty.union()?;
        let members = self.unions.members(union);
        let variants = crate::result_identity::variants(&self.program.records, &members)?;
        Some((union, variants))
    }

    pub(super) fn result_type_name(&self, ty: Type) -> Option<String> {
        let (_, variants) = self.result_variant_records(ty)?;
        let mut arguments = Vec::new();
        for record in variants {
            let fields = &self.program.records[record].fields;
            let payload = if let Some(field) = fields.first() { field.ty? } else { Type::Void };
            arguments.push(self.type_name(payload).into_owned());
        }
        Some(format!("Result<{}, {}>", arguments[0], arguments[1]))
    }

    #[inline(never)]
    pub(super) fn result_match_intrinsic(&mut self, context: &mut Context, expression: &mut Expr, loans: &Loans) -> Result<Option<Type>, Diagnostic> {
        let ExprKind::Call(callee, arguments) = &expression.kind else { return Ok(None); };
        let ExprKind::Name(name) = &callee.kind else { return Ok(None); };
        if !matches!(name.as_str(), "@match:coverage" | "@match:variant" | "@match:payload") { return Ok(None); }
        let kind = name.clone();
        let ty = self.expression_type_hint(context, &arguments[0]).ok_or_else(|| Diagnostic::new("match-type", "Match needs a known Result<T, E> type.", arguments[0].span))?;
        let (union, variants) = self.result_variant_records(ty).ok_or_else(|| Diagnostic::new("match-type", "This match requires a Result<T, E>; other pattern families are not implemented yet.", arguments[0].span))?;
        if kind == "@match:coverage" {
            let mut coverage = Coverage::new(2);
            for argument in &arguments[1..] {
                let ExprKind::String(label) = &argument.kind else { unreachable!("lowered pattern tag"); };
                let variant = match label.as_str() {
                    "Ok" => Some(0), "Err" => Some(1), "_" => None,
                    _ => return Err(Diagnostic::new("match-pattern", "Result patterns are Ok(payload), Err(error), or _.", argument.span)),
                };
                if let Err(error) = coverage.add(variant) {
                    let (code, message) = match error {
                        CoverageError::Unreachable => ("match-unreachable", "This match arm is unreachable because earlier arms cover its variant."),
                        CoverageError::UnknownVariant => ("match-pattern", "This variant is not part of the matched type."),
                    };
                    return Err(Diagnostic::new(code, message, argument.span));
                }
            }
            let missing = coverage.missing();
            if !missing.is_empty() {
                let names = missing.into_iter().map(|index| ["Ok", "Err"][index]).collect::<Vec<_>>().join(", ");
                return Err(Diagnostic::new("match-exhaustiveness", format!("Non-exhaustive match: add {names} or a final _ arm."), expression.span));
            }
            *expression = Expr::checked(ExprKind::Undefined, expression.span, Some(Type::Undefined));
            return Ok(Some(Type::Undefined));
        }
        let ExprKind::String(label) = &arguments[1].kind else { unreachable!("lowered pattern tag"); };
        let record = match label.as_str() { "Ok" => variants[0], "Err" => variants[1], _ => unreachable!("coverage validated pattern") };
        let value = arguments[0].clone();
        let mutable = Self::mutable_match_source(context, &value);
        if kind == "@match:variant" {
            *expression = Expr::new(ExprKind::Unary { op: UnaryOp::VariantIs(Type::Record(record)), operator: expression.span, value: Box::new(value) }, expression.span);
            return self.expression(context, expression, Mode::Read, loans, Some(Type::Boolean)).map(Some);
        }
        if self.program.records[record].fields.is_empty() {
            return Err(Diagnostic::new("match-payload", "A void variant has no payload to bind; use _ inside its pattern.", expression.span));
        }
        *expression = Expr::checked(ExprKind::Project { value: Box::new(value), union, narrowed: Type::Record(record) }, expression.span, Some(Type::Record(record)));
        let access = if mutable { Mode::Mutate } else if matches!(ty, Type::BorrowedUnion(_)) { Mode::Read } else { Mode::Consume };
        self.expression(context, expression, access, loans, Some(Type::Record(record))).map(Some)
    }
}
