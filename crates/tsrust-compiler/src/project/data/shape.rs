//! Structural inference for validated data; no dynamic or unchecked fallback.
use crate::data_module::{Kind, Value};
use crate::Span;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Shape {
    Null, Boolean, Int, Number, String,
    Array(Option<Box<Shape>>),
    Object(BTreeMap<String, Field>),
    Nullable(Box<Shape>),
    Union(Vec<Shape>),
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Field { pub shape: Shape, pub optional: bool }
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Error { pub span: Span, pub message: &'static str }
pub(crate) fn infer(value: &Value) -> Result<Shape, Error> {
    let shape = draft(value)?;
    validate(value, &shape)?;
    Ok(shape)
}

fn error(span: Span, message: &'static str) -> Error { Error { span, message } }
fn draft(value: &Value) -> Result<Shape, Error> {
    Ok(match &value.kind {
        Kind::Null => Shape::Null,
        Kind::Boolean(_) => Shape::Boolean,
        Kind::String(_) => Shape::String,
        Kind::Number(text) => {
            if text == "-0" || text.contains(['.', 'e', 'E']) { Shape::Number }
            else {
                text.parse::<i64>().map_err(|_| error(value.span, "JSON integer is outside the native int range."))?;
                Shape::Int
            }
        }
        Kind::Object(fields) => Shape::Object(fields.iter().map(|(name, field)| {
            Ok((name.clone(), Field { shape: draft(&field.value)?, optional: false }))
        }).collect::<Result<_, Error>>()?),
        Kind::Array(values) => {
            let mut element: Option<Box<Shape>> = None;
            for value in values {
                let next = draft(value)?;
                element = Some(Box::new(match element {
                    None => next,
                    Some(previous) => join(*previous, next, value.span)?,
                }));
            }
            Shape::Array(element)
        }
    })
}

fn join(left: Shape, right: Shape, span: Span) -> Result<Shape, Error> {
    use Shape::*;
    if left == right { return Ok(left); }
    Ok(match (left, right) {
        (Null, Nullable(inner)) | (Nullable(inner), Null) => Nullable(inner),
        (Null, other) | (other, Null) => Nullable(Box::new(other)),
        (Nullable(left), Nullable(right)) => Nullable(Box::new(join(*left, *right, span)?)),
        (Nullable(inner), other) | (other, Nullable(inner)) => Nullable(Box::new(join(*inner, other, span)?)),
        (Int, Number) | (Number, Int) => Number,
        (Array(None), Array(element)) | (Array(element), Array(None)) => Array(element),
        (Array(Some(left)), Array(Some(right))) => Array(Some(Box::new(join(*left, *right, span)?))),
        (Object(mut left), Object(mut right)) => {
            for (name, field) in &mut left {
                if let Some(other) = right.remove(name) {
                    field.shape = join(field.shape.clone(), other.shape, span)?;
                    field.optional |= other.optional;
                } else { field.optional = true; }
            }
            for (name, mut field) in right { field.optional = true; left.insert(name, field); }
            Object(left)
        }
        (left, right) => {
            let mut alternatives = match left { Union(values) => values, other => vec![other] };
            let incoming = match right { Union(values) => values, other => vec![other] };
            for next in incoming {
                let compatible = alternatives.iter().position(|previous| {
                    std::mem::discriminant(previous) == std::mem::discriminant(&next)
                        || matches!((previous, &next), (Int, Number) | (Number, Int))
                });
                if let Some(index) = compatible {
                    alternatives[index] = join(alternatives[index].clone(), next, span)?;
                } else { alternatives.push(next); }
            }
            alternatives.sort();
            alternatives.dedup();
            if alternatives.len() == 1 { alternatives.pop().unwrap() } else { Union(alternatives) }
        },
    })
}

fn validate(value: &Value, shape: &Shape) -> Result<(), Error> {
    // Revisit every actual value after joins. Numeric widening and nested empty
    // arrays cannot be validated using only the final structural type.
    let shape = match shape {
        Shape::Nullable(inner) if value.kind != Kind::Null => inner,
        other => other,
    };
    if let Shape::Union(alternatives) = shape {
        let selected = alternatives.iter().find(|shape| matches!((&value.kind, shape),
            (Kind::Boolean(_), Shape::Boolean) | (Kind::String(_), Shape::String)
            | (Kind::Number(_), Shape::Int | Shape::Number)
            | (Kind::Array(_), Shape::Array(_)) | (Kind::Object(_), Shape::Object(_))
        )).expect("inferred union retains each input kind");
        return validate(value, selected);
    }
    match (&value.kind, shape) {
        (Kind::Number(text), Shape::Number) if !text.contains(['.', 'e', 'E']) => {
            let integer = text.parse::<i64>().map_err(|_| error(value.span, "JSON integer is outside the native int range."))?;
            if integer as f64 as i128 != integer as i128 {
                return Err(error(value.span, "JSON integer cannot be widened to number without losing precision."));
            }
        }
        (Kind::Array(_), Shape::Array(None)) => return Err(error(value.span, "Cannot infer an empty array element type without a nonempty sibling.")),
        (Kind::Array(values), Shape::Array(Some(element))) => {
            for value in values { validate(value, element)?; }
        }
        (Kind::Object(values), Shape::Object(fields)) => {
            for (name, property) in values { validate(&property.value, &fields[name].shape)?; }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn value(text: &str) -> Value { crate::data_module::parse(text, false).unwrap() }
    #[test]
    fn data_shape_merges_record_fields_and_nullable_values() {
        let shape = infer(&value(r#"[{"port":80,"host":"local"},{"port":null,"tls":true}]"#)).unwrap();
        let Shape::Array(Some(element)) = shape else { panic!("array") };
        let Shape::Object(fields) = *element else { panic!("object") };
        assert_eq!(fields["port"], Field { shape: Shape::Nullable(Box::new(Shape::Int)), optional: false });
        assert_eq!(fields["host"], Field { shape: Shape::String, optional: true });
        assert_eq!(fields["tls"], Field { shape: Shape::Boolean, optional: true });
    }
    #[test]
    fn data_shape_resolves_empty_arrays_from_siblings() {
        assert_eq!(infer(&value("[[],[1,2]]")).unwrap(), Shape::Array(Some(Box::new(Shape::Array(Some(Box::new(Shape::Int)))))));
        assert!(infer(&value("[]")).unwrap_err().message.contains("empty array"));
        let text = r#"{"values":[]}"#;
        assert_eq!(infer(&value(text)).unwrap_err().span, Span::new(10,12));
    }
    #[test]
    fn data_shape_numeric_boundaries_do_not_silently_round_integers() {
        assert_eq!(infer(&value("9223372036854775807")).unwrap(), Shape::Int);
        assert!(infer(&value("18446744073709551615")).is_err());
        assert_eq!(infer(&value("-0")).unwrap(), Shape::Number);
        assert_eq!(infer(&value("[1,2.5]")).unwrap(), Shape::Array(Some(Box::new(Shape::Number))));
        let text = "[9007199254740993,2.5]";
        assert_eq!(infer(&value(text)).unwrap_err().span, Span::new(1,17));
        assert!(infer(&value("[18014398509481984,2.5]")).is_ok());
    }
    #[test]
    fn data_shape_supports_native_heterogeneous_unions() {
        assert_eq!(infer(&value("[1,true,\"x\",null]")).unwrap(),
            Shape::Array(Some(Box::new(Shape::Nullable(Box::new(Shape::Union(vec![Shape::Boolean, Shape::Int, Shape::String])))))));
        assert_eq!(infer(&value("[true,1,2.5]")).unwrap(),
            Shape::Array(Some(Box::new(Shape::Union(vec![Shape::Boolean, Shape::Number])))));
        assert!(infer(&value(r#"[{"value":1},{"value":"x"}]"#)).is_ok());
    }
}
