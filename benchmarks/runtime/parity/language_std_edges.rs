#[allow(dead_code)]
mod optional {
    include!("optional_parameter_int_guard.rs");
    #[test]
    fn omitted_explicit_none_and_value_keep_call_counts() {
        let mut trace = Trace {
            calls: 0,
            defaults: 0,
        };
        assert_eq!(apply(&mut trace, None), 1000);
        assert_eq!(apply(&mut trace, None), 1000);
        assert_eq!(apply(&mut trace, Some(10)), 13);
        assert_eq!((trace.calls, trace.defaults), (3, 0));
        assert_eq!(
            function0(2, 0, &String::Borrowed("abc")).0,
            vec![1000, 1, 1000, 1]
        );
    }
}
#[allow(dead_code)]
mod identity {
    include!("object_is_values.rs");
    #[test]
    fn nan_payloads_signed_zero_and_mixed_integer_precision() {
        assert!(number_is(
            f64::from_bits(0x7ff8000000000001),
            f64::from_bits(0x7ff8000000000002)
        ));
        assert!(!number_is(-0.0, 0.0));
        assert!(number_is(f64::INFINITY, f64::INFINITY));
        assert!(!number_int_is(-0.0, 0));
        assert!(!number_int_is(i64::MAX as f64, i64::MAX));
        assert!(number_int_is(42.0, 42));
        let rows = function0(3, 0, &String::Borrowed("abc"));
        assert_eq!(rows.0[0].0, vec![true, true, true, false, true, true]);
    }
}
#[allow(dead_code)]
mod codes {
    include!("string_from_codes_text.rs");
    #[test]
    fn utf16_pairs_replacements_wrapping_and_scalar_endpoints() {
        assert_eq!(from_units([0xd83d, 0xde00]), "😀");
        assert_eq!(from_points([0xd83d, 0xde00]), "😀");
        assert_eq!(from_units([0xd800]), "�");
        assert_eq!(from_units([65536]), "\0");
        assert_eq!(
            from_points([0, 0x10ffff]),
            format!("\0{}", char::from_u32(0x10ffff).unwrap())
        );
        assert!(std::panic::catch_unwind(|| from_points([-1])).is_err());
        assert!(std::panic::catch_unwind(|| from_points([0x110000])).is_err());
    }
}
#[allow(dead_code)]
mod case {
    include!("string_locale_case_text.rs");
    #[test]
    fn unicode_case_expansion_and_context() {
        assert_eq!("ßs".to_uppercase(), "SSS");
        assert_eq!("ΟΣ".to_lowercase(), "ος");
        let rows = function0(3, 0, &String::Borrowed("abc"));
        assert_eq!(rows[0].0[0], "ab");
        assert_eq!(rows[2].0[1], "SSSSSSSSS");
    }
}
#[allow(dead_code)]
mod raw {
    include!("string_raw_segments.rs");
    #[test]
    fn raw_segments_interleave_only_available_substitutions() {
        let template = Template {
            raw: vec!["a".to_owned(), "é".to_owned(), "tail\\n".to_owned()],
        };
        assert_eq!(
            raw(&template, &[Insert::Integer(7), Insert::Boolean(true)]),
            "a7étruetail\\n"
        );
        assert_eq!(raw(&template, &[Insert::Text("X")]), "aXétail\\n");
        assert_eq!(
            raw(
                &Template {
                    raw: vec!["only".to_owned()]
                },
                &[Insert::Text("unused")]
            ),
            "only"
        );
        assert_eq!(raw(&Template { raw: Vec::new() }, &[]), "");
    }
}
#[allow(dead_code)]
mod enumeration {
    include!("enum_values.rs");
    #[test]
    fn explicit_and_implicit_numeric_enum_values() {
        assert_eq!(choose(0) as i64, 0);
        assert_eq!(choose(1) as i64, 4);
        assert_eq!(choose(2) as i64, 5);
        let rows = function0(2, 0, &String::Borrowed("abc"));
        assert_eq!(rows[0].0[1], "s");
        assert_eq!(rows[1].0[2], "abc4");
    }
}
#[allow(dead_code)]
mod aliases {
    include!("type_aliases.rs");
    #[test]
    fn alias_erasure_retains_union_tags_and_owned_data() {
        let ids: Ids = vec![Id::Number(7), Id::Text("é".to_owned())];
        assert_eq!(kind(&ids[0]), "number");
        assert_eq!(kind(&ids[1]), "text");
        let rows = function0(1, 7, &String::Borrowed("abc"));
        assert_eq!(rows[0].0[2], "14");
        assert_eq!(rows[0].0[3], "7");
        assert_eq!(rows[0].0[4], "2");
    }
}
#[allow(dead_code)]
mod optional_fields {
    include!("optional_fields.rs");
    #[test]
    fn optional_fields_borrow_and_mutate_independently() {
        let mut point = Point { x: 5, y: None };
        assert_eq!(width(&point), 5);
        point.y = Some(3);
        assert_eq!(width(&point), 8);
        assert_eq!(function0(1, 0, &String::Borrowed("abc"))[0].0[3], "9");
    }
}
#[allow(dead_code)]
mod interfaces {
    include!("interfaces.rs");
    #[test]
    fn flattened_inherited_fields_retain_optional_presence() {
        let player = Player {
            name: "p".to_owned(),
            x: 5,
            y: Some(2),
            score: 1,
        };
        assert_eq!(describe(&player), "p:7:1");
        assert_eq!(player.y, Some(2));
        assert_eq!(function0(1, 0, &String::Borrowed("abc"))[0].0[0], "abc:0:0");
    }
}
#[allow(dead_code)]
mod truth {
    include!("truthiness.rs");
    #[test]
    fn logical_assignment_and_loop_state_produce_observed_values() {
        assert_eq!(
            function0(1, 0, &String::Borrowed("truthiness")).0[0].0,
            vec![2, 6, 7, 1, 0, 1]
        );
        assert!(
            function0(0, 0, &String::Borrowed("truthiness"))
                .0
                .is_empty()
        );
    }
}
#[allow(dead_code)]
mod tuples {
    include!("tuple_types_values.rs");
    #[test]
    fn tuple_copies_and_mapped_strings_are_independent() {
        let mut original = make_pair(7, "é");
        let copy = copy_pair(&original);
        original.0 = 8;
        original.1.push_str("changed");
        assert_eq!(copy, (7, "é".to_owned()));
        assert_eq!(pair_length(&copy), 2);
        assert!(pair_is_array(&copy));
        let rows = function0(1, 0, &String::Borrowed("abc"));
        assert_eq!(rows[0].0[0], "1");
        assert_eq!(rows[0].0[2], "0");
        assert_eq!(rows[0].0[4], "1");
        assert_eq!(rows[0].0[7], "true");
    }
}
#[allow(dead_code)]
mod assertions {
    include!("type_assertions.rs");
    #[test]
    fn assertions_keep_runtime_integer_tag_and_numeric_widening() {
        let rows = function0(1, 5, &String::Borrowed("abc"));
        assert_eq!(rows[0].0[0], "5");
        assert_eq!(rows[0].0[1], "5.5");
        assert_eq!(rows[0].0[3], "abc5");
    }
}
#[allow(dead_code)]
mod concatenation {
    include!("string_concatenation.rs");
    #[test]
    fn scalar_and_absent_rendering_match_concatenation_order() {
        let rows = function0(3, 0, &String::Borrowed("abc"));
        assert_eq!(rows[0].0[4], "absent:nullundefined");
        assert_eq!(rows[0].0[5], "union:0");
        assert_eq!(rows[1].0[5], "union:undefined");
        assert_eq!(rows[2].0[2], "fraction:1");
        assert_eq!(rows[1].0[6], "abc1false");
    }
}
