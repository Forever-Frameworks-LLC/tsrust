//! Registry calls keep deferred program effects separate from environment moves.
use super::*;

impl Analyzer {
    pub(super) fn program_callback_call(&mut self, context: &mut Context, expression: &mut Expr, loans: &Loans) -> Option<Result<Type, Diagnostic>> {
        let ExprKind::Call(callee, arguments) = &expression.kind else { return None; };
        let ExprKind::Name(name) = &callee.kind else { return None; };
        let &slot = context.names.get(name)?;
        let Type::Closure(origin) = context.bindings[slot].ty else { return None; };
        if !self.closures[origin].program_scoped { return None; }
        if !arguments.is_empty() {
            return Some(Err(Diagnostic::new("arity", "A registration callback expects no arguments.", expression.span)));
        }
        let span = expression.span;
        let callback = callee.as_ref().clone();
        let registry_name = crate::stdlib::testing::program::REGISTRY;
        let registry_slot = context.bindings.iter().position(|binding| matches!(binding.ty, Type::Record(id) if self.program.records[id].native_type == Some(crate::native_signatures::NativeTypeId::TestRegistry)));
        if let Some(slot) = registry_slot { context.names.insert(registry_name.into(), slot); }

        let mut arguments = vec![callback, Expr::new(ExprKind::Name(crate::stdlib::testing::program::REGISTRY.into()), span)];
        for declaration in &self.program.module_owner_bindings {
            let Some(&slot) = context.source_bindings.get(&declaration.start) else { continue; };
            let binding = &context.bindings[slot];
            if binding.name == crate::stdlib::testing::program::REGISTRY { continue; }
            let alias = format!("@program-input-{}", declaration.start);
            context.names.insert(alias.clone(), slot);
            arguments.push(Expr::new(ExprKind::Name(alias), span));
        }
        let symbol = "@native:std:testing-runtime#invokeRegistration";
        expression.kind = ExprKind::Call(Box::new(Expr::new(ExprKind::Name(symbol.into()), span)), arguments);
        let (id, _) = crate::native_bindings::by_symbol(symbol).expect("private callback invocation");
        Some(self.program_testing_call(context, expression, id, loans))
    }

    pub(super) fn program_testing_call(&mut self, context: &mut Context, expression: &mut Expr, id: usize, loans: &Loans) -> Result<Type, Diagnostic> {
        let binding = &crate::native_bindings::BINDINGS[id];
        let ExprKind::Call(_, arguments) = &mut expression.kind else { unreachable!() };
        let count = binding.parameters.len();
        if arguments.len() < count + 1 {
            return Err(Diagnostic::new("internal", "Test registry input was not lowered.", expression.span));
        }
        let registry = self.program.records.iter().position(|record| record.native_type == Some(crate::native_signatures::NativeTypeId::TestRegistry)).expect("lowered registry type");
        let mut contracts = binding.parameters.iter().map(|parameter| {
            Ok(ArgumentContract { ty: self.native_parameter_type(parameter.ty)?, mode: parameter.access, omission: crate::ParameterOmission::Required, rest: false })
        }).collect::<Result<Vec<_>, Diagnostic>>()?;
        // A registered task factory remains callable outside the runner. It
        // constructs a lazy task with the same explicit program loans; it does
        // not become a void suite-registration callback at those other uses.
        let factory=if binding.name=="invokeRegistration" {
            arguments.first().and_then(|arg|self.expression_type_hint(context,arg)).and_then(|ty|match ty{
                Type::Closure(origin) if self.is_task_type(self.closures[origin].function.result)=>Some((origin,self.closures[origin].function.result)),_=>None,
            })
        }else{None};
        if let Some((origin,_))=factory {contracts[0].ty=Type::Closure(origin);contracts[0].mode=self.closures[origin].invocation;}
        contracts.push(ArgumentContract { ty: Type::Record(registry), mode: Mode::Mutate, omission: crate::ParameterOmission::Required, rest: false });
        let mut declarations = Vec::new();
        let mut retained = Vec::new();
        for argument in arguments.drain(count + 1..) {
            let ExprKind::Name(name) = &argument.kind else { unreachable!("lowered program input") };
            let slot = *context.names.get(name).ok_or_else(|| Diagnostic::new("module-state", "Test execution requires initialized program storage.", argument.span))?;
            let declaration = context.source_bindings.iter().find_map(|(&declaration, &bound)| (bound == slot && self.program.module_owner_bindings.iter().any(|span| span.start == declaration)).then_some(declaration)).expect("program input identity");
            let Some(&mode) = self.program_access.get(&declaration) else { continue; };
            declarations.push(declaration);
            contracts.push(ArgumentContract { ty: context.bindings[slot].ty, mode, omission: crate::ParameterOmission::Required, rest: false });
            retained.push(argument);
        }
        arguments.extend(retained);
        self.check_call_arguments(context, arguments, &contracts, loans)?;
        if factory.is_some() {
            for (argument,contract) in arguments.iter().zip(&contracts) {
                if matches!(contract.mode,Mode::Read|Mode::Mutate) {
                    for root in Self::loan_roots(argument,contract.mode) {
                        let loan=(root,contract.mode,expression.span);
                        if !context.capture_loans.contains(&loan){context.capture_loans.push(loan);}
                    }
                }
            }
        }
        expression.capture_slots = Some(declarations.into_boxed_slice());
        expression.target = Some(CallTarget::NativeBinding(id));
        let result = factory.map_or_else(||native_type(binding.result, &self.program.records),|(_,result)|result);
        expression.ty = Some(result);
        Ok(result)
    }
}
