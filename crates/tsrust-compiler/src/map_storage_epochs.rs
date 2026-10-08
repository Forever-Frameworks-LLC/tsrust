//! Renew concrete scoped Map storage lifetimes across checked local paths.
use super::*;

impl Emitter<'_> {
    /// End a straight-line local storage lifetime before a later refill.
    /// Declarations outside this block must retain their surrounding storage.
    pub(super) fn local_map_clear_epoch(&self, fun: &CheckedFunction, body: &[Stmt], statement: &Stmt) -> Option<String> {
        let (place, native) = self.local_map_clear_storage(fun, body, statement)?;
        Some(format!("drop({place}); let mut {place}: {native} = <{native}>::new();"))
    }

    fn local_map_clear_storage(&self, fun: &CheckedFunction, body: &[Stmt], statement: &Stmt) -> Option<(String, String)> {
        let StmtKind::Expr(call) = &statement.kind else { return None; };
        if call.target != Some(CallTarget::MapMethod(MapMethod::Clear)) { return None; }
        let ExprKind::Call(callee, _) = &call.kind else { return None; };
        let ExprKind::Field(base, _) = &callee.kind else { return None; };
        let mut root = base.as_ref();
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
                StmtKind::Let { .. } | StmtKind::Expr(_) | StmtKind::Assign { .. } => {
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
        let indent = "    ".repeat(depth);
        let pattern = places.iter().map(|place| format!("mut {place}")).collect::<Vec<_>>().join(",");
        let values = places.join(",");
        let condition = self.expr(fun, condition, Mode::Copy);
        let _ = writeln!(output, "{indent}let ({pattern},) = if {condition} {{");
        self.control_block_epochs(output, fun, then_body, depth + 1, loop_label, Some(declarations));
        let _ = writeln!(output, "{indent}    ({values},)\n{indent}}} else {{");
        self.control_block_epochs(output, fun, else_body, depth + 1, loop_label, Some(declarations));
        let _ = writeln!(output, "{indent}    ({values},)\n{indent}}};");
        true
    }
}

impl Emitter<'_> {
    pub(super) fn map_storage_live_at_exit(&self, fun: &CheckedFunction, body: &[Stmt], place: &str) -> bool {
        for (binding, value) in fun.bindings.iter().enumerate() {
            if place == format!("value{binding}") { return fun.storage_exits.live(body, binding, &[]); }
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
                return fun.storage_exits.live(body, binding, &names);
            }
        }
        false
    }
}
