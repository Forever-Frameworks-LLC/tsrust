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
            Kind::Literal(_) | Kind::Named(_) | Kind::Polymorphic { .. } => {}
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
