#[allow(dead_code)]
mod factory {
    include!("closure_factory.rs");
    #[test]
    fn moved_factory_result_keeps_state_between_calls() {
        let mut action = counter(7);
        assert_eq!((action(), action()), (8, 9));
        assert_eq!(function0(2, 7), 17);
    }
}
#[allow(dead_code)]
mod contracts {
    include!("callback_contracts.rs");
    #[test]
    fn forwarded_arguments_and_once_contract_keep_call_count() {
        let mut calls = 0;
        assert_eq!(
            forward(
                |v, flag| {
                    calls += 1;
                    if flag { add(v, 7) } else { v }
                },
                3
            ),
            10
        );
        assert_eq!(calls, 1);
        let owned = "owner".to_owned();
        assert_eq!(
            single(move || i64::try_from(owned.into_bytes().len()).unwrap()),
            5
        );
        assert_eq!(function0(2, 7), 29);
    }
}
#[allow(dead_code)]
mod owned {
    include!("owned_callbacks.rs");
    #[test]
    fn forwarded_owned_callback_keeps_same_capture_and_returns_owned_record() {
        let action = counter(3);
        let moved = identity(action);
        let mut wrapped = retain(moved);
        assert_eq!((wrapped(), wrapped()), (4, 5));
        let user = User {
            name: "owned".to_owned().into(),
            value: 7,
        };
        let result = consume(|user| user, user);
        assert_eq!((result.name.as_ref(), result.value), ("owned", 7));
        assert_eq!(function0(2, 7), 29);
    }
}
#[allow(dead_code)]
mod replacement {
    include!("callable_replacement.rs");
    #[test]
    fn replacing_callback_discards_old_state_and_moves_new_capture() {
        let mut action = counter(3);
        assert_eq!(action(), 4);
        action = counter(7);
        let mut moved = action;
        assert_eq!((moved(), moved()), (8, 9));
        assert_eq!(function0(2, 7), 19);
    }
}
#[allow(dead_code)]
mod recursive {
    include!("recursive_callbacks.rs");
    #[test]
    fn recursion_reborrows_or_moves_one_mutable_environment() {
        let mut seen = vec![];
        assert_eq!(
            visit(
                &mut |v| {
                    seen.push(v);
                    v
                },
                3
            ),
            6
        );
        assert_eq!(seen, vec![3, 2, 1, 0]);
        assert_eq!(countdown(counter(0), 4), 15);
        assert_eq!(function0(1, 7), 114);
    }
}
#[allow(dead_code)]
mod once {
    include!("once_factory_results.rs");
    #[test]
    fn generic_once_factory_transfers_owned_result() {
        let result = make(vec!["owned".to_owned()])();
        assert_eq!(result, vec!["owned"]);
        assert_eq!(function0(2, 7), 22);
    }
}
#[allow(dead_code)]
mod fields {
    include!("owned_callable_fields.rs");
    #[test]
    fn owned_field_replacement_selects_new_function_environment() {
        let base = 7;
        let mut handler = Handler {
            run: Box::new(move |v| add(base, v)),
        };
        assert_eq!((handler.run)(3), 10);
        handler.run = Box::new(move |v| sub(base, v));
        assert_eq!((handler.run)(3), 4);
        assert_eq!(function0(2, 7), 37);
    }
}
#[allow(dead_code)]
mod mutable_array {
    include!("callable_array_mutable_pipeline.rs");
    #[test]
    fn mapping_and_flattening_reuse_each_callable_capture() {
        let mut actions = vec![state_action(7, 1), state_action(7, 2)];
        assert_eq!(
            actions.iter_mut().map(|f| f()).collect::<Vec<_>>(),
            vec![8, 9]
        );
        assert_eq!(
            actions.iter_mut().map(|f| f()).collect::<Vec<_>>(),
            vec![9, 11]
        );
        assert_eq!(function0(1, 7), 68);
    }
}
#[allow(dead_code)]
mod reduction {
    include!("callable_array_reduction_pipeline.rs");
    #[test]
    fn reverse_reduction_invokes_right_capture_first() {
        let mut actions = vec![state_action(7, 1), state_action(7, 2)];
        let forward = actions.iter_mut().fold(0, |sum, f| add(sum, f()));
        let reverse = actions
            .iter_mut()
            .rev()
            .fold(0, |sum, f| add(mul(sum, 10), f()));
        assert_eq!((forward, reverse), (17, 119));
        assert_eq!(function0(1, 7), 166);
    }
}
#[allow(dead_code)]
mod transfer {
    include!("callable_array_transfer_pipeline.rs");
    #[test]
    fn splice_and_removal_keep_callable_order_and_survivors() {
        assert_eq!(function0(1, 7), 47);
        assert_eq!(action(7, 4)(), 11);
    }
}
#[allow(dead_code)]
mod once_transfer {
    include!("callable_array_once_transfer.rs");
    #[test]
    fn once_array_transfer_empties_source_before_consumption() {
        assert_eq!(function0(1, 7), 45);
        let text = "owned".to_owned();
        assert_eq!(
            consume(move || i64::try_from(text.into_bytes().len()).unwrap()),
            5
        );
    }
}
