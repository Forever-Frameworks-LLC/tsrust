#[allow(dead_code)]
mod contexts {
    include!("generic_function_contexts.rs");
    #[test]
    fn one_environment_counts_all_generic_instantiations() {
        let mut context = Context { calls: 0 };
        assert_eq!(context.select(7), 7);
        assert_eq!(context.select("owned".to_owned()), "owned");
        assert_eq!(context.select(vec![2, 3]), vec![2, 3]);
        let mut values = vec!["head".to_owned(), "tail".to_owned()];
        assert_eq!(context.head(&mut values), "head");
        assert_eq!(values, vec!["tail"]);
        assert_eq!(context.calls, 4);
    }
}
#[allow(dead_code)]
mod deferred {
    include!("generic_function_deferred.rs");
    #[test]
    fn generic_owned_results_survive_argument_scope() {
        let result = identity(vec!["owned".to_owned()]);
        assert_eq!(first(result), "owned");
        assert_eq!(offset("selected".to_owned(), None), "selected");
        assert_eq!(size::<i64>(&[]), 0);
    }
}
#[allow(dead_code)]
mod captures {
    include!("generic_function_captures.rs");
    #[test]
    fn separate_generic_environments_retain_shared_per_instance_counts() {
        let mut first = Counter { calls: 1 };
        let mut second = Counter { calls: 11 };
        assert_eq!(first.visit(7), 2);
        assert_eq!(second.visit("text"), 12);
        assert_eq!(first.visit(vec![1, 2]), 3);
        assert_eq!(second.visit(false), 13);
    }
}
#[allow(dead_code)]
mod results {
    include!("generic_function_results.rs");
    #[test]
    fn generic_fresh_arrays_are_independently_owned() {
        let mut action = Fresh { calls: 0 };
        let mut first: Vec<i64> = action.fresh();
        let second: Vec<i64> = action.fresh();
        let mut text: Vec<std::string::String> = action.fresh();
        first.push(7);
        text.push("owned".to_owned());
        assert!(second.is_empty());
        assert_eq!(first, vec![7]);
        assert_eq!(text, vec!["owned"]);
        assert_eq!(action.calls, 3);
    }
}
#[allow(dead_code)]
mod replacement {
    include!("generic_callable_replacement.rs");
    #[test]
    fn selected_function_keeps_its_defaults_owned_returns_and_failures() {
        assert_eq!((defaulted(false), defaulted(true)), (3, 7));
        assert_eq!((attempt(false, 4), attempt(true, 4)), (4, 11));
        assert_eq!(selected_text(false), "head");
        assert_eq!(selected_text(true), "next-value");
        assert_eq!(nullable_length(false, None), 4);
        assert_eq!(nullable_length(true, None), 0);
        assert_eq!(nullable_length(true, Some("é🦀".to_owned())), 2);
    }
    #[test]
    fn callback_and_captured_errors_propagate_before_addition() {
        assert_eq!(callback_result(false, false, 3), 3);
        assert_eq!(callback_result(true, false, 3), 10);
        assert_eq!(callback_result(false, true, 3), 11);
        assert_eq!(callback_result(true, true, 3), 11);
        assert_eq!(captured_result(false, true, 3), 6);
        assert_eq!(captured_result(true, false, 3), 13);
        assert_eq!(captured_result(true, true, 3), 11);
    }
    #[test]
    fn replacement_returns_old_function_and_is_independent_per_factory() {
        assert_eq!(state_result(3), vec![3, 10, 3, 12]);
        assert_eq!(pool_result(3), vec![3, 3, 12]);
        assert_eq!(nested_pool_result(3), vec![3, 3, 12]);
        let mut state = state_factory(StateAction::First);
        assert_eq!(state(false).call("first".to_owned(), None), "first");
        assert_eq!(
            state(true).call("ignored".to_owned(), Some("second".to_owned())),
            "second"
        );
    }
}
