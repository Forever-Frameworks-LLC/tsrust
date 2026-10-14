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
        let result = self.result_variant_records(ty);
        let members = ty.union().map(|id| self.unions.members(id)).unwrap_or_else(|| vec![ty]);
        let variants = if let Some((_, variants)) = result {
            vec![("Ok".to_string(), variants[0]), ("Err".to_string(), variants[1])]
        } else {
            members.iter().filter_map(|member| match member {
                Type::Record(id) if self.program.records[*id].native_type.is_none() => Some((self.program.records[*id].name.text.clone(), *id)),
                _ => None,
            }).collect::<Vec<_>>()
        };
        if kind == "@match:coverage" {
            if variants.len() != members.len() { return self.literal_match_coverage(expression, ty).map(Some); }
            let mut coverage = Coverage::new(variants.len());
            for argument in &arguments[1..] {
                let ExprKind::String(label) = &argument.kind else { unreachable!("lowered pattern tag"); };
                let variant = if label == "_" { None } else {
                    Some(variants.iter().position(|(name, _)| name == label).ok_or_else(|| Diagnostic::new("match-pattern", "This variant is not part of the matched type.", argument.span))?)
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
                let names = missing.into_iter().map(|index| variants[index].0.as_str()).collect::<Vec<_>>().join(", ");
                return Err(Diagnostic::new("match-exhaustiveness", format!("Non-exhaustive match: add {names} or a final _ arm."), expression.span));
            }
            *expression = Expr::checked(ExprKind::Undefined, expression.span, Some(Type::Undefined));
            return Ok(Some(Type::Undefined));
        }
        let ExprKind::String(label) = &arguments[1].kind else { unreachable!("lowered pattern tag"); };
        let record = variants.iter().find(|(name, _)| name == label).ok_or_else(|| Diagnostic::new("match-pattern", "This variant is not part of the matched type.", expression.span))?.1;
        let value = arguments[0].clone();
        let mutable = Self::mutable_match_source(context, &value);
        let borrowed = self.borrowed_place_hint(context, &value);
        if kind == "@match:variant" {
            *expression = Expr::new(ExprKind::Unary { op: UnaryOp::VariantIs(Type::Record(record)), operator: expression.span, value: Box::new(value) }, expression.span);
            return self.expression(context, expression, Mode::Read, loans, Some(Type::Boolean)).map(Some);
        }
        if result.is_some() && self.program.records[record].fields.is_empty() {
            return Err(Diagnostic::new("match-payload", "A void variant has no payload to bind; use _ inside its pattern.", expression.span));
        }
        let projected = if let Some(union) = ty.union() {
            Expr::checked(ExprKind::Project { value: Box::new(value), union, narrowed: Type::Record(record) }, expression.span, Some(Type::Record(record)))
        } else { value };
        *expression = if result.is_some() { Expr::new(ExprKind::Field(Box::new(projected), "payload".into()), expression.span) } else { projected };
        let access = if mutable { Mode::Mutate } else if borrowed || matches!(ty, Type::BorrowedUnion(_)) { Mode::Read } else { Mode::Consume };
        self.expression(context, expression, access, loans, None).map(Some)
    }
}

impl Analyzer {
    fn literal_match_coverage(&mut self, expression: &mut Expr, ty: Type) -> Result<Type, Diagnostic> {
        use crate::literal_types::LiteralValue;
        let ExprKind::Call(_, arguments) = &expression.kind else { unreachable!() };
        let members = ty.union().map(|id| self.unions.members(id)).unwrap_or_else(|| vec![ty]);
        let mut required = HashSet::new();
        let mut open = HashSet::new();
        for member in members {
            match member {
                Type::Boolean => { required.insert("@boolean:true".to_string()); required.insert("@boolean:false".to_string()); }
                Type::Null => { required.insert("@null:".to_string()); }
                Type::Undefined => { required.insert("@undefined:".to_string()); }
                Type::String => { open.insert("@string"); }
                Type::Int => { open.insert("@int"); }
                Type::Number => { open.insert("@number"); }
                Type::Literal(id, _) => {
                    let key = match self.program.records[id].literal_definition.as_ref().expect("literal type") {
                        LiteralValue::String(value) => format!("@string:{value}"),
                        LiteralValue::Int(value) => format!("@int:{value}"),
                        LiteralValue::Number(bits) => format!("@number:{}", f64::from_bits(*bits)),
                        LiteralValue::Boolean(value) => format!("@boolean:{value}"),
                    };
                    required.insert(key);
                }
                _ => return Err(Diagnostic::new("match-type", "Match patterns need a scalar type or a union of named records.", arguments[0].span)),
            }
        }
        let mut seen = HashSet::new();
        let mut complete = false;
        for argument in &arguments[1..] {
            let ExprKind::String(label) = &argument.kind else { unreachable!() };
            let mut label = label.clone();
            if label.starts_with("@int:") && (open.contains("@number") || !required.contains(&label) && required.iter().any(|key| key.starts_with("@number:"))) {
                let value = label[5..].parse::<f64>().map_err(|_| Diagnostic::new("match-pattern", "Invalid numeric pattern.", argument.span))?;
                label = format!("@number:{value}");
            }
            if complete || !seen.insert(label.clone()) { return Err(Diagnostic::new("match-unreachable", "This match arm is already covered by earlier arms.", argument.span)); }
            if label == "_" { complete = true; continue; }
            let prefix = label.split_once(':').map(|(prefix, _)| prefix).unwrap_or("");
            if !required.contains(&label) && !open.contains(prefix) { return Err(Diagnostic::new("match-pattern", "This literal is not part of the matched type.", argument.span)); }
            complete = open.is_empty() && required.iter().all(|key| seen.contains(key));
        }
        if !complete { return Err(Diagnostic::new("match-exhaustiveness", "Non-exhaustive match: handle every alternative or add a final _ arm.", expression.span)); }
        *expression = Expr::checked(ExprKind::Undefined, expression.span, Some(Type::Undefined));
        Ok(Type::Undefined)
    }
}
