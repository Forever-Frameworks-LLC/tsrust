//! Point-sensitive nullable record results from checked dense-array removals.
use super::*;

/// Only direct stores in normal lexical sequences receive a new native epoch.
/// Other reaching stores retain their existing whole-function representation.
pub(super) fn eligible_bindings(context: &Context, body: &[Stmt]) -> HashSet<usize> {
    let mut declarations = HashMap::new();
    let mut direct = HashSet::new();
    let mut pending = body.iter().collect::<Vec<_>>();
    while let Some(statement) = pending.pop() {
        match &statement.kind {
            StmtKind::Block(body) if statement.labels.is_empty() => pending.extend(body),
            StmtKind::Let { binding: Some(binding), value, .. } if context.bindings[*binding].rebindable && context.bindings[*binding].ty.union().is_some() => { declarations.insert(*binding, value.span); }
            StmtKind::Assign { target, value, operator: None } if matches!(target.kind, ExprKind::Name(_)) => {
                if let Some(binding) = target.binding { direct.insert((binding, value.span.start, value.span.end)); }
            }
            _ => {},
        }
    }
    let mut eligible = declarations.keys().copied().filter(|&binding| context.bindings[binding].rebindable).collect::<HashSet<_>>();
    if eligible.is_empty() { return eligible; }
    for write in record_storage_writes::collect(body) {
        if declarations.get(&write.binding) != Some(&write.span)
            && (!write.path.is_empty() || write.array_element || !direct.contains(&(write.binding, write.span.start, write.span.end)))
        { eligible.remove(&write.binding); }
    }
    eligible
}

fn strip_carrier(value: &mut Expr) {
    while matches!(value.kind, ExprKind::Unary { op: UnaryOp::RecordCarrier, .. }) {
        let old = std::mem::replace(value, Expr::new(ExprKind::Null, value.span));
        let ExprKind::Unary { value: inner, .. } = old.into_kind() else { unreachable!() };
        *value = *inner;
    }
}

impl Analyzer {
    /// Preserve a new nominal record environment inside an optional contract.
    /// This changes its carrier, not nominal compatibility or ownership.
    pub(super) fn scoped_record_injection(&self, actual: Type, expected: UnionId, span: Span) -> Result<Option<Type>, Diagnostic> {
        let Some(Type::Record(target)) = self.unions.payload(Type::Union(expected)) else { return Ok(None); };
        // A named nullable value already carries its concrete environment. Keep
        // that environment when injecting into the same nominal contract.
        if let Type::Union(source) = actual {
            if source == expected { return Ok(None); }
            let Some(Type::Record(record)) = self.unions.payload(actual) else { return Ok(None); };
            let declaration = self.scoped_records.declaration(record);
            if declaration == self.scoped_records.declaration(target)
                && self.aggregate_callable_facts(Type::Record(declaration)).scoped
                && self.unions.members(source).into_iter().filter(|member| !matches!(member, Type::Record(_)))
                    .eq(self.unions.members(expected).into_iter().filter(|member| !matches!(member, Type::Record(_))))
            { return Ok(Some(actual)); }
            return Ok(None);
        }
        let Type::Record(record) = actual else { return Ok(None); };
        if actual == Type::Record(target) || !self.aggregate_callable_facts(Type::Record(self.scoped_records.declaration(record))).scoped
            || self.scoped_records.declaration(record) != self.scoped_records.declaration(target)
        { return Ok(None); }
        self.unions.intern(self.unions.members(expected).into_iter().map(|member| {
            if member == Type::Record(target) { actual } else { member }
        }), span).map(Some)
    }

    pub(super) fn array_result_replacement_type(&mut self, value: &mut Expr, selected: Option<Type>, previous: Type, results: &HashMap<usize, Type>) -> Result<Option<Type>, Diagnostic> {
        if let Some(selected) = selected {
            let ty = self.unions.intern([selected, Type::Undefined], value.span)?;
            value.ty = Some(ty);
            return Ok(Some(ty));
        }
        if matches!(value.kind, ExprKind::Name(_)) {
            return Ok(value.binding.and_then(|binding| results.get(&binding).copied()).or(value.ty));
        }
        let ExprKind::Inject { value: inner, union } = &mut value.kind else { return Ok(value.ty); };
        strip_carrier(inner);
        let concrete = match inner.ty {
            Some(Type::Record(record)) => Type::Record(record),
            Some(Type::Null | Type::Undefined) => {
                let Some(Type::Record(record)) = self.unions.payload(previous) else { return Ok(None); };
                self.empty_scoped_record_environment(self.scoped_records.declaration(record))?
            }
            _ => return Ok(None),
        };
        let nullish = previous.union().map(|union| self.unions.members(union)).unwrap_or_default()
            .into_iter().filter(|member|matches!(member,Type::Null|Type::Undefined));
        let ty = self.unions.intern(std::iter::once(concrete).chain(nullish), value.span)?;
        *union = ty.union().expect("nullable record result");
        value.ty = Some(ty);
        Ok(Some(ty))
    }
}

pub(super) fn unwrap_removal(value: &mut Expr) {
    loop {
        let ExprKind::Inject { value: inner, .. } = &value.kind else { break; };
        if !matches!(inner.target, Some(CallTarget::ArrayPop | CallTarget::ArrayShift)) { break; }
        let old = std::mem::replace(value, Expr::new(ExprKind::Null, value.span));
        let ExprKind::Inject { value: inner, .. } = old.into_kind() else { unreachable!() };
        *value = *inner;
    }
}

/// Newly accepted concrete replacements need either a direct epoch or a join.
/// Do not let unsupported branch stores escape as generated Rust type errors.
pub(super) fn reject_unplanned_replacements(context: &Context, body: &[Stmt], eligible: &HashSet<usize>) -> Result<(), Diagnostic> {
    if !context.bindings.iter().any(|binding| binding.rebindable && binding.ty.union().is_some()) { return Ok(()); }
    for write in record_storage_writes::collect(body) {
        if eligible.contains(&write.binding) || !context.bindings[write.binding].rebindable
            || !write.path.is_empty() || write.array_element
        { continue; }
        let ExprKind::Inject { value, .. } = &write.value.kind else { continue; };
        let mut value = value.as_ref();
        while let ExprKind::Unary { op: UnaryOp::RecordCarrier, value: inner, .. } = &value.kind { value = inner; }
        if matches!(value.kind, ExprKind::Record(_)) && write.value.ty != Some(context.bindings[write.binding].ty) {
            return Err(Diagnostic::new("unsupported", "Replacing a captured optional record inside a branch or loop is not supported yet. Use a direct replacement in a normal lexical sequence.", write.span));
        }
    }
    Ok(())
}

impl Analyzer {
    pub(super) fn retag_array_result_uses(&self, body: &mut [Stmt], results: &HashMap<usize, Type>) -> Result<(), Diagnostic> {
        crate::source_walk::body(body, &mut |_| Ok(()), &mut |value| {
            if let Some(ty) = self.planned_record_array_type(value, results) { value.ty = Some(ty); }
            if let ExprKind::Project { value: inner, union, narrowed } = &mut value.kind
                && matches!(narrowed, Type::Record(_))
                && let Some(Type::Union(id)) = self.planned_record_array_type(inner, results)
                && let Some(ty) = self.unions.payload(Type::Union(id))
            { *union = id; *narrowed = ty; value.ty = Some(ty); }
            if let ExprKind::Call(callee, _) = &mut value.kind
                && let Some(Type::BorrowedCallable(family)) = self.planned_record_array_type(callee, results)
            { callee.ty = Some(Type::BorrowedCallable(family)); value.target = Some(CallTarget::StoredCallableField(family)); }
            if let ExprKind::Call(callee, _) = &mut value.kind
                && matches!(value.target, Some(CallTarget::StoredCallableField(_) | CallTarget::ConditionalClosure(_)))
                && let Some(Type::Closure(origin)) = self.planned_record_array_type(callee, results)
            { callee.ty = Some(Type::Closure(origin)); value.target = Some(CallTarget::ConditionalClosure(origin)); }
            Ok(())
        })
    }
}
