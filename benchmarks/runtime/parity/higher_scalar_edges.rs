#[allow(dead_code)]
mod reusable {
    include!("higher_declared_reusable_int.rs");
    #[test]
    fn reusable_consumer_borrows_one_mutable_capture() {
        let mut calls = 0;
        let mut callback = |x| {
            calls = add(calls, 1);
            add(x, calls)
        };
        assert_eq!(apply(&mut callback, 7), 8);
        assert_eq!(apply(&mut callback, 7), 9);
        assert_eq!(calls, 2);
        assert_eq!(function0(1, 7), 939);
    }
}
#[allow(dead_code)]
mod once {
    include!("higher_declared_once_int.rs");
    #[test]
    fn once_consumer_can_move_owned_capture() {
        let owned = vec![7, 8];
        assert_eq!(
            apply(
                move |x| {
                    let values = owned;
                    add(x, values[0])
                },
                3
            ),
            10
        );
        assert_eq!(function0(1, 7), 939);
    }
}
#[allow(dead_code)]
mod owned {
    include!("higher_declared_owned_int.rs");
    #[test]
    fn owned_consumer_takes_mutable_environment_by_value() {
        let mut captured = vec![7];
        let callback = move |x| {
            captured[0] = add(captured[0], 1);
            add(x, captured[0])
        };
        assert_eq!(apply(callback, 3), 11);
    }
}
#[allow(dead_code)]
mod multiple {
    include!("higher_multiple_callbacks_int.rs");
    #[test]
    fn left_callback_runs_before_right_and_origins_remain_distinct() {
        let sequence = std::cell::RefCell::new(vec![]);
        let mut left = |x| {
            sequence.borrow_mut().push(1);
            add(x, 3)
        };
        let mut right = |x| {
            sequence.borrow_mut().push(2);
            mul(x, 2)
        };
        assert_eq!(pair(&mut left, &mut right, 7), 84);
        assert_eq!(*sequence.borrow(), vec![1, 2]);
        assert_eq!(function0(1, 7), 1109);
    }
}
#[allow(dead_code)]
mod borrowed_factory {
    include!("higher_borrowed_declared_factory_int.rs");
    #[test]
    fn generic_relay_preserves_independent_captured_factories() {
        let first = relay(&mut keep, plus(7));
        let second = relay(&mut keep, times(3));
        assert_eq!((first(2), second(2), first(3)), (9, 6, 10));
        assert_eq!(function0(1, 7), 974);
    }
}
#[allow(dead_code)]
mod nested {
    include!("higher_nested_consumer_int.rs");
    #[test]
    fn nested_function_factories_keep_callable_signature_and_capture() {
        let mut consumer = factory_factory()();
        let mut state = 0;
        let mut callback = |x| {
            state = add(state, 1);
            add(x, state)
        };
        assert_eq!(pass(&mut consumer, &mut callback, 7), 8);
        assert_eq!(pass(&mut consumer, &mut callback, 7), 9);
        assert_eq!(function0(1, 7), 1001);
    }
}
#[allow(dead_code)]
mod owned_nested {
    include!("higher_owned_once_nested_int.rs");
    #[test]
    fn nested_once_consumer_moves_capture_into_returned_factory() {
        let owned = "owned".to_owned();
        let callback = move |x| add(x, i64::try_from(owned.into_bytes().len()).unwrap());
        let factory = use_once(|f| bind(f, 7), callback);
        assert_eq!(factory(), 12);
        assert_eq!(function0(1, 7), 924);
    }
}
#[allow(dead_code)]
mod number {
    include!("higher_number_reusable.rs");
    #[test]
    fn reusable_number_callback_preserves_negative_zero_bits() {
        assert_eq!(apply(&mut identity, -0.0).to_bits(), (-0.0_f64).to_bits());
        assert_eq!(function0(2, 7), 3.75);
    }
}
#[allow(dead_code)]
mod boolean {
    include!("higher_boolean_reusable.rs");
    #[test]
    fn boolean_consumer_updates_parity_only_on_true_result() {
        assert!(!function0(0, 0));
        assert!(!function0(1, 0));
        assert!(function0(2, 0));
        assert!(!function0(3, 0));
    }
}
