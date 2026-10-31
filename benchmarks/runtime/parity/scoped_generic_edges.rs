#[allow(dead_code)]
mod generic {
    include!("generic_value_adapters.rs");
    #[test]
    fn one_environment_keeps_counts_across_generic_instantiations() {
        let mut environment = make(7);
        let first = environment.select(3);
        let text = "owned é🦀".to_owned();
        let pointer = text.as_ptr();
        let second = environment.select(text);
        let third = environment.select(true);
        assert_eq!((first.value, first.calls), (3, 8));
        assert_eq!(second.value.as_ptr(), pointer);
        assert_eq!((second.calls, third.calls), (9, 10));
        assert!(third.value);
        let textSeed = generic_factory("seed".to_owned(), 7);
        let scalarSeed = generic_factory(7_i64, 7);
        assert_eq!((textSeed.step, scalarSeed.step), (2, 1));
    }
    #[test]
    fn owned_patterns_move_payload_and_drop_unused_elements() {
        let mut environment = make(10);
        let value = "owned é🦀".to_owned();
        let pointer = value.as_ptr();
        let first = environment.head(vec![value, "unused".to_owned()]);
        assert_eq!(first.value.as_ptr(), pointer);
        assert_eq!(first.calls, 11);
        let second = environment.record(PatternBox { value: vec![7, 8] });
        assert_eq!(second.value, vec![7, 8]);
        assert_eq!(second.calls, 12);
    }
    #[test]
    fn defaults_and_rest_preserve_lazy_and_cumulative_calls() {
        let mut defaults = Defaults { calls: 0 };
        assert_eq!(defaults.select(7, None).calls, 1);
        assert_eq!(defaults.select(7, Some(9)).calls, 9);
        assert_eq!(defaults.calls, 1);
        assert_eq!(defaults.select(7, None).calls, 2);
        assert_eq!(defaults.select("after".to_owned(), None).calls, 3);
        let mut rest = Rest { calls: 0 };
        assert_eq!(rest.select(7, vec![]).calls, 1);
        assert_eq!(rest.select(8, vec![9, 10]).calls, 4);
        assert_eq!(
            rest.select("after".to_owned(), vec!["one".to_owned()])
                .calls,
            4
        );
    }
    #[test]
    fn reducer_predicate_and_comparator_share_state_across_types() {
        let mut reducer = Reducer { calls: 0 };
        let reduced = [1, 2, 3].into_iter().fold(4, |a, v| reducer.select(a, v));
        assert_eq!(reduced, 4);
        assert_eq!(
            reducer.select("before".to_owned(), "fourth".to_owned()),
            "fourth"
        );
        let mut predicate = Predicate { calls: 0 };
        assert_eq!(
            [1, 2, 3].into_iter().position(|v| predicate.accept(v)),
            Some(1)
        );
        assert!(!predicate.accept("after"));
        let mut comparator = Comparator { seen: 0 };
        let mut values = vec![3_i64, 1, 2];
        values.sort_by(|a, b| comparator.compare(a, b).cmp(&0));
        assert_eq!(values, vec![3, 1, 2]);
        assert_eq!(
            comparator.compare(&"after".to_owned(), &"unused".to_owned()),
            1
        );
    }
    #[test]
    fn payload_returned_functions_keep_default_and_rest_contracts() {
        let mut payload = payload_factory(10);
        let sum = payload.select(sum_payload as fn(Option<i64>, Vec<i64>) -> i64);
        assert_eq!(
            (
                sum(None, vec![]),
                sum(None, vec![1, 2]),
                sum(Some(7), vec![3, 4])
            ),
            (4, 7, 14)
        );
        payload.select(sum_payload as fn(Option<i64>, Vec<i64>) -> i64);
        assert!(payload.select(true));
        assert_eq!(payload.calls, 13);
    }
    #[test]
    #[should_panic]
    fn payload_boolean_rejects_wrong_shared_call_position() {
        payload_factory(10).select(true);
    }
    #[test]
    fn polymorphic_carriers_and_lexical_owners_keep_typed_values() {
        let first = retain_polymorphic(First)();
        let second = retain_polymorphic(Second)();
        assert_eq!(first.call(7), 7);
        assert_eq!(first.call("owned".to_owned()), "owned");
        assert_eq!(second.call(0, 9), 9);
        let value = "owned é🦀".to_owned();
        let pointer = value.as_ptr();
        let captured = lexical_capture(value);
        assert_eq!(captured().as_ptr(), pointer);
        let result = function0(1, 0, &"é🦀".into());
        assert_eq!(result.0[0].0[16], 6);
    }
}
#[allow(dead_code)]
mod scoped {
    include!("scoped_callable_records.rs");
    #[test]
    fn holder_replacement_releases_borrow_before_later_mutation() {
        let mut base = vec![7, 8];
        {
            let mut holder = Holder {
                run: Box::new(|| base[0]),
            };
            assert_eq!(call(&holder), 7);
            replace(&mut holder, 14);
            assert_eq!(call(&holder), 14);
        }
        base[0] = 0;
        assert_eq!(base, vec![0, 8]);
        assert_eq!(
            function0(1, 0, &"records".into()).0[0].0,
            vec![1, 2, 3, 8, 4, 2, 2]
        );
    }
}
#[allow(dead_code)]
mod fallible {
    include!("fallible_record_callbacks.rs");
    #[test]
    fn typed_callback_failures_keep_utf8_byte_lengths_and_numeric_codes() {
        let plain = Holder {
            run: Box::new(|| Ok(3)),
        };
        let text = Holder {
            run: Box::new(|| Err(Failure::Text("é🦀".to_owned()))),
        };
        let coded = Holder {
            run: Box::new(|| Err(Failure::Code(7))),
        };
        assert_eq!(
            (guarded(&plain), guarded(&text), guarded(&coded)),
            (3, 6, 1007)
        );
        assert!(matches!(checked(&plain,2),Err(Failure::Text(text)) if text=="big"));
        assert!(matches!(checked(&coded, 2), Err(Failure::Code(7))));
    }
}
