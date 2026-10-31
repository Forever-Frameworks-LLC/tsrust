#[allow(dead_code)]
mod patterns {
    include!("destructuring_patterns.rs");
    #[test]
    fn tuple_and_record_binding_keep_unicode_and_values() {
        assert_eq!(pair(7, "é"), (8, "é!".to_owned()));
        let rows = function0(1, 7, &String::Borrowed("é"));
        assert_eq!(rows[0].0[0], "15");
        assert_eq!(rows[0].0[1], "é!é");
        assert_eq!(rows[0].0[2], "ab");
        assert_eq!(rows[0].0[3], "15");
    }
}
#[allow(dead_code)]
mod defaults {
    include!("destructuring_defaults.rs");
    #[test]
    fn defaults_are_lazy_and_null_does_not_become_undefined() {
        let rows = function0(3, 0, &String::Borrowed("abc"));
        assert_eq!(rows[0].0[2], "none:0");
        assert_eq!(rows[1].0[2], "abcabc:null");
        assert_eq!(rows[1].0[3], "null");
        assert_eq!(rows[2].0[2], "abcabcabc:7");
        assert_eq!(rows[2].0[3], "8");
        assert_eq!(
            rows.iter().map(|row| row.0[4].as_ref()).collect::<Vec<_>>(),
            vec!["1", "1", "2"]
        );
    }
}
#[allow(dead_code)]
mod rest {
    include!("destructuring_rest.rs");
    #[test]
    fn spread_overwrite_preserves_order_and_owned_storage() {
        let row = Row {
            name: "é".to_owned(),
            count: 7,
            active: false,
            order: [Key::Active, Key::Name, Key::Count],
        };
        let mut merged = merge(&row, 1, 3);
        merged.count = 8;
        assert_eq!(keys(&merged.order), "before,active,name,count,after");
        assert_ne!(merged.name.as_ptr(), row.name.as_ptr());
        assert_eq!(row.count, 7);
        let rows = function0(1, 0, &String::Borrowed("abc"));
        assert_eq!(rows[0].0[5], "active,count");
        assert_eq!(rows[0].0[9], "3");
        assert_eq!(rows[0].0[10], "3");
    }
}
#[allow(dead_code)]
mod nested {
    include!("nested_rest_patterns.rs");
    #[test]
    fn nested_tail_default_and_original_length_remain_distinct() {
        assert_eq!(describe(&["unused".to_owned(), "é".to_owned()]), "é:tail");
        assert_eq!(
            describe(&["unused".to_owned(), "é".to_owned(), "present".to_owned()]),
            "é:present"
        );
        let rows = function0(1, 0, &String::Borrowed("abc"));
        assert_eq!(rows[0].0[2], "abcabc|end");
        assert_eq!(rows[0].0[3], "2");
        assert_eq!(rows[0].0[8], "3");
    }
}
#[allow(dead_code)]
mod for_patterns {
    include!("for_pattern_initializers.rs");
    #[test]
    fn continue_updates_and_rest_pushes_terminate_at_correct_length() {
        let rows = function0(1, 0, &String::Borrowed("abc"));
        assert_eq!(rows[0].0[0], "2");
        assert_eq!(rows[0].0[1], "3");
        assert_eq!(rows[0].0[2], "abc1abc2");
        assert_eq!(rows[0].0[3], "2");
    }
}
#[allow(dead_code)]
mod assignments {
    include!("destructuring_assignment.rs");
    #[test]
    fn simultaneous_swaps_fibonacci_and_partial_record_moves() {
        let rows = function0(1, 7, &String::Borrowed("é"));
        assert_eq!(rows[0].0[0], "7");
        assert_eq!(rows[0].0[1], "é");
        assert_eq!(rows[0].0[2], "5");
        assert_eq!(rows[0].0[3], "9");
        assert_eq!(rows[0].0[4], "é?");
        assert_eq!(rows[0].0[5], "16");
        assert_eq!(rows[0].0[6], "é?!");
    }
}
#[allow(dead_code)]
mod braceless {
    include!("braceless_bodies.rs");
    #[test]
    fn dangling_else_and_post_test_loop_keep_their_binding() {
        assert_eq!(grade(19), "low");
        assert_eq!(grade(20), "mid");
        assert_eq!(grade(60), "high");
        let rows = function0(1, 0, &String::Borrowed("abc"));
        assert_eq!(rows[0].0[3], "23");
        assert_eq!(rows[0].0[4], "5");
    }
}
#[allow(dead_code)]
mod switches {
    include!("switch_statements.rs");
    #[test]
    fn grouped_cases_and_continue_cross_the_switch_correctly() {
        assert_eq!(
            (0..7).map(tier).collect::<Vec<_>>(),
            vec!["zero", "low", "low", "other", "other", "five", "other"]
        );
        let rows = function0(1, 13, &String::Borrowed("abc"));
        assert_eq!(rows[0].0[1], "422");
        assert_eq!(rows[0].0[3], "thirteen");
        let rows = function0(1, 0, &String::Borrowed("dynamic é🙂 text"));
        assert_eq!(rows[0].0[2], "long");
        assert_eq!(rows[0].0[3], "THREE");
    }
}
#[allow(dead_code)]
mod deferred {
    include!("deferred_initialization.rs");
    #[test]
    fn definite_assignment_covers_each_branch_and_optional_starts_absent() {
        let mid = function0(1, 32, &String::Borrowed("abc"));
        assert_eq!(mid[0].0[0], "mid");
        assert_eq!(mid[0].0[1], "5");
        assert_eq!(mid[0].0[2], "few");
        let high = function0(1, 64, &String::Borrowed("abc"));
        assert_eq!(high[0].0[0], "high");
        assert_eq!(high[0].0[4], "true");
    }
}
#[allow(dead_code)]
mod labels {
    include!("labeled_control.rs");
    #[test]
    fn outer_continue_runs_updates_and_outer_break_skips_them() {
        let rows = function0(1, 0, &String::Borrowed("abc"));
        assert_eq!(rows[0].0[0], "101112202122");
        assert_eq!(rows[0].0[1], "3");
        assert_eq!(rows[0].0[2], "1");
        assert_eq!(rows[0].0[3], "1");
        assert_eq!(rows[0].0[4], "1");
        assert_eq!(rows[0].0[5], "0");
        let rows = function0(1, 1, &String::Borrowed("abc"));
        assert_eq!(rows[0].0[0], "000102202122");
        assert_eq!(rows[0].0[2], "2");
        assert_eq!(rows[0].0[3], "2");
        assert_eq!(rows[0].0[4], "2");
    }
}
#[allow(dead_code)]
mod owned {
    include!("owned_assignment_values.rs");
    #[test]
    fn assignment_results_borrow_the_new_owner_after_store() {
        let output = function0(1, 0, &String::Borrowed("assignment"));
        assert_eq!(output.0[0].0, vec![1, 5, 3, 3, 2, 2, 2, 1, 5]);
        let output = function0(1, 4, &String::Borrowed("assignment"));
        assert_eq!(output.0[0].0, vec![5, 5, 3, 7, 10, 6, 6, 5, 5]);
    }
}
#[allow(dead_code)]
mod scalar_assertions {
    include!("testing_scalar_assertions.rs");
    #[test]
    fn assertions_read_the_actual_table_and_preserve_checksum() {
        assert_eq!(function0(1, 3), 9);
        assert_eq!(function0(2, 3), 1179);
        assert_eq!(function0(0, 3), 0);
    }
}
#[allow(dead_code)]
mod predicates {
    include!("testing_expectation_predicates.rs");
    #[test]
    fn predicate_sign_nan_precision_and_unicode_length_edges() {
        assert!(same(
            Actual::Number(f64::NAN),
            Actual::Number(f64::from_bits(0x7ff8000000000001))
        ));
        assert!(!same(Actual::Number(-0.0), Actual::Number(0.0)));
        assert!(!Actual::Number(f64::NAN).truthy());
        assert!(!Actual::Number(-0.0).truthy());
        assert!(Actual::Number(f64::INFINITY).truthy());
        assert!(close(f64::INFINITY, f64::INFINITY, f64::NAN));
        assert!(!close(1.0, 1.0, 10000.0));
        assert!(!close(1.0, 1.0, f64::NAN));
        assert!(close(1.0, 1.001, 2.0));
        assert!(!close(1.0, 1.125, 5.0));
        assert_eq!("😀".encode_utf16().count(), 2);
        assert_eq!(function0(1, 0), 1);
    }
}
