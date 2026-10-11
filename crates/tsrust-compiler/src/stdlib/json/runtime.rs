pub use serde_json::Value;

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub bytes: usize,
    pub depth: usize,
    pub entries: usize,
    pub nodes: usize,
}

/// Structured private backend failure. No input bytes are retained or formatted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub code: &'static str,
    pub message: &'static str,
    pub offset: usize,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at byte {}", self.message, self.offset)
    }
}
impl std::error::Error for Error {}
fn message(code: &str) -> &'static str {
    match code {
        "output-limit" => "JSON output byte limit exceeded",
        "byte-limit" => "JSON input byte limit exceeded",
        "depth-limit" => "JSON nesting limit exceeded",
        "entry-limit" => "JSON collection entry limit exceeded",
        "node-limit" => "JSON value count limit exceeded",
        "invalid-limits" => "JSON limits must be nonnegative and depth must not exceed 64",
        "invalid-utf8" => "JSON input is not valid UTF-8",
        "unexpected-end" => "JSON input ended before the value was complete",
        "trailing-data" => "JSON input contains trailing non-whitespace data",
        _ => "JSON input is invalid",
    }
}
fn failure(code: &'static str, offset: usize) -> Error { Error { code, message: message(code), offset } }
fn located(input: &[u8], error: serde_json::Error, code: Option<&'static str>) -> Error {
    let code = code.unwrap_or(if error.is_eof() { "unexpected-end" } else { "invalid-json" });
    if error.is_eof() && code == "unexpected-end" { return failure(code, input.len()); }
    let start = if error.line() <= 1 { 0 } else {
        input.iter().enumerate().filter(|(_, byte)| **byte == b'\n').nth(error.line() - 2).map_or(input.len(), |(index, _)| index + 1)
    };
    failure(code, start.saturating_add(error.column().saturating_sub(1)).min(input.len()))
}
pub fn parse(input: &[u8], limits: Limits) -> Result<Value, Error> {
    if input.len() > limits.bytes { return Err(failure("byte-limit", limits.bytes)); }
    // Keep Serde's own recursion guard enabled as a second independent bound.
    if limits.depth > 64 { return Err(failure("invalid-limits", 0)); }
    if let Err(error) = std::str::from_utf8(input) { return Err(failure("invalid-utf8", error.valid_up_to())); }
    let mut budget = Budget { limits, nodes: 0, failed: None };
    let mut decoder = serde_json::Deserializer::from_slice(input);
    let value = serde::de::DeserializeSeed::deserialize(
        Seed { budget: &mut budget, depth: 0, permitted: true }, &mut decoder
    ).map_err(|error| located(input, error, budget.failed))?;
    decoder.end().map_err(|error| located(input, error, Some("trailing-data")))?;
    Ok(value)
}

pub fn stringify(value: &Value, max_bytes: usize) -> Result<std::string::String, Error> {
    struct Output { bytes: Vec<u8>, limit: usize, exceeded: bool }
    impl std::io::Write for Output {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > self.limit - self.bytes.len() { self.exceeded=true;return Err(std::io::Error::other("JSON output limit")); }
            self.bytes.extend_from_slice(bytes);Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
    }
    let mut output=Output{bytes:Vec::new(),limit:max_bytes,exceeded:false};
    if serde_json::to_writer(&mut output,value).is_err() {
        return Err(failure(if output.exceeded {"output-limit"} else {"invalid-json"},output.bytes.len()));
    }
    Ok(std::string::String::from_utf8(output.bytes).expect("JSON serialization emits UTF-8"))
}
pub fn invalid_limits() -> Error { failure("invalid-limits",0) }
pub fn kind(value: &Value) -> &'static str {
    match value {Value::Null=>"null",Value::Bool(_)=>"boolean",Value::Number(_)=>"number",Value::String(_)=>"string",Value::Array(_)=>"array",Value::Object(_)=>"object"}
}

struct Budget { limits: Limits, nodes: usize, failed: Option<&'static str> }
impl Budget {
    fn fail(&mut self, code: &'static str) -> &'static str { self.failed = Some(code); message(code) }
}
struct Seed<'a> { budget: &'a mut Budget, depth: usize, permitted: bool }
impl<'de> serde::de::DeserializeSeed<'de> for Seed<'_> {
    type Value = Value;
    fn deserialize<D: serde::Deserializer<'de>>(self, decoder: D) -> Result<Value, D::Error> {
        use serde::de::Error;
        if !self.permitted { return Err(D::Error::custom(self.budget.fail("entry-limit"))); }
        if self.budget.nodes >= self.budget.limits.nodes { return Err(D::Error::custom(self.budget.fail("node-limit"))); }
        self.budget.nodes += 1;
        decoder.deserialize_any(self)
    }
}
impl<'de> serde::de::Visitor<'de> for Seed<'_> {
    type Value = Value;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result { formatter.write_str("a strict JSON value") }
    fn visit_unit<E: serde::de::Error>(self) -> Result<Value, E> { Ok(Value::Null) }
    fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<Value, E> { Ok(Value::Bool(value)) }
    fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Value, E> { Ok(Value::from(value)) }
    fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Value, E> { Ok(Value::from(value)) }
    fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<Value, E> {
        serde_json::Number::from_f64(value).map(Value::Number).ok_or_else(|| E::custom("JSON number must be finite"))
    }
    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Value, E> { Ok(Value::String(value.to_owned())) }
    fn visit_string<E: serde::de::Error>(self, value: String) -> Result<Value, E> { Ok(Value::String(value)) }
    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut sequence: A) -> Result<Value, A::Error> {
        use serde::de::Error;
        if self.depth >= self.budget.limits.depth { return Err(A::Error::custom(self.budget.fail("depth-limit"))); }
        let mut values = Vec::new();
        loop {
            let permitted = values.len() < self.budget.limits.entries;
            let next = sequence.next_element_seed(Seed { budget: self.budget, depth: self.depth + 1, permitted })?;
            match next { Some(value) => values.push(value), None => break }
        }
        Ok(Value::Array(values))
    }
    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut object: A) -> Result<Value, A::Error> {
        use serde::de::Error;
        if self.depth >= self.budget.limits.depth { return Err(A::Error::custom(self.budget.fail("depth-limit"))); }
        let mut values = serde_json::Map::new();
        let mut entries = 0;
        // Count input entries, including duplicates, rather than the final map size.
        while let Some(key) = object.next_key_seed(KeySeed { permitted: entries < self.budget.limits.entries, budget: self.budget })? {
            let value = object.next_value_seed(Seed { budget: self.budget, depth: self.depth + 1, permitted: true })?;
            values.insert(key, value);
            entries += 1;
        }
        Ok(Value::Object(values))
    }
}
struct KeySeed<'a> { permitted: bool, budget: &'a mut Budget }
impl<'de> serde::de::DeserializeSeed<'de> for KeySeed<'_> {
    type Value = String;
    fn deserialize<D: serde::Deserializer<'de>>(self, decoder: D) -> Result<String, D::Error> {
        use serde::de::Error;
        if !self.permitted { return Err(D::Error::custom(self.budget.fail("entry-limit"))); }
        serde::Deserialize::deserialize(decoder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn limits() -> Limits { Limits { bytes: 1024, depth: 4, entries: 3, nodes: 20 } }
    #[test]
    fn encoding_limits_count_utf8_and_escaped_output_bytes() {
        for (input,expected) in [("\"é\"","\"é\""),("\"\\u0000\"","\"\\u0000\""),("-0.0","-0.0"),("18446744073709551615","18446744073709551615")] {
            let value=parse(input.as_bytes(),limits()).unwrap();
            assert_eq!(stringify(&value,expected.len()).unwrap(),expected);
            let error=stringify(&value,expected.len()-1).unwrap_err();
            assert_eq!(error.code,"output-limit");assert!(error.offset<=expected.len()-1);
            assert_eq!(stringify(&value,0).unwrap_err().code,"output-limit");
        }
    }
    #[test]
    fn errors_have_codes_byte_offsets_and_no_input_contents() {
        for (input,code,offset) in [
            (b"null secret".as_slice(),"trailing-data",5),
            (b"[1,".as_slice(),"unexpected-end",3),
            (b"".as_slice(),"unexpected-end",0),
            (b" \r\n".as_slice(),"unexpected-end",3),
            ("\"é".as_bytes(),"unexpected-end",3),
            (b"[1,\r\n]".as_slice(),"invalid-json",5),
            ("[\"é\",\nsecret]".as_bytes(),"invalid-json",7),
            (&[b'"',0xff,b'"'],"invalid-utf8",1),
        ] {
            let error=parse(input,limits()).unwrap_err();
            assert_eq!(error.code,code);assert_eq!(error.offset,offset);
            assert!(!error.to_string().contains("secret"));assert!(!error.message.contains("secret"));assert!(!format!("{error:?}").contains("secret"));
        }
        assert_eq!(parse(b"null",Limits{bytes:3,..limits()}).unwrap_err().code,"byte-limit");
        assert_eq!(parse(b"[]",Limits{depth:0,..limits()}).unwrap_err().code,"depth-limit");
        assert_eq!(parse(b"[0,1]",Limits{entries:1,..limits()}).unwrap_err().code,"entry-limit");
        assert_eq!(parse(b"[0]",Limits{nodes:1,..limits()}).unwrap_err().code,"node-limit");
        assert_eq!(parse(b"null",Limits{depth:65,..limits()}).unwrap_err().code,"invalid-limits");
    }
    #[test]
    fn exhausted_budgets_stop_before_decoding_the_next_value_or_key() {
        let one = Limits { entries: 1, ..limits() };
        assert!(parse(b"[0,invalid]", one).unwrap_err().message.contains("entry limit"));
        assert!(parse(br#"{"a":0,"unterminated"#, one).unwrap_err().message.contains("entry limit"));
        assert!(parse(b"[invalid]", Limits { nodes: 1, ..limits() }).unwrap_err().message.contains("value count"));
        assert!(parse(b"null", Limits { nodes: 0, ..limits() }).is_err());
        assert!(parse(b"null", Limits { depth: 0, ..limits() }).is_ok());
        assert!(parse(b"{}", Limits { depth: 0, ..limits() }).is_err());
    }
    #[test]
    fn bounded_depth_and_reference_value_semantics_match() {
        let nested = format!("{}0{}", "[".repeat(64), "]".repeat(64));
        assert!(parse(nested.as_bytes(), Limits { depth: 64, nodes: 65, ..limits() }).is_ok());
        assert!(parse(nested.as_bytes(), Limits { depth: 63, nodes: 65, ..limits() }).is_err());
        assert!(parse(b"null", Limits { depth: 65, ..limits() }).is_err());
        for input in ["null", "false", "0", "-0", "-0.0", "1e-300", "[]", "{}", " [true, null] ", r#"{"z":1,"a":2,"z":3}"#, r#""a\n\u0000é""#] {
            assert_eq!(parse(input.as_bytes(), limits()).unwrap(), serde_json::from_str::<Value>(input).unwrap());
        }
    }
    #[test]
    fn limits_reject_before_building_unbounded_values() {
        assert!(parse(b"null", Limits { bytes: 3, ..limits() }).is_err());
        assert!(parse(b"[[[[[]]]]]", limits()).is_err());
        assert!(parse(b"[0,1,2,3]", limits()).is_err());
        assert!(parse(br#"{"a":0,"a":1,"a":2,"a":3}"#, limits()).is_err());
        assert!(parse(b"[0,1]", Limits { nodes: 2, ..limits() }).is_err());
        assert!(parse(b"[]", Limits { entries: 0, nodes: 1, ..limits() }).is_ok());
        assert!(parse(b"[[]]", Limits { depth: 1, ..limits() }).is_err());
        assert!(parse(b"[0]", Limits { depth: 1, ..limits() }).is_ok());
    }
    #[test]
    fn strict_owned_values_preserve_scalar_boundaries() {
        let value = parse(br#"{"a":null,"a":true,"b":[-0.0,9223372036854775807,18446744073709551615,"\uD83E\uDD80"]}"#, Limits { entries: 4, ..limits() }).unwrap();
        assert_eq!(value["a"], true);
        assert!(value["b"][0].as_f64().unwrap().is_sign_negative());
        assert_eq!(value["b"][1].as_i64(), Some(i64::MAX));
        assert_eq!(value["b"][2].as_u64(), Some(u64::MAX));
        assert_eq!(value["b"][3], "🦀");
        for input in [b"null true".as_slice(), b"01", b"[1,]", b"NaN", b"1e999", br#""\uD800""#, &[b'"', 0xff, b'"']] {
            assert!(parse(input, limits()).is_err(), "{input:?}");
        }
    }
}
