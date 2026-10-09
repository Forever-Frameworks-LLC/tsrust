//! Caller-owned capture provenance retained by stores through record parameters.
use super::*;
use crate::capture_loan_owners::Place;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Borrow {
    pub(crate) target: Place,
    pub(crate) source: usize,
    pub(crate) mode: Mode,
    pub(crate) origin: usize,
    pub(crate) capture: usize,
}

/// Stable readonly views retain their owner's lifetime, not the local reference slot.
fn borrowed_owner(bindings: &[Binding], mut source: usize) -> usize {
    let mut seen = HashSet::new();
    while seen.insert(source) {
        let Some(binding) = bindings.get(source) else {
            break;
        };
        if binding.rebindable
            || (binding.borrowed_place.is_none() && !binding.callback_reference)
            || binding.view_roots.len() != 1
        {
            break;
        }
        let next = binding.view_roots[0];
        if next == source {
            break;
        }
        source = next;
    }
    source
}

pub(super) fn eligible(function: &CheckedFunction, closure: &CheckedClosure) -> bool {
    closure.captures.iter().all(|capture| {
        !matches!(capture.storage, Mode::Read | Mode::Mutate) || {
            let source = borrowed_owner(&function.bindings, capture.outer);
            function.bindings.get(source).is_some_and(|binding| {
                source < function.parameter_count
                    && binding.parameter
                    && matches!(binding.mode, Mode::Read | Mode::Mutate)
            })
        }
    })
}

fn capable(function: &CheckedFunction) -> bool {
    function.bindings[..function.parameter_count]
        .iter()
        .any(|binding| binding.mode == Mode::Mutate && matches!(binding.ty, Type::Record(_)))
        && function.bindings[..function.parameter_count]
            .iter()
            .any(|binding| {
                !binding.ty.is_copy() && matches!(binding.mode, Mode::Read | Mode::Mutate)
            })
}

fn direct(function: &CheckedFunction, closures: &[CheckedClosure]) -> Vec<Borrow> {
    let mut result = Vec::new();
    for write in record_storage_writes::collect(&function.body) {
        if write.binding >= function.parameter_count
            || function.bindings[write.binding].mode != Mode::Mutate
        {
            continue;
        }
        let mut pending = vec![(write.value, write.path)];
        while let Some((value, path)) = pending.pop() {
            match &value.kind {
                ExprKind::Record(fields) => {
                    for (name, value) in fields {
                        let mut path = path.clone();
                        path.push(name.text.clone());
                        pending.push((value, path));
                    }
                }
                ExprKind::Sequence(values) => {
                    if let Some(last) = values.last() {
                        pending.push((last, path));
                    }
                }
                ExprKind::Conditional {
                    consequent,
                    alternate,
                    ..
                } => {
                    pending.push((consequent, path.clone()));
                    pending.push((alternate, path));
                }
                ExprKind::Inject { value, .. }
                | ExprKind::TypeAssertion { value, .. }
                | ExprKind::Unary { value, .. } => pending.push((value, path)),
                _ => {
                    if let Some(Type::Closure(origin)) = value.ty {
                        let closure = &closures[origin];
                        if !eligible(function, closure) {
                            continue;
                        }
                        for (index, capture) in closure.captures.iter().enumerate() {
                            if matches!(capture.storage, Mode::Read | Mode::Mutate) {
                                result.push(Borrow {
                                    target: Place {
                                        binding: write.binding,
                                        fields: path.clone(),
                                    },
                                    source: borrowed_owner(&function.bindings, capture.outer),
                                    mode: capture.storage,
                                    origin,
                                    capture: index,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    result
}

fn argument_place(expr: &Expr) -> Option<Place> {
    let binding = Analyzer::root_binding(expr)?;
    let (_, fields) = Analyzer::direct_fields(expr)?;
    Some(Place { binding, fields })
}

pub(super) fn summary(
    functions: &[Option<CheckedFunction>],
    closures: &[CheckedClosure],
    root: usize,
) -> Vec<Borrow> {
    let Some(function) = functions.get(root).and_then(Option::as_ref) else {
        return Vec::new();
    };
    if !capable(function) {
        return Vec::new();
    }
    let mut summaries = HashMap::new();
    let mut calls = HashMap::new();
    let mut pending = vec![root];
    while let Some(id) = pending.pop() {
        if summaries.contains_key(&id) {
            continue;
        }
        let Some(function) = functions
            .get(id)
            .and_then(Option::as_ref)
            .filter(|function| capable(function))
        else {
            continue;
        };
        summaries.insert(id, direct(function, closures));
        let mut edges = Vec::new();
        crate::fixed_storage::visit_function_expressions(function, &mut |expr| {
            if let (Some(CallTarget::Function(callee)), ExprKind::Call(_, arguments)) =
                (expr.target, &expr.kind)
            {
                edges.push((
                    callee,
                    arguments.iter().map(argument_place).collect::<Vec<_>>(),
                ));
                pending.push(callee);
            }
        });
        calls.insert(id, edges);
    }
    loop {
        let mut added = Vec::new();
        for (&id, edges) in &calls {
            let function = functions[id].as_ref().unwrap();
            for (callee, arguments) in edges {
                for borrow in summaries.get(callee).into_iter().flatten() {
                    let (Some(Some(target)), Some(Some(source))) = (
                        arguments.get(borrow.target.binding),
                        arguments.get(borrow.source),
                    ) else {
                        continue;
                    };
                    let source_binding = borrowed_owner(&function.bindings, source.binding);
                    if target.binding >= function.parameter_count
                        || source_binding >= function.parameter_count
                    {
                        continue;
                    }
                    let mut mapped = borrow.clone();
                    mapped.target = target.clone();
                    mapped
                        .target
                        .fields
                        .extend(borrow.target.fields.iter().cloned());
                    mapped.source = source_binding;
                    if !summaries[&id].contains(&mapped) && !added.contains(&(id, mapped.clone())) {
                        added.push((id, mapped));
                    }
                }
            }
        }
        if added.is_empty() {
            break;
        }
        for (id, borrow) in added {
            summaries.get_mut(&id).unwrap().push(borrow);
        }
    }
    summaries.remove(&root).unwrap_or_default()
}

impl Analyzer {
    pub(super) fn check_retained_scope_borrows(
        context: &Context,
        first_local: usize,
    ) -> Result<(), Diagnostic> {
        for binding in 0..first_local {
            let place = Place {
                binding,
                fields: Vec::new(),
            };
            for id in context.capture_loan_owners.loans_under(&place) {
                if let Some((source, _, span)) = context.capture_loan_ids.loan(id)
                    && borrowed_owner(&context.bindings, source) >= first_local
                {
                    return Err(Diagnostic::new(
                        "callback-escape",
                        "The stored callback outlives its captured value. Declare the captured owner outside the holder's scope, or dispose of the callback before leaving this scope.",
                        span,
                    ));
                }
            }
        }
        Ok(())
    }

    /// A call can install a mutable wrapper even when the initial field was
    /// capture-free. Preserve all possible origins until exact stores are proved.
    fn retain_record_invocation_contract(
        &mut self,
        context: &mut Context,
        target: &Place,
        origin: usize,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let binding = &context.bindings[target.binding];
        if binding.parameter || binding.capture_reference {
            return Ok(());
        }
        let Type::Record(record) = binding.ty else {
            return Ok(());
        };
        let Some(previous) = self.record_field_type(binding.ty, &target.fields) else {
            return Ok(());
        };
        let incoming = Type::Closure(origin);
        if previous == incoming {
            return Ok(());
        }
        if let Some(union) = previous.union() {
            let mut members = self.unions.members(union);
            let old = self.unions.payload(previous).unwrap_or(previous);
            let payload = if self.absent_callable_type(previous) {
                let abstract_borrow = self.closures[origin]
                    .captures
                    .iter()
                    .any(|capture| matches!(capture.storage, Mode::Read | Mode::Mutate));
                if abstract_borrow {
                    self.borrowed_environment_family(origin, span)?
                        .map(Type::BorrowedCallable)
                        .unwrap_or(incoming)
                } else {
                    incoming
                }
            } else if let Some(family) =
                self.common_borrowed_conditional_family(old, incoming, span)?
            {
                Type::BorrowedCallable(family)
            } else if old == incoming {
                incoming
            } else {
                return Ok(());
            };
            for member in &mut members {
                if !matches!(member, Type::Null | Type::Undefined) {
                    *member = payload;
                }
            }
            let joined = self.unions.intern(members, span)?;
            if let Some(record) = self.record_type_with_field(record, &target.fields, joined) {
                context.bindings[target.binding].ty = Type::Record(record);
            }
            return Ok(());
        }
        if let Some(family) = self.common_borrowed_conditional_family(previous, incoming, span)?
            && let Some(record) =
                self.record_type_with_field(record, &target.fields, Type::BorrowedCallable(family))
        {
            context.bindings[target.binding].ty = Type::Record(record);
        }
        Ok(())
    }

    pub(super) fn retain_record_parameter_borrows(
        &mut self,
        context: &mut Context,
        callee: usize,
        args: &[Expr],
    ) -> Result<(), Diagnostic> {
        for borrow in summary(&self.checked, &self.closures, callee) {
            let (Some(target), Some(source)) =
                (args.get(borrow.target.binding), args.get(borrow.source))
            else {
                continue;
            };
            let (Some(mut target), Some(source_place)) =
                (argument_place(target), argument_place(source))
            else {
                return Err(Diagnostic::new(
                    "callback-escape",
                    "A retained callback borrow requires caller-owned named storage that outlives the record.",
                    source.span,
                ));
            };
            target.fields.extend(borrow.target.fields);
            self.retain_record_invocation_contract(context, &target, borrow.origin, source.span)?;
            let loan = (
                borrowed_owner(&context.bindings, source_place.binding),
                borrow.mode,
                source.span,
            );
            let id = context.capture_loan_ids.intern(loan);
            context.capture_loan_owners.attach(target, [id]);
            if !context.capture_loans.contains(&loan) {
                context.capture_loans.push(loan);
            }
        }
        Ok(())
    }
}
