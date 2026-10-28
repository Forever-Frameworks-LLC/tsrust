//! Parameter adaptation and argument loans retained across child evaluation.
use super::super::*;

enum Adaptation { Plain, MutableUnion, BorrowedUnion }

pub(super) struct ArgumentFrame {
    pub(super) mode: Mode,
    pub(super) expected: Option<Type>,
    contract: ArgumentContract,
    adaptation: Adaptation,
}

pub(super) struct Arguments {
    pub(super) params: Vec<ArgumentContract>,
    pub(super) active: std::rc::Rc<Loans>,
    next: usize,
    boundary: Option<crate::capture_loan_identity::Boundary>,
    precise: bool,
    mark_owned: bool,
}

impl Arguments {
    pub(super) fn new(params: Vec<ArgumentContract>, active: Loans, context: &Context, release_captures: bool, precise: bool, mark_owned: bool) -> Self {
        Self {
            params, active: std::rc::Rc::new(active), next: 0,
            boundary: release_captures.then(|| crate::capture_loan_identity::Boundary::capture(&context.capture_loans)),
            precise, mark_owned,
        }
    }

    pub(super) fn finish(&mut self, context: &mut Context) {
        if let Some(boundary) = self.boundary.take() { boundary.unwind(&mut context.capture_loans); }
    }
}

impl Analyzer {
    pub(in crate::analysis) fn parameter_argument(&mut self, context: &mut Context, argument: &mut Expr, contract: ArgumentContract, loans: &Loans) -> Result<Type, Diagnostic> {
        let frame = self.prepare_parameter_argument(context, argument, contract);
        let actual = self.expression(context, argument, frame.mode, loans, frame.expected)?;
        self.finish_parameter_argument(context, argument, frame, actual)
    }

    pub(super) fn prepare_parameter_argument(&self, context: &Context, argument: &Expr, contract: ArgumentContract) -> ArgumentFrame {
        let mut frame = ArgumentFrame { mode: contract.mode, expected: Some(contract.ty), contract, adaptation: Adaptation::Plain };
        // A known factory keeps its Task result while being evaluated. Adapt
        // only after evaluation, rather than contextualizing that result as void.
        if matches!(contract.ty,Type::Callback(id) if self.callables.callbacks[id].kind==CallbackKind::Program)
            && let Some(hint)=self.expression_type_hint(context,argument)
            && self.checked_generic_callable_shape(hint).is_some_and(|(params,result)|params.is_empty() && self.is_task_type(result)) {
            frame.expected=Some(hint);
        }
        if contract.mode == Mode::Mutate
            && let Type::Union(id) = contract.ty && !id.copy
            && (contract.omission.is_omittable() || self.unions.payload(contract.ty).is_none()) {
            let hint = self.expression_type_hint(context, argument);
            frame.expected = if hint.is_some_and(|ty| ty.union().is_some()) { None }
                else { hint.filter(|ty| matches!(ty, Type::Null | Type::Undefined)).or_else(|| self.unions.payload(contract.ty)) };
            frame.mode = if hint.is_some_and(|ty| matches!(ty, Type::Null | Type::Undefined)) { Mode::Copy } else { Mode::Mutate };
            frame.adaptation = Adaptation::MutableUnion;
        } else if contract.omission.is_omittable() && contract.mode == Mode::Read
            && self.nullable_scoped_record_declaration(contract.ty).is_none()
            && let Type::Union(id) = contract.ty && !id.copy
            && !matches!(argument.kind, ExprKind::Conditional { .. }) {
            let hint = self.expression_type_hint(context, argument);
            frame.expected = if hint.is_some_and(|ty| ty.union().is_some()) { None }
                else { hint.filter(|ty| matches!(ty, Type::Null | Type::Undefined)).or_else(|| self.unions.payload(contract.ty)) };
            frame.adaptation = Adaptation::BorrowedUnion;
        }
        frame
    }

    pub(super) fn finish_parameter_argument(&mut self, context:&Context, argument: &mut Expr, frame: ArgumentFrame, actual: Type) -> Result<Type, Diagnostic> {
        let actual=self.testing_callback_adapter(context,argument,actual,Some(frame.contract.ty))?;
        if let Type::Callback(id) = frame.contract.ty
            && matches!(self.callables.callbacks[id].kind, CallbackKind::Program | CallbackKind::ProgramReusable)
            && let Type::Closure(origin) = actual {
            let kind = self.callables.callbacks[id].kind;
            let site = self.closures[origin].span.start;
            self.program_callback_sites.entry(site).and_modify(|old| {
                if kind == CallbackKind::Program { *old = kind; }
            }).or_insert(kind);
        }
        match frame.adaptation {
            Adaptation::Plain => Ok(actual),
            Adaptation::MutableUnion => {
                let contract = frame.contract;
                let Type::Union(id) = contract.ty else { unreachable!("mutable parameter union") };
                self.require(actual, Some(contract.ty), argument.span)?;
                let span = argument.span;
                let value = std::mem::replace(argument, Expr::new(ExprKind::Undefined, span));
                *argument = Expr::new(ExprKind::Unary { op: UnaryOp::ParameterMutableInput, value: Box::new(value), operator: span }, span);
                let physical = if let Some(Type::Callback(callback)) = self.unions.payload(contract.ty) {
                    let payload = if matches!(actual, Type::Null | Type::Undefined) { self.absent_callback(callback) } else { self.unions.payload(actual).unwrap_or(actual) };
                    self.unions.intern(self.unions.members(id).into_iter().map(|member| if matches!(member, Type::Callback(_)) { payload } else { member }), span)?
                } else { contract.ty };
                argument.ty = Some(physical);
                Ok(physical)
            }
            Adaptation::BorrowedUnion => {
                let Type::Union(id) = frame.contract.ty else { unreachable!("borrowed parameter union") };
                self.require(actual, Some(frame.contract.ty), argument.span)?;
                argument.inject(id);
                argument.ty = Some(Type::BorrowedUnion(id));
                Ok(Type::BorrowedUnion(id))
            }
        }
    }

    pub(super) fn next_argument(&mut self, context: &Context, args: &[Expr], frame: &mut Arguments) -> Option<(usize, ArgumentFrame)> {
        let index = frame.next;
        let arg = args.get(index)?;
        let contract = *frame.params.get(index)?;
        frame.next += 1;
        if frame.mark_owned && let Type::Callback(id) = contract.ty
            && self.callables.callbacks[id].kind.is_owned()
            && let ExprKind::Closure { id, .. } = &arg.kind {
            self.escaping.insert(*id);
        }
        Some((index, self.prepare_parameter_argument(context, arg, contract)))
    }

    pub(super) fn finish_argument(&mut self, context:&Context, arg: &mut Expr, frame: &mut Arguments, argument: ArgumentFrame, actual: Type) -> Result<(), Diagnostic> {
        let contract = argument.contract;
        let actual = self.finish_parameter_argument(context, arg, argument, actual)?;
        self.require(actual, Some(contract.ty), arg.span)?;
        if !actual.is_copy() {
            // Child work has released its shared loan reference before resuming
            // this frame. Mutating the parent does not copy loans per argument.
            let active = std::rc::Rc::make_mut(&mut frame.active);
            for binding in Self::loan_roots(arg, contract.mode) {
                if frame.precise { active.argument(binding, contract.mode, arg); }
                else { active.entry(binding).or_default().push((contract.mode, arg.span)); }
            }
        }
        Ok(())
    }
}
