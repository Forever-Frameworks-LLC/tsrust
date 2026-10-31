// Independent Rust schema model: build each JSON document from its schema nodes.
type String = std::borrow::Cow<'static, str>;
use serde_json::Value;
enum Schema {
    String { min: usize, max: usize },
    Boolean,
    Array(Box<Schema>),
}
impl Schema {
    fn document(&self, root: bool) -> Value {
        let mut fields = serde_json::Map::new();
        if root {
            fields.insert(
                "$schema".to_owned(),
                Value::String("https://json-schema.org/draft/2020-12/schema".to_owned()),
            );
        }
        match self {
            Self::String { min, max } => {
                fields.insert("type".to_owned(), Value::String("string".to_owned()));
                fields.insert("minLength".to_owned(), Value::from(*min));
                fields.insert("maxLength".to_owned(), Value::from(*max));
            }
            Self::Boolean => {
                fields.insert("type".to_owned(), Value::String("boolean".to_owned()));
            }
            Self::Array(element) => {
                fields.insert("type".to_owned(), Value::String("array".to_owned()));
                fields.insert("items".to_owned(), element.document(false));
            }
        }
        Value::Object(fields)
    }
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let text = Schema::String { min: 2, max: 8 };
    let flags = Schema::Array(Box::new(Schema::Boolean));
    let mut checksum = value1;
    for _ in 0..value0 {
        let first = serde_json::to_string(&text.document(true)).unwrap();
        let second = serde_json::to_string(&flags.document(true)).unwrap();
        assert_eq!(first, value2.as_ref());
        assert_eq!(
            second,
            r#"{"$schema":"https://json-schema.org/draft/2020-12/schema","items":{"type":"boolean"},"type":"array"}"#
        );
        checksum = checksum
            .checked_add(i64::try_from(first.len()).unwrap())
            .unwrap()
            .checked_add(i64::try_from(second.len()).unwrap())
            .unwrap();
    }
    checksum
}
fn main() {}
