#[allow(dead_code)]
mod integer {
    include!("declared_callback_int_reduce.rs");
    #[test]
    fn ordered_fold_preserves_empty_single_and_full_prefixes() {
        assert_eq!(
            (count(0, 0), count(1, 0), count(2, 0), count(3, 0)),
            (0, 1, 4, 5)
        );
        assert_eq!([1, 2, 3].iter().fold(0, fold), 326);
        assert_eq!(function0(1, 0), 0);
        assert_eq!(function0(1, 1), 133);
    }
}
#[allow(dead_code)]
mod number {
    include!("declared_callback_number_reduce.rs");
    #[test]
    fn floating_fold_keeps_exact_quarters_and_empty_seed() {
        assert_eq!(function0(1, 0).to_bits(), 0.0_f64.to_bits());
        assert_eq!(function0(1, 1), 0.25);
        assert_eq!(function0(1, 2), 3.5);
        assert_eq!(function0(1, 3), 2.5);
    }
}
#[allow(dead_code)]
mod boolean {
    include!("declared_callback_boolean_reduce.rs");
    #[test]
    fn boolean_fold_keeps_ordered_parity_and_empty_identity() {
        assert!(!function0(1, 0));
        assert!(function0(1, 1));
        assert!(!function0(1, 2));
        assert!(!function0(1, 3));
    }
}
#[allow(dead_code)]
mod string_map {
    include!("declared_callback_string_map.rs");
    #[test]
    fn mapped_unicode_and_nul_text_has_independent_owner() {
        assert_eq!(paint("x😀xx"), "é\0😀é\0é\0");
        let input: String = "xé🦀".into();
        let output = function0(1, 1, &input);
        drop(input);
        assert_eq!(output[0][0], "é\0é🦀");
    }
}
#[allow(dead_code)]
mod record_map {
    include!("declared_callback_record_map.rs");
    #[test]
    fn mapped_record_copies_owned_fields_and_applies_index() {
        let mut source = Record0 {
            field0: 7,
            field1: "é🦀".to_owned().into(),
            field2: vec![1, 2],
        };
        let mapped = change(&source, 2);
        source.field1.to_mut().push('!');
        source.field2[0] = 99;
        assert_eq!(mapped.field0, 10);
        assert_eq!(mapped.field1, "é🦀");
        assert_eq!(mapped.field2, vec![1, 2]);
    }
}
#[allow(dead_code)]
mod returned {
    include!("declared_value_returned_function.rs");
    #[test]
    fn producing_function_runs_side_effect_before_returned_call() {
        let mut probe = Probe { state: 7 };
        let action = produce(&mut probe);
        assert_eq!(probe.state, 77);
        assert_eq!(action(7), 22);
        assert_eq!(function0(1, 7), 1016);
    }
}
#[allow(dead_code)]
mod factory {
    include!("declared_value_factory_target.rs");
    #[test]
    fn factory_owns_prefix_and_uses_original_source_for_tokens() {
        let action = make("é🦀".to_owned());
        assert_eq!(action("x😀"), "é🦀😀");
        assert_eq!(replace_x("axb", "$$:$&:$`:$'"), "a$:x:a:bb");
        assert_eq!(replace_x("xéx", "$`"), "éxé");
    }
}
#[allow(dead_code)]
mod moved_tree {
    include!("declared_value_owned_tree_move.rs");
    #[test]
    fn owned_tree_transfer_keeps_descendant_allocation() {
        let root = Record0 {
            field0: 7,
            field1: "é🦀".to_owned().into(),
            field2: vec![Record0 {
                field0: 1,
                field1: "child".into(),
                field2: vec![],
            }],
        };
        let pointer = root.field2.as_ptr();
        let result = take(root);
        assert_eq!(result.field2.as_ptr(), pointer);
        assert_eq!(result.field1, "é🦀");
        assert_eq!(result.field2[0].field0, 1);
    }
}
