//! Local Map values preserve concrete borrowed record environments.
use super::*;

impl Analyzer {
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

    #[inline(never)]
    pub(super) fn scoped_map_carriers(&mut self, context: &mut Context, body: &mut [Stmt]) -> Result<(), Diagnostic> {
        let plans = context.bindings.iter().enumerate().filter_map(|(binding, info)| {
            let Type::Map(id) = info.ty else { return None; };
            (!info.parameter && !info.capture_reference && self.aggregate_callable_facts(info.ty).scoped).then_some((binding, id))
        }).collect::<HashMap<_, _>>();
        if plans.is_empty() { return Ok(()); }
        let mut constructors = HashMap::new();
        for write in record_storage_writes::collect(body) {
            if write.path.is_empty() && matches!(write.value.kind, ExprKind::MapConstructor(_, _))
                && let Some(id) = plans.get(&write.binding)
            { constructors.insert((write.span.start, write.span.end), *id); }
        }
        let mut results = HashMap::new();
        let writes = record_storage_writes::collect(body);
        loop {
            let mut changed = false;
            for write in &writes {
                if !write.path.is_empty() { continue; }
                if let Some(ty) = self.planned_scoped_map_result(write.value, &plans, &results)
                    && results.get(&write.binding) != Some(&ty)
                { results.insert(write.binding, ty); changed = true; }
            }
            if !changed { break; }
        }
        for (binding, ty) in &results { context.bindings[*binding].ty = *ty; }
        crate::source_walk::body(body, &mut |_| Ok(()), &mut |value| {
            if let Some(ty) = self.planned_scoped_map_result(value, &plans, &results) { value.ty = Some(ty); }
            if let ExprKind::Project { value: inner, union, narrowed } = &mut value.kind
                && matches!(narrowed, Type::Record(_))
                && let Some(ty) = self.planned_scoped_map_result(inner, &plans, &results)
                && let Some(id) = ty.union()
                && let Some(payload) = self.unions.payload(ty.owning())
            { *union = id; *narrowed = payload; value.ty = Some(payload); }
            if let ExprKind::Call(callee, _) = &mut value.kind
                && let Some(Type::BorrowedCallable(family)) = self.planned_scoped_map_result(callee, &plans, &results)
            { callee.ty = Some(Type::BorrowedCallable(family)); value.target = Some(CallTarget::StoredCallableField(family)); }
            if let Some(id) = constructors.get(&(value.span.start, value.span.end)) { value.ty = Some(Type::Map(*id)); }
            if matches!(value.kind, ExprKind::Name(_)) && let Some(id) = value.binding.and_then(|binding| plans.get(&binding)) { value.ty = Some(Type::Map(*id)); }
            if let ExprKind::Call(callee, arguments) = &mut value.kind
                && let ExprKind::Field(base, _) = &mut callee.kind
                && let Some(id) = base.binding.and_then(|binding| plans.get(&binding)).copied()
            {
                base.ty = Some(Type::Map(id));
                let target = self.program.records[id].map_definition.as_ref().unwrap().value;
                if matches!(value.target, Some(CallTarget::MapMethod(MapMethod::Set)))
                    && arguments[1].ty != Some(target)
                { arguments[1].wrap_record_carrier(target); }
            }
            Ok(())
        })
    }
    fn planned_scoped_map_result(&self, value: &Expr, maps: &HashMap<usize, usize>, results: &HashMap<usize, Type>) -> Option<Type> {
        match &value.kind {
            ExprKind::Name(_) => results.get(&value.binding?).copied(),
            ExprKind::Call(callee, _) if matches!(value.target, Some(CallTarget::MapMethod(MapMethod::Get))) => {
                let ExprKind::Field(base, _) = &callee.kind else { return None; };
                let id = maps.get(&base.binding?)?;
                let payload = self.program.records[*id].map_definition.as_ref()?.value;
                let result = self.unions.intern([payload, Type::Undefined], value.span).ok()?;
                if matches!(value.ty, Some(Type::BorrowedUnion(_))) { Some(Type::BorrowedUnion(result.union()?)) } else { Some(result) }
            }
            ExprKind::Project { value: inner, narrowed, .. } if matches!(narrowed, Type::Record(_)) => self.unions.payload(self.planned_scoped_map_result(inner, maps, results)?.owning()),
            ExprKind::Field(base, name) => {
                let Type::Record(id) = self.planned_scoped_map_result(base, maps, results)? else { return None; };
                self.program.records[id].fields.iter().find(|field| field.name.text == *name)?.ty
            }
            _ => None,
        }
    }

}
