//! Prepare each source declaration independently of source argument access.
use super::*;

pub(super) struct Prepared {
    pub function: usize,
    pub arguments: Vec<Expr>,
    pub contracts: Vec<ArgumentContract>,
    pub accesses: Vec<named_capture_access::Access>,
    pub invocation: Mode,
}

pub(super) struct Pending { pub function: usize, pub arguments: Vec<Expr>, pub span: Span }

impl Analyzer {
    pub(super) fn same_deferred_argument_type(&self,left:Type,right:Type)->bool {
        if left==right {return true;}
        match (left,right) {
            (Type::Callback(left),Type::Callback(right))=>self.callables.callbacks[left].structural==self.callables.callbacks[right].structural,
            _=>false,
        }
    }

    /// Prepare an alternative from its own declaration after source arguments
    /// have been accessed. This must not replay their ownership effects.
    #[inline(never)]
    #[cfg(test)]
    pub(super) fn prepare_deferred_alternative_method(
        &mut self, context:&mut Context, origin:usize, arguments:Vec<Expr>,
        expected:Option<Type>, span:Span,
    )->Result<crate::polymorphic_invocation_plans::Pair,Diagnostic> {
        let prepared=self.prepare_deferred_generic_body(context,origin,arguments,expected,span)?;
        let (function,result)=self.resolved_call(context,prepared.function,&prepared.arguments,span)?;
        self.function(function)?;
        let method=self.materialize_deferred_method(origin,function,result,&prepared.arguments,&prepared.accesses,prepared.invocation,span)?;
        Ok(crate::polymorphic_invocation_plans::Pair{origin,method})
    }

    #[inline(never)]
    pub(super) fn request_deferred_generic_body(
        &mut self, context: &Context, origin: usize, arguments: Vec<Expr>,
        expected: Option<Type>, span: Span,
    ) -> Result<Pending, Diagnostic> {
        let mut source_call = Expr::new(ExprKind::Call(
            Box::new(Expr::new(ExprKind::Name(self.deferred_generic_values[&origin].declaration.clone()), span)), arguments), span);
        // Original template preparation solves constraints without evaluating
        // source arguments. It also retains omission and homogeneous rest packs.
        let preparation = self.deferred_generic_values[&origin].context.as_ref().map(|source| {
            let mut preparation = context.clone();
            let start = preparation.bindings.len();
            for binding in &source.bindings {
                let mut binding = binding.clone();
                for root in &mut binding.view_roots { *root += start; }
                preparation.bindings.push(binding);
            }
            preparation.source_bindings.extend(source.source_bindings.iter().map(|(&source, &slot)| (source, slot + start)));
            preparation
        });
        let preparation = preparation.as_ref().unwrap_or(context);
        self.prepare_generic_call(preparation, &mut source_call, expected)?;
        let ExprKind::Call(target, prepared) = source_call.into_kind() else { unreachable!() };
        let ExprKind::Name(name) = target.into_kind() else { unreachable!() };
        let function = self.function_names[&name];
        self.named_captures.prepare(function, preparation, &self.program)?;
        Ok(Pending { function, arguments: prepared, span })
    }

    pub(super) fn complete_deferred_generic_body(&mut self, pending: Pending) -> Result<Prepared, Diagnostic> {
        let Pending { function, arguments: prepared, span } = pending;
        let checked = self.checked[function].as_ref().ok_or_else(|| Self::pending_result(span))?;
        debug_assert_eq!(checked.parameter_count, self.program.functions[function].params.len());
        let contracts = checked.bindings[..checked.parameter_count].iter().map(ArgumentContract::from).collect::<Vec<_>>();
        let mut arguments = prepared;
        Self::prepare_arguments(&mut arguments, &contracts, span)?;
        let accesses = self.named_captures.access(function, Some(checked), &self.assumed_capture_access, checked.parameter_count);
        let method_invocation = accesses.iter().fold(Mode::Read, |mode, access| mode.max(access.mode));
        Ok(Prepared { function, arguments, contracts, accesses, invocation: method_invocation })
    }
    #[cfg(test)]
    pub(super) fn prepare_deferred_generic_body(&mut self, context: &Context, origin: usize, arguments: Vec<Expr>, expected: Option<Type>, span: Span) -> Result<Prepared, Diagnostic> {
        let pending = self.request_deferred_generic_body(context, origin, arguments, expected, span)?;
        self.function(pending.function)?;
        self.complete_deferred_generic_body(pending)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn alternative_declarations_keep_independent_bodies_and_arity_contracts() {
        let source = "function first<T>(value:T):T{return value;}function second<T>(value:T):int{return 10;}function pair<T>(left:T,right:T):T{return right;}function main(){}";
        let source = crate::format(source).unwrap();
        assert_eq!(crate::format(&source).unwrap(), source);
        let mut analyzer = Analyzer::new(crate::syntax::parse(&source).unwrap()).unwrap();
        let mut context = Context::new("main".into());
        let span = Span::new(400, 410);
        let first = analyzer.deferred_captured_generic_value(&mut context, "first", 1, span, &Loans::new()).unwrap();
        let second = analyzer.deferred_captured_generic_value(&mut context, "second", 2, span, &Loans::new()).unwrap();
        let pair = analyzer.deferred_captured_generic_value(&mut context, "pair", 3, span, &Loans::new()).unwrap();
        let argument = Expr::checked(ExprKind::Number(7.0), span, Some(Type::Int));
        let origin = |ty| if let Type::Closure(origin) = ty { origin } else { unreachable!() };
        let one = analyzer.prepare_deferred_generic_body(&context, origin(first), vec![argument.clone()], None, span).unwrap();
        let two = analyzer.prepare_deferred_generic_body(&context, origin(second), vec![argument.clone()], None, span).unwrap();
        assert_ne!(one.function, two.function, "different declarations must not reuse one checked body");
        assert_eq!(analyzer.checked[one.function].as_ref().unwrap().result, Type::Int);
        assert_eq!(analyzer.checked[two.function].as_ref().unwrap().result, Type::Int);
        assert_eq!(one.contracts[0].ty, Type::Int);
        assert_eq!(two.contracts[0].ty, Type::Int);
        assert_eq!(one.arguments[0].ty, argument.ty);
        assert_eq!(two.arguments[0].ty, argument.ty);
        let first_method = analyzer.materialize_deferred_method(origin(first), one.function, Type::Int, &one.arguments, &one.accesses, one.invocation, span).unwrap();
        let second_pair = analyzer.prepare_deferred_alternative_method(&mut context,origin(second),vec![argument.clone()],None,span).unwrap();
        let second_method = second_pair.method;
        assert_eq!(second_pair.origin,origin(second));
        let plan = crate::polymorphic_invocation_plans::Plan::checked([
            crate::polymorphic_invocation_plans::Pair { origin: origin(first), method: first_method },
            crate::polymorphic_invocation_plans::Pair { origin: origin(second), method: second_method },
        ], &analyzer.closures, span).unwrap();
        let second_body = &plan.cases[1].body.function;
        assert!(second_body.body.iter().any(|statement| matches!(&statement.kind,
            StmtKind::Return(Some(value)) if matches!(&value.kind, ExprKind::Integer(number) if number == "10"))),
            "the second environment must receive its own behavior, not the first body");
        let second_name=second_body.name.clone();
        let selected=analyzer.materialize_selected_generic_methods(&mut context,origin(first),one.function,Type::Int,&one.arguments,&one.accesses,one.invocation,&[origin(first),origin(second)],span).unwrap();
        let selected=analyzer.polymorphic_invocations.checked(selected,&analyzer.closures,span).unwrap();
        assert_eq!(selected.cases[1].body.function.name,second_name);
        let error = analyzer.prepare_deferred_generic_body(&context, origin(pair), vec![argument], None, span).err().expect("reachable wrong arity must reject");
        assert_eq!(error.code, "arity");
        assert_eq!(error.span, span);
    }
}
