#[allow(dead_code)]
mod strip {
    include!("native_schema_json_object_strip.rs");
    #[test]
    fn strip_discards_extras_but_keeps_count_checks() {
        assert_eq!(
            validate(parse(r#"{"count":2,"extra":true}"#))
                .unwrap()
                .count,
            2
        );
        let error = validate(parse(r#"{"extra":true}"#)).unwrap_err();
        assert_eq!(check_count(error), 1);
        let error = validate(parse(r#"{"count":1,"extra":true}"#)).unwrap_err();
        assert_eq!(error.issues[0].code, Code::TooSmall);
        assert_eq!(
            error.issues[0].path,
            vec![Segment::Field("count".to_owned())]
        );
    }
}
#[allow(dead_code)]
mod retain {
    include!("native_schema_json_object_retain.rs");
    #[test]
    fn extras_retain_owned_storage_and_exclude_known_fields() {
        let input = parse(r#"{"count":3,"extra":"é😊","flag":true}"#);
        let pointer = input["extra"].as_str().unwrap().as_ptr();
        let output = validate(input).unwrap();
        assert_eq!(output.count, 3);
        assert_eq!(output.extra.len(), 2);
        assert!(!output.extra.contains_key("count"));
        assert_eq!(output.extra["extra"].as_str().unwrap().as_ptr(), pointer);
        assert_eq!(output.extra["flag"], Value::Bool(true));
    }
}
#[allow(dead_code)]
mod presence {
    include!("native_schema_json_presence.rs");
    #[test]
    fn missing_optional_and_required_nullable_are_distinct() {
        let output = validate(parse(r#"{"label":null}"#)).unwrap();
        assert!(output.count.is_none() && output.label.is_none());
        let output = validate(parse(r#"{"count":7,"label":"é"}"#)).unwrap();
        assert_eq!(output.count, Some(7));
        assert_eq!(output.label.as_deref(), Some("é"));
        let error = validate(parse(r#"{"count":null}"#)).unwrap_err();
        assert_eq!(
            error.issues[0].path,
            vec![Segment::Field("count".to_owned())]
        );
        assert_eq!(
            error.issues[1].path,
            vec![Segment::Field("label".to_owned())]
        );
        assert_eq!(check_presence(error), 2);
        let error = validate(parse(r#"{"label":null,"extra":true}"#)).unwrap_err();
        assert_eq!(error.issues[0].code, Code::UnknownField);
        assert_eq!(error.issues[0].expected, Kind::Missing);
    }
}
#[allow(dead_code)]
mod defaults {
    include!("native_schema_json_defaults.rs");
    #[test]
    fn defaults_apply_only_to_missing_and_still_validate_siblings() {
        let output = validate(parse(r#"{"label":"ready"}"#)).unwrap();
        assert_eq!(output.count, 7);
        assert_eq!(output.label, "ready");
        assert_eq!(
            validate(parse(r#"{"count":8,"label":"ready"}"#))
                .unwrap()
                .count,
            8
        );
        let error = validate(parse(r#"{"count":null}"#)).unwrap_err();
        assert_eq!(check_presence(error), 2);
        let error = validate(parse(r#"{"count":4,"label":"ready"}"#)).unwrap_err();
        assert_eq!(error.issues[0].code, Code::TooSmall);
        let error = validate(parse(r#"{"label":null}"#)).unwrap_err();
        assert_eq!(error.issues[0].code, Code::TypeMismatch);
    }
}
#[allow(dead_code)]
mod literal {
    include!("native_schema_json_literal.rs");
    #[test]
    fn literal_validation_moves_the_matched_string() {
        let original = "ready".to_owned();
        let pointer = original.as_ptr();
        let output = validate(Value::String(original), &["ready"]).unwrap();
        assert_eq!(output.as_ptr(), pointer);
        check_root(
            validate(parse("\"waiting\""), &["ready"]).unwrap_err(),
            Code::LiteralMismatch,
            Kind::String,
            Kind::String,
        );
        check_root(
            validate(Value::Bool(true), &["ready"]).unwrap_err(),
            Code::TypeMismatch,
            Kind::String,
            Kind::Boolean,
        );
    }
}
#[allow(dead_code)]
mod enumeration {
    include!("native_schema_json_enum.rs");
    #[test]
    fn string_enum_accepts_every_member_and_rejects_nonmember() {
        for member in ["ready", "done"] {
            assert_eq!(
                validate(Value::String(member.to_owned()), &["ready", "done"]).unwrap(),
                member
            );
        }
        check_root(
            validate(Value::String("waiting".to_owned()), &["ready", "done"]).unwrap_err(),
            Code::LiteralMismatch,
            Kind::String,
            Kind::String,
        );
    }
}
#[allow(dead_code)]
mod declared {
    include!("native_schema_json_declared_enum.rs");
    #[test]
    fn numeric_enum_preserves_fractional_values_and_precision_checks() {
        assert_eq!(validate(parse("1.0")).unwrap(), CodeValue::Low);
        assert_eq!(validate(parse("2.5")).unwrap(), CodeValue::High);
        check_root(
            validate(parse("3.0")).unwrap_err(),
            Code::LiteralMismatch,
            Kind::Number,
            Kind::Number,
        );
        check_root(
            validate(parse("9007199254740993")).unwrap_err(),
            Code::NumberPrecision,
            Kind::Number,
            Kind::Number,
        );
    }
}
#[allow(dead_code)]
mod export {
    include!("native_schema_export.rs");
    #[test]
    fn schema_documents_derive_bounds_and_nested_nodes() {
        let schema = Schema::String { min: 3, max: 5 };
        let value = schema.document(true);
        assert_eq!(value["minLength"], 3);
        assert_eq!(value["maxLength"], 5);
        assert_eq!(value["type"], "string");
        assert!(value.get("$schema").is_some());
        let flags = Schema::Array(Box::new(Schema::Boolean)).document(true);
        assert_eq!(flags["items"]["type"], "boolean");
        assert!(flags["items"].get("$schema").is_none());
        assert_eq!(
            serde_json::to_string(&flags).unwrap(),
            r#"{"$schema":"https://json-schema.org/draft/2020-12/schema","items":{"type":"boolean"},"type":"array"}"#
        );
    }
}
