//! Constructor parsing lives outside the recursive primary-expression frame.
use super::*;

impl Parser<'_> {
    #[inline(never)]
    pub(super) fn set_constructor(&mut self, start: Token) -> Result<Expr, Diagnostic> {
        let name = self.name()?;
        if !matches!(name.text.as_str(), "Set" | "Map") {
            return Err(Diagnostic::new("unsupported", "Only the built-in Set and Map constructors are supported by new.", name.span));
        }
        let map = name.text == "Map";
        let annotation = if self.is("<") {
            let arguments = self.explicit_call_arguments()?;
            if arguments.len() != if map { 2 } else { 1 } {
                return Err(Diagnostic::new("arity", "Set requires one key type; Map requires key and value types.", name.span));
            }
            Some(Box::new(crate::type_syntax::ValueAnnotation {
                kind: crate::type_syntax::ValueAnnotationKind::Applied {
                    name: crate::type_syntax::TypeName { text: name.text, span: name.span },
                    arguments,
                },
                span: name.span,
            }))
        } else { None };
        self.expect("(")?;
        let (arguments, end) = self.call_arguments()?;
        Ok(Expr::new(if map { ExprKind::MapConstructor(arguments, annotation) } else { ExprKind::SetConstructor(arguments, annotation) }, Span::new(start.span.start, end.span.end)))
    }
}
