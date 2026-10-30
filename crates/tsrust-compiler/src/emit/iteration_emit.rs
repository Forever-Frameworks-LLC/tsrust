//! Shared for-of emission preserves iterator evaluation and binding layouts.
use super::*;
pub(super) struct IterationFrame<'a> {
    pub(super) scope: exception_contract_emit::ControlScope<'a>,
    pub(super) suffix: String,
}
impl Emitter<'_> {
    pub(super) fn schedule_for_of<'a, 's:'a>(&'s self, output: &mut String, fun: &'a CheckedFunction, statement: &'a Stmt, visible: std::rc::Rc<Vec<&'a Stmt>>, depth: usize, resets: &[(String, String)], pending: &mut Vec<statement_work::Work<'a, 's>>) {
        use statement_work::Work;
        let StmtKind::ForOf { body, iterable, .. } = &statement.kind else { unreachable!() };
        if matches!(iterable.ty,Some(Type::Map(_))) && self.exception_regions.carries_storage() {
            self.schedule_map_borrowed_iteration(fun,statement,visible,depth,resets,pending);
            return;
        }
        if let Some(frame) = self.prepare_for_of_loop(output, fun, statement, depth) {
            pending.push(Work::Scope(frame.scope));
            pending.push(Work::Text(frame.suffix));
            if !self.schedule_map_for_of_body(output, fun, statement, visible.clone(), depth + 1, pending) {
                for (place, native) in resets { let _ = writeln!(output, "let mut {place}: {native} = <{native}>::new();"); }
                pending.push(Work::Body(body, visible, depth + 1, None));
            }
        }
    }
    #[inline(never)]
    pub(super) fn prepare_for_of_loop<'s>(
        &'s self,
        output: &mut String,
        fun: &CheckedFunction,
        statement: &Stmt,
        _depth: usize,
    ) -> Option<IterationFrame<'s>> {
        let StmtKind::ForOf {
            iterable,
            binding,
            ..
        } = &statement.kind
        else {
            unreachable!("checked for-of");
        };


        let scope = self
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
            let _ = writeln!(output, "{{ {declarations}");
        }
        if iterable.ty.map(Type::scalar_base) == Some(Type::String) {
            let label = statement.span.start;
            let _ = writeln!(output, "{{ let string_source_{label}={iterable_code};");
            let _ = writeln!(output, "'loop_{label}: for string_character_{label} in string_source_{label}.as_ref().chars() {{");
            let _ = writeln!(output, "    let value{id}=&tsr_char_text(string_character_{label});");
            let suffix = format!("}} }}\n{}", if temporary_scope { format!("}}\n") } else { String::new() });
            return Some(IterationFrame { scope, suffix });
        }
        if self.expression_shape(iterable) == Some(crate::fixed_storage::Shape::LiteralStrings) {
            let _ = writeln!(
                output,
                "'loop_{}: for literal_element in ({}).literal_iter() {{",
                statement.span.start, iterable_code
            );
            let mutable = if fun.bindings[id].mutable { "mut " } else { "" };
            let _ = writeln!(
                output,
                "    let {mutable}value{id} = &String::Borrowed(literal_element);"
            );
            let suffix = format!("}}\n{}", if temporary_scope { format!("}}\n") } else { String::new() });
            return Some(IterationFrame { scope, suffix });
        }
        let mutable = if fun.bindings[id].mutable { "mut " } else { "" };
        if let Some(Type::Map(map))=iterable.ty
            && self.program.records[map].native_type==Some(crate::native_signatures::NativeTypeId::OrderedMap) {
            let _=writeln!(output,"'loop_{}: for (ordered_key,ordered_value) in ({iterable_code}).source_iter() {{",statement.span.start);
            let view=self.ordered_map_entry_view(map,"ordered_key","ordered_value");
            let _=writeln!(output,"    let {mutable}value{id}=&{view};");
            if let Some(entry)=self.record_parameter_entry_view(fun,id){let _=writeln!(output,"    {entry}");}
            let suffix = format!("}}\n{}", if temporary_scope { format!("}}\n") } else { String::new() });
            return Some(IterationFrame { scope, suffix });
        }
        let copied = if fun.bindings[id].ty.is_copy() {
            ".copied()"
        } else {
            ""
        };
        let iterator = if matches!(iterable.ty, Some(Type::Map(_))) {
            "entry_iter"
        } else if matches!(iterable.ty, Some(Type::Record(id)) if matches!(self.program.records[id].native_type,Some(crate::native_signatures::NativeTypeId::Heap | crate::native_signatures::NativeTypeId::OrderedSet))) {
            "values.key_iter"
        } else if matches!(iterable.ty, Some(Type::Record(id)) if self.program.records[id].native_element().is_some()) {
            "values.iter"
        } else {
            "iter"
        };
        let _ = writeln!(
            output,
            "'loop_{}: for {mutable}value{id} in ({}).{iterator}(){copied} {{",
            statement.span.start, iterable_code
        );
        if let Some(entry)=self.record_parameter_entry_view(fun,id){let _=writeln!(output,"    {entry}");}
        let suffix = format!("}}\n{}", if temporary_scope { format!("}}\n") } else { String::new() });
        Some(IterationFrame { scope, suffix })
    }

}

impl Emitter<'_> {
    /// Leave the iterator borrow scope before moving completion storage.
    #[inline(never)]
    fn schedule_map_borrowed_iteration<'a,'s:'a>(&'s self,fun:&'a CheckedFunction,statement:&'a Stmt,visible:std::rc::Rc<Vec<&'a Stmt>>,depth:usize,resets:&[(String,String)],work:&mut Vec<statement_work::Work<'a,'s>>) {
        use statement_work::Work;
        let (error,contract,runtime)=self.exception_regions.current_transport().expect("typed storage region");
        let id=self.completion_next.get();self.completion_next.set(id+1);
        let result=self.native_type(fun.result);
        let scope=self.exception_regions.enter_typed(id,vec![ControlTarget::Loop(statement.span.start)],error.clone(),contract,runtime);
        work.push(Work::Resume(Box::new(move |output,_,results| {
            let body=results.pop().expect("completed borrowed Map iteration");
            let pending=format!("{{let iteration_completion:TsrCompletion<(),{result},{error}>='exception_region{id}:{{{body} TsrCompletion::Normal(())}};iteration_completion}}");
            let dispatch=self.exception_regions.dispatch(&pending,&self.control_targets.snapshot(),statement.span).expect("enclosing iteration completion");
            let _=writeln!(output,"{dispatch};");
        })));
        work.push(Work::Region(scope));
        work.push(Work::Captured);
        let resets=resets.to_vec();
        work.push(Work::Resume(Box::new(move |output,work,_|self.schedule_for_of(output,fun,statement,visible,depth+1,&resets,work))));
        work.push(Work::Capture);
    }
}
