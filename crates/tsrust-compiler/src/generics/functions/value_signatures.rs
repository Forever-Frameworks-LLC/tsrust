//! Concrete generic value contracts precede environment and method construction.
use super::*;

/// A solved source contract has no runtime environment or checked body.
pub(crate) struct ValueParameter {
    pub annotation: ValueAnnotation,
    pub omission: crate::ParameterOmission,
    pub rest: bool,
}

pub(crate) struct ValueSignature {
    pub type_arguments: Vec<Type>,
    pub parameters: Vec<ValueParameter>,
    pub result: Option<ResultAnnotation>,
}

impl Cache {
    /// Solve the original declaration before choosing a value invocation method.
    /// The hints describe supplied fixed slots and an already formed rest pack.
    /// This does not evaluate arguments, allocate instances or copy captures.
    pub(crate) fn value_signature(
        &self,
        name: &str,
        hints: &[Option<Type>],
        expected: Option<Type>,
        types: &crate::generic_inference::Types<'_>,
        span: Span,
    ) -> Result<ValueSignature, Diagnostic> {
        let template = &self.templates[name];
        let type_arguments = self.infer(name, hints, expected, types, span)?;
        let replacements = template.type_parameters.iter().zip(&type_arguments)
            .map(|(variable, ty)| Ok((variable.text.clone(), types.annotation(*ty, span)?)))
            .collect::<Result<HashMap<_, _>, Diagnostic>>()?;
        let parameters = template.params.iter().enumerate().map(|(position, (_, annotation, _))| {
            let mut annotation = annotation.clone();
            crate::type_substitution::apply(&mut annotation, &replacements)?;
            Ok(ValueParameter {
                annotation,
                omission: template.parameter_initializers[position].omission(),
                rest: template.rest_parameter && position + 1 == template.params.len(),
            })
        }).collect::<Result<Vec<_>, Diagnostic>>()?;
        let mut result = template.result.clone();
        if let Some(result) = &mut result {
            crate::source_walk::result_annotation(result, &mut |annotation| crate::type_substitution::apply(annotation, &replacements))?;
        }
        Ok(ValueSignature { type_arguments, parameters, result })
    }

}

#[cfg(test)]
mod value_contract_tests {
    use super::*;

    #[test]
    fn value_signature_solves_arrays_and_preserves_default_rest_contracts() {
        let source = "function first<T>(fallback:T,offset:int=0,...values:T[]):T{return fallback;}function main(){}";
        let mut program = crate::syntax::parse(source).unwrap();
        let cache = specialize(&mut program).unwrap();
        let names = HashMap::new();
        let unions = crate::unions::UnionArena::default();
        let types = crate::generic_inference::Types { callable_annotations: None, callable_shapes:None, inline_shapes:None, records:&[], names:&names, unions:&unions };
        let array = Type::Array(crate::syntax::ArrayElement::Int(0));
        let optional_offset = unions.intern([Type::Int, Type::Undefined], Span::new(0,0)).unwrap();
        let contract = cache.value_signature("first", &[Some(Type::Int),Some(optional_offset),Some(array)], Some(Type::Int), &types, Span::new(0,0)).unwrap();
        assert_eq!(contract.type_arguments, vec![Type::Int]);
        assert_eq!(contract.parameters.len(), 3);
        assert_eq!(contract.parameters[1].omission, crate::ParameterOmission::Defaulted);
        assert!(contract.parameters[2].rest);
        assert_eq!(types.contextual_type(&contract.parameters[2].annotation, &HashMap::new()), Some(array));
        assert_eq!(types.contextual_type(match contract.result.as_ref().unwrap() { ResultAnnotation::Named(value) => value, _ => panic!("named result") }, &HashMap::new()), Some(Type::Int));
        assert!(cache.checked_instances.is_empty(), "signature solving does not allocate runtime environments or instances");
        assert!(cache.value_signature("first", &[Some(Type::String),Some(Type::Int),Some(array)], None, &types, Span::new(0,0)).is_err());
        let nullable_offset = unions.intern([Type::Int, Type::Null, Type::Undefined], Span::new(0,0)).unwrap();
        assert!(cache.value_signature("first", &[Some(Type::Int),Some(nullable_offset),Some(array)], None, &types, Span::new(0,0)).is_err(), "null remains a real supplied constraint");
    }

    #[test]
    fn value_signature_infers_from_result_and_rejects_inconsistent_constraints() {
        let mut program = crate::syntax::parse("function empty<T>():T[]{return [];}function pair<T>(left:T,right:T):T{return left;}function main(){}").unwrap();
        let cache = specialize(&mut program).unwrap();
        let names = HashMap::new();
        let unions = crate::unions::UnionArena::default();
        let types = crate::generic_inference::Types { callable_annotations: None, callable_shapes:None, inline_shapes:None, records:&[], names:&names, unions:&unions };
        let contract = cache.value_signature("empty", &[], Some(Type::Array(crate::syntax::ArrayElement::Int(0))), &types, Span::new(0,0)).unwrap();
        assert_eq!(contract.type_arguments, vec![Type::Int]);
        assert!(cache.value_signature("pair", &[Some(Type::Int),Some(Type::String)], None, &types, Span::new(0,0)).is_err());
        assert!(cache.value_signature("pair", &[Some(Type::Int)], None, &types, Span::new(0,0)).is_err());
    }
}
