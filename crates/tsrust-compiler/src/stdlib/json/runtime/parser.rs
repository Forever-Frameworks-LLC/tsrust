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
        "allocation" => "Unable to allocate bounded JSON storage",
        "undefined-value" => "JSON cannot encode undefined at this position",
        "shape-mismatch" => "JSON value does not match the requested type",
        "expected-boolean" => "Expected a JSON boolean",
        "expected-number" => "Expected a JSON number",
        "expected-string" => "Expected a JSON string",
        "expected-object" => "Expected a JSON object",
        "expected-array" => "Expected a JSON array",
        "index-range" => "JSON array index must be nonnegative",
        "integer-fractional" => "JSON number has a fractional part",
        "integer-range" => "JSON number is outside the signed integer range",
        "number-nonfinite" => "JSON numbers must be finite",
        "number-precision" => "JSON integer cannot be represented exactly as a number",
        "io-error" => "JSON Lines input read failed",
        "position-limit" => "JSON Lines input position limit exceeded",
        "unterminated-comment" => "JSONC block comment is not terminated",
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
    let mut budget = Budget { limits, nodes: 0, failed: None, saw_float: false };
    let mut decoder = serde_json::Deserializer::from_slice(input);
    let value = serde::de::DeserializeSeed::deserialize(
        Seed { budget: &mut budget, depth: 0, permitted: true }, &mut decoder
    ).map_err(|error| located(input, error, budget.failed))?;
    decoder.end().map_err(|error| located(input, error, Some("trailing-data")))?;
    if budget.saw_float { validate_integer_tokens(input)?; }
    Ok(value)
}

/// Run after syntax validation. Integer tokens must fit the exact integer storage
/// variants; serde otherwise falls back to f64 and can round an overflow into range.
fn validate_integer_tokens(input: &[u8]) -> Result<(), Error> {
    let mut index = 0;
    while index < input.len() {
        if input[index] == b'"' {
            index += 1;
            while index < input.len() {
                match input[index] {
                    b'\\' => index += 2,
                    b'"' => { index += 1; break; }
                    _ => index += 1,
                }
            }
        } else if input[index] == b'-' || input[index].is_ascii_digit() {
            let start = index;
            while index < input.len() && matches!(input[index], b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E') { index += 1; }
            let token = &input[start..index];
            if token.iter().any(|byte| matches!(byte, b'.' | b'e' | b'E')) { continue; }
            let (digits, limit): (&[u8], &[u8]) = if token[0] == b'-' {
                (&token[1..], b"9223372036854775808")
            } else { (token, b"18446744073709551615") };
            if digits.len() > limit.len() || (digits.len() == limit.len() && digits > limit) {
                return Err(failure("integer-range", start));
            }
        } else { index += 1; }
    }
    Ok(())
}

/// Replace comments with same-length whitespace so backend offsets still address
/// the original document. This bounded scratch buffer never mutates caller input.
pub fn parse_jsonc(input: &[u8], limits: Limits, trailing_comma: bool) -> Result<Value, Error> {
    parse(&normalize_jsonc(input,limits,trailing_comma)?,limits)
}

fn normalize_jsonc(input: &[u8], limits: Limits, trailing_comma: bool) -> Result<Vec<u8>, Error> {
    if input.len()>limits.bytes {return Err(failure("byte-limit",limits.bytes));}
    if limits.depth>64 {return Err(invalid_limits());}
    if let Err(error)=std::str::from_utf8(input) {return Err(failure("invalid-utf8",error.valid_up_to()));}
    let mut normalized=input.to_vec();let mut index=0;let mut quoted=false;
    while index<input.len() {
        let byte=input[index];
        if quoted {
            if byte==b'\\' {index=(index+2).min(input.len());continue;}
            if byte==b'"' {quoted=false;}
            index+=1;continue;
        }
        if byte==b'"' {quoted=true;index+=1;continue;}
        if byte==b'/' && input.get(index+1)==Some(&b'/') {
            let start=index;index+=2;
            while index<input.len() && !matches!(input[index],b'\r'|b'\n') {index+=1;}
            normalized[start..index].fill(b' ');continue;
        }
        if byte==b'/' && input.get(index+1)==Some(&b'*') {
            let start=index;index+=2;
            while index+1<input.len() && &input[index..index+2]!=b"*/" {index+=1;}
            if index+1>=input.len() {return Err(failure("unterminated-comment",start));}
            index+=2;
            for byte in &mut normalized[start..index] {if !matches!(*byte,b'\r'|b'\n') {*byte=b' ';}}
            continue;
        }
        index+=1;
    }
    if trailing_comma {
        let mut index=0;let mut quoted=false;let mut previous=None;
        while index<normalized.len() {
            let byte=normalized[index];
            if quoted {
                if byte==b'\\' {index=(index+2).min(normalized.len());continue;}
                if byte==b'"' {quoted=false;previous=Some(b'"');}
                index+=1;continue;
            }
            if byte==b'"' {quoted=true;index+=1;continue;}
            if byte==b',' && !matches!(previous,None|Some(b'['|b'{'|b','|b':')) {
                let next=normalized[index+1..].iter().copied().find(|byte|!matches!(byte,b' '|b'\t'|b'\r'|b'\n'));
                if matches!(next,Some(b']'|b'}')) {normalized[index]=b' ';}
            }
            if !matches!(byte,b' '|b'\t'|b'\r'|b'\n') {previous=Some(byte);}
            index+=1;
        }
    }
    Ok(normalized)
}

pub fn invalid_limits() -> Error { failure("invalid-limits",0) }
// `saw_float` records any number serde delivered as f64: an integer token too
// large for i64/u64 arrives that way, so only then do tokens need re-checking.
struct Budget { limits: Limits, nodes: usize, failed: Option<&'static str>, saw_float: bool }
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
        self.budget.saw_float = true;
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

