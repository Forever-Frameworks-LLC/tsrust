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
