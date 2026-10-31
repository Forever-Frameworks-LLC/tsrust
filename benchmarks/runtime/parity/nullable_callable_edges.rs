#[allow(dead_code)]
mod guard {
    include!("nullable_callable_declared_guard.rs");
    #[test]
    fn absent_guard_skips_arguments_and_calls() {
        let result = function0(3, 0, &"x".into());
        assert_eq!(result.0[0].0, vec![3]);
        assert_eq!(result.0[1].0, vec![1, 1, 0, 0]);
        for at in [2, 4] {
            assert!(result.0[at].0.is_empty());
            assert_eq!(result.0[at + 1].0, vec![0, 0, 0, 0]);
        }
    }
}
#[allow(dead_code)]
mod optional {
    include!("nullable_callable_declared_optional.rs");
    #[test]
    fn optional_call_skips_both_absent_variants() {
        let result = function0(3, 0, &"x".into());
        assert_eq!(result.0[0].0, vec![3]);
        assert_eq!(result.0[3].0, vec![0, 0, 0, 0]);
        assert_eq!(result.0[5].0, vec![0, 0, 0, 0]);
    }
}
#[allow(dead_code)]
mod read {
    include!("nullable_callable_read_forward.rs");
    #[test]
    fn forwarding_borrows_reusable_capture_and_gates_argument() {
        let mut trace = Trace::default();
        let bias = 7;
        let action = |v, trace: &mut Trace| {
            trace.calls = add(trace.calls, 1);
            add(v, bias)
        };
        assert_eq!(forward(Some(&action), 3, &mut trace), Some(10));
        assert_eq!(
            forward(Some(&action).filter(|_| false), 5, &mut trace),
            None
        );
        assert_eq!(forward(Some(&action), 5, &mut trace), Some(12));
        assert_eq!((trace.calls, trace.arguments), (2, 2));
    }
}
#[allow(dead_code)]
mod once {
    include!("nullable_callable_once_forward.rs");
    #[test]
    fn forwarding_moves_and_consumes_owned_capture() {
        let mut trace = Trace::default();
        let owned = "é🦀".to_owned();
        let action = move |v, trace: &mut Trace| {
            trace.calls = add(trace.calls, 1);
            add(v, length(&owned.into_boxed_str()))
        };
        assert_eq!(forward_once(Some(action), 7, &mut trace), Some(13));
        assert_eq!((trace.calls, trace.arguments), (1, 1));
        let result = function0(1, 1, &"x".into());
        assert!(result.0[0].0.is_empty());
        assert_eq!(result.0[1].0, vec![0, 0, 0, 0]);
    }
}
#[allow(dead_code)]
mod owned {
    include!("nullable_callable_owned_record.rs");
    #[test]
    fn owned_record_outlives_input_and_remains_reusable() {
        let mut trace = Trace::default();
        let input = "é🦀".to_owned();
        let action = make_owned(0, 3, &input, &mut trace).unwrap();
        drop(input);
        assert_eq!(action(7, &mut trace), 16);
        assert_eq!(action(8, &mut trace), 17);
        assert!(make_owned(1, 0, "x", &mut trace).is_none());
        assert!(make_owned(2, 0, "x", &mut trace).is_none());
        assert_eq!((trace.calls, trace.factories), (2, 3));
    }
}
#[allow(dead_code)]
mod owned_once {
    include!("nullable_callable_owned_once_return.rs");
    #[test]
    fn returned_once_action_owns_and_consumes_text() {
        let mut trace = Trace::default();
        let input = "é🦀".to_owned();
        let action = make_owned_once(0, 3, &input, &mut trace).unwrap();
        drop(input);
        assert_eq!(action(7, &mut trace), 16);
        assert!(make_owned_once(1, 0, "x", &mut trace).is_none());
        assert_eq!((trace.calls, trace.factories), (1, 2));
    }
}
#[allow(dead_code)]
mod coalesce {
    include!("nullable_callable_generic_coalesce.rs");
    #[test]
    fn generic_relay_keeps_lazy_fallback_and_function_identity() {
        let result = function0(3, 0, &"x".into());
        assert_eq!(result.0[0].0, vec![3]);
        assert_eq!(result.0[1].0, vec![1, 1, 0, 0]);
        assert_eq!(result.0[2].0, vec![324]);
        assert_eq!(result.0[3].0, vec![1, 1, 1, 0]);
        assert_eq!(result.0[4].0, vec![545]);
    }
}
#[allow(dead_code)]
mod assign {
    include!("nullable_callable_nullish_assign.rs");
    #[test]
    fn nullish_assignment_stores_fallback_for_second_call() {
        let result = function0(2, 0, &"x".into());
        assert_eq!(result.0[0].0, vec![3, 20]);
        assert_eq!(result.0[1].0, vec![2, 2, 0, 0]);
        assert_eq!(result.0[2].0, vec![224, 241]);
        assert_eq!(result.0[3].0, vec![2, 2, 1, 0]);
    }
}
#[allow(dead_code)]
mod higher {
    include!("nullable_callable_higher_order_factory.rs");
    #[test]
    fn absent_outer_factory_skips_inner_factory_and_arguments() {
        let result = function0(3, 0, &"abc".into());
        assert_eq!(result.0[0].0, vec![16]);
        assert_eq!(result.0[1].0, vec![1, 1, 1, 1, 1, 0]);
        for at in [2, 4] {
            assert!(result.0[at].0.is_empty());
            assert_eq!(result.0[at + 1].0, vec![1, 0, 0, 0, 0, 0]);
        }
    }
}
#[allow(dead_code)]
mod absent_higher {
    include!("nullable_callable_absent_higher_order_factory.rs");
    #[test]
    fn absent_forwarders_only_construct_inner_for_final_fallback() {
        let result = function0(3, 0, &"abc".into());
        for i in 0..3 {
            assert_eq!(result.0[i * 2 + 1].0, vec![3, 1, 1, 1, 1, 1]);
        }
        assert_eq!(result.0[0].0, vec![16]);
        assert_eq!(result.0[2].0, vec![56]);
    }
}
