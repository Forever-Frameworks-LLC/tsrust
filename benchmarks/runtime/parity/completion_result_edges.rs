#[allow(dead_code)]
mod conditional {
    include!("conditional_record_value.rs");
    #[test]
    fn only_selected_record_branch_updates_counters_and_payload() {
        assert_eq!(
            function0(2, 0).0,
            vec![101, 0, 202, 3, 104, 105, 106, 2, 1, 1]
        );
        let mut trace = Trace::default();
        let choice = left_value(&mut trace, 2, 2);
        assert_eq!((choice.tag, choice.payload), (101, vec![16, 17]));
        assert_eq!((trace.left, trace.right), (1, 0));
    }
}
#[allow(dead_code)]
mod logical {
    include!("logical_nullable_string_values.rs");
    #[test]
    fn logical_values_distinguish_missing_empty_and_truthy_unicode() {
        assert_eq!(
            function0(4, 0).0,
            vec![
                "Guest",
                "1",
                "undefined",
                "1",
                "Guest",
                "2",
                "",
                "2",
                "é",
                "2",
                "YES",
                "3",
                "😀",
                "3",
                "YES",
                "4"
            ]
        );
    }
}
#[allow(dead_code)]
mod completion {
    include!("exception_completion.rs");
    #[test]
    fn caught_failure_and_success_both_run_final_trace() {
        let mut trace = vec![];
        assert_eq!(attempt(0, &mut trace), 2);
        assert_eq!(attempt(1, &mut trace), 2);
        assert_eq!(trace, vec![0, 1]);
        assert_eq!(replaced(7), 11);
    }
    #[test]
    fn moved_field_return_keeps_sibling_callback_for_cleanup() {
        let text = "owned é🦀".to_owned();
        let pointer = text.as_ptr();
        let mut trace = vec![];
        let result = retained_field(RetainedOwner { text, stamp: 11 }, true, &mut trace);
        assert_eq!(result.as_ptr(), pointer);
        assert_eq!(trace, vec![11]);
        assert_eq!(
            retained_map(
                RetainedOwner {
                    text: "early".to_owned(),
                    stamp: 1
                },
                true,
                &mut trace
            ),
            "early"
        );
        assert_eq!(
            retained_map(
                RetainedOwner {
                    text: "late".to_owned(),
                    stamp: 1
                },
                false,
                &mut trace
            ),
            "late"
        );
        assert_eq!(trace, vec![11, 4, 32, 8]);
    }
    #[test]
    fn loop_and_catch_completions_run_cleanup_in_order() {
        for (mode, expected) in [
            (0, vec![20]),
            (1, vec![20]),
            (2, vec![20, 21]),
            (3, vec![20, 21]),
        ] {
            let mut trace = vec![];
            assert_eq!(
                retained_loop(
                    RetainedOwner {
                        text: "owned".to_owned(),
                        stamp: 0
                    },
                    mode,
                    &mut trace
                ),
                "owned"
            );
            assert_eq!(trace, expected);
        }
        for (mode, expected) in [(0, vec![31]), (1, vec![30, 31]), (2, vec![29, 31])] {
            let mut trace = vec![];
            assert_eq!(
                retained_catch(
                    RetainedOwner {
                        text: "owned".to_owned(),
                        stamp: 0
                    },
                    mode,
                    &mut trace
                ),
                "owned"
            );
            assert_eq!(trace, expected);
        }
    }
    #[test]
    fn cleanup_return_overrides_pending_owned_return() {
        assert_eq!(
            conditional_cleanup("owned".to_owned(), true, false),
            "owned"
        );
        assert_eq!(
            conditional_cleanup("owned".to_owned(), true, true),
            "alternate"
        );
        assert_eq!(
            conditional_cleanup("owned".to_owned(), false, false),
            "alternate"
        );
    }
}
#[allow(dead_code)]
mod callbacks {
    include!("array_callback_errors.rs");
    #[test]
    fn failed_map_discards_partial_results_and_keeps_input() {
        let values = vec![2, 0, 1];
        assert_eq!(mapped(&values, 0, "é🦀").unwrap_err(), "é🦀");
        assert_eq!(values, vec![2, 0, 1]);
        assert_eq!(sorted(&values).unwrap(), vec![0, 1, 2]);
        assert_eq!(sorted(&[2, -1, 0]).unwrap_err(), "negative");
        assert_eq!(
            function0(1, 0, &"é🦀".into())[0].0,
            vec!["é🦀", "1", "2,0,1", "0,1,2"]
        );
    }
}
#[allow(dead_code)]
mod results {
    include!("result_match.rs");
    #[test]
    fn result_read_mutation_and_owned_match_preserve_variant() {
        let original = make(3);
        assert_eq!(inspect(&original), 24);
        assert_eq!(inspect(&original), 24);
        let mut success = Ok(User { value: 3 });
        update(&mut success);
        update(&mut success);
        assert!(matches!(success, Ok(User { value: 7 })));
        let mut failure = Err(vec![3]);
        update(&mut failure);
        update(&mut failure);
        assert!(matches!(failure,Err(values) if values==vec![3,7,7]));
    }
    #[test]
    fn callback_result_keeps_mutable_state_or_owned_failure() {
        assert!(matches!(counter(4), Err(4)));
        let mut action = counter(3).unwrap();
        assert_eq!((action(), action()), (4, 5));
        assert_eq!(
            (
                failure_scenario(5),
                failure_scenario(1),
                failure_scenario(4)
            ),
            (6, 3, 2)
        );
        assert_eq!(
            function0(1, 0, &"results".into()).0[0].0,
            vec![2, 2, 8, 2, 3, 10, 3]
        );
    }
}
