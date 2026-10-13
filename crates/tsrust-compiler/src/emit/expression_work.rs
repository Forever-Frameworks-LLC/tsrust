use super::*;

struct Boundary {
    depth: usize,
    first_temporary: usize,
}

enum CallState<'p> {
    Callable,
    Declared {
        index: usize,
        native_name: String,
        result_target: Option<(String, String)>,
    },
    Ready(String),
    Forward {
        function: usize,
        once: bool,
    },
    Method {
        closure: usize,
        once: bool,
        mutable: bool,
        plan: crate::polymorphic_invocation_plans::Plan<'p>,
    },
}

struct CallFrame<'a, 'p> {
    children: Vec<(&'a Expr, Mode)>,
    rendered: Vec<String>,
    invocation: Option<Mode>,
    state: CallState<'p>,
}

struct Frame<'a, 'p> {
    expr: &'a Expr,
    mode: Mode,
    boundary: Option<Boundary>,
    call: Option<CallFrame<'a, 'p>>,
}

impl<'a, 'p> Frame<'a, 'p> {
    fn new(expr: &'a Expr, mode: Mode) -> Self {
        Self {
            expr,
            mode,
            boundary: None,
            call: None,
        }
    }
}

impl<'p> Emitter<'p> {
    // A suspended parent owns its wrapper boundary and completed child fragments.
    // Legacy expression variants still dispatch synchronously during migration.
    pub(super) fn emit_expression_work(
        &self,
        fun: &CheckedFunction,
        expr: &Expr,
        mode: Mode,
    ) -> String {
        let mut work = vec![Frame::new(expr, mode)];
        let mut completed = None;
        loop {
            let frame = work.last_mut().expect("active expression");
            if let Some(code) = completed.take() {
                frame
                    .call
                    .as_mut()
                    .expect("suspended call")
                    .rendered
                    .push(code);
            }
            if frame.boundary.is_none() {
                // Sequence has its own temporary policy and precedes copy adaptation.
                if let ExprKind::Sequence(values) = &frame.expr.kind {
                    let code = self.sequence_expression(
                        fun,
                        values,
                        frame.mode,
                        frame.expr.ty == Some(Type::Void),
                    );
                    work.pop();
                    if work.is_empty() {
                        return code;
                    }
                    completed = Some(code);
                    continue;
                }
                if frame.expr.ty.is_some_and(Type::is_copy) {
                    frame.mode = Mode::Copy;
                }
                let depth = self.expression_depth.get();
                frame.boundary = Some(Boundary {
                    depth,
                    first_temporary: self.conditional_temporaries.borrow().len(),
                });
                self.expression_depth.set(depth + 1);
                frame.call = self.prepare_callable_work(fun, frame.expr, frame.mode);
            }
            let code = if let Some(call) = &mut frame.call {
                if let Some(&(child, mode)) = call.children.get(call.rendered.len()) {
                    work.push(Frame::new(child, mode));
                    continue;
                }
                self.finish_callable_work(
                    fun,
                    frame.expr,
                    frame.mode,
                    call.invocation,
                    std::mem::replace(&mut call.state, CallState::Callable),
                    std::mem::take(&mut call.rendered),
                )
            } else {
                self.expr_dispatch(fun, frame.expr, frame.mode)
            };
            let boundary = frame.boundary.take().expect("entered expression");
            let code = self.finish_expression_work(
                fun,
                frame.expr,
                frame.mode,
                boundary.depth,
                boundary.first_temporary,
                code,
            );
            work.pop();
            if work.is_empty() {
                return code;
            }
            completed = Some(code);
        }
    }

    fn prepare_callable_work<'a>(
        &self,
        fun: &CheckedFunction,
        expr: &'a Expr,
        mode: Mode,
    ) -> Option<CallFrame<'a, 'p>> {
        let ExprKind::Call(callee, args) = &expr.kind else {
            return None;
        };
        let mut children = Vec::new();
        let mut invocation = None;
        let mut state = CallState::Callable;
        match expr.target? {
            CallTarget::Function(index) => {
                if let Some(code) = self.direct_record_store_call(fun, index, args) {
                    let code = match mode {
                        Mode::Read => format!("&({code})"),
                        Mode::Mutate => format!("&mut ({code})"),
                        Mode::Consume | Mode::Copy => code,
                    };
                    state = CallState::Ready(code);
                } else {
                    state = CallState::Declared {
                        index,
                        result_target: self.result_functions.borrow().get(&index).cloned(),
                        native_name: self.native_call_name(index, args),
                    };
                    children.extend(
                        args.iter()
                            .zip(&self.program.functions[index].bindings)
                            .map(|(arg, p)| (arg, p.mode)),
                    );
                }
            }
            CallTarget::ForwardFunction(function) => {
                let Type::Closure(origin) = callee.ty.expect("checked forwarded receiver") else {
                    unreachable!()
                };
                let once = matches!(
                    self.program.closures[origin].forward,
                    Some(ClosureForward::Once(_))
                );
                let mode = self.conditional_invocation(
                    fun,
                    callee,
                    self.program.closures[origin].invocation,
                );
                children.push((callee.as_ref(), mode));
                children.extend(
                    args.iter()
                        .zip(&self.program.functions[function].bindings)
                        .map(|(arg, p)| (arg, p.mode)),
                );
                state = CallState::Forward { function, once };
            }
            CallTarget::ClosureMethod(method) => {
                let Type::Closure(origin) = callee.ty.expect("checked method receiver") else {
                    unreachable!()
                };
                let (closure, once) = match self.program.closures[origin].forward {
                    Some(ClosureForward::Once(inner)) => (inner, true),
                    _ => (origin, false),
                };
                let pairs = [crate::polymorphic_invocation_plans::Pair {
                    origin: closure,
                    method,
                }];
                let plan = if let Some(id) = callee
                    .generic_invocation
                    .or_else(|| self.program.polymorphic_invocations.index(&pairs))
                {
                    self.program.polymorphic_invocations.checked(
                        id,
                        &self.program.closures,
                        callee.span,
                    )
                } else {
                    crate::polymorphic_invocation_plans::Plan::checked(
                        pairs,
                        &self.program.closures,
                        callee.span,
                    )
                }
                .expect("checked materialized invocation plan");
                let mode = if once {
                    Mode::Consume
                } else {
                    self.conditional_invocation(fun, callee, plan.invocation)
                };
                let mutable = mode == Mode::Consume && plan.invocation == Mode::Mutate;
                children.push((callee.as_ref(), mode));
                children.extend(
                    args.iter()
                        .zip(&plan.argument_modes)
                        .map(|(arg, &mode)| (arg, mode)),
                );
                state = CallState::Method {
                    closure,
                    once,
                    mutable,
                    plan,
                };
            }
            CallTarget::ConditionalClosure(origin) => {
                let closure = &self.program.closures[origin];
                let mode = self.conditional_invocation(fun, callee, closure.invocation);
                invocation = Some(mode);
                children.push((callee.as_ref(), mode));
                children.extend(
                    args.iter()
                        .zip(&closure.parameters)
                        .map(|(arg, &id)| (arg, closure.function.bindings[id].mode)),
                );
            }
            CallTarget::ConditionalCallback(contract) => {
                let checked = &self.program.callback_contracts[contract];
                children.push((callee.as_ref(), checked.kind.invocation()));
                children.extend(
                    args.iter()
                        .zip(&checked.params)
                        .map(|(arg, p)| (arg, p.mode)),
                );
            }
            CallTarget::Closure(binding) => {
                let Type::Closure(id) = fun.bindings[binding].ty else {
                    unreachable!("checked concrete closure");
                };
                let closure = &self.program.closures[id];
                children.extend(
                    args.iter()
                        .zip(&closure.parameters)
                        .map(|(arg, &id)| (arg, closure.function.bindings[id].mode)),
                );
            }
            CallTarget::StoredCallable(binding) => {
                let (Type::StoredCallable(family) | Type::BorrowedCallable(family)) =
                    fun.bindings[binding].ty
                else {
                    unreachable!("checked stored binding");
                };
                let declaration = self
                    .program
                    .stored_contracts
                    .declaration(family)
                    .expect("checked storage signature");
                children.extend(
                    args.iter()
                        .zip(&self.program.callback_contracts[declaration].params)
                        .map(|(arg, p)| (arg, p.mode)),
                );
            }
            CallTarget::StoredCallableField(family) => {
                let declaration = self
                    .program
                    .stored_contracts
                    .declaration(family)
                    .expect("checked field storage signature");
                let contract = &self.program.callback_contracts[declaration];
                children.push((
                    callee.as_ref(),
                    contract.invocation(&self.program.callable_shapes),
                ));
                children.extend(
                    args.iter()
                        .zip(&contract.params)
                        .map(|(arg, p)| (arg, p.mode)),
                );
            }
            CallTarget::Callback(binding) => {
                let Type::Callback(id) = fun.bindings[binding].ty else {
                    unreachable!("checked callback parameter");
                };
                children.extend(
                    args.iter()
                        .zip(&self.program.callback_contracts[id].params)
                        .map(|(arg, p)| (arg, p.mode)),
                );
            }
            _ => return None,
        }
        Some(CallFrame {
            children,
            rendered: Vec::new(),
            invocation,
            state,
        })
    }

    fn finish_callable_work(
        &self,
        fun: &CheckedFunction,
        expr: &Expr,
        mode: Mode,
        invocation: Option<Mode>,
        state: CallState<'_>,
        rendered: Vec<String>,
    ) -> String {
        let ExprKind::Call(callee, args) = &expr.kind else {
            unreachable!("scheduled call");
        };
        let mut rendered = rendered.into_iter();
        let code = match state {
            CallState::Ready(code) => return code,
            CallState::Declared {
                index,
                native_name,
                result_target,
            } => {
                let mut values = rendered.by_ref().collect::<Vec<_>>();
                values.extend(self.named_capture_arguments(fun, index, expr));
                values.extend(self.owner_call_arguments(fun, index, &expr.kind));
                let arguments = values.join(", ");
                let call = format!("{native_name}({arguments})");
                let code = if let Some((_, source_name)) = result_target {
                    let unions = crate::unions::UnionArena::from_snapshot_with_records(
                        self.program.unions.clone(),
                        &self.program.records,
                    );
                    self.exception_regions
                        .function_result_call(
                            &call,
                            index,
                            &source_name,
                            &self.program.error_effects,
                            &unions,
                            expr.span,
                        )
                        .expect("checked registered Result call")
                } else {
                    call
                };
                let updates = self.record_call_reification(fun, args, index);
                if updates.is_empty() {
                    code
                } else {
                    format!("{{let record_call_result={code};{updates}record_call_result}}")
                }
            }
            CallState::Forward { function, once } => {
                let receiver = rendered.next().expect("queued receiver");
                let mut text = format!("{{ let forward_environment = {receiver}; ");
                if once {
                    text.push_str("let forward_environment = forward_environment.capture0; ");
                }
                for (index, value) in rendered.by_ref().enumerate() {
                    text.push_str(&format!("let forward_argument{index} = {value}; "));
                }
                let mut arguments = (0..args.len())
                    .map(|index| format!("forward_argument{index}"))
                    .collect::<Vec<_>>();
                arguments.extend(self.owner_argument_markers(fun, function, args));
                let arguments = arguments.join(", ");
                let result_target = self.result_functions.borrow().get(&function).cloned();
                let name = self.native_call_name(function, args);
                text.push_str(&format!("{name}({arguments}) }}"));
                let code = if let Some((_, source_name)) = result_target {
                    let unions = crate::unions::UnionArena::from_snapshot_with_records(
                        self.program.unions.clone(),
                        &self.program.records,
                    );
                    self.exception_regions
                        .function_result_call(
                            &text,
                            function,
                            &source_name,
                            &self.program.error_effects,
                            &unions,
                            callee.span,
                        )
                        .expect("checked forwarded function Result call")
                } else {
                    text
                };
                let updates = self.record_call_reification(fun, args, function);
                if updates.is_empty() {
                    code
                } else {
                    format!("{{let forward_result={code};{updates}forward_result}}")
                }
            }
            CallState::Method {
                closure,
                once,
                mutable,
                plan,
            } => {
                let receiver = rendered.next().expect("queued receiver");
                let receiver = if once {
                    format!("({receiver}).capture0")
                } else {
                    receiver
                };
                let values = rendered.by_ref().collect::<Vec<_>>();
                let family = self.program.polymorphic_storage.contains(closure).then(|| {
                    self.program
                        .polymorphic_storage
                        .checked(closure, &self.program.closures, callee.span)
                        .expect("checked invocation storage")
                });
                let unions = crate::unions::UnionArena::from_snapshot_with_records(
                    self.program.unions.clone(),
                    &self.program.records,
                );
                plan.render_with_transport(
                    &receiver,
                    &values,
                    &expr
                        .supplied_count
                        .unwrap_or(CallArgumentCount::Static(args.len()))
                        .to_string(),
                    |pair| {
                        matches!(
                            self.program.closures[pair.origin].forward,
                            Some(ClosureForward::RestAdapter { counted: true, .. })
                        )
                    },
                    mutable,
                    family
                        .as_ref()
                        .map(|family| (family.id, family.origins.as_slice())),
                    callee.span,
                    |pair, text| {
                        if let Some(name) = self
                            .result_methods
                            .borrow()
                            .get(&(pair.origin, pair.method))
                            .cloned()
                        {
                            self.exception_regions.result_call(
                                &text,
                                &name,
                                &self.program.error_effects.method_errors
                                    [&(pair.origin, pair.method)],
                                &unions,
                                callee.span,
                            )
                        } else {
                            Ok(text)
                        }
                    },
                )
                .expect("checked materialized invocation arguments and Result transport")
            }
            CallState::Callable => {
                return self.finish_bound_callable_work(fun, expr, mode, invocation, &mut rendered);
            }
        };
        debug_assert!(rendered.next().is_none());
        Self::access_wrapped(expr, mode, code)
    }

    fn finish_bound_callable_work(
        &self,
        fun: &CheckedFunction,
        expr: &Expr,
        mode: Mode,
        invocation: Option<Mode>,
        rendered: &mut std::vec::IntoIter<String>,
    ) -> String {
        let ExprKind::Call(callee, args) = &expr.kind else {
            unreachable!("scheduled call");
        };
        let code = match expr.target.expect("checked call target") {
            CallTarget::ConditionalClosure(origin) => {
                let closure = &self.program.closures[origin];
                let invocation = invocation.expect("queued conditional invocation");
                let receiver = rendered.next().expect("queued receiver");
                let mut code = format!("{{ let mut conditional_environment = {receiver}; ");
                for (index, (_arg, &_parameter)) in args.iter().zip(&closure.parameters).enumerate()
                {
                    code.push_str(&format!(
                        "let conditional_argument{index} = {}; ",
                        rendered.next().expect("queued argument")
                    ));
                }
                let arguments = (0..args.len())
                    .map(|index| format!("conditional_argument{index}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                if let Some(call) = self.record_point_call(
                    fun,
                    callee,
                    Type::Closure(origin),
                    "conditional_environment",
                    &arguments,
                    expr.supplied_count
                        .unwrap_or(CallArgumentCount::Static(args.len())),
                ) {
                    code.push_str(&format!("{call} }}"));
                    return self.registered_closure_call(code, origin, expr.span);
                }
                if invocation == Mode::Consume && closure.invocation != Mode::Consume {
                    let arguments =
                        tuple((0..args.len()).map(|index| format!("conditional_argument{index}")));
                    code.push_str(&format!(
                        "{} }}",
                        self.counted_invocation(
                            "OnceCallback::invoke_once",
                            "conditional_environment",
                            &arguments,
                            expr.supplied_count
                                .unwrap_or(CallArgumentCount::Static(args.len()))
                        )
                    ));
                } else {
                    code.push_str(&format!(
                        "{} }}",
                        self.counted_call(
                            "conditional_environment",
                            &arguments,
                            expr.supplied_count
                                .unwrap_or(CallArgumentCount::Static(args.len()))
                        )
                    ));
                }
                // Carrier planning can widen the stored field to a family
                // after this call was checked; its Result layout then applies.
                match self.record_callee_family(fun, callee) {
                    Some(family) => self.registered_storage_call(code, family, expr.span),
                    None => self.registered_closure_call(code, origin, expr.span),
                }
            }
            CallTarget::ConditionalCallback(contract) => {
                let checked = &self.program.callback_contracts[contract];
                let once = checked.kind.is_once();
                let receiver = rendered.next().expect("queued receiver");
                let mut code = format!("{{ let conditional_environment = {receiver}; ");
                for (index, (_arg, _parameter)) in args.iter().zip(&checked.params).enumerate() {
                    code.push_str(&format!(
                        "let conditional_argument{index} = {}; ",
                        rendered.next().expect("queued argument")
                    ));
                }
                let arguments =
                    tuple((0..args.len()).map(|index| format!("conditional_argument{index}")));
                let method = if matches!(
                    checked.kind,
                    crate::syntax::CallbackKind::Read | crate::syntax::CallbackKind::SharedRead
                ) {
                    "ReadCallback::invoke_read"
                } else if once {
                    "OnceCallback::invoke_once"
                } else {
                    "Callback::invoke"
                };
                if self.program.absent_callback_payloads.contains(&contract) {
                    code.push_str(&format!(
                        "let absent_result: {} = {}; absent_result }}",
                        self.callback_result_type(
                            contract,
                            expr.ty.expect("checked absent callback result")
                        ),
                        self.counted_invocation(
                            method,
                            "conditional_environment",
                            &arguments,
                            expr.supplied_count
                                .unwrap_or(CallArgumentCount::Static(args.len()))
                        )
                    ));
                } else {
                    code.push_str(&format!(
                        "{} }}",
                        self.counted_invocation(
                            method,
                            "conditional_environment",
                            &arguments,
                            expr.supplied_count
                                .unwrap_or(CallArgumentCount::Static(args.len()))
                        )
                    ));
                }
                self.registered_callback_call(code, contract, expr.span)
            }
            CallTarget::Closure(binding) => {
                let Type::Closure(id) = fun.bindings[binding].ty else {
                    unreachable!("checked concrete closure")
                };
                let closure = &self.program.closures[id];
                let mut arguments = args
                    .iter()
                    .zip(&closure.parameters)
                    .map(|(_arg, &_id)| rendered.next().expect("queued argument"))
                    .collect::<Vec<_>>();
                arguments.extend(self.closure_owner_arguments(fun, id, args, binding));
                let args = arguments.join(", ");
                let call = if fun.bindings[binding].callback_invocation == Some(Mode::Consume) {
                    let args = if args.is_empty() {
                        "()".into()
                    } else {
                        format!("({args},)")
                    };
                    self.counted_invocation(
                        "OnceCallback::invoke_once",
                        &format!("value{binding}"),
                        &args,
                        expr.supplied_count
                            .unwrap_or(CallArgumentCount::Static(closure.parameters.len())),
                    )
                } else {
                    self.counted_call(
                        &format!("value{binding}"),
                        &args,
                        expr.supplied_count
                            .unwrap_or(CallArgumentCount::Static(closure.parameters.len())),
                    )
                };
                let call = self.registered_closure_call(call, id, expr.span);
                let ExprKind::Call(_, arguments) = &expr.kind else {
                    unreachable!("closure call")
                };
                let reification = self.record_callable_reification(
                    fun,
                    arguments,
                    borrowed_owner_provenance::Scope::Closure(id),
                );
                if reification.is_empty() {
                    call
                } else {
                    format!("{{let closure_call_result={call};{reification} closure_call_result}}")
                }
            }
            CallTarget::StoredCallable(binding) => {
                let (Type::StoredCallable(family) | Type::BorrowedCallable(family)) =
                    fun.bindings[binding].ty
                else {
                    unreachable!("checked stored binding");
                };
                let declaration = self
                    .program
                    .stored_contracts
                    .declaration(family)
                    .expect("checked storage signature");
                let arguments = args
                    .iter()
                    .zip(&self.program.callback_contracts[declaration].params)
                    .map(|(_arg, _contract)| rendered.next().expect("queued argument"))
                    .collect::<Vec<_>>()
                    .join(", ");
                let call = self.counted_call(
                    &format!("value{binding}"),
                    &arguments,
                    expr.supplied_count
                        .unwrap_or(CallArgumentCount::Static(args.len())),
                );
                self.registered_storage_call(call, family, expr.span)
            }
            CallTarget::StoredCallableField(family) => {
                let declaration = self
                    .program
                    .stored_contracts
                    .declaration(family)
                    .expect("checked field storage signature");
                let contract = &self.program.callback_contracts[declaration];
                let receiver = rendered.next().expect("queued receiver");
                let arguments = args
                    .iter()
                    .zip(&contract.params)
                    .map(|(_arg, _parameter)| rendered.next().expect("queued argument"))
                    .collect::<Vec<_>>()
                    .join(", ");
                let count = expr
                    .supplied_count
                    .unwrap_or(CallArgumentCount::Static(args.len()));
                let call = self
                    .record_point_call(
                        fun,
                        callee,
                        Type::BorrowedCallable(family),
                        &format!("({receiver})"),
                        &arguments,
                        count,
                    )
                    .unwrap_or_else(|| {
                        self.counted_call(&format!("({receiver})"), &arguments, count)
                    });
                self.registered_storage_call(call, family, expr.span)
            }
            CallTarget::Callback(binding) => {
                let Type::Callback(id) = fun.bindings[binding].ty else {
                    unreachable!("checked callback parameter")
                };
                let args = tuple(
                    args.iter()
                        .zip(&self.program.callback_contracts[id].params)
                        .map(|(_arg, _contract)| rendered.next().expect("queued argument")),
                );
                let call = if self.program.absent_callback_payloads.contains(&id) {
                    let kind = self.program.callback_contracts[id].kind;
                    let (method, receiver) = if matches!(
                        kind,
                        crate::syntax::CallbackKind::Read | crate::syntax::CallbackKind::SharedRead
                    ) {
                        ("ReadCallback::invoke_read", format!("&value{binding}"))
                    } else if kind.is_once() {
                        ("OnceCallback::invoke_once", format!("value{binding}"))
                    } else {
                        ("Callback::invoke", format!("&mut value{binding}"))
                    };
                    format!(
                        "{{let absent_result: {} = {}; absent_result}}",
                        self.callback_result_type(
                            id,
                            expr.ty.expect("checked absent callback result")
                        ),
                        self.counted_invocation(
                            method,
                            &receiver,
                            &args,
                            expr.supplied_count.unwrap_or(CallArgumentCount::Static(
                                self.program.callback_contracts[id].params.len()
                            ))
                        )
                    )
                } else if matches!(
                    self.program.callback_contracts[id].kind,
                    crate::syntax::CallbackKind::Read | crate::syntax::CallbackKind::SharedRead
                ) {
                    let receiver = if fun.bindings[binding].initializer.is_some()
                        || (!fun.bindings[binding].parameter
                            && !fun.bindings[binding].capture_reference)
                        || matches!(fun.bindings[binding].mode, Mode::Consume | Mode::Copy)
                    {
                        format!("&value{binding}")
                    } else {
                        format!("value{binding}")
                    };
                    self.counted_invocation(
                        "ReadCallback::invoke_read",
                        &receiver,
                        &args,
                        expr.supplied_count.unwrap_or(CallArgumentCount::Static(
                            self.program.callback_contracts[id].params.len(),
                        )),
                    )
                } else if self.program.callback_contracts[id].kind.is_once() {
                    self.counted_invocation(
                        "OnceCallback::invoke_once",
                        &format!("value{binding}"),
                        &args,
                        expr.supplied_count.unwrap_or(CallArgumentCount::Static(
                            self.program.callback_contracts[id].params.len(),
                        )),
                    )
                } else {
                    let receiver = if fun.bindings[binding].initializer.is_some()
                        || (!fun.bindings[binding].parameter
                            && !fun.bindings[binding].capture_reference)
                        || matches!(fun.bindings[binding].mode, Mode::Consume | Mode::Copy)
                    {
                        format!("&mut value{binding}")
                    } else {
                        format!("value{binding}")
                    };
                    self.counted_invocation(
                        "Callback::invoke",
                        &receiver,
                        &args,
                        expr.supplied_count.unwrap_or(CallArgumentCount::Static(
                            self.program.callback_contracts[id].params.len(),
                        )),
                    )
                };
                self.registered_callback_call(call, id, expr.span)
            }
            _ => unreachable!("scheduled callable target"),
        };
        debug_assert!(rendered.next().is_none());
        Self::access_wrapped(expr, mode, code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deep_checked_call_arguments_use_one_emission_stack() {
        // Phase isolation: the parser still has migration debt. This does not
        // establish source acceptance or a supported maximum nesting depth.
        for (setup, call) in [
            ("", "identity"),
            ("const action=(value:int):int=>{return value;};", "action"),
            ("", "((value:int):int=>{return value;})"),
            (
                "const holder:Holder={run:(value:int):int=>{return value;}};",
                "holder.run",
            ),
        ] {
            let source = format!(
                "type Holder={{run:ReadOnly<Function<int,[value:int]>>;}}; function identity(value:int):int{{return value;}} function probe():int{{{setup} return {call}(987654321);}} function main(){{console.log(probe());}}"
            );
            let checked = crate::check(&source).unwrap();
            let fun = checked
                .program
                .functions
                .iter()
                .find(|fun| fun.name == "probe")
                .unwrap();
            let StmtKind::Return(Some(template)) = &fun.body.last().unwrap().kind else {
                panic!("call result");
            };
            let emitter = super::super::emitter_for(&checked.program, "");
            let shallow = emitter.expr(fun, template, Mode::Copy);
            let ExprKind::Call(_, args) = &template.kind else {
                panic!("checked call");
            };
            let leaf = emitter.expr(fun, &args[0], Mode::Copy);
            let (prefix, suffix) = shallow.split_once(&leaf).expect("literal emission");
            let mut nested = args[0].clone();
            for _ in 0..4096 {
                let mut parent = template.clone();
                let ExprKind::Call(_, args) = &mut parent.kind else {
                    unreachable!()
                };
                args[0] = nested;
                nested = parent;
            }
            let emitted = emitter.expr(fun, &nested, Mode::Copy);
            assert!(
                emitted == format!("{}{leaf}{}", prefix.repeat(4096), suffix.repeat(4096)),
                "{call}: nested output differs"
            );
            assert_eq!(emitter.expression_depth.get(), 0);
            assert!(emitter.conditional_temporaries.borrow().is_empty());
            drop(nested);
        }
    }
}
