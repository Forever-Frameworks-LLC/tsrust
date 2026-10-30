use super::*;

struct Boundary {
    depth: usize,
    first_temporary: usize,
}

enum CallState<'p> {
    Spread,
    CollectionConstructor,
    BuiltinReceiver,
    ObjectSpread { snapshots:Vec<Option<String>> },
    Sequence { discards: Vec<bool>, first_temporary: usize },
    Assignment { discards: Vec<bool>, first_temporary: usize },
    Logical { plan: super::logical_values_emit::LogicalPlan, temporary: Option<usize> },
    Selected,
    Assertion { widen: bool },
    Coalesce { plan: super::coalesce_work::Plan, temporary: Option<usize>, pattern_slot: Option<usize> },
    Conditional { infer: bool, temporary: Option<usize> },
    BorrowedInjection { union: UnionId, infer: bool, temporary: Option<usize> },
    Unary { temporary: Option<usize> },
    Binary,
    NullableParameterField { method: String },
    NamedAccess { discards: Vec<bool>, first_temporary: usize },
    TemporaryAccess {
        borrowed: bool,
        fixed: Option<crate::fixed_storage::Shape>,
    },
    Array,
    Record,
    Template {
        first_temporary: usize,
    },
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
    entered: bool,
    carrier_spine: bool,
    call: Option<CallFrame<'a, 'p>>,
}

impl<'a, 'p> Frame<'a, 'p> {
    fn new(expr: &'a Expr, mode: Mode) -> Self {
        Self {
            expr,
            mode,
            boundary: None,
            entered: false,
            carrier_spine: false,
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
        let mut completed: Option<String> = None;
        loop {
            let frame = work.last_mut().expect("active expression");
            if let Some(mut code) = completed.take() {
                let call = frame.call.as_mut().expect("suspended expression");
                if let CallState::Template { first_temporary } = &call.state {
                    let declarations = self
                        .conditional_temporaries
                        .borrow_mut()
                        .drain(*first_temporary..)
                        .collect::<Vec<_>>()
                        .join(" ");
                    let part = call.children[call.rendered.len()].0;
                    code = self.finish_template_append(part, &code, &declarations);
                }
                if let CallState::Sequence { discards, first_temporary } | CallState::NamedAccess { discards, first_temporary } | CallState::Assignment { discards, first_temporary } = &call.state
                    && discards[call.rendered.len()]
                {
                    let declarations = self.conditional_temporaries.borrow_mut().drain(*first_temporary..).collect::<Vec<_>>().join(" ");
                    code = format!("{{{declarations} let _={code};}}");
                }
                if let CallState::Logical { temporary, .. } = &call.state {
                    let (child, mode) = call.children[call.rendered.len()];
                    code = self.finish_conditional_arm_work(child, mode, *temporary, code);
                }
                if let CallState::Coalesce { plan, temporary, pattern_slot } = &call.state {
                    let index = call.rendered.len();
                    let child = call.children[index].0;
                    let mode = if index == 0 { plan.left_mode } else if pattern_slot.is_some() { Mode::Consume } else { plan.right_mode };
                    code = self.finish_conditional_arm_work(child, mode, *temporary, code);
                    if index == 1 { code = self.finish_pattern_fallback_work(child, *pattern_slot, code); }
                }
                if let CallState::Conditional { temporary, .. } = &call.state
                    && !call.rendered.is_empty() {
                    code = self.finish_conditional_arm_work(call.children[call.rendered.len()].0, frame.mode, *temporary, code);
                }
                call.rendered.push(code);
            }
            if !frame.entered {
                frame.entered = true;
                // Sequence has its own temporary policy and precedes copy adaptation.
                if let ExprKind::Sequence(values) = &frame.expr.kind {
                    let discarded = frame.expr.ty == Some(Type::Void);
                    let discards = (0..values.len()).map(|index| discarded || index + 1 < values.len()).collect::<Vec<_>>();
                    let children = values.iter().zip(&discards).map(|(value, discard)| (value, if *discard { Mode::Read } else { frame.mode })).collect();
                    frame.call = Some(CallFrame { children, rendered: Vec::new(), invocation: None,
                        state: CallState::Sequence { discards, first_temporary: 0 } });
                } else {
                if !frame.carrier_spine {
                    if frame.expr.ty.is_some_and(Type::is_copy) { frame.mode = Mode::Copy; }
                    let depth = self.expression_depth.get();
                    frame.boundary = Some(Boundary { depth, first_temporary: self.conditional_temporaries.borrow().len() });
                    self.expression_depth.set(depth + 1);
                }
                frame.call = self.prepare_callable_work(fun, frame.expr, frame.mode, frame.carrier_spine);
                }
            }

            let code = if let Some(call) = &mut frame.call {
                if let Some(&(child, mode)) = call.children.get(call.rendered.len()) {
                    if let CallState::Template { first_temporary } = &mut call.state {
                        *first_temporary = self.conditional_temporaries.borrow().len();
                    }
                    if let CallState::Sequence { first_temporary, .. } | CallState::NamedAccess { first_temporary, .. } | CallState::Assignment { first_temporary, .. } = &mut call.state {
                        *first_temporary = self.conditional_temporaries.borrow().len();
                    }
                    let mut child_frame = Frame::new(child, mode);
                    if let CallState::Logical { temporary, .. } = &mut call.state {
                        *temporary = None;
                        if !child.ty.unwrap().is_copy() {
                            let (access, pending) = self.prepare_conditional_arm_work(fun, child, mode);
                            child_frame.mode = access;
                            *temporary = pending;
                        }
                    }
                    if let CallState::Coalesce { plan, temporary, pattern_slot } = &mut call.state {
                        let index = call.rendered.len();
                        *temporary = None;
                        let mode = if index == 1 && plan.undefined_only {
                            let (access, slot) = self.prepare_pattern_fallback_work(child, mode);
                            *pattern_slot = slot;
                            access
                        } else { mode };
                        if index != 0 || !plan.absent_only {
                            let (access, pending) = self.prepare_conditional_arm_work(fun, child, mode);
                            child_frame.mode = access;
                            *temporary = pending;
                        }
                    }
                    if let CallState::Conditional { infer, temporary } = &mut call.state
                        && !call.rendered.is_empty() {
                        let spine = *infer && (matches!(child.kind, ExprKind::Conditional { .. })
                            || matches!(child.kind, ExprKind::Inject { .. }) && matches!(child.ty, Some(Type::BorrowedUnion(_))));
                        if spine {
                            *temporary = None;
                            child_frame.carrier_spine = true;
                        } else {
                            let (access, pending) = self.prepare_conditional_arm_work(fun, child, mode);
                            child_frame.mode = access;
                            *temporary = pending;
                        }
                    }
                    work.push(child_frame);
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
            let code = if let Some(boundary) = frame.boundary.take() {
                self.finish_expression_work(fun, frame.expr, frame.mode, boundary.depth, boundary.first_temporary, code)
            } else { code };
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
        force_carrier: bool,
    ) -> Option<CallFrame<'a, 'p>> {
        if let ExprKind::SpreadCall(plan)=&expr.kind {
            return Some(CallFrame{children:self.spread_expression_children(plan,mode),rendered:Vec::new(),invocation:None,state:CallState::Spread});
        }
        if let Some(receiver)=Self::builtin_chain_receiver(expr){
            return Some(CallFrame{children:vec![(receiver,Mode::Read)],rendered:Vec::new(),invocation:None,state:CallState::BuiltinReceiver});
        }
        if let ExprKind::SetConstructor(arguments,_)|ExprKind::MapConstructor(arguments,_)=&expr.kind{
            return Some(CallFrame{children:arguments.iter().map(|source|(source,Mode::Read)).collect(),rendered:Vec::new(),invocation:None,state:CallState::CollectionConstructor});
        }
        if matches!(expr.target,Some(CallTarget::RecordSpread(_))){
            let ExprKind::Call(_,args)=&expr.kind else{unreachable!("object spread call")};
            let mut children=Vec::new();let mut snapshots=Vec::new();
            for arg in args{
                if let ExprKind::Unary{op:UnaryOp::Spread,value,..}=&arg.kind{
                    let snapshot=if arg.moved_spread {None}else{self.record_parameter_fields_clone(fun,value)};
                    if snapshot.is_none(){children.push((value.as_ref(),if arg.moved_spread {Mode::Consume}else{Mode::Read}));}
                    snapshots.push(snapshot);
                }else{children.push((arg,Mode::Consume));snapshots.push(None);}
            }
            return Some(CallFrame{children,rendered:Vec::new(),invocation:None,state:CallState::ObjectSpread{snapshots}});
        }
        let assignment = match &expr.kind {
            ExprKind::Assign { target, value, .. } => Some((target, Some((value, if expr.ty == Some(Type::Void) { Mode::Consume } else { Mode::Copy })))),
            ExprKind::LogicalAssign { target, value, .. } => Some((target, Some((value, Mode::Copy)))),
            ExprKind::CompoundAssign { target, value, .. } => Some((target, Some((value, if target.ty == Some(Type::String) { Mode::Read } else { Mode::Copy })))),
            ExprKind::CoalesceAssign { target, value, branch, .. } => Some((target, if *branch == crate::syntax::NullishBranch::Present { None } else { Some((value, Mode::Consume)) })),
            ExprKind::Update { target, .. } => Some((target, None)),
            _ => None,
        };
        if let Some((target, value)) = assignment {
            let (_, steps) = self.storage_steps(target);
            let mut children = Vec::new();
            let mut discards = Vec::new();
            for step in steps {
                match step {
                    StorageStep::Index(index, _) => { children.push((index, Mode::Copy)); discards.push(false); }
                    StorageStep::Discard(prefix) => { children.push((prefix, Mode::Read)); discards.push(true); }
                    _ => (),
                }
            }
            if let Some((value, mode)) = value { children.push((value.as_ref(), mode)); discards.push(false); }
            return Some(CallFrame { children, rendered: Vec::new(), invocation: None, state: CallState::Assignment { discards, first_temporary: 0 } });
        }
        if let ExprKind::Binary { op, left, right, .. } = &expr.kind
            && matches!(op, BinaryOp::And | BinaryOp::Or)
            && !(left.ty == Some(Type::Boolean) && right.ty == Some(Type::Boolean)) {
            let plan = self.prepare_logical_emission(fun, expr, *op, left, right, mode);
            let mut children = vec![(left.as_ref(), plan.left_mode)];
            if !plan.skip_right { children.push((right.as_ref(), plan.right_mode)); }
            return Some(CallFrame { children, rendered: Vec::new(), invocation: None, state: CallState::Logical { plan, temporary: None } });
        }
        if let ExprKind::Unary { op: UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected | UnaryOp::RestPack, value, .. } = &expr.kind {
            return Some(CallFrame { children: vec![(value, mode)], rendered: Vec::new(), invocation: None, state: CallState::Selected });
        }
        if let ExprKind::Binary { op: BinaryOp::Coalesce | BinaryOp::PatternDefault, left, right, .. } = &expr.kind {
            let (plan, left) = self.prepare_coalesce_emission(expr, left, mode);
            return Some(CallFrame { children: vec![(left, plan.left_mode), (right, plan.right_mode)],
                rendered: Vec::new(), invocation: None, state: CallState::Coalesce { plan, temporary: None, pattern_slot: None } });
        }
        if let ExprKind::Conditional { predicate, consequent, alternate } = &expr.kind {
            let infer = force_carrier || (matches!(expr.ty, Some(Type::BorrowedUnion(_)))
                && self.callback_payload(expr.ty.unwrap()).is_some() && !Self::absent_callable_initializer(expr));
            return Some(CallFrame { children: vec![(predicate, Mode::Copy), (consequent, mode), (alternate, mode)],
                rendered: Vec::new(), invocation: None, state: CallState::Conditional { infer, temporary: None } });
        }
        if let ExprKind::Inject { value, union } = &expr.kind && matches!(expr.ty, Some(Type::BorrowedUnion(_))) {
            let (access, temporary) = if force_carrier && !value.ty.unwrap().is_copy() {
                self.prepare_conditional_arm_work(fun, value, Mode::Read)
            } else { (Mode::Read, None) };
            return Some(CallFrame { children: vec![(value, access)], rendered: Vec::new(), invocation: None,
                state: CallState::BorrowedInjection { union: *union, infer: force_carrier, temporary } });
        }
        if let Some((root, method)) = self.nullable_record_parameter_field_work(fun, expr, mode) {
            return Some(CallFrame {
                children: vec![(root, Mode::Read)],
                rendered: Vec::new(),
                invocation: None,
                state: CallState::NullableParameterField { method },
            });
        }
        if let ExprKind::TypeAssertion { value, satisfies, .. } = &expr.kind {
            return Some(CallFrame { children: vec![(value, mode)], rendered: Vec::new(), invocation: None,
                state: CallState::Assertion { widen: !satisfies && value.ty == Some(Type::Int) && expr.ty == Some(Type::Number) } });
        }
        if let Some(access) = self.prepare_storage_access_work(fun, expr) {
            return Some(access);
        }
        if let ExprKind::Unary { op, value, .. } = &expr.kind
            && matches!(op, UnaryOp::Typeof | UnaryOp::Void | UnaryOp::VariantIs(_) | UnaryOp::Not | UnaryOp::Positive | UnaryOp::Negate | UnaryOp::BitNot)
        {
            let unresolved = matches!(op, UnaryOp::Typeof | UnaryOp::Void)
                && matches!(&value.kind, ExprKind::Name(_) if value.binding.is_none() && value.ty == Some(Type::Undefined));
            let logical = *op == UnaryOp::Not && value.ty.is_some_and(|ty| !matches!(ty, Type::Boolean | Type::Null | Type::Undefined));
            let conditional = (*op == UnaryOp::Typeof && value.ty.and_then(Type::union).is_some())
                || (logical && !value.ty.unwrap().is_copy());
            let (child_mode, temporary) = if conditional { self.prepare_conditional_arm_work(fun, value, Mode::Read) }
                else { (if matches!(op, UnaryOp::Typeof | UnaryOp::Void | UnaryOp::VariantIs(_)) { Mode::Read } else { Mode::Copy }, None) };
            let children = if unresolved { Vec::new() } else { vec![(value.as_ref(), child_mode)] };
            return Some(CallFrame { children, rendered: Vec::new(), invocation: None, state: CallState::Unary { temporary } });
        }
        if let ExprKind::Binary { op, left, right, .. } = &expr.kind
            && !matches!(op, BinaryOp::Coalesce | BinaryOp::PatternDefault)
            && (!matches!(op, BinaryOp::And | BinaryOp::Or) || (left.ty == Some(Type::Boolean) && right.ty == Some(Type::Boolean)))
        {
            let comparison = matches!(op, BinaryOp::Equal | BinaryOp::NotEqual)
                && (matches!(left.ty, Some(Type::Null | Type::Undefined))
                    || matches!(right.ty, Some(Type::Null | Type::Undefined))
                    || left.ty.and_then(Type::union).is_some() || right.ty.and_then(Type::union).is_some());
            let mode = if comparison || left.ty.map(Type::scalar_base) == Some(Type::String) { Mode::Read } else { Mode::Copy };
            // A chain of string additions renders its operands, not the inner `+` nodes.
            if let Some(operands) = Self::string_addition_chain(expr) {
                let children = operands.into_iter().map(Self::chain_operand).collect();
                return Some(CallFrame { children, rendered: Vec::new(), invocation: None, state: CallState::Binary });
            }
            // A numeric `String(n)` appended to text renders only its argument.
            let right = match Self::appended_number(right).filter(|_| *op == BinaryOp::Add && mode == Mode::Read && !comparison) {
                Some(argument) => (argument, Mode::Copy),
                None => (right.as_ref(), mode),
            };
            return Some(CallFrame { children: vec![(left, mode), right], rendered: Vec::new(), invocation: None, state: CallState::Binary });
        }
        let structural = match &expr.kind {
            ExprKind::Array(values) | ExprKind::ArrayOf(values, _) => Some((
                CallState::Array,
                values.iter().map(|value| (value, Mode::Consume)).collect(),
            )),
            ExprKind::Record(fields) => Some((
                CallState::Record,
                fields
                    .iter()
                    .map(|(_, value)| (value, Mode::Consume))
                    .collect(),
            )),
            ExprKind::Template(parts) => Some((
                CallState::Template { first_temporary: 0 },
                parts
                    .iter()
                    .skip(1)
                    .step_by(2)
                    .map(|part| (part, Mode::Read))
                    .collect(),
            )),
            _ => None,
        };
        if let Some((state, children)) = structural {
            return Some(CallFrame {
                children,
                rendered: Vec::new(),
                invocation: None,
                state,
            });
        }
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
        if let CallState::ObjectSpread{snapshots}=state{
            let Some(CallTarget::RecordSpread(result))=expr.target else{unreachable!("object spread target")};
            let ExprKind::Call(_,args)=&expr.kind else{unreachable!("object spread call")};
            let mut rendered=rendered.into_iter();
            let copied=args.iter().zip(snapshots).map(|(arg,snapshot)|{
                if let Some(snapshot)=snapshot{return snapshot;}
                let code=rendered.next().expect("completed object spread source");
                if let ExprKind::Unary{op:UnaryOp::Spread,value,..}=&arg.kind && !arg.moved_spread {self.finish_clone_argument(fun,value,code)}else{code}
            }).collect();
            debug_assert!(rendered.next().is_none());
            return Self::access_wrapped(expr,mode,self.object_spread_value_with(result,args,copied));
        }
        if matches!(state,CallState::BuiltinReceiver){
            let receiver=Self::builtin_chain_receiver(expr).expect("builtin chain receiver");
            let code=rendered.into_iter().next().expect("rendered builtin receiver");
            self.rendered_receivers.borrow_mut().push((receiver as *const Expr as usize,code));
            let code=self.expr_dispatch(fun,expr,mode);
            let mut rendered=self.rendered_receivers.borrow_mut();
            debug_assert!(rendered.last().is_none_or(|(address,_)|*address!=receiver as *const Expr as usize),"builtin receiver rendered twice");
            if rendered.last().is_some_and(|(address,_)|*address==receiver as *const Expr as usize){rendered.pop();}
            return code;
        }
        if matches!(state,CallState::CollectionConstructor){
            let source=rendered.into_iter().next();
            return if matches!(expr.kind,ExprKind::SetConstructor(_,_)){self.set_constructor_expression(expr,mode,source)}else{self.map_constructor_expression(expr,mode,source)};
        }
        if matches!(state,CallState::Spread){
            let ExprKind::SpreadCall(plan)=&expr.kind else{unreachable!("spread emission frame")};
            let mut rendered=rendered.into_iter();
            let code=self.spread_call_expression_with(plan,mode,&mut |_,_|rendered.next().expect("completed spread operand"));
            debug_assert!(rendered.next().is_none());return code;
        }
        if matches!(
            state,
            CallState::Sequence { .. } | CallState::Assignment { .. } | CallState::Logical { .. } | CallState::Selected | CallState::Assertion { .. } | CallState::Coalesce { .. } | CallState::Conditional { .. } | CallState::BorrowedInjection { .. } | CallState::Unary { .. } | CallState::Binary | CallState::Array | CallState::Record | CallState::Template { .. } | CallState::TemporaryAccess { .. } | CallState::NamedAccess { .. } | CallState::NullableParameterField { .. }
        ) {
            return self.finish_structural_work(fun, expr, mode, state, rendered);
        }
        let ExprKind::Call(callee, args) = &expr.kind else {
            unreachable!("scheduled call");
        };
        let mut rendered = rendered.into_iter();
        let code = match state {
            CallState::Spread|CallState::CollectionConstructor|CallState::BuiltinReceiver|CallState::ObjectSpread{..}=>unreachable!("spread emission handled before ordinary assembly"),
            CallState::Sequence { .. } | CallState::Assignment { .. } | CallState::Logical { .. } | CallState::Selected | CallState::Assertion { .. } | CallState::Coalesce { .. } | CallState::Conditional { .. } | CallState::BorrowedInjection { .. } | CallState::Unary { .. } | CallState::Binary | CallState::Array | CallState::Record | CallState::Template { .. } | CallState::TemporaryAccess { .. } | CallState::NamedAccess { .. } | CallState::NullableParameterField { .. } => {
                unreachable!("structural completion handled above")
            }
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

    fn prepare_storage_access_work<'a>(
        &self,
        fun: &CheckedFunction,
        expr: &'a Expr,
    ) -> Option<CallFrame<'a, 'p>> {
        let (base, index) = match &expr.kind {
            ExprKind::Field(base, name) => {
                // These property forms precede storage selection in dispatch.
                if matches!(&base.kind, ExprKind::Name(namespace) if NumericConstant::lookup(namespace, name).is_some())
                    || matches!(base.ty, Some(Type::Union(_)))
                    || (name == "size" && (matches!(base.ty, Some(Type::Set(_) | Type::Map(_)))
                        || matches!(base.ty, Some(Type::Record(id)) if self.program.records[id].native_element().is_some())))
                    || (name == "length" && (base.ty.map(Type::scalar_base) == Some(Type::String)
                        || matches!(base.ty, Some(Type::Array(_)))
                        || matches!(base.ty, Some(Type::Record(id)) if self.program.records[id].is_tuple())))
                {
                    return None;
                }
                (base.as_ref(), None)
            }
            ExprKind::Index(base, index) => {
                if base.ty.map(Type::scalar_base) == Some(Type::String)
                    || self.expression_shape(base) == Some(crate::fixed_storage::Shape::LiteralStrings)
                {
                    return None;
                }
                (base.as_ref(), Some(index.as_ref()))
            }
            _ => return None,
        };
        if !Self::temporary_storage(expr) {
            let (_, steps) = self.storage_steps(expr);
            let mut children = Vec::new();
            let mut discards = Vec::new();
            for step in steps {
                match step {
                    StorageStep::Index(index, _) => {
                        children.push((index, Mode::Copy));
                        discards.push(false);
                    }
                    StorageStep::Discard(prefix) => {
                        children.push((prefix, Mode::Read));
                        discards.push(true);
                    }
                    _ => (),
                }
            }
            return Some(CallFrame {
                children,
                rendered: Vec::new(),
                invocation: None,
                state: CallState::NamedAccess { discards, first_temporary: 0 },
            });
        }
        let borrowed = Self::borrowed_place_expression(fun, base);
        let fixed = if index.is_some() && !borrowed {
            self.fixed.temporary_shape(self.program, fun, base).cloned()
        } else {
            None
        };
        let mut children = Vec::new();
        if fixed.is_some() {
            // Temporary fixed storage is proved only for scalar array literals.
            // Its elements have expression boundaries; the literal itself does not.
            let mut literal = base;
            while let ExprKind::Unary { op: UnaryOp::OptionalSelected, value, .. } = &literal.kind {
                literal = value;
            }
            let ExprKind::Array(items) = &literal.kind else {
                unreachable!("proved temporary scalar literal");
            };
            children.extend(items.iter().map(|item| (item, Mode::Consume)));
        } else {
            children.push((base, if borrowed { Mode::Read } else { Mode::Consume }));
        }
        if let Some(index) = index {
            children.push((index, Mode::Copy));
        }
        Some(CallFrame {
            children,
            rendered: Vec::new(),
            invocation: None,
            state: CallState::TemporaryAccess { borrowed, fixed },
        })
    }

    fn finish_structural_work(
        &self,
        fun: &CheckedFunction,
        expr: &Expr,
        mode: Mode,
        state: CallState<'_>,
        rendered: Vec<String>,
    ) -> String {
        let code = match state {
            CallState::Spread|CallState::CollectionConstructor|CallState::BuiltinReceiver|CallState::ObjectSpread{..}=>unreachable!("spread emission handled before ordinary assembly"),
            CallState::Sequence { discards, .. } => {
                let mut code = String::from("{");
                for (value, discard) in rendered.into_iter().zip(discards) {
                    code.push_str(&value);
                    if !discard { code.push(' '); }
                }
                code.push('}');
                return code;
            }
            CallState::Assignment { .. } => {
                let mut values = rendered.into_iter();
                let mut render = |_: &Expr, _: Mode, _: bool| values.next().expect("scheduled assignment operand");
                let code = match &expr.kind {
                    ExprKind::Assign { target, value, .. } if expr.ty == Some(Type::Void) => {
                        if matches!(target.kind, ExprKind::Name(_)) {
                            let id = target.binding.expect("checked assignment binding");
                            let assigned = render(value, Mode::Consume, false);
                            if self.restored_capture(fun, id) { format!("{{{}value{id}=Some({assigned});}}", if fun.bindings[id].capture_reference { "*" } else { "" }) }
                            else if let Some(store) = self.owned_record_assignment(fun, id, &assigned) { format!("{{{store}}}") }
                            else { format!("{{{}value{id}={assigned};}}", if fun.bindings[id].capture_reference { "*" } else { "" }) }
                        } else { self.storage_assignment_with(fun, target, value, &mut render) }
                    }
                    ExprKind::Assign { target, value, .. } => self.scalar_assignment_with(fun, target, value, &mut render),
                    ExprKind::LogicalAssign { target, value, and, .. } => self.logical_assignment_with(fun, target, value, *and, &mut render),
                    ExprKind::CompoundAssign { target, value, op, operator } => self.compound_assignment_with(fun, target, value, *op, *operator, &mut render),
                    ExprKind::CoalesceAssign { target, value, branch, .. } => self.coalesce_assignment_with(fun, expr, target, value, *branch, &mut render),
                    ExprKind::Update { target, increment, prefix, operator } => self.update_expression_with(fun, target, *increment, *prefix, *operator, &mut render),
                    _ => unreachable!("assignment work"),
                };
                debug_assert!(values.next().is_none());
                return code;
            }
            CallState::Logical { plan, .. } => {
                let mut values = rendered.into_iter();
                return self.finish_logical_emission(plan, values.next().expect("logical left"), values.next());
            }
            CallState::Selected => return rendered.into_iter().next().expect("selected child"),
            CallState::Assertion { widen } => {
                let code = rendered.into_iter().next().expect("assertion operand");
                return if widen { format!("({code}) as f64") } else { code };
            },
            CallState::Coalesce { plan, .. } => {
                let ExprKind::Binary { right, .. } = &expr.kind else { unreachable!("coalescing operands"); };
                let mut values = rendered.into_iter();
                return self.finish_coalesce_emission(plan, right, values.next().unwrap(), values.next().unwrap());
            }
            CallState::Conditional { .. } => {
                let mut rendered = rendered.into_iter();
                let condition = rendered.next().expect("predicate");
                let then_value = rendered.next().expect("consequent");
                let else_value = rendered.next().expect("alternate");
                return format!("(if {condition} {{ {then_value} }} else {{ {else_value} }})");
            }
            CallState::BorrowedInjection { union, infer, temporary } => {
                let ExprKind::Inject { value, .. } = &expr.kind else { unreachable!("view injection"); };
                let source = self.finish_conditional_arm_work(value, Mode::Read, temporary, rendered.into_iter().next().expect("view source"));
                return self.finish_borrowed_union_injection(value, union, infer, source);
            }

            CallState::Unary { temporary } => {
                let ExprKind::Unary { op, value, .. } = &expr.kind else { unreachable!("unary work"); };
                let mut rendered = rendered.into_iter().next().map(|code| self.finish_conditional_arm_work(value, Mode::Read, temporary, code));
                if *op == UnaryOp::BitNot && value.ty == Some(Type::Number) {
                    let code = rendered.take().expect("bitwise operand");
                    return format!("{{ let number_value: f64 = {code}; (!tsr_math_uint32(number_value) as i32) as f64 }}");
                }
                if *op == UnaryOp::Not && value.ty.is_some_and(|ty| !matches!(ty, Type::Boolean | Type::Null | Type::Undefined)) {
                    return self.finish_logical_not(value, rendered.take().expect("truthiness operand"));
                }
                return match self.unary_expression_code_with(expr, mode, &mut |_, _, _| rendered.take().expect("completed unary operand")) {
                    Ok(code) => Self::access_wrapped(expr, mode, code),
                    Err(complete) => complete,
                };
            }

            CallState::Binary => {
                let ExprKind::Binary { op, operator, left, .. } = &expr.kind else { unreachable!("binary work"); };
                let mut rendered = rendered.into_iter();
                if op.bitwise_or_power() {
                    let first = rendered.next().expect("left operand");
                    let second = rendered.next().expect("right operand");
                    return if left.ty == Some(Type::Int) { self.int_bitwise_or_power(*op, &first, &second, *operator) }
                        else { Self::number_operation(*op, &first, &second) };
                }
                return match self.binary_expression_code_with(expr, &mut |_, _| rendered.next().expect("completed binary operand")) {
                    Ok(code) => Self::access_wrapped(expr, mode, code),
                    Err(complete) => complete,
                };
            }

            CallState::NullableParameterField { method } => {
                return Self::finish_nullable_record_parameter_field(expr, &method, rendered.into_iter().next().expect("completed nullable receiver"));
            }

            CallState::NamedAccess { .. } => {
                let mut rendered = rendered.into_iter();
                let code = self.storage_access_with(fun, expr, mode, &mut |_, _, _| rendered.next().expect("completed storage operand"));
                debug_assert!(rendered.next().is_none());
                return code;
            }

            CallState::TemporaryAccess { borrowed, fixed } => {
                let mut rendered = rendered;
                let index = if matches!(expr.kind, ExprKind::Index(..)) {
                    rendered.pop().expect("completed index")
                } else {
                    String::new()
                };
                let array = if fixed.is_some() {
                    format!("[{}]", rendered.join(", "))
                } else {
                    rendered.pop().expect("completed receiver")
                };
                return self.finish_temporary_storage_access(expr, mode, borrowed, fixed.as_ref(), array, index);
            }

            CallState::Array => {
                let (ExprKind::Array(values) | ExprKind::ArrayOf(values, _)) = &expr.kind else {
                    unreachable!("array expression");
                };
                return self.finish_array_literal_expression(
                    expr,
                    values,
                    mode,
                    rendered.join(", "),
                );
            }
            CallState::Record => {
                let ExprKind::Record(fields) = &expr.kind else {
                    unreachable!("record expression");
                };
                let Type::Record(index) = expr.ty.expect("checked record type") else {
                    unreachable!("record literal");
                };
                let values = fields
                    .iter()
                    .zip(rendered)
                    .map(|((key, _), value)| {
                        format!("field{}: {}", self.fields[index][key.text.as_str()], value)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let values = self.order_values(index, fields, values);
                format!(
                    "Record{} {{ {values} }}",
                    self.native_record_declaration(index)
                )
            }
            CallState::Template { .. } => {
                let ExprKind::Template(parts) = &expr.kind else {
                    unreachable!("template expression");
                };
                let capacity: usize = parts
                    .iter()
                    .step_by(2)
                    .map(|part| {
                        let ExprKind::String(chunk) = &part.kind else {
                            unreachable!("template chunk");
                        };
                        chunk.len()
                    })
                    .sum();
                let mut code = format!(
                    "{{ use std::fmt::Write as _; let mut template_output = std::string::String::with_capacity({capacity}); "
                );
                let mut rendered = rendered.into_iter();
                for (index, part) in parts.iter().enumerate() {
                    if index % 2 == 0 {
                        let ExprKind::String(chunk) = &part.kind else {
                            unreachable!("template chunk");
                        };
                        code.push_str(&format!("template_output.push_str({chunk:?}); "));
                    } else {
                        code.push_str(&rendered.next().expect("completed interpolation"));
                    }
                }
                code.push_str("String::Owned(template_output) }");
                code
            }
            _ => unreachable!("structural emission frame"),
        };
        match mode {
            Mode::Read => format!("&({code})"),
            Mode::Mutate => format!("&mut ({code})"),
            Mode::Consume | Mode::Copy => code,
        }
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
                if closure.program_scoped {
                    let context=if self.program.task_type(closure.function.result){self.test_factory_context()}else{"test_program".into()};
                    arguments.extend([context,"test_registry".into()]);
                }
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
    fn deep_assertions_preserve_casts_and_release_emission_boundaries() {
        let checked = crate::check("function probe():number{return 987654321 as number;}function main(){console.log(probe());}").unwrap();
        let fun = checked.program.functions.iter().find(|fun| fun.name == "probe").unwrap();
        let template = fun.body.iter().find_map(|stmt| if let StmtKind::Return(Some(expr)) = &stmt.kind { Some(expr) } else { None }).unwrap();
        let ExprKind::TypeAssertion { value, .. } = &template.kind else { panic!("assertion") };
        let emitter = super::super::emitter_for(&checked.program, "");
        let mut nested = (**value).clone();
        let leaf = emitter.expr(fun, &nested, Mode::Copy);
        for layer in 0..16_384 {
            let mut parent = template.clone();
            parent.ty = Some(Type::Int);
            let ExprKind::TypeAssertion { value, satisfies, .. } = &mut parent.kind else { unreachable!() };
            *satisfies = layer % 2 == 0;
            *value = Box::new(nested);
            nested = parent;
        }
        assert_eq!(emitter.expr(fun, &nested, Mode::Copy), leaf);
        let mut outer = template.clone();
        let ExprKind::TypeAssertion { value, .. } = &mut outer.kind else { unreachable!() };
        *value = Box::new(nested);
        assert_eq!(emitter.expr(fun, &outer, Mode::Copy), format!("({leaf}) as f64"));
        assert_eq!(emitter.expression_depth.get(), 0);
        assert!(emitter.conditional_temporaries.borrow().is_empty());
        drop(outer);
    }

    #[test]
    fn sequence_operands_share_the_emission_stack() {
        for source in [
            "function probe():int{return (0,987654321);} function main(){console.log(probe());}",
            "function probe():int{let text=\"old\";return (text=\"new\",987654321);} function main(){console.log(probe());}",
        ] {
            let checked = crate::check(source).unwrap();
            let fun = checked.program.functions.iter().find(|fun| fun.name == "probe").unwrap();
            let template = fun.body.iter().find_map(|stmt| if let StmtKind::Return(Some(expr)) = &stmt.kind { Some(expr) } else { None }).unwrap();
            let ExprKind::Sequence(values) = &template.kind else { panic!("sequence"); };
            let emitter = super::super::emitter_for(&checked.program, "");
            let mut nested = values.last().unwrap().clone();
            let leaf = emitter.expr(fun, &nested, Mode::Copy);
            let shallow = emitter.expr(fun, template, Mode::Copy);
            let (prefix, suffix) = shallow.split_once(&leaf).expect("sequence final value");
            for _ in 0..4096 {
                let mut parent = template.clone();
                let ExprKind::Sequence(values) = &mut parent.kind else { unreachable!() };
                *values.last_mut().unwrap() = nested;
                nested = parent;
            }
            assert!(emitter.expr(fun, &nested, Mode::Copy) == format!("{}{leaf}{}", prefix.repeat(4096), suffix.repeat(4096)));
            assert_eq!(emitter.expression_depth.get(), 0);
            assert!(emitter.conditional_temporaries.borrow().is_empty());
        }
    }

    #[test]
    fn assignment_operands_share_the_emission_stack() {
        for source in [
            "function probe():int{let value=0;return value=987654321;} function main(){console.log(probe());}",
            "function probe():int{let value=0;return value+=987654321;} function main(){console.log(probe());}",
            "function probe():boolean{let value=false;return value&&=true;} function main(){console.log(probe());}",
            "function probe():int{let value:int|null=null;return value??=987654321;} function main(){console.log(probe());}",
            "function probe():int{const values=[1];return ++values[987654321];} function main(){console.log(probe());}",
        ] {
            let checked = crate::check(source).unwrap();
            let fun = checked.program.functions.iter().find(|fun| fun.name == "probe").unwrap();
            let StmtKind::Return(Some(template)) = &fun.body[1].kind else { panic!("return"); };
            fn operand(expr: &mut Expr) -> &mut Expr {
                match &mut expr.kind {
                    ExprKind::Assign { value, .. } | ExprKind::CompoundAssign { value, .. }
                    | ExprKind::LogicalAssign { value, .. } | ExprKind::CoalesceAssign { value, .. } => value,
                    ExprKind::Update { target, .. } => {
                        let ExprKind::Index(_, index) = &mut target.kind else { panic!("update index"); };
                        index
                    }
                    _ => panic!("assignment expression"),
                }
            }
            let emitter = super::super::emitter_for(&checked.program, "");
            let mut parent = template.clone();
            let mut nested = operand(&mut parent).clone();
            let leaf = emitter.expr(fun, &nested, Mode::Copy);
            let shallow = emitter.expr(fun, template, Mode::Copy);
            let (prefix, suffix) = shallow.split_once(&leaf).expect("assignment operand emission");
            // Phase isolation verifies the traversal independently of parser guards.
            for _ in 0..4096 {
                let mut parent = template.clone();
                *operand(&mut parent) = nested;
                nested = parent;
            }
            assert!(emitter.expr(fun, &nested, Mode::Copy) == format!("{}{leaf}{}", prefix.repeat(4096), suffix.repeat(4096)));
            assert_eq!(emitter.expression_depth.get(), 0);
            assert!(emitter.conditional_temporaries.borrow().is_empty());
        }
    }

    #[test]
    fn logical_operands_share_the_emission_stack() {
        for source in [
            "function probe():int{return null||987654321;} function main(){console.log(probe());}",
            "function probe():int{return 0||987654321;} function main(){console.log(probe());}",
            "function probe(value:boolean):boolean{return true&&value;} function main(){console.log(probe(false));}",
        ] {
            let checked = crate::check(source).unwrap();
            let fun = checked.program.functions.iter().find(|fun| fun.name == "probe").unwrap();
            let StmtKind::Return(Some(template)) = &fun.body[0].kind else { panic!("return"); };
            let ExprKind::Binary { right, .. } = &template.kind else { panic!("logical expression"); };
            let emitter = super::super::emitter_for(&checked.program, "");
            let mut nested = right.as_ref().clone();
            let leaf = emitter.expr(fun, &nested, Mode::Copy);
            let shallow = emitter.expr(fun, template, Mode::Copy);
            let (prefix, suffix) = shallow.split_once(&leaf).expect("logical operand emission");
            // Phase-isolated witness, not a source-language depth policy.
            for _ in 0..4096 {
                let mut parent = template.clone();
                let ExprKind::Binary { right, .. } = &mut parent.kind else { unreachable!() };
                **right = nested;
                nested = parent;
            }
            assert!(emitter.expr(fun, &nested, Mode::Copy) == format!("{}{leaf}{}", prefix.repeat(4096), suffix.repeat(4096)));
            assert_eq!(emitter.expression_depth.get(), 0);
            assert!(emitter.conditional_temporaries.borrow().is_empty());
        }
    }

    #[test]
    fn coalescing_fallbacks_and_selected_values_share_the_emission_stack() {
        for source in [
            "function probe():int{return null??987654321;} function main(){console.log(probe());}",
            "function probe(value:int|null):int{return value??987654321;} function main(){console.log(probe(null));}",
            "function probe():int{return 987654321??0;} function main(){console.log(probe());}",
        ] {
            let checked = crate::check(source).unwrap();
            let fun = checked.program.functions.iter().find(|fun| fun.name == "probe").unwrap();
            let StmtKind::Return(Some(template)) = &fun.body[0].kind else { panic!("return"); };
            fn payload(expr: &mut Expr) -> &mut Expr {
                match &mut expr.kind {
                    ExprKind::Binary { right, .. } => right,
                    ExprKind::Unary { op: UnaryOp::CoalesceSelected, value, .. } => value,
                    _ => panic!("coalescing expression"),
                }
            }
            let emitter = super::super::emitter_for(&checked.program, "");
            let mut parent = template.clone();
            let mut nested = payload(&mut parent).clone();
            let leaf = emitter.expr(fun, &nested, Mode::Copy);
            let shallow = emitter.expr(fun, template, Mode::Copy);
            let (prefix, suffix) = shallow.split_once(&leaf).expect("fallback emission");
            // Phase-isolated witness; no source-language depth policy.
            for _ in 0..4096 {
                let mut parent = template.clone();
                *payload(&mut parent) = nested;
                nested = parent;
            }
            assert!(emitter.expr(fun, &nested, Mode::Copy) == format!("{}{leaf}{}", prefix.repeat(4096), suffix.repeat(4096)));
            assert_eq!(emitter.expression_depth.get(), 0);
            assert!(emitter.conditional_temporaries.borrow().is_empty());
        }
    }

    #[test]
    fn conditional_arms_share_the_emission_stack() {
        let checked = crate::check("function probe():int{return true?987654321:0;} function main(){console.log(probe());}").unwrap();
        let fun = checked.program.functions.iter().find(|fun| fun.name == "probe").unwrap();
        let StmtKind::Return(Some(template)) = &fun.body[0].kind else { panic!("return"); };
        let ExprKind::Conditional { consequent, .. } = &template.kind else { panic!("conditional"); };
        let emitter = super::super::emitter_for(&checked.program, "");
        let leaf = emitter.expr(fun, consequent, Mode::Copy);
        let shallow = emitter.expr(fun, template, Mode::Copy);
        let (prefix, suffix) = shallow.split_once(&leaf).expect("arm emission");
        let mut nested = consequent.as_ref().clone();
        // Phase isolation, not a language nesting ceiling.
        for _ in 0..4096 {
            let mut parent = template.clone();
            let ExprKind::Conditional { consequent, .. } = &mut parent.kind else { unreachable!() };
            **consequent = nested;
            nested = parent;
        }
        assert!(emitter.expr(fun, &nested, Mode::Copy) == format!("{}{leaf}{}", prefix.repeat(4096), suffix.repeat(4096)));
        assert_eq!(emitter.expression_depth.get(), 0);
        assert!(emitter.conditional_temporaries.borrow().is_empty());
    }

    #[test]
    fn mixed_unary_binary_operands_share_the_emission_stack() {
        fn payload(expr: &mut Expr) -> &mut Expr {
            let ExprKind::Unary { value, .. } = &mut expr.kind else { panic!("unary"); };
            let ExprKind::Binary { left, .. } = &mut value.kind else { panic!("binary"); };
            left
        }
        let checked = crate::check("function probe():int{return -(987654321+1);} function main(){console.log(probe());}").unwrap();
        let fun = checked.program.functions.iter().find(|fun| fun.name == "probe").unwrap();
        let StmtKind::Return(Some(template)) = &fun.body[0].kind else { panic!("return"); };
        let emitter = super::super::emitter_for(&checked.program, "");
        let mut parent = template.clone();
        let mut nested = payload(&mut parent).clone();
        let leaf = emitter.expr(fun, &nested, Mode::Copy);
        let shallow = emitter.expr(fun, template, Mode::Copy);
        let (prefix, suffix) = shallow.split_once(&leaf).expect("operand emission");
        // Emission isolation; this is a witness, not a source-language ceiling.
        for _ in 0..4096 {
            let mut parent = template.clone();
            *payload(&mut parent) = nested;
            nested = parent;
        }
        assert!(emitter.expr(fun, &nested, Mode::Copy) == format!("{}{leaf}{}", prefix.repeat(4096), suffix.repeat(4096)));
        assert_eq!(emitter.expression_depth.get(), 0);
        assert!(emitter.conditional_temporaries.borrow().is_empty());
    }

    #[test]
    fn named_index_operands_share_the_emission_stack() {
        let checked = crate::check("function main(){const a=[0];console.log(a[987654321]);}").unwrap();
        let fun = checked.program.functions.iter().find(|fun| fun.name == "main").unwrap();
        let StmtKind::Expr(console) = &fun.body[1].kind else { panic!("console statement"); };
        let ExprKind::Call(_, args) = &console.kind else { panic!("console call"); };
        let template = &args[0];
        let ExprKind::Index(_, index) = &template.kind else { panic!("indexed expression"); };
        let emitter = super::super::emitter_for(&checked.program, "");
        let leaf = emitter.expr(fun, index, Mode::Copy);
        let shallow = emitter.expr(fun, template, Mode::Copy);
        let (prefix, suffix) = shallow.split_once(&leaf).expect("index emission");
        let mut nested = index.as_ref().clone();
        // A phase-isolated structural witness, not a language depth limit.
        for _ in 0..4096 {
            let mut parent = template.clone();
            let ExprKind::Index(_, index) = &mut parent.kind else { unreachable!() };
            **index = nested;
            nested = parent;
        }
        let emitted = emitter.expr(fun, &nested, Mode::Copy);
        assert!(emitted == format!("{}{leaf}{}", prefix.repeat(4096), suffix.repeat(4096)));
        assert_eq!(emitter.expression_depth.get(), 0);
        assert!(emitter.conditional_temporaries.borrow().is_empty());
    }

    #[test]
    fn mixed_aggregate_access_and_calls_share_the_emission_stack() {
        fn payload(expr: &mut Expr) -> &mut Expr {
            let ExprKind::Call(_, args) = &mut expr.kind else { panic!("call"); };
            let ExprKind::Index(base, _) = &mut args[0].kind else { panic!("index"); };
            let ExprKind::Field(base, _) = &mut base.kind else { panic!("field"); };
            let ExprKind::Record(fields) = &mut base.kind else { panic!("record"); };
            let ExprKind::Array(items) = &mut fields[0].1.kind else { panic!("array"); };
            &mut items[0]
        }
        let checked = crate::check("function identity(value:int):int{return value;} function main(){console.log(identity(({value:[987654321]}).value[0]));}").unwrap();
        let fun = checked.program.functions.iter().find(|fun| fun.name == "main").unwrap();
        let StmtKind::Expr(console) = &fun.body[0].kind else { panic!("console statement"); };
        let ExprKind::Call(_, args) = &console.kind else { panic!("console call"); };
        let template = &args[0];
        let emitter = super::super::emitter_for(&checked.program, "");
        let mut parent = template.clone();
        let mut nested = payload(&mut parent).clone();
        let leaf = emitter.expr(fun, &nested, Mode::Copy);
        let shallow = emitter.expr(fun, template, Mode::Copy);
        let (prefix, suffix) = shallow.split_once(&leaf).expect("leaf emission");
        // This isolates emission from remaining parser/checker migrations.
        // The witness size is not a source-language support ceiling.
        for _ in 0..4096 {
            let mut parent = template.clone();
            *payload(&mut parent) = nested;
            nested = parent;
        }
        let emitted = emitter.expr(fun, &nested, Mode::Copy);
        assert!(emitted == format!("{}{leaf}{}", prefix.repeat(4096), suffix.repeat(4096)));
        assert_eq!(emitter.expression_depth.get(), 0);
        assert!(emitter.conditional_temporaries.borrow().is_empty());
    }

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

impl Emitter<'_> {
    /// A builtin method whose receiver is itself a call renders that receiver
    /// as a child frame. Each listed target reads its receiver once, in read
    /// mode (copy for copyable receivers), through `expr`.
    fn builtin_chain_receiver(expr:&Expr)->Option<&Expr>{
        let ExprKind::Call(callee,_)=&expr.kind else{return None};
        let ExprKind::Field(base,_)=&callee.kind else{return None};
        if !matches!(base.kind,ExprKind::Call(..)){return None;}
        // Lowercasing a trimmed string renders the trim input directly.
        if expr.target==Some(CallTarget::Lower)&&base.target==Some(CallTarget::Trim){return None;}
        matches!(expr.target,Some(CallTarget::Trim|CallTarget::TrimStart|CallTarget::TrimEnd|CallTarget::Lower|CallTarget::Upper
            |CallTarget::StringSlice|CallTarget::StringSubstring|CallTarget::StringSubstr|CallTarget::StringCharAt
            |CallTarget::ArraySlice|CallTarget::ArrayJoin|CallTarget::IntToString|CallTarget::NumberToString|CallTarget::BooleanToString|CallTarget::StringIdentity|CallTarget::StringIsWellFormed))
            .then_some(base.as_ref())
    }
    pub(super) fn take_rendered_receiver(&self,expr:&Expr,mode:Mode)->Option<String>{
        let mut rendered=self.rendered_receivers.borrow_mut();
        let readable=mode==Mode::Read||mode==Mode::Copy&&expr.ty.is_some_and(Type::is_copy);
        (readable&&rendered.last().is_some_and(|(address,_)|*address==expr as *const Expr as usize)).then(||rendered.pop().unwrap().1)
    }
}
