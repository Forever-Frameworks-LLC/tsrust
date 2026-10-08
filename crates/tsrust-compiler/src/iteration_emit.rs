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
            self.iteration_body(output, fun, body, depth + 1, declaring_body, resets);
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
        } else {
            "iter"
        };
        let _ = writeln!(
            output,
            "{indent}'loop_{}: for {mutable}value{id} in ({}).{iterator}(){copied} {{",
            statement.span.start, iterable_code
        );
        self.iteration_body(output, fun, body, depth + 1, declaring_body, resets);
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
    ) {
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
