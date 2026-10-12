//! Explicit JSON decoding validates the requested owned shape before emission.
use super::*;
impl Analyzer {
    pub(super) fn json_typed_decode(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Option<Type>, Diagnostic> {
        let ExprKind::Call(callee, args) = &mut expr.kind else {
            return Ok(None);
        };
        if !matches!(&callee.kind,ExprKind::Name(name) if name=="@json:decode") {
            return Ok(None);
        }
        let span = self
            .program
            .functions
            .iter()
            .find(|function| function.name.text == context.function_name)
            .and_then(|function| function.application.as_ref())
            .and_then(|application| application.arguments.first())
            .map_or(expr.span, |argument| argument.span);
        let result = expected.ok_or_else(|| {
            Diagnostic::new(
                "json-type",
                "Typed JSON decoding requires a concrete result type.",
                span,
            )
        })?;
        let (_, variants) = self.result_variant_records(result).ok_or_else(|| {
            Diagnostic::new(
                "json-type",
                "Typed JSON decoding requires a Result context.",
                span,
            )
        })?;
        let ty = self.program.records[variants[0]]
            .fields
            .first()
            .and_then(|field| field.ty)
            .ok_or_else(|| {
                Diagnostic::new(
                    "json-type",
                    "Typed JSON decoding requires a value target; void is not a JSON value.",
                    span,
                )
            })?;
        let mut pending = vec![ty];
        let mut seen = std::collections::HashSet::new();
        while let Some(ty) = pending.pop() {
            if !seen.insert(ty) {
                continue;
            }
            match ty {
                Type::Int | Type::Number | Type::Boolean | Type::String | Type::Null => {}
                Type::Array(element) => pending.push(element.ty()),
                Type::Record(id)
                    if self.program.records[id].native_type.is_none()
                        && self.program.records[id].map_definition.is_none()
                        && self.program.records[id].set_definition.is_none() =>
                {
                    pending.extend(
                        self.program.records[id]
                            .fields
                            .iter()
                            .filter_map(|field| field.ty),
                    )
                }
                Type::Union(id) => {
                    let members = self.unions.members(id);
                    let values: Vec<_> = members
                        .into_iter()
                        .filter(|ty| !matches!(ty, Type::Null | Type::Undefined))
                        .collect();
                    if values.len() != 1 {
                        return Err(Diagnostic::new(
                            "json-type",
                            "Typed JSON decoding currently supports nullable or optional values, not unions of multiple value shapes.",
                            span,
                        ));
                    }
                    pending.extend(values);
                }
                _ => {
                    return Err(Diagnostic::new(
                        "json-type",
                        format!(
                            "Typed JSON decoding does not support {}.",
                            self.type_name(ty)
                        ),
                        span,
                    ));
                }
            }
        }
        if args.len() != 2 {
            return Err(Diagnostic::new(
                "arity",
                "Typed JSON decoding expects text and limits.",
                span,
            ));
        }
        let limits = self
            .program
            .records
            .iter()
            .position(|record| {
                record.native_type == Some(crate::native_signatures::NativeTypeId::JsonLimits)
            })
            .expect("prepared JSON limits");
        let contracts = [
            ArgumentContract {
                ty: Type::String,
                mode: Mode::Read,
                omission: crate::ParameterOmission::Required,
                rest: false,
            },
            ArgumentContract {
                ty: Type::Record(limits),
                mode: Mode::Read,
                omission: crate::ParameterOmission::Required,
                rest: false,
            },
        ];
        self.check_call_arguments(context, args, &contracts, loans)?;
        expr.target = Some(CallTarget::JsonDecode);
        expr.ty = Some(result);
        Ok(Some(result))
    }
}
