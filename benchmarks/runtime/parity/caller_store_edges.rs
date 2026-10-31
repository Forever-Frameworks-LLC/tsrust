#[allow(dead_code)]
mod stores {
    include!("caller_borrowed_record_stores.rs");
    #[test]
    fn installed_failure_keeps_new_callback_but_early_failure_keeps_old() {
        let source = Cell::new(7);
        let action = || Ok(source.get());
        let mut holder = Holder {
            run: Box::new(|| Ok(3)),
        };
        assert_eq!(
            fail_before_install(&mut holder, &action, true),
            Err("before")
        );
        assert_eq!(holder.call(), 3);
        assert_eq!(install_and_fail(&mut holder, &action), Err("installed"));
        assert_eq!(holder.call(), 7);
    }
    #[test]
    fn paired_callback_error_skips_right_callback() {
        let calls = Cell::new(0);
        let left = || Err(Failure::Code(7));
        let right = || {
            bump(&calls, 1);
            Ok(9)
        };
        let mut holder = Holder {
            run: Box::new(|| Ok(0)),
        };
        install_pair(&mut holder, &left, &right);
        assert!(matches!((holder.run)(), Err(Failure::Code(7))));
        assert_eq!(calls.get(), 0);
    }
    #[test]
    fn replacement_and_clear_drop_captured_owner_immediately() {
        struct Guard<'a>(&'a Cell<i64>);
        impl Drop for Guard<'_> {
            fn drop(&mut self) {
                bump(self.0, 1);
            }
        }
        let drops = Cell::new(0);
        let source = Cell::new(7);
        let guard = Guard(&drops);
        let mut holder = Nullable {
            run: Some(Box::new(move || {
                let _keep = &guard;
                3
            })),
        };
        assert_eq!(drops.get(), 0);
        install_captured_source(&mut holder, &source);
        assert_eq!(drops.get(), 1);
        assert_eq!(holder.call(), 7);
        holder.clear();
        assert!(holder.run.is_none());
        let guard = Guard(&drops);
        holder.run = Some(Box::new(move || {
            let _keep = &guard;
            4
        }));
        holder.clear();
        assert_eq!(drops.get(), 2);
    }
    #[test]
    fn replacement_releases_original_state_and_retains_replacement_state() {
        let original = Cell::new(7);
        let replacement = Cell::new(11);
        let original_action = || bump(&original, 1);
        let replacement_action = || bump(&replacement, 3);
        let mut holder = Nullable::empty();
        install_nullable(&mut holder, &original_action);
        install_nullable(&mut holder, &replacement_action);
        assert_eq!(original_action(), 8);
        assert_eq!(holder.call(), 14);
        assert_eq!(original.get(), 8);
        holder.clear();
        assert_eq!(replacement_action(), 17);
        assert!(holder.run.is_none());
    }
    #[test]
    fn conditional_installer_keeps_branch_and_evaluation_effect_order() {
        let original = Cell::new(7);
        let replacement = Cell::new(11);
        let calls = Cell::new(0);
        let mut holder = Nullable::empty();
        install_conditional(&mut holder, &original, true, &calls);
        assert_eq!((holder.call(), calls.get()), (8, 119));
        install_conditional(&mut holder, &replacement, false, &calls);
        assert_eq!((holder.call(), calls.get()), (13, 343));
        bump(&original, 1);
        assert_eq!(holder.call(), 13);
    }
    #[test]
    fn nullable_data_and_nested_installers_keep_shared_effects() {
        let original = Cell::new(7);
        let replacement = Cell::new(11);
        let calls = Cell::new(0);
        let mut holder = Nullable::empty();
        install_nullable_data(&mut holder, &original, &calls);
        assert_eq!(holder.call(), 8);
        install_nullable_data(&mut holder, &replacement, &calls);
        assert_eq!((holder.call(), calls.get()), (12, 2));
        holder.clear();
        with_installer(
            |target| {
                with_installer(
                    |nested| {
                        bump(&calls, 1);
                        install_captured_source(nested, &original);
                    },
                    target,
                )
            },
            &mut holder,
        );
        assert_eq!((holder.call(), calls.get()), (8, 3));
    }
    #[test]
    fn owned_transfer_keeps_state_and_region_cleanup_selects_last() {
        let mut holder = OwnedNullable { run: None };
        let values = vec![7];
        let transfer = move |holder: &mut OwnedNullable| install_owned_mutation(holder, values);
        transfer(&mut holder);
        assert_eq!(holder.run.as_mut().unwrap()(), 8);
        assert_eq!(holder.run.as_mut().unwrap()(), 9);
        holder.run = None;
        let original = Cell::new(7);
        let replacement = Cell::new(11);
        let mut borrowed = Nullable::empty();
        install_through_finally(&mut borrowed, &original, &replacement);
        assert_eq!(borrowed.call(), 11);
        install_through_catch(&mut borrowed, &original, &replacement, false);
        assert_eq!(borrowed.call(), 7);
        install_through_catch(&mut borrowed, &original, &replacement, true);
        assert_eq!(borrowed.call(), 11);
    }
}
