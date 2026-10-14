//! Thread calls preserve concrete callback ownership and typed completion payloads.
use super::*;
use crate::native_signatures::NativeTypeId;

impl Analyzer {
    fn owned_pool_receiver(&self, context: &Context, base: &Expr) -> bool {
        matches!(self.expression_type_hint(context, base), Some(Type::Record(id)) if self.program.records[id].native_type == Some(NativeTypeId::Pool))
    }

    pub(super) fn thread_call(&mut self, context: &mut Context, expression: &mut Expr, loans: &Loans, expected: Option<Type>) -> Result<Option<Type>, Diagnostic> {
        let ExprKind::Call(callee, arguments) = &mut expression.kind else { return Ok(None); };
        if let ExprKind::Field(base, member) = &callee.kind {
            if self.owned_pool_receiver(context, base) {
                let name = match member.as_str() {
                    "task" => "__ownedPoolTask", "stats" => "__ownedPoolStats", "shutdown" => "__ownedPoolShutdown",
                    _ => return Err(Diagnostic::new("pool-member", "Pool supports task, stats and shutdown; its limits are fixed at creation.", callee.span)),
                };
                arguments.insert(0, (**base).clone());
                callee.kind = ExprKind::Name(format!("@native:std:threads#{name}"));
            }
        }
        let ExprKind::Name(name) = &callee.kind else { return Ok(None); };
        let Some((id, binding)) = crate::native_bindings::by_symbol(name).filter(|(_, binding)| binding.module == "std:threads") else { return Ok(None); };
        self.threads_enabled = true;
        let span = expression.span;
        let owned = binding.name.starts_with("__ownedPool");
        if !binding.accepts_arity(arguments.len()) { return Err(Diagnostic::new("arity", format!("{} expects {} arguments.", match binding.name { "__poolTask" | "__ownedPoolTask" => "pool.task", "__poolCreate" => "new Pool", "__poolConfigure" => "pool.configure", "__poolStats" | "__ownedPoolStats" => "pool.stats", "__poolShutdown" | "__ownedPoolShutdown" => "pool.shutdown", name => name }, binding.parameters.len() - usize::from(owned)), span)); }
        let mut active = loans.clone();
        if owned {
            let receiver = self.expression(context, &mut arguments[0], Mode::Read, loans, None)?;
            if !matches!(receiver, Type::Record(id) if self.program.records[id].native_type == Some(NativeTypeId::Pool)) { return Err(Diagnostic::new("pool-receiver", "Expected an owned Pool or a borrow of it.", span)); }
            for root in Self::loan_roots(&arguments[0], Mode::Read) { active.entry(root).or_default().push((Mode::Read, arguments[0].span)); }
        }
        let callback_index = usize::from(owned);
        let result = if matches!(binding.name, "__poolStats" | "__ownedPoolStats") { Type::Record(self.schema_native_dependency(NativeTypeId::PoolStats, span)?)
        } else if binding.name == "cancellationRequested" { Type::Boolean
        } else if matches!(binding.name, "__poolConfigure" | "__poolShutdown" | "__poolCreate" | "__ownedPoolShutdown") {
            if !matches!(binding.name, "__poolShutdown" | "__ownedPoolShutdown") {
                let options = Type::Record(self.schema_native_dependency(NativeTypeId::PoolOptions, span)?);
                let actual = self.expression(context, &mut arguments[0], Mode::Read, loans, Some(options))?;
                let valid = match actual {
                    Type::Record(id) => {
                        let fields = &self.program.records[id].fields;
                        fields.len() == 2 && ["workers", "capacity"].into_iter().all(|name| fields.iter().any(|field| field.name.text == name && !field.optional && field.ty == Some(Type::Int)))
                    },
                    _ => false,
                };
                if !valid { return Err(Diagnostic::new("pool-options", "Pool options must contain exactly workers: int and capacity: int.", arguments[0].span)); }
            }
            let failure = Type::Record(self.schema_native_dependency(NativeTypeId::SpawnError, span)?);
            let output = if binding.name == "__poolCreate" { Type::Record(self.schema_native_dependency(NativeTypeId::Pool, span)?) } else { Type::Void };
            self.thread_result(output, failure, span)?
        } else if matches!(binding.name, "spawn" | "spawnScoped" | "__poolTask" | "__ownedPoolTask") {
            if let ExprKind::Closure { id, .. } = arguments[callback_index].kind { self.escaping.insert(id); }
            let actual = self.expression(context, &mut arguments[callback_index], Mode::Consume, &active, None)?;
            let Type::Closure(origin) = actual else { return Err(Diagnostic::new("thread-callback", "spawn requires an owned concrete zero-argument callback.", arguments[callback_index].span)); };
            let closure = &self.closures[origin];
            if !closure.parameters.is_empty() || closure.program_scoped || !closure.program_captures.is_empty() {
                return Err(Diagnostic::new("thread-callback", "A worker must take no arguments and cannot borrow program state.", arguments[callback_index].span));
            }
            thread_transfer::require_owned_captures(closure, &self.program.records, &|id| self.unions.members(id))
                .map_err(|_| Diagnostic::new("thread-transfer", "Worker captures must own transferable data; caller loans and stored callbacks cannot cross threads.", arguments[callback_index].span))?;
            let output = closure.function.result;
            let effects = self.discovery_error_effects();
            let failures = &effects.closure_errors[origin];
            let runtime = failures.runtime.then(|| self.materialize_discovery_runtime_error());
            thread_transfer::require_owned_completion(output, failures, runtime, &self.program.records, &|id| self.unions.members(id))
                .map_err(|_| Diagnostic::new("thread-transfer", "Worker results and thrown payloads must be resolved owned transferable data.", span))?;
            let error = failures.catch_type(&self.unions, runtime, span)?.unwrap_or(Type::Undefined);
            let handle = self.thread_application(if binding.name == "spawnScoped" { NativeTypeId::ScopedThread } else { NativeTypeId::Thread }, &[output, error], span)?;
            let failure = Type::Record(self.schema_native_dependency(NativeTypeId::SpawnError, span)?);
            self.thread_result(handle, failure, span)?
        } else {
            let handle = self.expression(context, &mut arguments[0], if binding.name == "cancel" { Mode::Read } else { Mode::Consume }, loans, None)?;
            let Type::Record(record) = handle else { return Err(Diagnostic::new("thread-handle", "join and detach consume an owned Thread handle.", span)); };
            if !matches!(self.program.records[record].native_type, Some(NativeTypeId::Thread | NativeTypeId::ScopedThread)) {
                return Err(Diagnostic::new("thread-handle", "join and detach consume an owned Thread handle.", span));
            }
            let parameters = self.program.records[record].application.as_ref().expect("applied thread").resolved_arguments.clone();
            if binding.name == "cancel" { Type::Boolean } else if binding.name == "detach" { Type::Void } else {
                let error = self.thread_application(NativeTypeId::ThreadError, &[parameters[1]], span)?;
                self.thread_result(parameters[0], error, span)?
            }
        };
        self.require(result, expected, span)?;
        expression.ty = Some(result);
        expression.target = Some(CallTarget::NativeBinding(id));
        Ok(Some(result))
    }

    fn thread_application(&mut self, identity: NativeTypeId, arguments: &[Type], span: Span) -> Result<Type, Diagnostic> {
        self.thread_applied(&crate::native_type_resolution::linked_name(identity), arguments, span)
    }
    fn thread_result(&mut self, output: Type, error: Type, span: Span) -> Result<Type, Diagnostic> {
        self.thread_applied("Result", &[output, error], span)
    }
    fn thread_applied(&mut self, name: &str, arguments: &[Type], span: Span) -> Result<Type, Diagnostic> {
        use crate::type_syntax::{ValueAnnotation, ValueAnnotationKind, TypeName};
        let mut annotation = ValueAnnotation { span, kind: ValueAnnotationKind::Applied { name: TypeName { text: name.into(), span }, arguments: arguments.iter().map(|&ty| self.checked_value_annotation(ty, span)).collect::<Result<_, _>>()? } };
        self.generic_records.materialize(&mut annotation, &mut self.program.records, &mut self.record_names, &self.unions, &mut self.callables)?;
        append_checked_record_fields(&mut self.program.records, &self.record_names, &self.unions, &mut self.fields, &mut self.callables)?;
        self.resolve_value_annotation(&annotation, false)
    }
}

pub(super) fn contains_handle(root: Type, records: &[Record], members: &impl Fn(UnionId) -> Vec<Type>) -> bool {
    let mut pending = vec![root]; let mut seen = HashSet::new();
    while let Some(ty) = pending.pop() {
        if !seen.insert(ty) { continue; }
        match ty {
            Type::Record(id) => {
                let record = &records[id];
                if matches!(record.native_type, Some(NativeTypeId::Thread | NativeTypeId::ScopedThread | NativeTypeId::Pool)) { return true; }
                pending.extend(record.fields.iter().filter_map(|field| field.ty));
                if matches!(record.native_type, Some(NativeTypeId::Deque | NativeTypeId::Heap | NativeTypeId::OrderedMap | NativeTypeId::OrderedSet)) {
                    if let Some(application) = &record.application { pending.extend(&application.resolved_arguments); }
                }
            },
            Type::Array(element) => pending.push(element.ty()),
            Type::Union(id) | Type::BorrowedUnion(id) => pending.extend(members(id)),
            Type::Map(id) => if let Some(map) = &records[id].map_definition { pending.extend([map.key, map.value]); },
            Type::Set(id) => if let Some(set) = &records[id].set_definition { pending.push(set.key); },
            _ => {},
        }
    }
    false
}

impl Analyzer {
    pub(super) fn thread_discard(&self, ty: Type, span: Span) -> Result<(), Diagnostic> {
        if self.threads_enabled && contains_handle(ty, &self.program.records, &|id| self.unions.members(id)) {
            return Err(Diagnostic::new("thread-lifecycle", "Cannot discard a worker or pool owner, or an unhandled creation Result. Bind, settle, or transfer ownership explicitly.", span));
        }
        Ok(())
    }
}

pub(super) fn validate(program: &CheckedProgram) -> Result<(), Diagnostic> {
    if !program.records.iter().any(|record| matches!(record.native_type, Some(NativeTypeId::Thread | NativeTypeId::ScopedThread | NativeTypeId::Pool))) { return Ok(()); }
    let contains = |ty| contains_handle(ty, &program.records, &|id| program.unions[id.index()].members.clone());
    for record in &program.records {
        if matches!(record.native_type, Some(NativeTypeId::Thread | NativeTypeId::ScopedThread)) || record.application.as_ref().is_some_and(|app| app.declaration.text.starts_with("Result#")) { continue; }
        if record.fields.iter().filter_map(|field| field.ty).any(&contains)
            || matches!(record.native_type, Some(NativeTypeId::Deque | NativeTypeId::Heap | NativeTypeId::OrderedMap | NativeTypeId::OrderedSet))
                && record.application.as_ref().is_some_and(|app| app.resolved_arguments.iter().copied().any(&contains)) {
            return Err(Diagnostic::new("thread-storage", "Worker and pool owners currently transfer directly or through Result; storing them in records or collections requires lifecycle support for those containers.", record.name.span));
        }
    }
    for closure in &program.closures {
        if closure.captures.iter().any(|capture| contains(closure.function.bindings[capture.inner].ty)) {
            return Err(Diagnostic::new("thread-storage", "A callback cannot retain a worker or pool owner. Settle the owner before capturing other data.", closure.span));
        }
    }
    for function in program.functions.iter().chain(program.closures.iter().map(|closure| &closure.function)).filter(|function| !function.template) {
        for binding in function.bindings.iter().take(function.parameter_count).filter(|binding| contains(binding.ty)) {
            if binding.mode != Mode::Consume && !matches!(binding.ty, Type::Record(id) if program.records[id].native_type == Some(NativeTypeId::Pool)) {
                return Err(Diagnostic::new("thread-lifecycle", format!("Thread-bearing parameter {:?} of {:?} must consume and settle or transfer its owner.",binding.name,function.name), binding.declaration_span.unwrap_or_default()));
            }
        }
        let expressions = crate::error_effects::checked_expressions(function);
        for expression in &expressions {
            if let Some(CallTarget::NativeBinding(id)) = expression.target {
                let binding = &crate::native_bindings::BINDINGS[id];
                if binding.module == "std:threads" && matches!(binding.name, "spawn" | "spawnScoped" | "__poolTask" | "__ownedPoolTask") {
                    let ExprKind::Call(_, arguments) = &expression.kind else { unreachable!() };
                    let Type::Closure(origin) = arguments[usize::from(binding.name == "__ownedPoolTask")].ty.unwrap() else { unreachable!() };
                    let closure = &program.closures[origin];
                    let runtime = program.records.iter().position(|record| record.name.text == "#RuntimeError").map(Type::Record);
                    let members = |id:UnionId| program.unions[id.index()].members.clone();
                    thread_transfer::require_owned_captures(closure, &program.records, &members)
                        .map_err(|_| Diagnostic::new("thread-transfer", "The final worker capture layout must own transferable data.", expression.span))?;
                    let failures = &program.error_effects.closure_errors[origin];
                    thread_transfer::require_owned_completion(closure.function.result, failures, runtime, &program.records, &members)
                        .map_err(|_| Diagnostic::new("thread-transfer", "The final worker completion contract must own transferable data.", expression.span))?;
                    let unions = crate::unions::UnionArena::from_snapshot_with_records(program.unions.clone(), &program.records);
                    let actual = failures.catch_type(&unions,runtime,expression.span)?.unwrap_or(Type::Undefined);
                    let Type::Union(result) = expression.ty.unwrap() else { unreachable!() };
                    let handle = members(result).into_iter().find_map(|ty| {
                        let Type::Record(record) = ty else { return None; };
                        program.records[record].fields.first().and_then(|field| match field.ty {
                            Some(Type::Record(id)) if matches!(program.records[id].native_type, Some(NativeTypeId::Thread | NativeTypeId::ScopedThread)) => Some(id),
                            _ => None,
                        })
                    }).expect("checked spawn handle");
                    if program.records[handle].application.as_ref().unwrap().resolved_arguments[1] != actual {
                        return Err(Diagnostic::new("thread-transfer", "Worker error inference changed after checking; use a callback with a resolved completion contract.", expression.span));
                    }
                }
            }
            if let ExprKind::Call(_, arguments) = &expression.kind {
                for (position, argument) in arguments.iter().enumerate() {
                    if argument.ty.is_some_and(&contains) && arguments[position+1..].iter().any(|later| later.may_fail) {
                        return Err(Diagnostic::new("thread-lifecycle", "Evaluate fallible arguments before transferring a thread owner into this call.", expression.span));
                    }
                }
            }
            if matches!(expression.kind, ExprKind::Conditional { .. } | ExprKind::Binary { op: BinaryOp::And | BinaryOp::Or | BinaryOp::Coalesce, .. } | ExprKind::Optional { .. })
                && expressions.iter().any(|child| child.span.start >= expression.span.start && child.span.end <= expression.span.end && child.ty.is_some_and(&contains)) {
                return Err(Diagnostic::new("thread-lifecycle", "Use explicit if branches to settle thread ownership; conditional and short-circuit owner expressions are not supported.", expression.span));
            }
            if let Some(ty @ (Type::Array(_) | Type::Map(_) | Type::Set(_))) = expression.ty {
                if contains(ty) { return Err(Diagnostic::new("thread-storage", "Worker and pool owners cannot be stored in collections until element lifecycle checking is supported.", expression.span)); }
            }
            if let ExprKind::Sequence(values) = &expression.kind {
                if values.iter().take(values.len().saturating_sub(1)).any(|value| value.ty.is_some_and(&contains)) {
                    return Err(Diagnostic::new("thread-lifecycle", "A comma expression cannot discard thread ownership.", expression.span));
                }
            }
        }
    }
    Ok(())
}
