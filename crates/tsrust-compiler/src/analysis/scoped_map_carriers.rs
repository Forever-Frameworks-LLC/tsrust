//! Local Map values preserve concrete borrowed record environments.
use super::*;

impl Analyzer {


    pub(super) fn contains_scoped_map_storage(&self, ty: Type) -> bool {
        let mut pending = vec![ty];
        let mut seen = HashSet::new();
        while let Some(ty) = pending.pop() {
            if !seen.insert(ty) { continue; }
            match ty {
                Type::Map(id) if self.aggregate_callable_facts(ty).scoped
                    || matches!(self.program.records[id].map_definition.as_ref().map(|map|map.value),Some(Type::Record(record)) if self.absent_record_environment(record)) => return true,
                Type::Array(element) => pending.push(element.ty()),
                Type::Record(id) => pending.extend(self.program.records[id].fields.iter().filter_map(|field| field.ty)),
                _ => {}
            }
        }
        false
    }
    pub(super) fn scoped_map_value_instance(&mut self, id: usize, value: Type) -> usize {
        let key = self.program.records[id].map_definition.as_ref().unwrap().key;
        let family=self.program.records[id].native_type;
        if let Some(found) = self.program.records.iter().position(|record| record.native_type==family && record.map_definition.as_ref().is_some_and(|map| map.key == key && map.value == value && map.entry.is_none())) { return found; }
        let mut record = self.program.records[id].clone();
        let map = record.map_definition.as_mut().unwrap();
        map.value = value;
        map.entry = None;
        if let Some(application)=&mut record.application {application.resolved_arguments=vec![key,value];}
        let instance = self.program.records.len();
        self.program.records.push(record);
        self.fields.push(HashMap::new());
        self.unions.register_map_type(instance);
        instance
    }

    pub(super) fn scoped_map_contract(&self, actual: Type, expected: Type) -> bool {
        let (Type::Map(actual), Type::Map(expected)) = (actual, expected) else { return false; };
        if self.program.records[actual].native_type!=self.program.records[expected].native_type{return false;}
        let (Some(actual), Some(expected)) = (self.program.records[actual].map_definition.as_ref(), self.program.records[expected].map_definition.as_ref()) else { return false; };
        let (Type::Record(value), Type::Record(contract)) = (actual.value, expected.value) else { return false; };
        actual.key == expected.key && self.aggregate_callable_facts(actual.value).scoped
            && value != contract && (self.scoped_records.declaration(value) == contract || self.scoped_records.parameter_contract(contract) && self.scoped_records.declaration(value) == self.scoped_records.declaration(contract))
    }

    #[inline(never)]
    pub(super) fn scoped_map_carriers(&mut self, context: &mut Context, body: &mut [Stmt]) -> Result<(), Diagnostic> {
        let mut roots = context.bindings.iter().enumerate().filter_map(|(binding, info)| {
            (!info.parameter && !info.capture_reference && self.contains_scoped_map_storage(info.ty)).then_some((binding, info.ty))
        }).collect::<HashMap<_, _>>();
        if roots.is_empty() { return Ok(()); }
        let local_declarations=empty_scoped_maps::declared_storage_bindings(body);
        crate::source_walk::body(body, &mut |_| Ok(()), &mut |value| {
            if !matches!(value.target, Some(CallTarget::MapMethod(MapMethod::Set))) { return Ok(()); }
            let ExprKind::Call(callee, arguments) = &value.kind else { return Ok(()); };
            let ExprKind::Field(base, _) = &callee.kind else { return Ok(()); };
            let Some(binding) = Self::root_binding(base) else { return Ok(()); };
            let Some((_, path)) = Self::scoped_array_storage_path(base) else { return Ok(()); };
            let Some(root) = roots.get(&binding).copied() else { return Ok(()); };
            let Some(Type::Map(id)) = self.scoped_array_field_type(root, &path) else { return Ok(()); };
            let previous = self.program.records[id].map_definition.as_ref().unwrap().value;
            let incoming=context.storage_exits.map_entry_write(value.span).or(arguments[1].ty);
            let (Type::Record(previous), Some(Type::Record(incoming))) = (previous, incoming) else { return Ok(()); };
            let joined = if previous == incoming { Type::Record(previous) } else {
                self.join_record_types(previous, incoming, value.span)?.ok_or_else(|| Diagnostic::new("type", "Map insertion environments have incompatible contracts.", value.span))?
            };
            let map = Type::Map(self.scoped_map_value_instance(id, joined));
            let Some(ty) = self.scoped_array_replace_field(root, &path, map) else { return Ok(()); };
            roots.insert(binding, ty);
            Ok(())
        })?;
        let writes = record_storage_writes::collect(body);
        // Record literal fields and direct moves share the same storage edges.
        let mut leaves = Vec::new();
        for write in &writes {
            let mut pending = vec![(write.path.clone(), write.value)];
            while let Some((path, value)) = pending.pop() {
                if let ExprKind::Unary { op: UnaryOp::RecordCarrier, value: inner, .. } = &value.kind {
                    pending.push((path, inner));
                } else if let ExprKind::Array(elements) = &value.kind {
                    for element in elements { pending.push((path.clone(), element)); }
                } else if let ExprKind::Record(fields) = &value.kind {
                    for (name, field) in fields { let mut nested = path.clone(); nested.push(name.text.clone()); pending.push((nested, field)); }
                } else if matches!(value.ty, Some(Type::Map(_))) {
                    leaves.push((write.binding, path, value));
                }
            }
        }
        let transfers=self.map_storage_edges(&writes,&roots);
        loop {
            let mut changed = false;
            for edge in &transfers {
                let source_binding=edge.source;
                let destination=&edge.destination;
                let source_path=&edge.source_path;
                let path=&edge.destination_path;
                let (Some(source_ty),Some(destination_ty))=(roots.get(&source_binding),roots.get(destination)) else {continue;};
                let (Some(Type::Map(source_id)),Some(Type::Map(destination_id)))=(self.scoped_array_field_type(*source_ty,source_path),self.scoped_array_field_type(*destination_ty,path)) else {continue;};
                let source_value = self.program.records[source_id].map_definition.as_ref().unwrap().value;
                let destination_value = self.program.records[destination_id].map_definition.as_ref().unwrap().value;
                let (Type::Record(left), Type::Record(right)) = (source_value, destination_value) else { continue; };
                let joined = if left == right { source_value }
                    else if left == self.scoped_records.declaration(right) { destination_value }
                    else if right == self.scoped_records.declaration(left) { source_value }
                    else { self.join_record_types(left, right, edge.span)?.ok_or_else(|| Diagnostic::new("type", "Moved Map callback environments have incompatible contracts.", edge.span))? };
                for (binding, path, id) in [(source_binding, source_path.as_slice(), source_id), (*destination, path.as_slice(), destination_id)] {
                    let next = Type::Map(self.scoped_map_value_instance(id, joined));
                    let root = roots[&binding];
                    let Some(next) = self.scoped_array_replace_field(root, path, next) else { continue; };
                    if root != next { roots.insert(binding, next); changed = true; }
                }
            }
            if !changed { break; }
        }
        for (binding,ty) in &mut roots {
            if local_declarations.contains(binding) {*ty=self.empty_scoped_map_layout(*ty)?;}
        }
        for (binding, ty) in &roots { context.bindings[*binding].ty = *ty; }
        let mut constructors = HashMap::new();
        for (binding, path, value) in &leaves {
            if matches!(value.kind, ExprKind::MapConstructor(_, _) | ExprKind::NativeConstructor(_, _))
                && let Some(ty) = roots.get(binding).and_then(|root| self.scoped_array_field_type(*root, path))
            { constructors.insert((value.span.start, value.span.end), ty); }
        }
        let mut results = roots.clone();
        self.empty_map_iteration_carriers(context,body,&mut results)?;
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
        self.retarget_scoped_map_body(context,body,&results,&constructors)
    }
    pub(super) fn retarget_scoped_map_body(&mut self, context:&mut Context, body:&mut [Stmt], results:&HashMap<usize,Type>, constructors:&HashMap<(usize,usize),Type>) -> Result<(),Diagnostic> {
        crate::source_walk::body(body, &mut |_| Ok(()), &mut |value| {
            if let ExprKind::Unary { op: UnaryOp::RecordCarrier, value: inner, .. } = &value.kind
                && value.ty.is_some_and(|ty| self.contains_scoped_map_storage(ty))
            { value.ty = Some(self.record_value_carrier_type(context, inner, results)?); }
            if let Some(ty) = self.planned_scoped_map_result(value, results) { value.ty = Some(ty); }
            if let ExprKind::Project { value: inner, union, narrowed } = &mut value.kind
                && matches!(narrowed, Type::Record(_))
                && let Some(ty) = self.planned_scoped_map_result(inner, results)
                && let Some(id) = ty.union()
                && let Some(payload) = self.unions.payload(ty.owning())
            { *union = id; *narrowed = payload; value.ty = Some(payload); }
            if let ExprKind::Call(callee, _) = &mut value.kind
                && let Some(Type::BorrowedCallable(family)) = self.planned_scoped_map_result(callee, results)
            { callee.ty = Some(Type::BorrowedCallable(family)); value.target = Some(CallTarget::StoredCallableField(family)); }
            if matches!(value.target,Some(CallTarget::StoredCallableField(_)|CallTarget::ConditionalClosure(_)))
                && let ExprKind::Call(callee,_) = &mut value.kind
                && let Some(Type::Closure(origin))=self.planned_scoped_map_result(callee,results)
            {callee.ty=Some(Type::Closure(origin));value.target=Some(CallTarget::ConditionalClosure(origin));}
            if let ExprKind::Call(callee,_) = &mut value.kind
                && matches!(value.target,Some(CallTarget::ConditionalCallback(_)|CallTarget::ConditionalClosure(_)|CallTarget::StoredCallableField(_)))
                && let Some(Type::Callback(contract))=self.planned_scoped_map_result(callee,results)
                && self.absent_callable_type(Type::Callback(contract))
            {callee.ty=Some(Type::Callback(contract));value.target=Some(CallTarget::ConditionalCallback(contract));}
            if let Some(ty) = constructors.get(&(value.span.start, value.span.end)) { value.ty = Some(*ty); }
            if let ExprKind::Call(callee, arguments) = &mut value.kind
                && let ExprKind::Field(base, _) = &mut callee.kind
                && let Some(Type::Map(id)) = self.planned_scoped_map_result(base, results)
            {
                base.ty = Some(Type::Map(id));
                let target = self.program.records[id].map_definition.as_ref().unwrap().value;
                if matches!(value.target, Some(CallTarget::MapMethod(MapMethod::Set))) && arguments[1].ty != Some(target) { arguments[1].wrap_record_carrier(target); }
            }
            if matches!(value.target, Some(CallTarget::DequeMethod(DequeMethod::PushFront | DequeMethod::PushBack)))
                && let ExprKind::Call(callee, arguments) = &mut value.kind
                && let ExprKind::Field(base, _) = &mut callee.kind
                && let Some(Type::Record(id)) = self.planned_scoped_map_result(base, results)
                && let Some(target) = self.program.records[id].native_element()
            {
                base.ty = Some(Type::Record(id));
                if arguments[0].ty != Some(target) { arguments[0].wrap_record_carrier(target); }
            }
            if matches!(value.kind, ExprKind::Record(_)) { value.ty = Some(self.record_value_carrier_type(context, value, results)?); }
            Ok(())
        })
    }
    pub(super) fn planned_scoped_map_result(&self, value: &Expr, results: &HashMap<usize, Type>) -> Option<Type> {
        match &value.kind {
            ExprKind::Name(_) => results.get(&value.binding?).copied(),
            ExprKind::Call(callee, _) if matches!(value.target, Some(CallTarget::DequeMethod(DequeMethod::Front | DequeMethod::Back | DequeMethod::At | DequeMethod::PopFront | DequeMethod::PopBack))) => {
                let ExprKind::Field(base, _) = &callee.kind else { return None; };
                let Type::Record(id) = self.planned_scoped_map_result(base, results)? else { return None; };
                let payload = self.program.records[id].native_element()?;
                let result = self.unions.intern([payload, Type::Undefined], value.span).ok()?;
                if matches!(value.ty, Some(Type::BorrowedUnion(_))) { Some(Type::BorrowedUnion(result.union()?)) } else { Some(result) }
            }
            ExprKind::Call(callee, _) if matches!(value.target, Some(CallTarget::MapMethod(MapMethod::Get))) => {
                let ExprKind::Field(base, _) = &callee.kind else { return None; };
                let Type::Map(id) = self.planned_scoped_map_result(base, results)? else { return None; };
                let payload = self.program.records[id].map_definition.as_ref()?.value;
                let result = self.unions.intern([payload, Type::Undefined], value.span).ok()?;
                if matches!(value.ty, Some(Type::BorrowedUnion(_))) { Some(Type::BorrowedUnion(result.union()?)) } else { Some(result) }
            }
            ExprKind::Unary { op: UnaryOp::RecordCarrier | UnaryOp::NonNull, value: inner, .. } => self.planned_scoped_map_result(inner, results),
            ExprKind::Project { value: inner, narrowed, .. } if matches!(narrowed, Type::Record(_)) => self.unions.payload(self.planned_scoped_map_result(inner, results)?.owning()),
            ExprKind::Index(base, _) => match self.planned_scoped_map_result(base, results)? { Type::Array(element) => Some(element.ty()), _ => None },
            ExprKind::Field(base, name) => {
                let Type::Record(id) = self.planned_scoped_map_result(base, results)? else { return None; };
                self.program.records[id].fields.iter().find(|field| field.name.text == *name)?.ty
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod ordered_identity_tests {
    use super::*;
    #[test]
    fn ordered_map_value_specialization_keeps_its_family() {
        use crate::native_signatures::{NativeTypeId,Signature};
        let mut program=crate::syntax::parse("function main(){const first=new Map<int,int>();const second=new Map<int,string>();}").unwrap();
        let annotation=crate::native_type_resolution::project_with_parameters(&mut program.records,&Signature::Applied(NativeTypeId::OrderedMap,&[Signature::Parameter(0),Signature::Parameter(1)]),&[Signature::Int,Signature::Int],Span::default()).unwrap();
        program.type_argument_checks.push(annotation);
        let mut analyzer=Analyzer::new(program).unwrap();
        let ordered=analyzer.program.records.iter().position(|record|record.native_type==Some(NativeTypeId::OrderedMap)).unwrap();
        let ordinary=analyzer.program.records.iter().position(|record|record.native_type.is_none() && record.map_definition.as_ref().is_some_and(|map|map.value==Type::String)).unwrap();
        let specialized=analyzer.scoped_map_value_instance(ordered,Type::String);
        assert_ne!(specialized,ordinary);
        assert_eq!(analyzer.program.records[specialized].native_type,Some(NativeTypeId::OrderedMap));
        assert_eq!(analyzer.program.records[specialized].map_definition.as_ref().unwrap().value,Type::String);
        assert_eq!(analyzer.program.records[specialized].application.as_ref().unwrap().resolved_arguments,[Type::Int,Type::String]);
        assert_eq!(analyzer.scoped_map_value_instance(ordered,Type::String),specialized);
        assert_eq!(analyzer.unions.named_type(specialized),Type::Map(specialized));
        assert_eq!(analyzer.type_name(Type::Map(specialized)),"OrderedMap");
    }
}
