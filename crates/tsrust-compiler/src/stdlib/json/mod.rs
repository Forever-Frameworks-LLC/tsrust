//! Private S10 backend. Public bindings and native dependency packaging follow separately.
use serde_json::Value;

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub bytes: usize,
    pub depth: usize,
    pub entries: usize,
    pub nodes: usize,
}

pub fn parse(input: &[u8], limits: Limits) -> Result<Value, String> {
    if input.len() > limits.bytes { return Err("JSON input byte limit exceeded".into()); }
    // Keep Serde's own recursion guard enabled as a second independent bound.
    if limits.depth > 64 { return Err("JSON depth limit must not exceed 64".into()); }
    let mut budget = Budget { limits, nodes: 0 };
    let mut decoder = serde_json::Deserializer::from_slice(input);
    let value = serde::de::DeserializeSeed::deserialize(
        Seed { budget: &mut budget, depth: 0, permitted: true }, &mut decoder
    ).map_err(|error| error.to_string())?;
    decoder.end().map_err(|error| error.to_string())?;
    Ok(value)
}

struct Budget { limits: Limits, nodes: usize }
struct Seed<'a> { budget: &'a mut Budget, depth: usize, permitted: bool }
impl<'de> serde::de::DeserializeSeed<'de> for Seed<'_> {
    type Value = Value;
    fn deserialize<D: serde::Deserializer<'de>>(self, decoder: D) -> Result<Value, D::Error> {
        use serde::de::Error;
        if !self.permitted { return Err(D::Error::custom("JSON collection entry limit exceeded")); }
        if self.budget.nodes >= self.budget.limits.nodes { return Err(D::Error::custom("JSON value count limit exceeded")); }
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
        if self.depth >= self.budget.limits.depth { return Err(A::Error::custom("JSON nesting limit exceeded")); }
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
        if self.depth >= self.budget.limits.depth { return Err(A::Error::custom("JSON nesting limit exceeded")); }
        let mut values = serde_json::Map::new();
        let mut entries = 0;
        // Count input entries, including duplicates, rather than the final map size.
        while let Some(key) = object.next_key_seed(KeySeed(entries < self.budget.limits.entries))? {
            let value = object.next_value_seed(Seed { budget: self.budget, depth: self.depth + 1, permitted: true })?;
            values.insert(key, value);
            entries += 1;
        }
        Ok(Value::Object(values))
    }
}
struct KeySeed(bool);
impl<'de> serde::de::DeserializeSeed<'de> for KeySeed {
    type Value = String;
    fn deserialize<D: serde::Deserializer<'de>>(self, decoder: D) -> Result<String, D::Error> {
        use serde::de::Error;
        if !self.0 { return Err(D::Error::custom("JSON collection entry limit exceeded")); }
        serde::Deserialize::deserialize(decoder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn limits() -> Limits { Limits { bytes: 1024, depth: 4, entries: 3, nodes: 20 } }
    #[test]
    fn exhausted_budgets_stop_before_decoding_the_next_value_or_key() {
        let one = Limits { entries: 1, ..limits() };
        assert!(parse(b"[0,invalid]", one).unwrap_err().contains("entry limit"));
        assert!(parse(br#"{"a":0,"unterminated"#, one).unwrap_err().contains("entry limit"));
        assert!(parse(b"[invalid]", Limits { nodes: 1, ..limits() }).unwrap_err().contains("value count"));
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
