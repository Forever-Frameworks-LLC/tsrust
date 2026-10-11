//! Independent owning record leaves for local storage and static projections.
use super::*;
impl Emitter<'_> {
    /// A checked point type retains precise capture origins. Emitted owning
    /// storage can have a wider carrier after whole-body planning.
    pub(super) fn owned_record_source_type(
        &self,
        fun: &CheckedFunction,
        expr: &Expr,
    ) -> Option<Type> {
        let mut current = expr;
        let mut fields = Vec::new();
        loop {
            match &current.kind {
                ExprKind::Sequence(values) => current = values.last()?,
                ExprKind::TypeAssertion { value, .. } => current = value,
                ExprKind::Field(base, name) => {
                    fields.push(name);
                    current = base;
                }
                ExprKind::Name(_) => break,
                _ => return None,
            }
        }
        let id = current.binding?;
        self.owned_record_layout(fun, id)?;
        let mut ty = fun.bindings[id].ty;
        for name in fields.into_iter().rev() {
            let Type::Record(record) = ty else {
                return None;
            };
            ty = self.program.records[record]
                .fields
                .iter()
                .find(|field| field.name.text == *name)?
                .ty?;
        }
        Some(ty)
    }

    pub(super) fn owned_record_layout(
        &self,
        fun: &CheckedFunction,
        id: usize,
    ) -> Option<crate::scoped_record_places::Layout> {
        let binding = fun.bindings.get(id)?;
        if binding.parameter && id >= fun.parameter_count {
            return None;
        }
        let callback_storage = self.callback_record_native_parameter(binding.ty)
            || (self.native_views.contains(binding.ty) && crate::aggregate_callable_facts::inspect(
                binding.ty,
                &self.program.records,
                |id| self.program.unions[id.index()].members.clone(),
                |_| false,
            ).retains);
        crate::scoped_record_places::Layout::for_binding(
            binding,
            &self.program.records,
            // Only callback records use the nominal field-borrowing bridges.
            // Native views alone have ordinary Rust record borrowing, including
            // recursive records whose fields share one slice lifetime.
            callback_storage,
        )
    }
    pub(super) fn owned_record_value(
        &self,
        fun: &CheckedFunction,
        id: usize,
        path: &[usize],
        mode: Mode,
    ) -> Option<String> {
        let layout = self.owned_record_layout(fun, id)?;
        let mut ty = Type::Record(layout.record);
        for &field in path {
            let Type::Record(record) = ty else {
                return None;
            };
            ty = self.program.records[record].fields.get(field)?.ty?;
        }
        Some(self.owned_record_render(id, ty, &mut path.to_vec(), mode, false, false))
    }
    pub(super) fn owned_record_declaration(
        &self,
        fun: &CheckedFunction,
        id: usize,
        initializer: &str,
    ) -> Option<String> {
        let layout = self.owned_record_layout(fun, id)?;
        let pattern = self.owned_record_render(
            id,
            Type::Record(layout.record),
            &mut Vec::new(),
            Mode::Consume,
            true,
            fun.bindings[id].mutable,
        );
        // Bind every leaf directly. A named, partially moved generic record
        // can otherwise retain its old callback lifetime until scope exit.
        Some(format!(
            "let {pattern}:{}={initializer};",
            self.local_callback_environment_type(fun.bindings[id].ty)
        ))
    }
    pub(super) fn owned_record_assignment(
        &self,
        fun: &CheckedFunction,
        id: usize,
        initializer: &str,
    ) -> Option<String> {
        let layout = self.owned_record_layout(fun, id)?;
        let target = self.owned_record_render(
            id,
            Type::Record(layout.record),
            &mut Vec::new(),
            Mode::Consume,
            false,
            false,
        );
        // A destructuring assignment evaluates the replacement before committing fields.
        Some(format!("{target}={initializer};"))
    }
    pub(super) fn owned_record_deferred(&self, fun: &CheckedFunction, id: usize) -> Option<String> {
        let layout = self.owned_record_layout(fun, id)?;
        let mut declarations = layout
            .leaves
            .iter()
            .map(|leaf| {
                format!(
                    "let mut {}:{};",
                    leaf.native_name(id),
                    if self.scoped_record_native_parameter(leaf.ty) {
                        "_".to_owned()
                    } else {
                        self.native_type(leaf.ty)
                    }
                )
            })
            .collect::<String>();
        for (path, width) in self.record_order_paths(layout.record) {
            let name = crate::scoped_record_places::Leaf {
                path,
                ty: Type::Record(layout.record),
            }
            .native_name(id);
            let _ = write!(declarations, "let mut {name}_key_order:[usize;{width}];");
        }
        Some(declarations)
    }
    pub(super) fn owned_record_leaf_place(
        &self,
        fun: &CheckedFunction,
        id: usize,
        path: &[usize],
    ) -> Option<String> {
        let layout = self.owned_record_layout(fun, id)?;
        layout
            .leaves
            .iter()
            .find(|leaf| leaf.path == path)
            .map(|leaf| leaf.native_name(id))
    }
    fn owned_record_render(
        &self,
        id: usize,
        ty: Type,
        path: &mut Vec<usize>,
        mode: Mode,
        pattern: bool,
        mutable: bool,
    ) -> String {
        let name = crate::scoped_record_places::Leaf {
            path: path.clone(),
            ty,
        }
        .native_name(id);
        if let Type::Record(record) = ty
            && !self.program.records[record].fields.is_empty()
        {
            let mut fields=self.program.records[record].fields.iter().enumerate().map(|(index,field)|{
                path.push(index);let mut value=self.owned_record_render(id,field.ty.unwrap(),path,mode,pattern,mutable);if !pattern && matches!(mode,Mode::Read|Mode::Mutate) && matches!(field.ty,Some(Type::Record(child)) if !self.program.records[child].fields.is_empty()){let prefix=if mode==Mode::Mutate{"&mut ("}else{"&("};value=value.strip_prefix(prefix).unwrap().strip_suffix(')').unwrap().to_owned();}path.pop();format!("field{index}:{value}")
            }).collect::<Vec<_>>().join(",");
            if self.record_runtime.ordered.contains(&record) {
                let metadata = format!("{name}_key_order");
                let value = if pattern {
                    format!("{}{metadata}", if mutable { "mut " } else { "" })
                } else {
                    format!(
                        "{}{metadata}",
                        match mode {
                            Mode::Read => "&",
                            Mode::Mutate => "&mut ",
                            _ => "",
                        }
                    )
                };
                let _ = write!(fields, ",key_order:{value}");
            }
            let symbol = if pattern || !matches!(mode, Mode::Read | Mode::Mutate) {
                format!("Record{}", self.program.scoped_records.declaration(record))
            } else {
                format!(
                    "Tsr{}View{}",
                    if mode == Mode::Mutate {
                        "Mutate"
                    } else {
                        "Read"
                    },
                    self.program.scoped_records.declaration(record)
                )
            };
            let value = format!("{symbol}{{{fields}}}");
            return if !pattern && matches!(mode, Mode::Read | Mode::Mutate) {
                format!(
                    "{}({value})",
                    if mode == Mode::Mutate { "&mut " } else { "&" }
                )
            } else {
                value
            };
        }
        format!(
            "{}{name}",
            if pattern {
                if mutable { "mut " } else { "" }
            } else {
                match mode {
                    Mode::Read => "&",
                    Mode::Mutate => "&mut ",
                    _ => "",
                }
            }
        )
    }
}
