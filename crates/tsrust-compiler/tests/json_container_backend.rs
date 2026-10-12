#[allow(dead_code)]
#[path = "../src/stdlib/json/runtime.rs"]
mod runtime;
use runtime::*;

fn parsed(text: &str) -> Value {
    parse(text.as_bytes(), Limits { bytes: 4096, depth: 64, entries: 100, nodes: 100 }).unwrap()
}

#[test]
fn child_reads_distinguish_missing_null_and_wrong_kind() {
    let value = parsed(r#"{"null":null,"array":[null,"crab 🦀"],"":true}"#);
    assert!(get_key(&value, "missing").unwrap().is_none());
    assert_eq!(get_key(&value, "null").unwrap(), Some(&Value::Null));
    assert_eq!(get_key(&value, "").unwrap(), Some(&Value::Bool(true)));
    let array = {
        let temporary_key = String::from("array");
        get_key(&value, &temporary_key).unwrap().unwrap()
    };
    assert_eq!(get_index(array, 0).unwrap(), Some(&Value::Null));
    assert!(get_index(array, 2).unwrap().is_none());
    assert!(get_index(array, i64::MAX).unwrap().is_none());
    assert_eq!(get_index(array, -1).unwrap_err().code, "index-range");
    assert_eq!(get_key(array, "0").unwrap_err().code, "expected-object");
    assert_eq!(get_index(&value, 0).unwrap_err().code, "expected-array");
    assert_eq!(get_index(&Value::Null, 0).unwrap_err().code, "expected-array");
}

#[test]
fn child_reads_borrow_and_takes_transfer_original_storage() {
    let text = "owned 🦀".repeat(20);
    let pointer = text.as_ptr();
    let array = Value::Array(vec![Value::Null, Value::String(text), parsed("{\"discarded\":[1,2,3]}")]);
    assert_eq!(read_string(get_index(&array, 1).unwrap().unwrap()).unwrap().as_ptr(), pointer);
    let selected = take_index(array, 1).unwrap().unwrap();
    let selected = take_string(selected).unwrap();
    assert_eq!(selected.as_ptr(), pointer);

    let text = "second allocation".repeat(20);
    let pointer = text.as_ptr();
    let mut object = serde_json::Map::new();
    object.insert("selected".into(), Value::String(text));
    object.insert("discarded".into(), parsed("[1,2,3]"));
    let object = Value::Object(object);
    assert_eq!(read_string(get_key(&object, "selected").unwrap().unwrap()).unwrap().as_ptr(), pointer);
    let selected = take_string(take_key(object, "selected").unwrap().unwrap()).unwrap();
    assert_eq!(selected.as_ptr(), pointer);
}

#[test]
fn consuming_selection_keeps_absence_errors_and_redaction() {
    assert_eq!(take_key(parsed("{\"x\":null}"), "x").unwrap(), Some(Value::Null));
    assert!(take_key(parsed("{}"), "secret").unwrap().is_none());
    assert!(take_index(parsed("[]"), 0).unwrap().is_none());
    assert!(take_index(parsed("[1]"), i64::MAX).unwrap().is_none());
    assert_eq!(take_index(parsed("[1]"), -1).unwrap_err().code, "index-range");
    for error in [take_key(parsed("[\"secret\"]"), "secret").unwrap_err(), take_index(parsed("{\"secret\":true}"), 0).unwrap_err()] {
        assert_eq!(error.offset, 0);
        assert!(!format!("{error:?}").contains("secret"));
    }
}
