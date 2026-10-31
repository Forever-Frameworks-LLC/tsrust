#[allow(dead_code)]
mod declarations {
    include!("nested_function_declarations.rs");
    #[test]
    fn recursion_shadowing_and_mapping_keep_function_identity() {
        assert_eq!(row(2), vec![4, 120, 1, 6, 4, 6, 10]);
        assert_eq!(row(3)[2], 0);
    }
}
#[allow(dead_code)]
mod captures {
    include!("nested_function_captures.rs");
    #[test]
    fn mutable_captures_persist_across_iterations() {
        let result = function0(2, 0, &"x".into());
        assert_eq!(result.0[0].0, vec![2, 2, 2, 3, 5, 8, 2]);
        assert_eq!(result.0[1].0, vec![6, 4, 3, 4, 6, 10, 4]);
    }
}
#[allow(dead_code)]
mod escape {
    include!("nested_function_owned_escape.rs");
    #[test]
    fn escaped_closures_keep_separate_state_and_owned_results() {
        let mut actions = [make(1), make(11)];
        assert_eq!((actions[0](), actions[1](), actions[0]()), (2, 12, 3));
        let recurse = recursive(vec![7]);
        assert_eq!((recurse(0), recurse(3), recurse(4)), (7, 8, 7));
        let take = owner(23);
        assert_eq!(take().value, 23);
        assert_eq!(failure(17)(), Err(17));
    }
}
#[allow(dead_code)]
mod results {
    include!("nested_function_results.rs");
    #[test]
    fn inferred_results_retain_unicode_scalar_length() {
        let input: String = "é🦀".into();
        let result = function0(1, 0, &input);
        assert_eq!(result.0[0].0, vec![2, 4, 3, 1, 2, 2]);
    }
}
#[allow(dead_code)]
mod defaults {
    include!("nested_function_defaults.rs");
    #[test]
    fn only_missing_arguments_evaluate_default() {
        let mut total = 1;
        let mut hits = 0;
        {
            let mut action = next(&mut total, &mut hits);
            assert_eq!(action(Some(0)), 1);
            assert_eq!(action(None), 3);
            assert_eq!(action(Some(3)), 6);
            assert_eq!(action(None), 8);
        }
        assert_eq!(hits, 2);
        let result = function0(1, 0, &"x".into());
        assert_eq!(result.0[0].0, vec![3, 6, 8, 2, 5, 2, 5, 1]);
    }
}
#[allow(dead_code)]
mod rest {
    include!("nested_function_rest.rs");
    #[test]
    fn empty_packs_and_explicit_zero_preserve_prefix_rules() {
        assert_eq!(sum(7, vec![]), 7);
        assert_eq!(sum(7, vec![2, 3]), 12);
        assert_eq!(front(7, Some(0), vec![1, 2]), 2);
        assert_eq!(front(7, None, vec![1, 2]), 9);
        let result = function0(1, 0, &"x".into());
        assert_eq!(result.0[0].0, vec![1, 6, 10, 1, 3, 4, 1]);
    }
}
