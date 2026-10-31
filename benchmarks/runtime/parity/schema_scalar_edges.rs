#[allow(dead_code)]
mod string {
    include!("native_schema_string.rs");
    #[test]
    fn owned_string_moves_and_wrong_kinds_keep_metadata() {
        let original = "é😊".to_owned();
        let pointer = original.as_ptr();
        let output = validate(Value::String(original)).unwrap();
        assert_eq!(output.as_ptr(), pointer);
        assert_eq!(len(&output), 6);
        let error = validate(Value::Null).unwrap_err();
        assert_eq!(
            check_failure(error, Code::TypeMismatch, Kind::String, Kind::Null),
            1
        );
    }
}
#[allow(dead_code)]
mod unicode_bounds {
    include!("native_schema_string_bounds.rs");
    #[test]
    fn constraints_count_unicode_scalars_while_output_length_counts_bytes() {
        assert_eq!(validate(Value::String("é😊".to_owned())).unwrap(), "é😊");
        assert_eq!(
            validate(Value::String("é".to_owned())).unwrap_err().issues[0].code,
            Code::TooSmall
        );
        assert_eq!(
            validate(Value::String("1234567".to_owned()))
                .unwrap_err()
                .issues[0]
                .code,
            Code::TooLarge
        );
    }
}
#[allow(dead_code)]
mod messages {
    include!("native_schema_constraint_messages.rs");
    #[test]
    fn custom_messages_follow_constraint_direction() {
        let low = validate(parse("\"x\"")).unwrap_err();
        assert_eq!(low.issues[0].message.as_deref(), Some("Too short"));
        let high = validate(parse("\"toolong\"")).unwrap_err();
        assert_eq!(high.issues[0].code, Code::TooLarge);
        assert_eq!(high.issues[0].message.as_deref(), Some("Too long"));
    }
}
#[allow(dead_code)]
mod boolean {
    include!("native_schema_boolean.rs");
    #[test]
    fn false_is_a_boolean_success_and_number_is_not() {
        assert!(!validate(Value::Bool(false)).unwrap());
        assert_eq!(
            check_failure(
                validate(parse("0")).unwrap_err(),
                Code::TypeMismatch,
                Kind::Boolean,
                Kind::Number
            ),
            1
        );
    }
}
#[allow(dead_code)]
mod null {
    include!("native_schema_null.rs");
    #[test]
    fn only_null_is_valid() {
        validate(Value::Null).unwrap();
        assert_eq!(
            check_failure(
                validate(Value::Bool(false)).unwrap_err(),
                Code::TypeMismatch,
                Kind::Null,
                Kind::Boolean
            ),
            1
        );
    }
}
#[allow(dead_code)]
mod integer {
    include!("native_schema_int.rs");
    #[test]
    fn whole_floats_exact_endpoints_fractional_and_range_errors() {
        assert_eq!(validate(parse("7.0")).unwrap(), 7);
        assert_eq!(validate(parse("-0.0")).unwrap(), 0);
        assert_eq!(validate(parse("9223372036854775807")).unwrap(), i64::MAX);
        assert_eq!(validate(parse("-9223372036854775808")).unwrap(), i64::MIN);
        for (text, code) in [
            ("1.5", Code::IntegerFractional),
            ("18446744073709551615", Code::IntegerRange),
            ("9223372036854775808.0", Code::IntegerRange),
        ] {
            assert_eq!(
                check_failure(
                    validate(parse(text)).unwrap_err(),
                    code,
                    Kind::Integer,
                    Kind::Number
                ),
                1
            );
        }
    }
}
#[allow(dead_code)]
mod inclusive {
    include!("native_schema_int_bounds.rs");
    #[test]
    fn inclusive_bounds_and_complete_diagnostics() {
        assert_eq!(validate(parse("2")).unwrap(), 2);
        assert_eq!(validate(parse("9")).unwrap(), 9);
        assert_eq!(
            check_failure(
                validate(parse("10")).unwrap_err(),
                Code::TooLarge,
                Kind::Integer,
                Kind::Number
            ),
            1
        );
        assert_eq!(
            check_failure(
                validate(parse("1")).unwrap_err(),
                Code::TooSmall,
                Kind::Integer,
                Kind::Number
            ),
            1
        );
    }
}
#[allow(dead_code)]
mod exclusive {
    include!("native_schema_int_exclusive.rs");
    #[test]
    fn exclusive_endpoints_reject_and_inside_values_pass() {
        assert_eq!(validate(parse("3")).unwrap(), 3);
        assert_eq!(validate(parse("9")).unwrap(), 9);
        assert_eq!(
            validate(parse("2")).unwrap_err().issues[0].code,
            Code::TooSmall
        );
        assert_eq!(
            validate(parse("10")).unwrap_err().issues[0].code,
            Code::TooLarge
        );
    }
}
#[allow(dead_code)]
mod numbers {
    include!("native_schema_number.rs");
    #[test]
    fn exact_integer_conversion_and_signed_zero() {
        assert_eq!(
            validate(parse("9007199254740994")).unwrap(),
            9007199254740994.0
        );
        assert_eq!(
            validate(parse("-0.0")).unwrap().to_bits(),
            (-0.0_f64).to_bits()
        );
        assert_eq!(
            check_failure(
                validate(parse("9007199254740993")).unwrap_err(),
                Code::NumberPrecision,
                Kind::Number,
                Kind::Number
            ),
            1
        );
        assert_eq!(
            check_failure(
                native_number(i64::MAX).unwrap_err(),
                Code::NumberPrecision,
                Kind::Number,
                Kind::Integer
            ),
            1
        );
    }
}
#[allow(dead_code)]
mod number_bounds {
    include!("native_schema_number_bounds.rs");
    #[test]
    fn intersected_inclusive_and_exclusive_ranges() {
        assert_eq!(validate(parse("1.5")).unwrap(), 1.5);
        for (text, code) in [
            ("0", Code::TooSmall),
            ("1", Code::TooSmall),
            ("2", Code::TooLarge),
            ("3", Code::TooLarge),
        ] {
            assert_eq!(validate(parse(text)).unwrap_err().issues[0].code, code);
        }
    }
}
#[allow(dead_code)]
mod refine {
    include!("native_schema_super_refine.rs");
    #[test]
    fn refinement_accumulates_in_callback_order_after_integer_validation() {
        assert_eq!(validate(parse("5")).unwrap(), 5);
        let error = validate(parse("0")).unwrap_err();
        assert_eq!(error.issues.len(), 2);
        assert_eq!(
            error
                .issues
                .iter()
                .map(|i| i.message.as_deref().unwrap())
                .collect::<Vec<_>>(),
            vec!["Must be positive", "Must be odd"]
        );
        assert!(error.issues.iter().all(|i| i.code == Code::RefinementFailed
            && i.expected == Kind::Integer
            && i.observed == Kind::Integer
            && i.path.is_empty()
            && !i.path_truncated));
        let error = validate(parse("true")).unwrap_err();
        assert_eq!(error.issues[0].code, Code::TypeMismatch);
        assert!(error.issues[0].message.is_none());
    }
}
#[allow(dead_code)]
mod owned_array {
    include!("native_schema_owned_array.rs");
    #[test]
    fn owned_array_reuses_success_and_retains_ordered_error_paths() {
        let values = vec![2, 3];
        let pointer = values.as_ptr();
        let output = validate(values).unwrap();
        assert_eq!(output.as_ptr(), pointer);
        let error = validate(vec![2, 1, 3, 0]).unwrap_err();
        assert_eq!(error.issues[0].path, vec![Segment::Index(1)]);
        assert_eq!(error.issues[1].path, vec![Segment::Index(3)]);
        assert!(error.issues.iter().all(|i| i.observed == Kind::Integer));
    }
}
#[allow(dead_code)]
mod number_array {
    include!("native_schema_number_array_int.rs");
    #[test]
    fn number_array_checks_all_indices_without_lossy_output() {
        assert_eq!(
            validate(vec![42, 9007199254740994]).unwrap(),
            vec![42.0, 9007199254740994.0]
        );
        let error = validate(vec![42, 9007199254740993, 43, 9007199254740995]).unwrap_err();
        assert_eq!(error.issues.len(), 2);
        assert_eq!(error.issues[0].path, vec![Segment::Index(1)]);
        assert_eq!(error.issues[1].path, vec![Segment::Index(3)]);
        assert!(error.issues.iter().all(|i| i.code == Code::NumberPrecision));
    }
}
#[allow(dead_code)]
mod parse {
    include!("native_schema_parse.rs");
    #[test]
    fn parse_observes_success_and_caught_failure() {
        assert_eq!(function0(2, 7, &String::Borrowed("hello")), 19);
        assert_eq!(
            validate(std::string::String::new()).unwrap_err().issues[0].code,
            Code::TooSmall
        );
    }
}
