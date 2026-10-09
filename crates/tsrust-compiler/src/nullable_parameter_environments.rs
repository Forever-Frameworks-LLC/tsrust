//! Each nullable parameter leaf binds its own callback environment variables.
use super::*;
impl Analyzer {
    pub(super) fn nullable_parameter_environment(&mut self, ty: Type, span: Span) -> Result<Type, Diagnostic> {
        self.nullable_parameter_environment_inner(ty, false, span, &mut HashSet::new())
    }
    fn nullable_parameter_environment_inner(&mut self, ty: Type, nested: bool, span: Span, active: &mut HashSet<Type>) -> Result<Type, Diagnostic> {
        if !active.insert(ty) { return Ok(ty); }
        let result = match ty {
            Type::Callback(id) if nested => {
                let contract = &self.callables.callbacks[id];
                let contract = CallbackContract { structural: contract.structural, parameter_names: contract.parameter_names.clone(), params: contract.params.clone(), result: contract.result, kind: contract.kind };
                let fresh = self.callables.callbacks.len();
                self.callables.callbacks.push(contract);
                Type::Callback(fresh)
            }
            Type::Record(record) => {
                let original = self.program.records[record].fields.iter().map(|field|field.ty.unwrap()).collect::<Vec<_>>();
                let mut fields = Vec::with_capacity(original.len());
                for &field in &original { fields.push(self.nullable_parameter_environment_inner(field, nested, span, active)?); }
                if fields == original { ty } else {
                    let instance = self.scoped_records.materialize(record, fields, &mut self.program.records);
                    self.scoped_records.mark_parameter_contract(instance);
                    if self.fields.len() <= instance { self.fields.push(self.program.records[instance].fields.iter().map(|field|(field.name.text.clone(),field.ty.unwrap())).collect()); }
                    Type::Record(instance)
                }
            }
            Type::Union(id) | Type::BorrowedUnion(id) => {
                let nested = nested || self.nullable_scoped_record_declaration(ty.owning()).is_some();
                let mut members = Vec::new();
                for member in self.unions.members(id) { members.push(self.nullable_parameter_environment_inner(member,nested,span,active)?); }
                let result = self.unions.intern(members,span)?;
                if matches!(ty,Type::BorrowedUnion(_)) { Type::BorrowedUnion(result.union().expect("nullable parameter")) } else { result }
            }
            Type::Map(id) => {
                let original=self.program.records[id].map_definition.as_ref().expect("map parameter").value;
                let nested=nested || self.aggregate_callable_facts(original).scoped;
                let value=self.nullable_parameter_environment_inner(original,nested,span,active)?;
                if value==original {ty} else {Type::Map(self.scoped_map_value_instance(id,value))}
            }
            Type::Array(element) => {
                let nested=nested || self.aggregate_callable_facts(element.ty()).scoped;
                let element_ty = self.nullable_parameter_environment_inner(element.ty(),nested,span,active)?;
                Type::Array(ArrayElement::from_value_type(element_ty).expect("existing array element remains representable"))
            }
            _ => ty,
        };
        active.remove(&ty);
        Ok(result)
    }
}
