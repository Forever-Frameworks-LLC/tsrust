//! Data annotation resolution uses postorder heap continuations.
use super::*;

pub(super) fn resolve(root: &crate::type_syntax::ValueAnnotation, allow_void: bool, names: &HashMap<String, usize>, unions: &crate::unions::UnionArena) -> Result<Type, Diagnostic> {
    use crate::type_syntax::{ValueAnnotation, ValueAnnotationKind as Kind};
    enum Work<'a> { Value(&'a ValueAnnotation, bool), Union(Span, usize), Array(Span, u8) }
    let mut pending = vec![Work::Value(root, allow_void)];
    let mut results = Vec::new();
    let array_error = |span| Diagnostic::new("array-element", "Array leaves must be int, number, boolean, string, or a named record.", span);
    while let Some(work) = pending.pop() {
        match work {
            Work::Union(span, start) => {
                let members = results.drain(start..).collect::<Vec<_>>();
                results.push(unions.intern(members, span)?);
            }
            Work::Array(span, depth) => {
                let ty = results.pop().expect("resolved array leaf");
                let element = match ty {
                    Type::Union(id) => Some(ArrayElement::Union(id, depth)),
                    single => ArrayElement::from_value_type(single).map(|leaf| leaf.with_depth(depth)),
                };
                results.push(Type::Array(element.ok_or_else(|| array_error(span))?));
            }
            Work::Value(value, allow_void) => match &value.kind {
                Kind::Group(inner) => pending.push(Work::Value(inner, allow_void)),
                Kind::Union(members) => {
                    pending.push(Work::Union(value.span, results.len()));
                    pending.extend(members.iter().rev().map(|member| Work::Value(member, true)));
                }
                Kind::Array(_) => {
                    let (inner, dimensions) = crate::callable_annotations::storage_leaf(value);
                    if dimensions > 128 {
                        return Err(Diagnostic::new("unsupported", "Array type nesting exceeds the prototype limit of 128.", value.span));
                    }
                    let depth = (dimensions - 1) as u8;
                    match &inner.kind {
                        Kind::Literal(_) | Kind::Union(_) => {
                            pending.push(Work::Array(value.span, depth));
                            pending.push(Work::Value(inner, false));
                        }
                        Kind::Named(name) => {
                            let element = match name.text.as_str() {
                                "int" => Some(ArrayElement::Int(depth)),
                                "number" => Some(ArrayElement::Number(depth)),
                                "boolean" => Some(ArrayElement::Boolean(depth)),
                                "string" => Some(ArrayElement::String(depth)),
                                "null" => Some(ArrayElement::Null(depth)),
                                "undefined" => Some(ArrayElement::Undefined(depth)),
                                _ => names.get(&name.text).copied().and_then(|id| ArrayElement::from_value_type(unions.named_type(id))).map(|leaf| leaf.with_depth(depth)),
                            };
                            results.push(Type::Array(element.ok_or_else(|| array_error(value.span))?));
                        }
                        Kind::Applied { .. } => return Err(Diagnostic::new("unsupported", "Generic type applications are not yet supported by the type checker.", inner.span)),
                        _ => return Err(array_error(value.span)),
                    }
                }
                _ => results.push(leaf(value, allow_void, names, unions)?),
            }
        }
    }
    Ok(results.pop().expect("resolved data annotation"))
}

fn leaf(
    annotation: &crate::type_syntax::ValueAnnotation,
    allow_void: bool,
    record_names: &HashMap<String, usize>,
    unions: &crate::unions::UnionArena,
) -> Result<Type, Diagnostic> {
    use crate::type_syntax::ValueAnnotationKind as Kind;
    match &annotation.kind {
        Kind::Literal(value) => {
            let literal=crate::literal_types::LiteralValue::checked(value,annotation.span)?;
            record_names.get(&literal.key()).map(|&id|Type::Literal(id,literal.kind())).ok_or_else(||Diagnostic::new("literal-type","Literal type identity has not been materialized.",annotation.span))
        },
        Kind::Polymorphic { origin, .. } => Ok(Type::Closure(*origin)),
        Kind::Group(_) | Kind::Union(_) | Kind::Array(_) => unreachable!("scheduled data annotation"),
        Kind::Tuple(_) => Err(Diagnostic::new("type", "Tuple annotation has not been materialized.", annotation.span)),
        Kind::Applied { .. } => Err(Diagnostic::new(
            "unsupported",
            "Generic type applications are not yet supported by the type checker.",
            annotation.span,
        )),
        Kind::Callable { .. } => Err(Diagnostic::new(
            "unsupported",
            "Callable value annotations are not yet supported in this position.",
            annotation.span,
        )),
        Kind::Named(name) => match name.text.as_str() {
            "int" => Ok(Type::Int),
            "number" => Ok(Type::Number),
            "boolean" => Ok(Type::Boolean),
            "string" => Ok(Type::String),
            "null" => Ok(Type::Null),
            "undefined" => Ok(Type::Undefined),
            "void" if allow_void => Ok(Type::Void),
            _ => record_names
                .get(&name.text)
                .copied()
                .map(|id| unions.named_type(id))
                .ok_or_else(|| {
                    Diagnostic::new(
                        "type",
                        match name.text.as_str() {
                            "Function" => "Function needs a result type, such as Function<int> or Function<int, [string]>.".to_owned(),
                            "Mutable" | "ReadOnly" | "Once" | "Owned" => format!("{} wraps one type, such as {}<Function<int>>.", name.text, name.text),
                            _ => format!("Unknown or unsupported type {:?}.", name.text),
                        },
                        name.span,
                    )
                }),
        },
    }
}

