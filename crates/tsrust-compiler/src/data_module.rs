//! Source-preserving, validated data modules. Linking consumes this tree directly.
use crate::Span;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Value { pub span: Span, pub kind: Kind }
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Kind {
    Null, Boolean(bool), Number(String), String(String),
    Array(Vec<Value>), Object(BTreeMap<String, Property>),
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Property { pub name_span: Span, pub value: Value }

// Reuse the bounded public JSON implementation's acceptance and diagnostics.
use crate::stdlib::json as backend;

pub(crate) fn parse(text: &str, jsonc: bool) -> Result<Value, backend::Error> {
    let limits = backend::Limits { bytes: 16 * 1024 * 1024, depth: 64, entries: 1_000_000, nodes: 1_000_000 };
    // Validate before walking: one acceptance contract, bounded recursion, and
    // original UTF-8 byte offsets even when JSONC comments contain Unicode.
    if jsonc { backend::parse_jsonc(text.as_bytes(), limits, true)?; }
    else { backend::parse(text.as_bytes(), limits)?; }
    Ok(Cursor { text, offset: 0 }.value())
}

struct Cursor<'a> { text: &'a str, offset: usize }
impl Cursor<'_> {
    fn skip(&mut self) {
        let bytes = self.text.as_bytes();
        loop {
            while bytes.get(self.offset).is_some_and(u8::is_ascii_whitespace) { self.offset += 1; }
            if bytes.get(self.offset..self.offset + 2) == Some(b"//") {
                self.offset += 2;
                while bytes.get(self.offset).is_some_and(|b| !matches!(b, b'\r' | b'\n')) { self.offset += 1; }
            } else if bytes.get(self.offset..self.offset + 2) == Some(b"/*") {
                self.offset += 2;
                while &bytes[self.offset..self.offset + 2] != b"*/" { self.offset += 1; }
                self.offset += 2;
            } else { return; }
        }
    }
    fn string(&mut self) -> (String, Span) {
        let start = self.offset;
        self.offset += 1;
        let bytes = self.text.as_bytes();
        while bytes[self.offset] != b'"' {
            if bytes[self.offset] == b'\\' { self.offset += 1; }
            self.offset += 1;
        }
        self.offset += 1;
        let span = Span::new(start, self.offset);
        (serde_json::from_str(&self.text[start..self.offset]).expect("validated JSON string"), span)
    }
    fn value(&mut self) -> Value {
        self.skip();
        let start = self.offset;
        let bytes = self.text.as_bytes();
        let kind = match bytes[self.offset] {
            b'{' => {
                self.offset += 1;
                let mut fields = BTreeMap::new();
                loop {
                    self.skip();
                    if bytes[self.offset] == b'}' { self.offset += 1; break; }
                    let (name, name_span) = self.string();
                    self.skip();
                    self.offset += 1; // Validated colon.
                    let value = self.value();
                    fields.insert(name, Property { name_span, value });
                    self.skip();
                    if bytes[self.offset] == b',' { self.offset += 1; }
                }
                Kind::Object(fields)
            }
            b'[' => {
                self.offset += 1;
                let mut values = Vec::new();
                loop {
                    self.skip();
                    if bytes[self.offset] == b']' { self.offset += 1; break; }
                    values.push(self.value());
                    self.skip();
                    if bytes[self.offset] == b',' { self.offset += 1; }
                }
                Kind::Array(values)
            }
            b'"' => Kind::String(self.string().0),
            b'n' => { self.offset += 4; Kind::Null }
            b't' => { self.offset += 4; Kind::Boolean(true) }
            b'f' => { self.offset += 5; Kind::Boolean(false) }
            _ => {
                while bytes.get(self.offset).is_some_and(|b| matches!(b, b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E')) { self.offset += 1; }
                // Keep the exact lexeme for representability checks. Do not
                // round through serde_json::Number before type inference.
                Kind::Number(self.text[start..self.offset].to_owned())
            }
        };
        Value { span: Span::new(start, self.offset), kind }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn data_module_preserves_original_spans_and_decoded_names() {
        let text = "{/*é*/\"na\\u006de\": [true, null, \"雪\",],}";
        let root = parse(text, true).unwrap();
        assert_eq!(root.span, Span::new(0, text.len()));
        let Kind::Object(fields) = root.kind else { panic!("object") };
        let property = &fields["name"];
        assert_eq!(&text[property.name_span.start..property.name_span.end], "\"na\\u006de\"");
        let Kind::Array(items) = &property.value.kind else { panic!("array") };
        assert_eq!(items[0].kind, Kind::Boolean(true));
        assert_eq!(items[1].kind, Kind::Null);
        assert_eq!(items[2].kind, Kind::String("雪".into()));
        assert_eq!(&text[items[2].span.start..items[2].span.end], "\"雪\"");
        assert!(parse(text, false).is_err());
    }
    #[test]
    fn data_module_retains_number_lexemes_and_last_duplicate() {
        let text = r#"{"x":0,"x":18446744073709551615,"n":-0,"f":1.234567890123456789e-20,"empty":[]}"#;
        let Kind::Object(fields) = parse(text, false).unwrap().kind else { panic!("object") };
        assert_eq!(fields["x"].value.kind, Kind::Number("18446744073709551615".into()));
        assert_eq!(fields["x"].name_span.start, text.rfind("\"x\"").unwrap());
        assert_eq!(fields["n"].value.kind, Kind::Number("-0".into()));
        assert_eq!(fields["f"].value.kind, Kind::Number("1.234567890123456789e-20".into()));
        assert_eq!(fields["empty"].value.kind, Kind::Array(vec![]));
    }
    #[test]
    fn data_module_handles_scalar_roots_and_comment_like_strings() {
        for (text, expected) in [
            (" null ", Kind::Null),
            ("false", Kind::Boolean(false)),
            (r#""/* not a comment */ // \\ \" \uD83E\uDD80""#, Kind::String("/* not a comment */ // \\ \" 🦀".into())),
            ("/* start */ 42 // end", Kind::Number("42".into())),
        ] {
            assert_eq!(parse(text, true).unwrap().kind, expected);
        }
        let value = parse("[/* first */ {}, // line\n [null,],]", true).unwrap();
        let Kind::Array(items) = value.kind else { panic!("array") };
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].kind, Kind::Object(BTreeMap::new()));
        assert!(matches!(items[1].kind, Kind::Array(_)));
    }
    #[test]
    fn data_module_reports_original_errors_and_bounds() {
        let text = "{/*雪*/\"x\": nope}";
        let error = parse(text, true).unwrap_err();
        assert_eq!(error.offset, text.find("nope").unwrap() + 1); // The invalid byte after the valid null prefix.
        assert_eq!(parse("[1,]", false).unwrap_err().code, "invalid-json");
        assert!(parse("[1,]", true).is_ok());
        assert_eq!(parse("null false", false).unwrap_err().code, "trailing-data");
        let deep = format!("{}0{}", "[".repeat(65), "]".repeat(65));
        assert_eq!(parse(&deep, false).unwrap_err().code, "depth-limit");
        assert!(parse("1e9999", false).is_err());
    }
}
