//! Renew concrete scoped Map storage lifetimes across checked local paths.
use super::*;

impl Emitter<'_> {
    /// Leading clears are infallible and execute before every completion path.
    /// Keep their fresh storage in the enclosing scope, then enter the checked
    /// exception region for all remaining effects and handlers.
    #[inline(never)]
    pub(super) fn map_try_prefix_epoch(&self, output: &mut String, fun: &CheckedFunction, statement: &Stmt, declarations: &[Stmt], depth: usize) -> bool {
        if !statement.labels.is_empty() { return false; }
        let StmtKind::Try(region) = &statement.kind else { return false; };
        let mut resets = String::new();
        let mut count = 0;
        for item in &region.body {
            if !item.labels.is_empty() { break; }
            let Some(reset) = self.local_map_clear_epoch(fun, declarations, item) else { break; };
            resets.push_str(&reset);
            count += 1;
        }
        if count == 0 { return false; }
        let mut remaining = region.as_ref().clone();
        remaining.body.drain(..count);
        let indent = "    ".repeat(depth);
        let _ = writeln!(output, "{indent}{resets}");
        let code = self.nested_completion(fun, statement, &remaining);
        let _ = writeln!(output, "{indent}{code}");
        true
    }

    /// End a straight-line local storage lifetime before a later refill.
    /// Declarations outside this block must retain their surrounding storage.
    pub(super) fn local_map_clear_epoch(&self, fun: &CheckedFunction, body: &[Stmt], statement: &Stmt) -> Option<String> {
        let (place, native) = self.local_map_clear_storage(fun, body, statement)?;
        let drop = if matches!(statement.kind, StmtKind::Assign { .. }) && !self.exception_regions.has_native_storage()
            && !self.map_storage_live_before(fun, statement.span.start, &place) { String::new() }
            else { format!("drop({place});") };
        Some(format!("{drop} let mut {place}: {native} = <{native}>::new();"))
    }

    pub(super) fn local_map_clear_storage(&self, fun: &CheckedFunction, body: &[Stmt], statement: &Stmt) -> Option<(String, String)> {
        let base = match &statement.kind {
            StmtKind::Expr(call) if call.target == Some(CallTarget::MapMethod(MapMethod::Clear)) => {
                let ExprKind::Call(callee, _) = &call.kind else { return None; };
                let ExprKind::Field(base, _) = &callee.kind else { return None; };
                base.as_ref()
            },
            StmtKind::Assign { target, value, operator: None }
                if matches!(&value.kind, ExprKind::MapConstructor(arguments, _) if arguments.is_empty()) => target,
            _ => return None,
        };
        let mut root = base;
        let mut names = Vec::new();
        while let ExprKind::Field(parent, name) = &root.kind { names.push(name); root = parent; }
        if !matches!(root.kind, ExprKind::Name(_)) { return None; }
        let binding = root.binding?;
        if !body.iter().any(|item| matches!(item.kind, StmtKind::Let { binding: Some(id), .. } if id == binding)) { return None; }
        let mut ty = fun.bindings[binding].ty;
        let mut path = Vec::new();
        for name in names.into_iter().rev() {
            let Type::Record(record) = ty else { return None; };
            let index = self.fields[record].get(name.as_str()).copied()?;
            path.push(index);
            ty = self.program.records[record].fields[index].ty?;
        }
        let place = if path.is_empty() { format!("value{binding}") }
            else { self.owned_record_leaf_place(fun, binding, &path)? };
        let Type::Map(id) = ty else { return None; };
        let value = self.program.records[id].map_definition.as_ref()?.value;
        if !self.scoped_record_native_parameter(value) { return None; }
        let native = self.native_type(Type::Map(id));
        Some((place, native))
    }
}

impl Emitter<'_> {
    /// A block containing only storage resets introduces no bindings or control
    /// flow. Emit its resets in the declaring scope to retain fresh lifetimes.
    pub(super) fn map_clear_block_epoch(&self, fun: &CheckedFunction, body: &[Stmt], statement: &Stmt) -> Option<String> {
        if !statement.labels.is_empty() { return None; }
        let StmtKind::Block(inner) = &statement.kind else { return None; };
        if inner.is_empty() { return None; }
        let mut code = String::new();
        for item in inner {
            let reset = self.local_map_clear_epoch(fun, body, item)
                .or_else(|| self.map_clear_block_epoch(fun, body, item))?;
            code.push_str(&reset);
        }
        Some(code)
    }
}

impl Emitter<'_> {
    pub(super) fn map_block_return_places(&self, fun: &CheckedFunction, declaring_body: &[Stmt], inner: &[Stmt]) -> Vec<String> {
        let mut places = Vec::new();
        let mut pending = inner.iter().collect::<Vec<_>>();
        while let Some(statement) = pending.pop() {
            if !statement.labels.is_empty() { return Vec::new(); }
            match &statement.kind {
                StmtKind::Block(body) => pending.extend(body.iter()),
                StmtKind::If { then_body, else_body, .. } => {
                    let left = self.map_block_return_places(fun, declaring_body, then_body);
                    let right = self.map_block_return_places(fun, declaring_body, else_body);
                    for place in left.into_iter().chain(right) { if !places.contains(&place) { places.push(place); } }
                },
                StmtKind::Let { .. } | StmtKind::Expr(_) | StmtKind::Assign { .. } | StmtKind::Throw(_) => {
                    if let Some((place, _)) = self.local_map_clear_storage(fun, declaring_body, statement)
                        && !places.contains(&place) { places.push(place); }
                },
                _ => return Vec::new(),
            }
        }
        places.sort();
        places
    }
}

impl Emitter<'_> {
    #[inline(never)]
    pub(super) fn map_clear_if_epoch(&self, output: &mut String, fun: &CheckedFunction, statement: &Stmt, declarations: &[Stmt], depth: usize, loop_label: Option<usize>) -> bool {
        let StmtKind::If { condition, then_body, else_body } = &statement.kind else { return false; };
        let mut places = self.map_block_return_places(fun, declarations, then_body);
        for place in self.map_block_return_places(fun, declarations, else_body) {
            if !places.contains(&place) { places.push(place); }
        }
        if places.is_empty() { return false; }
        places.sort();
        places.retain(|place| self.map_storage_live_at_exit(fun, then_body, place) || self.map_storage_live_at_exit(fun, else_body, place));
        let comma = if places.is_empty() { "" } else { "," };
        let indent = "    ".repeat(depth);
        let pattern = places.iter().map(|place| format!("mut {place}")).collect::<Vec<_>>().join(",");
        let then_values = places.iter().map(|place| self.map_branch_exit_value(fun, then_body, place)).collect::<Vec<_>>().join(",");
        let else_values = places.iter().map(|place| self.map_branch_exit_value(fun, else_body, place)).collect::<Vec<_>>().join(",");
        let condition = self.expr(fun, condition, Mode::Copy);
        let _ = writeln!(output, "{indent}let ({pattern}{comma}) = if {condition} {{");
        self.control_block_epochs(output, fun, then_body, depth + 1, loop_label, Some(declarations));
        let _ = writeln!(output, "{indent}    ({then_values}{comma})\n{indent}}} else {{");
        self.control_block_epochs(output, fun, else_body, depth + 1, loop_label, Some(declarations));
        let _ = writeln!(output, "{indent}    ({else_values}{comma})\n{indent}}};");
        true
    }
}

impl Emitter<'_> {
    pub(super) fn map_storage_live_at_exit(&self, fun: &CheckedFunction, body: &[Stmt], place: &str) -> bool {
        self.map_storage_live(fun, place, |binding,names| fun.storage_exits.live(body,binding,names))
    }
    pub(super) fn map_storage_live_before(&self, fun: &CheckedFunction, start: usize, place: &str) -> bool {
        self.map_storage_live(fun, place, |binding,names| fun.storage_exits.live_before(start,binding,names))
    }
    pub(super) fn map_storage_live_at_expression(&self, fun: &CheckedFunction, point: (usize,usize), place: &str) -> bool {
        self.map_storage_live(fun, place, |binding,names| fun.storage_exits.live_at_expression(point,binding,names))
    }
    fn map_storage_live(&self, fun: &CheckedFunction, place: &str, live: impl Fn(usize,&[String])->bool) -> bool {
        for (binding, value) in fun.bindings.iter().enumerate() {
            if place == format!("value{binding}") { return live(binding, &[]); }
            let Some(layout) = self.owned_record_layout(fun, binding) else { continue; };
            for leaf in &layout.leaves {
                if leaf.native_name(binding) != place { continue; }
                let mut ty = value.ty;
                let mut names = Vec::new();
                for &index in &leaf.path {
                    let Type::Record(record) = ty else { return false; };
                    let field = &self.program.records[record].fields[index];
                    names.push(field.name.text.clone());
                    ty = field.ty.unwrap();
                }
                return live(binding, &names);
            }
        }
        false
    }
}

impl Emitter<'_> {
    pub(super) fn map_branch_exit_value(&self, fun: &CheckedFunction, body: &[Stmt], place: &str) -> String {
        if self.map_storage_live_at_exit(fun, body, place) { return place.to_owned(); }
        self.map_empty_storage_value(fun, place)
    }
    pub(super) fn map_empty_storage_value(&self, fun: &CheckedFunction, place: &str) -> String {
        // Source availability remains consumed. This empty native owner only
        // balances branch storage; it has no captured payload to copy or drop.
        for (binding, value) in fun.bindings.iter().enumerate() {
            let ty = if place == format!("value{binding}") { Some(value.ty) }
                else { self.owned_record_layout(fun, binding).and_then(|layout| layout.leaves.iter().find(|leaf| leaf.native_name(binding) == place).map(|leaf| leaf.ty)) };
            if let Some(ty @ Type::Map(_)) = ty { return format!("<{}>::new()", self.native_type(ty)); }
        }
        unreachable!("checked Map storage leaf")
    }
}
