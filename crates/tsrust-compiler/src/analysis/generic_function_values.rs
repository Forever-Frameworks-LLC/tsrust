//! Contextual generic values retain the original template call as their checking boundary.
use super::*;

pub(crate) fn parameter(
    name: &Token,
    annotation: ParameterAnnotation,
    variables: &[crate::type_syntax::TypeName],
) -> ParameterAnnotation {
    let value = match &annotation {
        ParameterAnnotation::Named(value) => value.as_ref().clone(),
        ParameterAnnotation::Callback(value) => value.value_annotation(),
        ParameterAnnotation::Inferred(_) => return annotation,
    };
    let mut pending = vec![&value];
    while let Some(value) = pending.pop() {
        use crate::type_syntax::ValueAnnotationKind as Kind;
        match &value.kind {
            Kind::Named(ty) if variables.iter().any(|variable| variable.text == ty.text) => {
                return ParameterAnnotation::Inferred(name.clone());
            }
            Kind::Literal(_) | Kind::Named(_) | Kind::NativeCallable { .. } | Kind::Polymorphic { .. } => {}
            Kind::Array(value) | Kind::Group(value) => pending.push(value),
            Kind::Applied { arguments, .. } | Kind::Union(arguments) | Kind::Tuple(arguments) => pending.extend(arguments),
            Kind::Callable { parameters, result, .. } => {
                pending.push(result);
                pending.extend(parameters.iter().map(|parameter| &parameter.annotation));
            }
        }
    }
    annotation
}

impl Analyzer {
    #[inline(never)]
    pub(super) fn prepare_contextual_generic_value(&mut self, expression: &Expr, mode: Mode, expected: Type) -> Result<continuations::adapters::Request, Diagnostic> {
        let ExprKind::Name(name) = &expression.kind else { unreachable!() };
        let template = self.generic_functions.template(name).expect("generic value template").clone();
        let (parameters, result) = self.checked_generic_callable_shape(self.unions.payload(expected).unwrap_or(expected)).expect("contextual generic contract");
        let hints = parameters.iter().copied().map(Some).collect::<Vec<_>>();
        let shapes = self.checked_inference_callable_shapes(parameters.iter().copied().chain([result]));
        let annotations = self.checked_inference_callable_annotations(parameters.iter().copied().chain([result]))?;
        let types = crate::generic_inference::Types { callable_annotations: Some(&annotations),
            callable_shapes: Some(&shapes), inline_shapes: None, records: &self.program.records,
            names: &self.record_names, unions: &self.unions,
        };
        let contract = self.generic_functions.value_signature(name, &hints, Some(result), &types, expression.span)?;
        // Capture-free template values need no lexical environment. Captured
        // templates are wrapped during lexical resolution, before escape analysis.
        let mut value = crate::local_function_values::value(&template, usize::MAX - expression.span.start, expression.span);
        if let ExprKind::Closure { params, result, .. } = &mut value.kind {
            debug_assert_eq!(contract.type_arguments.len(), template.type_parameters.len());
            if let Some(solved) = contract.result { *result = solved; }
            for (parameter, solved) in params.iter_mut().zip(contract.parameters) {
                debug_assert_eq!(parameter.rest, solved.rest);
                debug_assert_eq!(parameter.initializer.omission().is_omittable(), solved.omission.is_omittable());
                if matches!(parameter.ty, ParameterAnnotation::Inferred(_)) {
                    parameter.ty = ParameterAnnotation::Named(Box::new(solved.annotation));
                }
            }
        }
        Ok(continuations::adapters::Request { value: Box::new(value), mode, expected: Some(expected), cleanup: continuations::adapters::Cleanup::None, restore_on_error: false })
    }

}

#[cfg(test)]
mod nested_task_value_shape_tests {
 use super::*;
 #[test]
 fn contextual_generic_async_value_has_every_nested_callable_shape(){
  let source="async function observe<T>(action:Once<Function<Task<T>>>):Task<void>{try{await action();}catch{}}function main(){const stored:Function<Task<void>,[action:Once<Function<Task<int>>>]>=observe;}";
  let program=crate::syntax::parse(source).unwrap();
  let annotation=program.functions.iter().find(|function|function.name.text=="main").unwrap().body.iter().find_map(|statement|if let StmtKind::Let{annotation:Some(annotation),..}=&statement.kind{Some(annotation.clone())}else{None}).expect("contextual generic declaration");
  let mut analyzer=Analyzer::new(program).unwrap();
  let expected=analyzer.callable_resolver().resolve_value_annotation(&annotation,false).unwrap();
  let (parameters,result)=analyzer.checked_generic_callable_shape(expected).expect("declared stored signature");
  let callback=parameters[0];
  let (_,task)=analyzer.checked_generic_callable_shape(callback).expect("declared Once input");
  let shapes=analyzer.checked_inference_callable_shapes(parameters.iter().copied().chain([result]));
  assert!(shapes.contains_key(&callback),"missing callback {callback:?}: {shapes:?}");
  assert!(shapes.contains_key(&task),"missing nested task {task:?}: {shapes:?}");
  let annotations=analyzer.checked_inference_callable_annotations(parameters.iter().copied().chain([result])).unwrap();
  let types=crate::generic_inference::Types{callable_annotations:Some(&annotations),callable_shapes:Some(&shapes),inline_shapes:None,records:&analyzer.program.records,names:&analyzer.record_names,unions:&analyzer.unions};
  let hints=parameters.iter().copied().map(Some).collect::<Vec<_>>();
  analyzer.generic_functions.value_signature("observe",&hints,Some(result),&types,annotation.span)
   .unwrap_or_else(|error|panic!("expected={expected:?}; callback={callback:?}; nested task={task:?}; result={result:?}; shapes={shapes:?}; error={error:?}"));
 }
}
