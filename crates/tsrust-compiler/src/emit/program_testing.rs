//! Program storage is supplied to registered callbacks only at invocation.
use super::*;

impl Emitter<'_> {
    pub(super) fn test_program_inputs(&self) -> std::collections::BTreeMap<usize, (Type, Mode)> {
        let mut inputs = std::collections::BTreeMap::new();
        for closure in &self.program.closures {
            for capture in &closure.program_captures {
                let ty = closure.function.bindings[capture.inner].ty;
                if self.test_registry_type(ty) { continue; }
                inputs.entry(capture.declaration.start)
                    .and_modify(|(_, mode): &mut (Type, Mode)| *mode = (*mode).max(capture.access))
                    .or_insert((ty, capture.access));
            }
        }
        inputs
    }

    pub(super) fn test_registry_type(&self, ty: Type) -> bool {
        matches!(ty, Type::Record(id) if self.program.records[id].native_type == Some(crate::native_signatures::NativeTypeId::TestRegistry))
    }

    pub(super) fn test_program_declarations(&self, output: &mut String) {
        if !self.program.records.iter().any(|record| record.native_type == Some(crate::native_signatures::NativeTypeId::TestRegistry)) { return; }
        output.push_str("struct TsrTestContext<'p> { marker: std::marker::PhantomData<&'p ()>,");
        for (declaration, (ty, mode)) in self.test_program_inputs() {
            let mutable = if mode == Mode::Mutate { "mut " } else { "" };
            let _ = writeln!(output, "g{declaration}: Option<&'p {mutable}{}>,", self.native_type(ty));
        }
        output.push_str("}\ntrait ProgramCallback<A,R> { fn invoke_program(&mut self, args:A, program:&mut TsrTestContext<'_>, registry:&mut TsrTestRegistry)->R; }\n");
        if self.callback_traits { output.push_str("impl<C:Callback<A,R>,A,R> ProgramCallback<A,R> for C { fn invoke_program(&mut self,args:A,_program:&mut TsrTestContext<'_>,_registry:&mut TsrTestRegistry)->R { self.invoke(args) } }\n"); }
    }

    pub(super) fn test_program_capture_bindings(&self, output: &mut String, closure: &crate::analysis::CheckedClosure) {
        let factory=closure.program_scoped && self.program.task_type(closure.function.result);
        for capture in &closure.program_captures {
            let binding = &closure.function.bindings[capture.inner];
            if factory {
                let source=if self.test_registry_type(binding.ty){"test_registry".to_owned()}else{format!("test_program.g{}.expect(\"checked program input\")",capture.declaration.start)};
                let _=writeln!(output,"let value{} = {source};",capture.inner);
                continue;
            }
            let source = if self.test_registry_type(binding.ty) { "*test_registry".to_owned() } else {
                let view=if self.test_program_inputs()[&capture.declaration.start].1==Mode::Mutate{"as_deref_mut"}else{"as_deref"};
                format!("*test_program.g{}.{view}().expect(\"checked program input\")", capture.declaration.start)
            };
            let borrow = if capture.access == Mode::Mutate { "&mut " } else { "&" };
            let _ = writeln!(output, "let value{} = {borrow}{source};", capture.inner);
        }
    }

    pub(super) fn test_factory_context(&self)->String {
        let mut fields=String::from("marker:std::marker::PhantomData,");
        for (declaration,(_,mode)) in self.test_program_inputs() {
            let view=if mode==Mode::Mutate{"as_deref_mut"}else{"as_deref"};
            let _=write!(fields,"g{declaration}:test_program.g{declaration}.{view}(),");
        }
        format!("TsrTestContext{{{fields}}}")
    }

    fn test_callback_output(&self,ty:Type)->String {
        let error=match ty {
            Type::Closure(id)=>self.result_closures.borrow().get(&id).cloned(),
            Type::Callback(id)=>self.result_callbacks.borrow().get(&id).map(|(error,_)|error.clone()),
            Type::StoredCallable(id)|Type::BorrowedCallable(id)=>self.result_storage.borrow().get(&id).map(|(error,_)|error.clone()),
            _=>None,
        };
        error.map_or_else(||"()".into(),|error|format!("Result<(),{error}>"))
    }

    pub(super) fn program_testing_native(&self, fun: &CheckedFunction, expression: &Expr, id: usize, args: &[Expr]) -> Option<String> {
        let binding = &crate::native_bindings::BINDINGS[id];
        if binding.module != "std:testing-runtime" || !matches!(binding.name, "invokeSuite" | "invokeRegistration" | "registerTest" | "registerHook" | "runTests" | "finishTests" | "beginSuite" | "endSuite" | "beginFile") { return None; }
        self.native_bindings.borrow_mut().insert(id);
        let count = binding.parameters.len();
        let mut arguments = args[..count].iter().zip(binding.parameters).map(|(arg, parameter)| self.expr(fun, arg, parameter.access)).collect::<Vec<_>>();
        let registry = self.expr(fun, &args[count], Mode::Mutate);
        arguments.push(registry.clone());
        if matches!(binding.name, "registerTest" | "registerHook") {
            if let Some(call) = self.typed_test_registration(&arguments, args[1].ty.expect("checked callback"), expression.span, None, binding.name == "registerHook") { return Some(call); }
            let helper = if binding.name == "registerHook" { "tsrust_testing_register_hook" } else { "tsrust_testing_register_test" };
            return Some(format!("{helper}({registry}, {}, {})", arguments[0], arguments[1]));
        }
        let call = if crate::stdlib::testing::program::executes(binding.name) {
            let inputs = self.test_program_inputs();
            let mut fields = String::from("marker:std::marker::PhantomData,");
            for (argument, &declaration) in args[count + 1..].iter().zip(expression.capture_slots.as_deref().expect("program identities")) {
                let mode = inputs[&declaration].1;
                let value = if argument.ty.is_some_and(|ty| ty.is_copy()) {
                    let slot = argument.binding.expect("program input binding");
                    let borrow = if mode == Mode::Mutate { "&mut " } else { "&" };
                    let deref = if fun.bindings[slot].capture_reference { "*" } else { "" };
                    format!("{borrow}{deref}value{slot}")
                } else { self.expr(fun, argument, mode) };
                let _ = write!(fields, "g{declaration}:Some({value}),");
            }
            for declaration in inputs.keys().filter(|declaration|!expression.capture_slots.as_deref().expect("program identities").contains(declaration)) {
                let _=write!(fields,"g{declaration}:None,");
            }
            if binding.name=="invokeSuite" {
                // Evaluate name and callback once, in source argument order,
                // before entering the suite. Preserve typed callback failures.
                let result=self.test_callback_output(args[1].ty.expect("checked suite callback"));
                let call=format!("tsrust_testing_suite(test_suite_registry,test_suite_name,|test_suite_registry|ProgramCallback::<(),{result}>::invoke_program(test_suite_callback,(),&mut TsrTestContext{{{fields}}},test_suite_registry))");
                let call=self.typed_runner_result(&call,expression.span,None)
                    .unwrap_or_else(||format!("{call}.unwrap_or_else(|message|panic!(\"{{message}}\"))"));
                let invoke=self.registered_traversal_call(call,&args[1]);
                return Some(format!("{{let test_suite_name={};let test_suite_callback={};let test_suite_registry={registry};{invoke}}}",arguments[0],arguments[1]));
            }
            if binding.name == "invokeRegistration" {
                if let Some(Type::Closure(origin))=args[0].ty
                    && self.program.task_type(self.program.closures[origin].function.result) {
                    let call=format!("({}).call(TsrTestContext{{{fields}}},{registry})",arguments[0]);
                    return Some(self.registered_closure_call(call,origin,expression.span));
                }
                let result=self.test_callback_output(args[0].ty.expect("checked registration callback"));
                let call = format!("ProgramCallback::<(),{result}>::invoke_program({},(),&mut TsrTestContext{{{fields}}},{registry})", arguments[0]);
                return Some(self.registered_traversal_call(call, &args[0]));
            }
            let helper = if binding.name == "runTests" { "tsrust_testing_run_tests_result" } else { "tsrust_testing_finish_tests_result" };
            format!("{helper}({registry}, &mut TsrTestContext{{{fields}}})")
        } else {
            let helper = match binding.name { "beginSuite" => "tsrust_testing_begin_suite_result", "beginFile" => "tsrust_testing_begin_file_result", _ => "tsrust_testing_end_suite_result" };
            let suffix = if count == 0 { String::new() } else { format!(", {}", arguments[0]) };
            format!("{helper}({registry}{suffix})")
        };
        Some(self.typed_runner_result(&call, expression.span, None).unwrap_or_else(|| {
            if binding.name == "finishTests" { call.replace("tsrust_testing_finish_tests_result", "tsrust_testing_finish_tests") }
            else { format!("{call}.unwrap_or_else(|message|panic!(\"{{message}}\"))") }
        }))
    }
}
