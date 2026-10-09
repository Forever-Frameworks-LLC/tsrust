//! Caller-owned capture provenance retained by stores through record parameters.
use super::record_callable_identity::Callable;
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CapturedBorrow {
    pub(crate) target: Place,
    pub(crate) source_capture: usize,
    pub(crate) capture: usize,
    pub(crate) mode: Mode,
    pub(crate) origin: usize,
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
    eligible_parameters(
        function,
        closure,
        &(0..function.parameter_count).collect::<Vec<_>>(),
    )
}

pub(super) fn eligible_parameters(
    function: &CheckedFunction,
    closure: &CheckedClosure,
    parameters: &[usize],
) -> bool {
    closure.captures.iter().all(|capture| {
        !matches!(capture.storage, Mode::Read | Mode::Mutate) || {
            let source = borrowed_owner(&function.bindings, capture.outer);
            function.bindings.get(source).is_some_and(|binding| {
                parameters.contains(&source) && matches!(binding.mode, Mode::Read | Mode::Mutate)
            })
        }
    })
}

fn capable(function: &CheckedFunction, parameters: &[usize]) -> bool {
    parameters.iter().any(|&id| {
        let binding = &function.bindings[id];
        binding.mode == Mode::Mutate && matches!(binding.ty, Type::Record(_))
    }) && parameters.iter().any(|&id| {
        let binding = &function.bindings[id];
        !binding.ty.is_copy() && matches!(binding.mode, Mode::Read | Mode::Mutate)
    })
}

fn direct(
    function: &CheckedFunction,
    closures: &[CheckedClosure],
    storage: &crate::callable_storage::Registry,
    parameters: &[usize],
) -> Vec<Borrow> {
    let mut result = Vec::new();
    for write in record_storage_writes::collect(&function.body) {
        if !parameters.contains(&write.binding)
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
                    let origins = match value.ty {
                        Some(Type::Closure(origin)) => vec![origin],
                        Some(Type::BorrowedCallable(family)) => storage
                            .origins(family)
                            .map(<[usize]>::to_vec)
                            .unwrap_or_default(),
                        _ => Vec::new(),
                    };
                    for origin in origins {
                        let closure = &closures[origin];
                        if !eligible_parameters(function, closure, parameters) {
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
    storage: &crate::callable_storage::Registry,
    root: usize,
) -> Vec<Borrow> {
    callable_summary(functions, closures, storage, Callable::Function(root))
}

pub(super) fn callable_summary(
    functions: &[Option<CheckedFunction>],
    closures: &[CheckedClosure],
    storage: &crate::callable_storage::Registry,
    root: Callable,
) -> Vec<Borrow> {
    summary_parts(functions, closures, storage, root).0
}

pub(super) fn summary_parts(
    functions: &[Option<CheckedFunction>],
    closures: &[CheckedClosure],
    storage: &crate::callable_storage::Registry,
    root: Callable,
) -> (Vec<Borrow>, Vec<CapturedBorrow>) {
    let Some(view) = root.view(functions, closures) else {
        return (Vec::new(), Vec::new());
    };
    let mut ordinary = Vec::new();
    let mut captured = Vec::new();
    for mut borrow in binding_summary(functions, closures, storage, root) {
        let Some(target) = view
            .parameters
            .iter()
            .position(|id| *id == borrow.target.binding)
        else {
            continue;
        };
        borrow.target.binding = target;
        if let Some(source) = view.parameters.iter().position(|id| *id == borrow.source) {
            borrow.source = source;
            ordinary.push(borrow);
        } else if let Callable::Closure(origin) = root {
            if let Some(source_capture) = closures[origin]
                .captures
                .iter()
                .position(|capture| capture.inner == borrow.source)
            {
                let effect = CapturedBorrow {
                    target: borrow.target,
                    source_capture,
                    capture: borrow.capture,
                    mode: borrow.mode,
                    origin: borrow.origin,
                };
                if !captured.contains(&effect) {
                    captured.push(effect);
                }
            }
        }
    }
    (ordinary, captured)
}

fn binding_summary(
    functions: &[Option<CheckedFunction>],
    closures: &[CheckedClosure],
    storage: &crate::callable_storage::Registry,
    root: Callable,
) -> Vec<Borrow> {
    let Some(view) = root.view(functions, closures) else {
        return Vec::new();
    };
    if !capable(view.function, &view.inputs) {
        return Vec::new();
    }

    let mut summaries = HashMap::new();
    let mut calls = HashMap::new();
    let mut pending = vec![root];
    while let Some(id) = pending.pop() {
        if summaries.contains_key(&id) {
            continue;
        }
        let Some(view) = id.view(functions, closures) else {
            continue;
        };
        let function = view.function;
        if !capable(function, &view.inputs) {
            continue;
        }
        summaries.insert(id, direct(function, closures, storage, &view.inputs));
        let mut local_closures = HashSet::new();
        let mut edges = Vec::new();
        crate::fixed_storage::visit_function_expressions(function, &mut |expr| {
            if matches!(expr.kind, ExprKind::Closure { .. })
                && let Some(Type::Closure(origin)) = expr.ty
            {
                local_closures.insert(origin);
            }
            if let (Some(callee), ExprKind::Call(_, arguments)) = (Callable::call(expr), &expr.kind)
            {
                edges.push((
                    callee,
                    arguments.iter().map(argument_place).collect::<Vec<_>>(),
                ));
                pending.push(callee);
            }
        });
        let edges = edges
            .into_iter()
            .map(|(callee, arguments)| {
                let local_captures =
                    matches!(callee, Callable::Closure(origin) if local_closures.contains(&origin));
                (callee, arguments, local_captures)
            })
            .collect::<Vec<_>>();
        calls.insert(id, edges);
    }
    loop {
        let mut added = Vec::new();
        for (&id, edges) in &calls {
            let view = id.view(functions, closures).unwrap();
            let function = view.function;
            for (callee, arguments, local_capture_layout) in edges {
                let Some(callee_view) = callee.view(functions, closures) else {
                    continue;
                };
                for borrow in summaries.get(callee).into_iter().flatten() {
                    let Some(target_slot) = callee_view
                        .parameters
                        .iter()
                        .position(|id| *id == borrow.target.binding)
                    else {
                        continue;
                    };
                    let Some(Some(target)) = arguments.get(target_slot) else {
                        continue;
                    };
                    let source = if let Some(source_slot) = callee_view
                        .parameters
                        .iter()
                        .position(|id| *id == borrow.source)
                    {
                        arguments
                            .get(source_slot)
                            .and_then(Option::as_ref)
                            .map(|place| place.binding)
                    } else if *local_capture_layout && let Callable::Closure(origin) = callee {
                        // Capture outer IDs belong to the construction context.
                        // Use them only when this caller constructs the callee.
                        closures[*origin]
                            .captures
                            .iter()
                            .find(|capture| capture.inner == borrow.source)
                            .map(|capture| capture.outer)
                    } else {
                        None
                    };
                    let Some(source) = source else {
                        continue;
                    };
                    let source_binding = borrowed_owner(&function.bindings, source);
                    if !view.inputs.contains(&target.binding)
                        || !view.inputs.contains(&source_binding)
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
    pub(super) fn retain_record_invocation_contract(
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
        let borrows = summary(
            &self.checked,
            &self.closures,
            &self.callables.storage,
            callee,
        );
        self.retain_record_borrow_summary(context, borrows, args)
    }

    pub(super) fn retain_record_closure_borrows(
        &mut self,
        context: &mut Context,
        closure: usize,
        receiver: usize,
        args: &[Expr],
        span: Span,
    ) -> Result<(), Diagnostic> {
        let (borrows, captured) = summary_parts(
            &self.checked,
            &self.closures,
            &self.callables.storage,
            Callable::Closure(closure),
        );
        self.retain_record_borrow_summary(context, borrows, args)?;
        self.retain_captured_record_borrows(context, closure, receiver, args, span, captured)
    }

    fn retain_record_borrow_summary(
        &mut self,
        context: &mut Context,
        borrows: Vec<Borrow>,
        args: &[Expr],
    ) -> Result<(), Diagnostic> {
        for borrow in borrows {
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
