//! Shared for-of emission preserves iterator evaluation and binding layouts.
use super::*;
impl Emitter<'_> {
    #[inline(never)]
    pub(super) fn for_of_loop(
        &self,
        output: &mut String,
        fun: &CheckedFunction,
        statement: &Stmt,
        depth: usize,
        declaring_body: Option<&[Stmt]>,
        resets: &[(String, String)],
    ) {
        let StmtKind::ForOf {
            iterable,
            body,
            binding,
            ..
        } = &statement.kind
        else {
            unreachable!("checked for-of");
        };
        if matches!(iterable.ty,Some(Type::Map(_))) && self.exception_regions.carries_storage() {
            self.map_borrowed_iteration_completion(output,fun,statement,depth,declaring_body,resets);
            return;
        }
        let indent = "    ".repeat(depth);
        let _control_scope = self
            .control_targets
            .enter(ControlTarget::Loop(statement.span.start));
        let id = binding.expect("checked iteration binding");
        let iterable_code = self.expr(fun, iterable, Mode::Read);
        let declarations = self
            .conditional_temporaries
            .borrow_mut()
            .drain(..)
            .collect::<Vec<_>>()
            .join(" ");
        let temporary_scope = !declarations.is_empty();
        if temporary_scope {
            let _ = writeln!(output, "{indent}{{ {declarations}");
        }
        if iterable.ty == Some(Type::String) {
            self.string_iteration(
                output,
                fun,
                statement,
                depth,
                &iterable_code,
                declaring_body,
                resets,
            );
            if temporary_scope {
                let _ = writeln!(output, "{indent}}}");
            }
            return;
        }
        if self.expression_shape(iterable) == Some(crate::fixed_storage::Shape::LiteralStrings) {
            let _ = writeln!(
                output,
                "{indent}'loop_{}: for literal_element in ({}).literal_iter() {{",
                statement.span.start, iterable_code
            );
            let mutable = if fun.bindings[id].mutable { "mut " } else { "" };
            let _ = writeln!(
                output,
                "{indent}    let {mutable}value{id} = &String::Borrowed(literal_element);"
            );
            self.iteration_body(output, fun, body, depth + 1, declaring_body, resets,statement);
            let _ = writeln!(output, "{indent}}}");
            if temporary_scope {
                let _ = writeln!(output, "{indent}}}");
            }
            return;
        }
        let mutable = if fun.bindings[id].mutable { "mut " } else { "" };
        let copied = if fun.bindings[id].ty.is_copy() {
            ".copied()"
        } else {
            ""
        };
        let iterator = if matches!(iterable.ty, Some(Type::Map(_))) {
            "entry_iter"
        } else if matches!(iterable.ty, Some(Type::Record(id)) if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::Heap)) {
            "values.key_iter"
        } else if matches!(iterable.ty, Some(Type::Record(id)) if self.program.records[id].native_element().is_some()) {
            "values.iter"
        } else {
            "iter"
        };
        let _ = writeln!(
            output,
            "{indent}'loop_{}: for {mutable}value{id} in ({}).{iterator}(){copied} {{",
            statement.span.start, iterable_code
        );
        self.iteration_body(output, fun, body, depth + 1, declaring_body, resets,statement);
        let _ = writeln!(output, "{indent}}}");
        if temporary_scope {
            let _ = writeln!(output, "{indent}}}");
        }
        return;
    }
    #[inline(never)]
    pub(super) fn iteration_body(
        &self,
        output: &mut String,
        fun: &CheckedFunction,
        body: &[Stmt],
        depth: usize,
        declaring_body: Option<&[Stmt]>,
        resets: &[(String, String)],
        statement:&Stmt,
    ) {
        if self.map_for_of_iteration_body(output,fun,statement,depth,declaring_body){return;}
        let indent = "    ".repeat(depth);
        for (place, native) in resets {
            let _ = writeln!(
                output,
                "{indent}let mut {place}: {native} = <{native}>::new();"
            );
        }
        self.control_block_epochs(output, fun, body, depth, None, declaring_body);
    }
}

impl Emitter<'_> {
    /// Leave the iterator borrow scope before moving completion storage.
    #[inline(never)]
    fn map_borrowed_iteration_completion(&self,output:&mut String,fun:&CheckedFunction,statement:&Stmt,depth:usize,declaring_body:Option<&[Stmt]>,resets:&[(String,String)]) {
        let (error,contract,runtime)=self.exception_regions.current_transport().expect("typed storage region");
        let id=self.completion_next.get();self.completion_next.set(id+1);
        let result=self.native_type(fun.result);
        let mut body=String::new();
        {
            let _scope=self.exception_regions.enter_typed(id,vec![ControlTarget::Loop(statement.span.start)],error.clone(),contract,runtime);
            self.for_of_loop(&mut body,fun,statement,depth+1,declaring_body,resets);
        }
        let pending=format!("{{let iteration_completion:TsrCompletion<(),{result},{error}>='exception_region{id}:{{{body} TsrCompletion::Normal(())}};iteration_completion}}");
        let dispatch=self.exception_regions.dispatch(&pending,&self.control_targets.snapshot(),statement.span).expect("enclosing iteration completion");
        let indent="    ".repeat(depth);
        let _=writeln!(output,"{indent}{dispatch};");
    }
}
