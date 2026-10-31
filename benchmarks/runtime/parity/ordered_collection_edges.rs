#[allow(dead_code)]
mod values {
    include!("set_values.rs");
    #[test]
    fn set_duplicate_delete_reinsert_preserves_insertion_order() {
        let mut values = set([3, 4, 3]);
        assert_eq!(values, vec![3, 4]);
        assert!(delete(&mut values, &3));
        assert!(!delete(&mut values, &3));
        insert(&mut values, 3);
        assert_eq!(values, vec![4, 3]);
        assert_eq!(
            function0(1, 0, &"tail".into())[0].0,
            vec!["2", "true", "1,0", "1", "0", "1:0:", "tail"]
        );
    }
}
#[allow(dead_code)]
mod maps {
    include!("map_values.rs");
    #[test]
    fn update_keeps_key_position_and_reinsert_appends() {
        let mut values = map([("a", 1), ("b", 2), ("a", 3)]);
        assert_eq!(values, vec![("a", 3), ("b", 2)]);
        put(&mut values, "b", 4);
        assert_eq!(get(&values, &"b"), Some(&4));
        assert!(remove(&mut values, &"a"));
        put(&mut values, "a", 5);
        assert_eq!(values, vec![("b", 4), ("a", 5)]);
    }
}
#[allow(dead_code)]
mod snapshots {
    include!("map_snapshots.rs");
    #[test]
    fn snapshots_keep_owned_entries_after_receiver_clear() {
        let mut values = map([(0, "é🦀".to_owned()), (1, "tail".to_owned())]);
        let snapshot = values.clone();
        values.clear();
        assert_eq!(
            snapshot,
            vec![(0, "é🦀".to_owned()), (1, "tail".to_owned())]
        );
        assert_eq!(
            function0(1, 0, &"é🦀".into())[0].0,
            vec!["1,0", "tail|é🦀", "1", "tail", "0", "é🦀", "0"]
        );
    }
}
#[allow(dead_code)]
mod algebra {
    include!("set_algebra.rs");
    #[test]
    fn algebra_uses_defined_order_including_smaller_right_intersection() {
        let left = vec![3, 1, 2];
        let right = vec![2, 4, 1];
        let small = vec![2, 1];
        assert_eq!(union(&left, &right), vec![3, 1, 2, 4]);
        assert_eq!(intersection(&left, &right), vec![1, 2]);
        assert_eq!(intersection(&left, &small), vec![2, 1]);
        assert_eq!(difference(&left, &right), vec![3]);
        assert_eq!(symmetric(&left, &right), vec![3, 4]);
        assert_eq!(
            union(&["tail"], ["tail", "last"].as_slice()),
            vec!["tail", "last"]
        );
    }
}
#[allow(dead_code)]
mod relations {
    include!("set_relations.rs");
    #[test]
    fn empty_relations_and_string_key_collisions_remain_exact() {
        assert!(subset::<i64>(&[], &[1]));
        assert!(disjoint::<i64>(&[], &[]));
        assert!(!disjoint(&[1], &[1]));
        assert_eq!(
            function0(1, 0, &"tail".into())[0].0,
            vec!["true", "true", "true", "false", "false", "true"]
        );
    }
}
#[allow(dead_code)]
mod each {
    include!("map_for_each.rs");
    #[test]
    fn callbacks_observe_replaced_value_and_actual_source_size() {
        assert_eq!(
            function0(1, 0, &"tail".into())[0].0,
            vec!["12", "2", "tail:updated|", "1"]
        );
    }
}
#[allow(dead_code)]
mod iteration {
    include!("map_iteration.rs");
    #[test]
    fn map_iteration_keeps_duplicate_key_position_and_count() {
        assert_eq!(
            function0(1, 0, &"tail".into())[0].0,
            vec!["tail:1|", "tail:1|", "2"]
        );
        assert_eq!(
            function0(1, 0, &"last".into())[0].0,
            vec!["last:0|tail:1|", "last:0|tail:1|", "2"]
        );
    }
}
#[allow(dead_code)]
mod unicode {
    include!("string_iteration.rs");
    #[test]
    fn string_iteration_counts_scalars_and_preserves_nul() {
        assert_eq!(
            function0(1, 0, &"é🦀\0".into())[0].0,
            vec!["0:é|🦀|\0|", "3", "🦀\0", "éaéb🦀a🦀b\0a\0b"]
        );
        assert_eq!(function0(1, 0, &"".into())[0].0, vec!["0:", "0", "", ""]);
    }
}
#[allow(dead_code)]
mod grouping {
    include!("map_group_by.rs");
    #[test]
    fn groups_keep_first_key_order_and_own_values_after_source_mutation() {
        let mut source = vec![1, 2, 3, 4];
        let groups = group_by(source.iter().copied(), |v, _| v % 2);
        source[0] = 99;
        assert_eq!(groups, vec![(1, vec![1, 3]), (0, vec![2, 4])]);
        assert_eq!(
            function0(1, 0, &"a🦀aé".into())[0].0,
            vec![
                "2",
                "0,1",
                "0,2",
                "1,3",
                "10",
                "0,1,2,3,4",
                "2",
                "a|a",
                "🦀|é"
            ]
        );
    }
}
