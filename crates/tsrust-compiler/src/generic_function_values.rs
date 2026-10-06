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
            Kind::Named(_) => {}
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
    pub(super) fn contextual_generic_value(
        &mut self,
        context: &mut Context,
        expression: &mut Expr,
        mode: Mode,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        let ExprKind::Name(name) = &expression.kind else { unreachable!() };
        let template = self.generic_functions.template(name).expect("generic value template").clone();
        if expected.and_then(|ty| self.checked_generic_callable_shape(self.unions.payload(ty).unwrap_or(ty))).is_none() {
            return Err(self.generic_value_pending(name, expression.span));
        }
        // Capture-free template values need no lexical environment. Captured
        // templates are wrapped during lexical resolution, before escape analysis.
        *expression = crate::local_function_values::value(&template, usize::MAX - expression.span.start, expression.span);
        self.expression(context, expression, mode, loans, expected)
    }

    pub(super) fn generic_value_pending(&self, name: &str, span: Span) -> Diagnostic {
        let written = self.program.local_declarations.iter()
            .find(|declaration| declaration.emitted_name == name)
            .map_or(name, |declaration| declaration.source_name.text.as_str());
        Diagnostic::new("unsupported", format!("The generic function {written:?} needs a concrete callable context when used as a value. Unannotated polymorphic values are not implemented yet."), span)
    }
}
