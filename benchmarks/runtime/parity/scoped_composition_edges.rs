#[allow(dead_code, unused_assignments)]
mod composition {
    include!("scoped_record_composition.rs");
    #[test]
    fn ordered_replacement_and_clear_release_captured_owners() {
        struct Guard<'a>(&'a Cell<i64>);
        impl Drop for Guard<'_> {
            fn drop(&mut self) {
                bump(self.0, 1);
            }
        }
        let drops = Cell::new(0);
        let mut map = Map::new();
        let guard = Guard(&drops);
        put(&mut map, "first", move || {
            let _keep = &guard;
            3
        });
        put(&mut map, "second", || 5);
        let guard = Guard(&drops);
        put(&mut map, "first", move || {
            let _keep = &guard;
            7
        });
        assert_eq!(drops.get(), 1);
        assert_eq!(
            map.iter().map(|(k, _)| *k).collect::<Vec<_>>(),
            vec!["first", "second"]
        );
        assert_eq!(observe(&map), 12);
        assert!(delete(&mut map, "first"));
        assert_eq!(drops.get(), 2);
        map.clear();
        assert_eq!(size(&map), 0);
    }
    #[test]
    fn pending_returns_are_copied_before_finally_mutates_sources() {
        for (mode, result) in [(0, Ok(8)), (1, Ok(18)), (2, Err(18))] {
            let mut trace = [0];
            assert_eq!(return_carrier_result(7, mode, &mut trace), result);
            assert_eq!(trace, [9]);
        }
        let mut trace = [0, 0, 0];
        assert_eq!(nested_returned_cleanup(false, 7, &mut trace), 14);
        assert_eq!(trace, [1, 0, 0]);
        let mut trace = [0, 0, 0];
        assert_eq!(nested_returned_cleanup(true, 7, &mut trace), 15);
        assert_eq!(trace, [1, 0, 0]);
    }
    #[test]
    fn cleanup_can_replace_return_error_or_loop_completion() {
        for (mode, result, trace_expected) in [
            (0, Ok(9), [32, 16]),
            (1, Ok(27), [32, 16]),
            (2, Ok(10), [63, 24]),
            (3, Err(9), [32, 16]),
            (4, Ok(37), [32, 16]),
        ] {
            let mut trace = [0, 0];
            assert_eq!(return_cleanup_entry_result(7, mode, &mut trace), result);
            assert_eq!(trace, trace_expected);
        }
        let expected = cleanup_override_entry(7, 0);
        for mode in 1..4 {
            assert_eq!(cleanup_override_entry(7, mode), expected);
            assert_eq!(handler_cleanup_override_entry(7, mode), expected);
        }
    }
    #[test]
    fn inner_cleanup_continue_overrides_pending_outer_edge() {
        assert_eq!(nested_cleanup_entry_mutation(7, false, false), 652);
        assert_eq!(nested_cleanup_entry_mutation(7, false, true), 643);
        assert_eq!(nested_cleanup_entry_mutation(7, true, false), 129);
        assert_eq!(nested_cleanup_entry_mutation(7, true, true), 331);
    }
    #[test]
    fn zero_iteration_and_post_test_loops_keep_different_stores() {
        assert_eq!(counted_entry_loop_release(7, 0), 19);
        assert_eq!(for_of_entry_loop_release(7, 0), 19);
        assert_eq!(post_entry_loop_release(7, 0), 29);
        assert_eq!(counted_entry_loop_release(7, 2), 30);
    }
    #[test]
    fn retained_removed_callback_tracks_its_original_source() {
        let first = Cell::new(7);
        let second = Cell::new(11);
        let mut entries = vec![held(&first), held(&second)];
        let removed = entries.pop().unwrap();
        entries[0] = held(&second);
        first.set(13);
        second.set(17);
        assert_eq!(removed.call(), 17);
        assert_eq!(entries[0].call(), 17);
        assert_eq!(first.get(), 13);
    }
    #[test]
    fn nullable_views_and_independent_parameter_owners() {
        assert!(nullable_owned_result(7).is_none());
        assert_eq!(nullable_owned_result(8).unwrap().call(), 9);
        assert_eq!(borrowed_nullable_reader(None), 1);
        assert_eq!(borrowed_nullable_views(7), 23);
        let left_capture = Cell::new(7);
        let right_capture = Cell::new(11);
        let mut left = Slot {
            saved: Some(held(&left_capture)),
        };
        let mut right = Slot {
            saved: Some(held(&right_capture)),
        };
        assert_eq!(independent_parameter_read(&left, &right), 18);
        independent_parameter_fill(&mut left, &mut right, 20);
        left_capture.set(13);
        right_capture.set(17);
        assert_eq!(independent_parameter_read(&left, &right), 42);
        assert_eq!((left_capture.get(), right_capture.get()), (13, 17));
    }
}
