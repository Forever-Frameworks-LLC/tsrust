#[allow(dead_code)]
mod keys {
    include!("object_keys_order.rs");
    #[test]
    fn constructor_order_clones_and_utf16_positions() {
        let source = Row::new(7, "é", true);
        assert_eq!(source.keys(), vec!["gamma", "alpha", "beta"]);
        let mut copied = source.clone();
        copied.beta.push_str("changed");
        let mut names = source.keys();
        names[0] = "changed";
        assert_eq!(copied.keys(), vec!["gamma", "alpha", "beta"]);
        assert_eq!(source.beta, "é");
        assert_eq!(
            Row::new(7, "é", false).keys(),
            vec!["beta", "gamma", "alpha"]
        );
        assert_eq!(text_keys("é😀"), vec!["0", "1", "2"]);
    }
}
#[allow(dead_code)]
mod entries {
    include!("object_values_entries.rs");
    #[test]
    fn values_and_entries_follow_construction_order_and_borrow_text() {
        let row = Row::new(7, "é", true);
        let values = row.values();
        assert_eq!(
            values
                .iter()
                .copied()
                .map(Field::render)
                .collect::<Vec<_>>(),
            vec!["true", "7", "é"]
        );
        if let Field::Text(text) = values[2] {
            assert_eq!(text.as_ptr(), row.beta.as_ptr());
        } else {
            panic!()
        };
        assert_eq!(
            row.entries()
                .iter()
                .map(|(key, _)| *key)
                .collect::<Vec<_>>(),
            vec!["gamma", "alpha", "beta"]
        );
        let sides = Sides { h: 7, w: 2 };
        assert_eq!(sides.values(), vec![7, 2]);
    }
}
#[allow(dead_code)]
mod own {
    include!("object_has_own.rs");
    #[test]
    fn undefined_fields_length_and_canonical_indices() {
        let row = Row {
            label: None,
            amount: 7,
        };
        assert!(row.has_own("label"));
        assert!(!row.has_own("toString"));
        assert!(has_position(2, &Property::Number(-0.0)));
        assert!(has_position(2, &Property::Text("0")));
        assert!(!has_position(2, &Property::Text("01")));
        assert!(!has_position(2, &Property::Number(0.5)));
        assert!(!has_position(2, &Property::Integer(-1)));
        assert!(has_position(0, &Property::Text("length")));
        let rows = function0(4, 0, &String::Borrowed("é😀"));
        assert_eq!(rows[2].0[6], "true");
        assert_eq!(rows[3].0[6], "false");
    }
}
#[allow(dead_code)]
mod membership {
    include!("in_operator.rs");
    #[test]
    fn record_and_array_membership_keep_distinct_key_forms() {
        let sample = Sample {
            name: "abc".to_owned(),
            amount: 7,
        };
        assert!(sample.contains("name"));
        assert!(!sample.contains("missing"));
        assert_eq!(sample.keys(), ["name", "amount"]);
        assert!(has_position(3, &Property::Integer(2)));
        assert!(!has_position(3, &Property::Integer(3)));
        assert!(!has_position(3, &Property::Number(1.5)));
        let rows = function0(1, 0, &String::Borrowed("abc"));
        assert_eq!(rows[0].0[4], "name,amount,");
        assert_eq!(rows[0].0[5], "012");
    }
}
#[allow(dead_code)]
mod data {
    include!("data_union_values.rs");
    #[test]
    fn record_clones_and_copied_owner_text_remain_independent() {
        let source = Row {
            value: Scalar::Text("é".to_owned()),
            owner: Owner::Integers(vec![7, 8]),
        };
        let mut copy = source.clone();
        copy.owner = Owner::Integers(vec![12]);
        if let Scalar::Text(text) = &mut copy.value {
            text.push_str("changed");
        }
        assert_eq!(source.value.render(), "é");
        assert_eq!(source.owner.render(), "7,8");
        let mut owner = Owner::Text("é".to_owned());
        let output = copied_text(&mut owner);
        if let Owner::Text(text) = &mut owner {
            assert_ne!(output.as_ptr(), text.as_ptr());
            text.push_str("changed");
        }
        assert_eq!(output, "é");
        assert_eq!(describe(&Scalar::Null), "missing");
        assert_eq!(Scalar::Null.kind(), "object");
    }
}
#[allow(dead_code)]
mod union {
    include!("union_array_values.rs");
    #[test]
    fn null_join_and_numeric_search_preserve_union_tags() {
        let values = vec![
            Item::Integer(7),
            Item::Text("é".to_owned()),
            Item::Null,
            Item::Integer(8),
        ];
        assert_eq!(joined(&values), "7|é||8");
        assert_eq!(Item::Null.render(), "null");
        let rows = function0(2, 0, &String::Borrowed("abc"));
        assert_eq!(rows[0].0[4], "-1");
        assert_eq!(rows[1].0[4], "3");
        assert_eq!(rows[0].0[6], "2");
        assert_eq!(rows[1].0[6], "1");
    }
}
#[allow(dead_code)]
mod heap {
    include!("union_array_heap.rs");
    #[test]
    fn heap_growth_changes_search_position_and_retains_every_number() {
        let rows = function0(2, 0, &String::Borrowed("abc"));
        assert_eq!(rows[0].0[0], "136");
        assert_eq!(rows[0].0[4], "3");
        assert_eq!(rows[1].0[4], "3");
        assert_eq!(rows[0].0[5], "tail");
        assert_eq!(rows[0].0[6], "2");
    }
}
#[allow(dead_code)]
mod fallback {
    include!("borrowed_optional_fallback.rs");
    #[test]
    fn none_falls_back_but_present_empty_string_does_not() {
        let rows = function0(1, 0, &String::Borrowed("é"));
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].0[0], "É");
        assert_eq!(rows[1].0[0], "NONE");
        assert_eq!(rows[1].0[1], "none");
        assert_eq!(rows[2].0[0], "");
        assert_eq!(rows[2].0[1], "");
    }
}
