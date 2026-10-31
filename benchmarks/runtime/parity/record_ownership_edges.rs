#[allow(dead_code)]
mod mutation {
    include!("record_mutation.rs");
    #[test]
    fn mutation_updates_value_before_active_predicate() {
        let mut counter = Counter {
            value: 7,
            active: true,
        };
        increment(&mut counter);
        assert_eq!((counter.value, counter.active), (8, true));
        increment(&mut counter);
        assert_eq!((counter.value, counter.active), (9, false));
        assert_eq!(function0(2, 7), 8);
    }
}
#[allow(dead_code)]
mod deep {
    include!("deep_clone.rs");
    #[test]
    fn nested_clone_owns_its_array_independently() {
        let mut source = User {
            details: Details {
                name: "é🦀".to_owned().into(),
                scores: vec![1, 2],
            },
        };
        let saved = source.clone();
        source.details.scores[0] = 9;
        source.details.name.to_mut().push('!');
        assert_eq!(saved.details.scores, vec![1, 2]);
        assert_eq!(saved.details.name, "é🦀");
        assert_eq!(function0(2, 7), 29);
    }
}
#[allow(dead_code)]
mod restoration {
    include!("field_restore.rs");
    #[test]
    fn moved_fields_restore_record_without_losing_siblings() {
        assert_eq!(function0(1, 7), 18);
        assert_eq!(function0(2, 7), 33);
    }
}
#[allow(dead_code)]
mod recursive {
    include!("recursive_data.rs");
    #[test]
    fn nested_total_and_mutual_recursion_keep_base_case() {
        let tree = Tree {
            value: 1,
            children: vec![Tree {
                value: 2,
                children: vec![Tree {
                    value: 3,
                    children: vec![],
                }],
            }],
        };
        assert_eq!(total(&tree), 6);
        assert_eq!((even(0, 7), odd(0, 7), even(8, 7)), (7, 7, 43));
        assert_eq!(function0(1, 7), 51);
    }
}
#[allow(dead_code)]
mod factory {
    include!("recursive_factory.rs");
    #[test]
    fn recursive_factories_return_independent_mutable_state() {
        let mut first = counter(1, 8);
        let mut second = counter(11, 8);
        assert_eq!((first(), second(), first()), (2, 12, 3));
        assert_eq!(chain(7, 4)(), 7);
        assert_eq!(function0(1, 7), 10);
    }
}
#[allow(dead_code)]
mod arguments {
    include!("owned_arguments.rs");
    #[test]
    fn read_mutate_then_move_preserves_single_record_owner() {
        let mut user = User {
            name: "owned".to_owned().into(),
            value: 3,
        };
        assert_eq!(read_twice(|u| u.value, &user), 6);
        apply(|u| u.value = add(u.value, 7), &mut user);
        let saved = take_once(|u| u, user);
        assert_eq!((saved.value, saved.name.as_ref()), (10, "owned"));
        assert_eq!(function0(2, 7), 39);
    }
}
#[allow(dead_code)]
mod capture {
    include!("capture_replacement.rs");
    #[test]
    fn capture_replacement_keeps_previous_values_and_once_transfer() {
        let mut action = make(
            User {
                name: "Pat".into(),
                value: 0,
            },
            7,
        );
        assert_eq!((action(), action()), (8, 22));
        let owned = User {
            name: "owned".to_owned().into(),
            value: 17,
        };
        assert_eq!(once_value(owned)(), 17);
        assert_eq!(function0(2, 7), 85);
    }
}
#[allow(dead_code)]
mod growth {
    include!("recursive_growth.rs");
    #[test]
    fn empty_and_single_child_removal_preserve_root() {
        assert_eq!(function0(1, 0), 0);
        assert_eq!(function0(1, 1), 2);
        let mut tree = Tree {
            value: 1,
            children: vec![
                Tree {
                    value: 2,
                    children: vec![],
                },
                Tree {
                    value: 3,
                    children: vec![],
                },
            ],
        };
        assert_eq!(tree.children.remove(1).value, 3);
        assert_eq!(total(&tree), 3);
    }
}
#[allow(dead_code)]
mod recursive_copy {
    include!("recursive_clone.rs");
    #[test]
    fn copied_nested_tree_remains_separate_after_root_and_child_mutation() {
        let result = function0(1, 1);
        assert_eq!(result.0[0].0, vec![1, 0, 1]);
        assert_eq!(result.0[1].0, vec![2000, 3001, 1]);
        let leaf = function0(1, 0);
        assert_eq!(leaf.0[0].0, vec![0]);
        assert_eq!(leaf.0[1].0, vec![2000]);
    }
}
#[allow(dead_code)]
mod retained {
    include!("retained_rows.rs");
    #[test]
    fn retained_rows_have_independent_empty_and_growing_owners() {
        let mut result = function0(3, 0);
        assert!(result.0[0].0.is_empty());
        assert_eq!(result.0[1].0, vec![3]);
        assert_eq!(result.0[2].0, vec![6, 7]);
        result.0[1].0.push(9);
        assert_eq!(result.0[2].0, vec![6, 7]);
    }
}
#[allow(dead_code)]
mod generic {
    include!("generic_function_record_pipeline.rs");
    #[test]
    fn generic_pipeline_moves_owned_nested_values() {
        assert_eq!(
            unwrap(wrap(identity(vec!["owned".to_owned()]))),
            vec!["owned"]
        );
        assert_eq!(function0(2, 7), 22);
    }
}
