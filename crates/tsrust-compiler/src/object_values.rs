//! `Object.values` and `Object.entries`: owning snapshots of a borrowed
//! record, tuple or array. A record with several field types yields a union
//! element; each value keeps the alternative of its field.
use super::*;

impl Analyzer {
    #[inline(never)]
    pub(super) fn object_values(
        &mut self,
        context: &mut Context,
        args: &mut [Expr],
        loans: &Loans,
        expected: Option<Type>,
        span: Span,
        entries: bool,
    ) -> Result<(Type, Option<usize>), Diagnostic> {
        let method = if entries { "Object.entries" } else { "Object.values" };
        if args.len() != 1 {
            return Err(Diagnostic::new("arity", format!("{method} expects one value."), span));
        }
        let prior = self.builtin_literals;
        self.builtin_literals = true;
        let actual = self.expression(context, &mut args[0], Mode::Read, loans, None);
        self.builtin_literals = prior;
        let element = match actual? {
            Type::Record(id) => {
                let types = self.program.records[id]
                    .fields
                    .iter()
                    .map(|field| field.ty.expect("checked record field"))
                    .collect::<Vec<_>>();
                if types.is_empty() {
                    return Err(Diagnostic::new(
                        "unsupported",
                        format!("{method} needs a record with at least one field; an empty record gives no element type."),
                        args[0].span,
                    ));
                }
                if types.iter().any(|ty| {
                    matches!(
                        self.unions.payload(*ty).unwrap_or(*ty),
                        Type::Closure(_) | Type::Callable(_) | Type::Callback(_) | Type::StoredCallable(_) | Type::Void
                    )
                }) {
                    return Err(Diagnostic::new(
                        "unsupported",
                        format!("{method} cannot copy a record that has a function field; read the data fields by name."),
                        args[0].span,
                    ));
                }
                self.require_cloneable(Type::Record(id), args[0].span)?;
                self.unions.intern(types, span)?
            }
            Type::Array(element) if !entries => {
                self.require_cloneable(Type::Array(element), args[0].span)?;
                let result = Type::Array(element);
                self.require(result, expected, span)?;
                return Ok((result, None));
            }
            Type::Array(_) => {
                return Err(Diagnostic::new(
                    "unsupported",
                    "Object.entries on an array is not available; use array.entries() for [index, value] pairs with int positions.",
                    args[0].span,
                ));
            }
            _ => {
                return Err(Diagnostic::new(
                    "unsupported",
                    format!("{method} accepts records, tuples{}; strings, numbers, booleans, nullable values and functions are unsupported.", if entries { "" } else { " and arrays" }),
                    args[0].span,
                ));
            }
        };
        let Some(leaf) = ArrayElement::from_value_type(element) else {
            return Err(Diagnostic::new(
                "unsupported",
                format!("{method} cannot form an array of these field types."),
                args[0].span,
            ));
        };
        if !entries {
            let result = Type::Array(leaf);
            self.require(result, expected, span)?;
            return Ok((result, None));
        }
        let types = crate::generic_inference::Types { callable_annotations: None,
            callable_shapes: None,
            inline_shapes: None,
            records: &self.program.records,
            names: &self.record_names,
            unions: &self.unions,
        };
        let mut annotation = crate::type_syntax::ValueAnnotation {
            kind: crate::type_syntax::ValueAnnotationKind::Tuple(vec![
                types.annotation(Type::String, span)?,
                types.annotation(element, span)?,
            ]),
            span,
        };
        self.generic_records.materialize(&mut annotation, &mut self.program.records, &mut self.record_names, &self.unions, &mut self.callables)?;
        append_checked_record_fields(&mut self.program.records, &self.record_names, &self.unions, &mut self.fields, &mut self.callables)?;
        let Type::Record(pair) = self.resolve_value_annotation(&annotation, false)? else {
            unreachable!("materialized tuple")
        };
        let result = Type::Array(ArrayElement::Record(pair, 0));
        self.require(result, expected, span)?;
        Ok((result, Some(pair)))
    }
}
