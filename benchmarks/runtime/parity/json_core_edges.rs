#[allow(dead_code)]
mod tree {
    include!("native_json_roundtrip.rs");
    #[test]
    fn nested_tree_reencodes_and_uses_default_recursion_limit() {
        let text = String::Borrowed(" { \"a\": [1,true,null], \"b\": \"hello\" } ");
        assert_eq!(function0(2, 7, &text), 69);
        assert_eq!(parse(r#"{"a":[true,null,"é"]}"#)["a"][2], "é");
        let deep = format!("{}null{}", "[".repeat(256), "]".repeat(256));
        assert!(serde_json::from_str::<Value>(&deep).is_err());
    }
}
#[allow(dead_code)]
mod scalar {
    include!("native_json_scalars.rs");
    #[test]
    fn scalar_kinds_and_owned_string() {
        assert_eq!(function0(2, 7, &String::Borrowed("42")), 91);
        assert!(parse("42").as_i64().is_some());
        assert!(parse("1.25").as_i64().is_none());
        assert!(parse("true").as_bool().unwrap());
        let Value::String(text) = parse(r#""é😊""#) else {
            panic!()
        };
        assert_eq!(text, "é😊");
    }
}
#[allow(dead_code)]
mod kinds {
    include!("native_json_typeof.rs");
    #[test]
    fn json_type_labels() {
        for (text, expected) in [
            ("null", "object"),
            ("[]", "object"),
            ("{}", "object"),
            ("true", "boolean"),
            ("42", "number"),
            (r#""hi""#, "string"),
        ] {
            assert_eq!(label(&parse(text)), expected);
        }
    }
}
#[allow(dead_code)]
mod views {
    include!("native_json_views.rs");
    #[test]
    fn view_keeps_owner_and_preserves_missing_null_distinction() {
        let root = parse(r#"{"items":[42,null]}"#);
        let items = root
            .as_object()
            .unwrap()
            .get("items")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(items[0].as_i64(), Some(42));
        assert!(items[1].is_null());
        assert!(items.get(2).is_none());
        assert_eq!(
            function0(2, 7, &String::Borrowed(r#"{"items":[42,null]}"#)),
            91
        );
    }
}
#[allow(dead_code)]
mod strings {
    include!("native_json_string_views.rs");
    #[test]
    fn byte_lengths_indices_and_borrowed_storage() {
        let root = parse(r#""éhello😊hello""#);
        let text = root.as_str().unwrap();
        assert_eq!(length(text), 16);
        assert_eq!(string_score(text), 29);
        let Value::String(owner) = &root else {
            panic!()
        };
        assert_eq!(text.as_ptr(), owner.as_ptr());
        assert_eq!(string_score("missing"), 5);
    }
}
#[allow(dead_code)]
mod typed {
    include!("native_json_typed.rs");
    #[test]
    fn owned_record_fields_reject_wrong_types_and_missing_values() {
        let item: Item =
            serde_json::from_str(r#"{"name":"é😊","values":[20,17],"active":true,"extra":1}"#)
                .unwrap();
        assert_eq!(length(&item.name), 6);
        assert_eq!(item.values, vec![20, 17]);
        assert!(item.active);
        for text in [
            r#"{"name":"hi","values":[true],"active":true}"#,
            r#"{"name":"hi","values":[20],"active":"true"}"#,
            r#"{"name":"hi","active":true}"#,
        ] {
            assert!(serde_json::from_str::<Item>(text).is_err());
        }
    }
}
#[allow(dead_code)]
mod literals {
    include!("native_json_literal_union.rs");
    #[test]
    fn literal_union_and_true_only_field() {
        for name in ["hello", "other"] {
            let text = format!("{{\"name\":\"{name}\",\"values\":[20,17],\"active\":true}}");
            let item: Item = serde_json::from_str(&text).unwrap();
            assert_eq!(item.name.as_str(), name);
            assert!(item.active);
        }
        for text in [
            r#"{"name":"wrong","values":[20,17],"active":true}"#,
            r#"{"name":"hello","values":[20,17],"active":false}"#,
        ] {
            assert!(serde_json::from_str::<Item>(text).is_err());
        }
    }
}
#[allow(dead_code)]
mod construction {
    include!("native_json_construction.rs");
    #[test]
    fn scalar_construction_and_utf8_encoding() {
        assert_eq!(function0(2, 7, &String::Borrowed("hello")), 49);
        assert!(serde_json::Number::from_f64(f64::NAN).is_none());
        assert_eq!(
            serde_json::to_string(&Value::String("é😊".to_owned())).unwrap(),
            "\"é😊\""
        );
    }
}
#[allow(dead_code)]
mod containers {
    include!("native_json_containers.rs");
    #[test]
    fn duplicate_key_last_value_wins_without_losing_other_members() {
        let input = String::Borrowed("hello");
        assert_eq!(function0(2, 7, &input), 75);
        let mut map = serde_json::Map::new();
        map.insert("name".to_owned(), Value::String("old".to_owned()));
        map.insert(
            "items".to_owned(),
            Value::Array(vec![42.into(), Value::Null]),
        );
        map.insert("name".to_owned(), Value::String("hello".to_owned()));
        assert_eq!(map.len(), 2);
        assert_eq!(
            serde_json::to_string(&map).unwrap(),
            r#"{"items":[42,null],"name":"hello"}"#
        );
    }
}
#[allow(dead_code)]
mod raw {
    include!("native_json_raw.rs");
    #[test]
    fn borrowed_raw_retains_whitespace_and_rejects_invalid_documents() {
        let input = " \n {\"é\": [1,true,null]} \t";
        let retained = raw(input).unwrap();
        assert_eq!(retained, input);
        assert_eq!(retained.as_ptr(), input.as_ptr());
        for bad in ["", "[1,]", "true false", "{\"x\":}"] {
            assert!(raw(bad).is_err());
        }
    }
}
#[allow(dead_code)]
mod raw_encoding {
    include!("native_json_raw_encoding.rs");
    #[test]
    fn envelope_preserves_unmodified_raw_source() {
        let input = String::Borrowed(" { \"a\": [1,true,null], \"b\": \"hello\" } ");
        assert_eq!(function0(1, 0, &input), 50);
    }
}
