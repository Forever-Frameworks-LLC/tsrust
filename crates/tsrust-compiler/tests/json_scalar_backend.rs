#[allow(dead_code)]
#[path = "../src/stdlib/json/runtime.rs"]
mod runtime;
use runtime::*;

#[test]
fn scalar_reads_reject_coercion_and_integer_rounding() {
    assert_eq!(read_bool(&Value::Bool(true)).unwrap(), true);
    assert_eq!(read_bool(&Value::from("true")).unwrap_err().code, "expected-boolean");
    for integer in [i64::MIN, -1, 0, 1, i64::MAX] {
        assert_eq!(read_i64(&Value::from(integer)).unwrap(), integer);
    }
    assert_eq!(read_i64(&Value::from(42.0)).unwrap(), 42);
    assert_eq!(read_i64(&Value::from(-0.0)).unwrap(), 0);
    assert_eq!(read_i64(&Value::from(1.5)).unwrap_err().code, "integer-fractional");
    for value in [Value::from(u64::MAX), Value::from(9223372036854775808.0), Value::from(-9223372036854777856.0)] {
        assert_eq!(read_i64(&value).unwrap_err().code, "integer-range");
    }
    assert_eq!(read_i64(&Value::Null).unwrap_err().code, "expected-number");
}

#[test]
fn floating_reads_preserve_bits_and_reject_inexact_integer_conversion() {
    for value in [-0.0, 0.0, 1.5, f64::MIN_POSITIVE, f64::from_bits(1), f64::MAX] {
        assert_eq!(read_f64(&Value::from(value)).unwrap().to_bits(), value.to_bits());
    }
    for integer in [0u64, 9007199254740992, 9007199254740994, 9223372036854775808] {
        assert_eq!(read_f64(&Value::from(integer)).unwrap(), integer as f64);
    }
    for value in [Value::from(9007199254740993u64), Value::from(i64::MAX), Value::from(u64::MAX), Value::from(-9007199254740993i64)] {
        assert_eq!(read_f64(&value).unwrap_err().code, "number-precision");
    }
    assert_eq!(read_f64(&Value::from(i64::MIN)).unwrap(), i64::MIN as f64);
    assert_eq!(read_f64(&Value::Bool(true)).unwrap_err().code, "expected-number");
}

#[test]
fn string_extraction_moves_storage_and_errors_retain_no_input() {
    let text="secret 🦀\0".repeat(20);let pointer=text.as_ptr();
    let value=Value::String(text);
    assert_eq!(read_string(&value).unwrap().as_ptr(),pointer);
    let text=take_string(value).unwrap();assert_eq!(text.as_ptr(),pointer);
    let error=take_string(Value::from(vec!["secret"])).unwrap_err();
    assert_eq!(error.code,"expected-string");assert_eq!(error.offset,0);
    assert!(!format!("{error:?}").contains("secret"));
}

#[test]
fn integer_tokens_outside_json_storage_never_round_into_range() {
    let limits=Limits{bytes:4096,depth:16,entries:32,nodes:64};
    for text in ["-9223372036854775809","18446744073709551616","[-9223372036854775809]","{\"n\":18446744073709551616}"] {
        assert_eq!(parse(text.as_bytes(),limits).unwrap_err().code,"integer-range", "{text}");
    }
    for text in ["-9223372036854775808","18446744073709551615","\"-9223372036854775809\"","1e30","-0.0"] {
        assert!(parse(text.as_bytes(),limits).is_ok(), "{text}");
    }
}

#[test]
fn plain_parser_checks_overwritten_integer_tokens_before_discarding_them() {
    for (text, token) in [
        (r#"{"x":18446744073709551616,"x":0}"#, "18446744073709551616"),
        (r#"{"x":-9223372036854775809,"x":null}"#, "-9223372036854775809"),
        (r#"{"x":[18446744073709551616],"x":true}"#, "18446744073709551616"),
        (r#"{"x":{"y":18446744073709551616},"x":"replacement"}"#, "18446744073709551616"),
        (r#"{"x":18446744073709551616,"\u0078":0}"#, "18446744073709551616"),
    ] {
        let error = parse_plain(text).unwrap_err();
        assert_eq!(error.code, "integer-range", "{text}");
        assert_eq!(error.offset, text.find(token).unwrap(), "{text}");
    }
    assert_eq!(parse_plain(r#"{"x":18446744073709551616,"x":0} trailing"#).unwrap_err().code, "trailing-data");
    assert_eq!(parse_plain(r#"{"x":18446744073709551616,"x":}"#).unwrap_err().code, "invalid-json");
}

#[test]
fn plain_parser_keeps_serde_values_and_default_recursion_boundaries() {
    for text in [
        r#""18446744073709551616""#,
        " \t\r\n\"hello\" ",
        r#""crab \uD83E\uDD80 \" \\""#,
        r#"{"x":18446744073709551615,"x":0}"#,
        r#"{"x":-9223372036854775808,"x":null}"#,
        r#"{"x":"18446744073709551616","x":false}"#,
        r#"{"x":1e30,"x":2.5}"#,
        r#"[true,null,"crab 🦀\"\\",-0.0,1e-400,2.5]"#,
        r#"{"\u0078":1,"x":2,"nested":[{"value":3}]}"#,
    ] {
        let expected: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parse_plain(text).unwrap().to_string(), expected.to_string(), "{text}");
    }
    for (text, code) in [
        ("\"ok\" false", "trailing-data"),
        (r#""\x""#, "invalid-json"),
        ("\u{000b}\"ok\"", "invalid-json"),
    ] { assert_eq!(parse_plain(text).unwrap_err().code, code, "{text}"); }
    for depth in 0..=130 {
        let text = format!("{}0{}", "[".repeat(depth), "]".repeat(depth));
        let expected = serde_json::from_str::<Value>(&text);
        let actual = parse_plain(&text);
        assert_eq!(actual.is_ok(), expected.is_ok(), "depth {depth}");
        if let (Ok(actual), Ok(expected)) = (actual, expected) { assert_eq!(actual, expected); }
    }
}

#[test]
fn plain_parser_preserves_property_names_that_match_serde_internal_tokens() {
    for (text, expected) in [
        (r#"{"$serde_json::private::RawValue":"0"}"#, "0"),
        (r#"{"$serde_json::private::RawValue":"true"}"#, "true"),
        (r#"{"$serde_json::private::RawValue":"not JSON"}"#, "not JSON"),
        (r#"{"first":1,"$serde_json::private::RawValue":"0"}"#, "0"),
    ] {
        let value = parse_plain(text).unwrap();
        let Value::Object(fields) = value else { panic!("ordinary property changed object kind: {text}") };
        assert_eq!(fields.get("$serde_json::private::RawValue"), Some(&Value::from(expected)));
        assert_eq!(fields.len(), if text.contains("first") { 2 } else { 1 });
    }
}
