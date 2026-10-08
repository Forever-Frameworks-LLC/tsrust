//! Local Map values preserve concrete borrowed record environments.
use super::*;

impl Analyzer {
    fn contains_scoped_map_storage(&self, ty: Type) -> bool {
        let mut pending = vec![ty];
        let mut seen = HashSet::new();
        while let Some(ty) = pending.pop() {
            if !seen.insert(ty) { continue; }
            match ty {
                Type::Map(_) if self.aggregate_callable_facts(ty).scoped => return true,
                Type::Record(id) => pending.extend(self.program.records[id].fields.iter().filter_map(|field| field.ty)),
                _ => {}
            }
        }
        false
    }
    pub(super) fn scoped_map_value_instance(&mut self, id: usize, value: Type) -> usize {
        let key = self.program.records[id].map_definition.as_ref().unwrap().key;
        if let Some(found) = self.program.records.iter().position(|record| record.map_definition.as_ref().is_some_and(|map| map.key == key && map.value == value && map.entry.is_none())) { return found; }
        let mut record = self.program.records[id].clone();
        let map = record.map_definition.as_mut().unwrap();
        map.value = value;
        map.entry = None;
        let instance = self.program.records.len();
        self.program.records.push(record);
        self.fields.push(HashMap::new());
        self.unions.register_map_type(instance);
        instance
    }

    pub(super) fn scoped_map_contract(&self, actual: Type, expected: Type) -> bool {
        let (Type::Map(actual), Type::Map(expected)) = (actual, expected) else { return false; };
        let (Some(actual), Some(expected)) = (self.program.records[actual].map_definition.as_ref(), self.program.records[expected].map_definition.as_ref()) else { return false; };
        let (Type::Record(value), Type::Record(contract)) = (actual.value, expected.value) else { return false; };
        actual.key == expected.key && self.aggregate_callable_facts(actual.value).scoped
            && value != contract && self.scoped_records.declaration(value) == contract
    }

    #[inline(never)]
    pub(super) fn scoped_map_carriers(&mut self, context: &mut Context, body: &mut [Stmt]) -> Result<(), Diagnostic> {
        let mut roots = context.bindings.iter().enumerate().filter_map(|(binding, info)| {
            (!info.parameter && !info.capture_reference && self.contains_scoped_map_storage(info.ty)).then_some((binding, info.ty))
        }).collect::<HashMap<_, _>>();
        if roots.is_empty() { return Ok(()); }
        let writes = record_storage_writes::collect(body);
        // Record literal fields and direct moves share the same storage edges.
        let mut leaves = Vec::new();
        for write in &writes {
            let mut pending = vec![(write.path.clone(), write.value)];
            while let Some((path, value)) = pending.pop() {
                if let ExprKind::Unary { op: UnaryOp::RecordCarrier, value: inner, .. } = &value.kind {
                    pending.push((path, inner));
                } else if let ExprKind::Record(fields) = &value.kind {
                    for (name, field) in fields { let mut nested = path.clone(); nested.push(name.text.clone()); pending.push((nested, field)); }
                } else if matches!(value.ty, Some(Type::Map(_))) {
                    leaves.push((write.binding, path, value));
                }
            }
        }
        loop {
            let mut changed = false;
            for (destination, path, value) in &leaves {
                let Some((_, source_path)) = Self::direct_fields(value) else { continue; };
                let Some(source_binding) = Self::root_binding(value) else { continue; };
                let (Some(source_ty), Some(destination_ty)) = (roots.get(&source_binding), roots.get(destination)) else { continue; };
                let (Some(Type::Map(source_id)), Some(Type::Map(destination_id))) = (self.record_field_type(*source_ty, &source_path), self.record_field_type(*destination_ty, path)) else { continue; };
                let source_value = self.program.records[source_id].map_definition.as_ref().unwrap().value;
                let destination_value = self.program.records[destination_id].map_definition.as_ref().unwrap().value;
                let (Type::Record(left), Type::Record(right)) = (source_value, destination_value) else { continue; };
                let joined = if left == right { source_value } else { self.join_record_types(left, right, value.span)?.ok_or_else(|| Diagnostic::new("type", "Moved Map callback environments have incompatible contracts.", value.span))? };
                for (binding, path, id) in [(source_binding, source_path.as_slice(), source_id), (*destination, path.as_slice(), destination_id)] {
                    let next = Type::Map(self.scoped_map_value_instance(id, joined));
                    let root = roots[&binding];
                    let next = if path.is_empty() { next } else {
                        let Type::Record(record) = root else { continue; };
                        let Some(record) = self.record_type_with_field(record, path, next) else { continue; };
                        Type::Record(record)
                    };
                    if root != next { roots.insert(binding, next); changed = true; }
                }
            }
            if !changed { break; }
        }
        for (binding, ty) in &roots { context.bindings[*binding].ty = *ty; }
        let mut constructors = HashMap::new();
        for (binding, path, value) in &leaves {
            if matches!(value.kind, ExprKind::MapConstructor(_, _))
                && let Some(ty) = roots.get(binding).and_then(|root| self.record_field_type(*root, path))
            { constructors.insert((value.span.start, value.span.end), ty); }
        }
        let mut results = roots.clone();
        loop {
            let mut changed = false;
            for write in &writes {
                if !write.path.is_empty() || roots.contains_key(&write.binding) { continue; }
                if let Some(ty) = self.planned_scoped_map_result(write.value, &results)
                    && results.get(&write.binding) != Some(&ty)
                { results.insert(write.binding, ty); changed = true; }
            }
            if !changed { break; }
        }
        for (binding, ty) in &results { context.bindings[*binding].ty = *ty; }
        crate::source_walk::body(body, &mut |_| Ok(()), &mut |value| {
            if let ExprKind::Unary { op: UnaryOp::RecordCarrier, value: inner, .. } = &value.kind
                && value.ty.is_some_and(|ty| self.contains_scoped_map_storage(ty))
            { value.ty = Some(self.record_value_carrier_type(context, inner, &results)?); }
            if let Some(ty) = self.planned_scoped_map_result(value, &results) { value.ty = Some(ty); }
            if let ExprKind::Project { value: inner, union, narrowed } = &mut value.kind
                && matches!(narrowed, Type::Record(_))
                && let Some(ty) = self.planned_scoped_map_result(inner, &results)
                && let Some(id) = ty.union()
                && let Some(payload) = self.unions.payload(ty.owning())
            { *union = id; *narrowed = payload; value.ty = Some(payload); }
            if let ExprKind::Call(callee, _) = &mut value.kind
                && let Some(Type::BorrowedCallable(family)) = self.planned_scoped_map_result(callee, &results)
            { callee.ty = Some(Type::BorrowedCallable(family)); value.target = Some(CallTarget::StoredCallableField(family)); }
            if let Some(ty) = constructors.get(&(value.span.start, value.span.end)) { value.ty = Some(*ty); }
            if let ExprKind::Call(callee, arguments) = &mut value.kind
                && let ExprKind::Field(base, _) = &mut callee.kind
                && let Some(Type::Map(id)) = self.planned_scoped_map_result(base, &results)
            {
                base.ty = Some(Type::Map(id));
                let target = self.program.records[id].map_definition.as_ref().unwrap().value;
                if matches!(value.target, Some(CallTarget::MapMethod(MapMethod::Set))) && arguments[1].ty != Some(target) { arguments[1].wrap_record_carrier(target); }
            }
            if matches!(value.kind, ExprKind::Record(_)) { value.ty = Some(self.record_value_carrier_type(context, value, &results)?); }
            Ok(())
        })
    }
    fn planned_scoped_map_result(&self, value: &Expr, results: &HashMap<usize, Type>) -> Option<Type> {
        match &value.kind {
            ExprKind::Name(_) => results.get(&value.binding?).copied(),
            ExprKind::Call(callee, _) if matches!(value.target, Some(CallTarget::MapMethod(MapMethod::Get))) => {
                let ExprKind::Field(base, _) = &callee.kind else { return None; };
                let Type::Map(id) = self.planned_scoped_map_result(base, results)? else { return None; };
                let payload = self.program.records[id].map_definition.as_ref()?.value;
                let result = self.unions.intern([payload, Type::Undefined], value.span).ok()?;
                if matches!(value.ty, Some(Type::BorrowedUnion(_))) { Some(Type::BorrowedUnion(result.union()?)) } else { Some(result) }
            }
            ExprKind::Project { value: inner, narrowed, .. } if matches!(narrowed, Type::Record(_)) => self.unions.payload(self.planned_scoped_map_result(inner, results)?.owning()),
            ExprKind::Field(base, name) => {
                let Type::Record(id) = self.planned_scoped_map_result(base, results)? else { return None; };
                self.program.records[id].fields.iter().find(|field| field.name.text == *name)?.ty
            }
            _ => None,
        }
    }
}
