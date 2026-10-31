#[allow(dead_code)]
mod returned {
    include!("higher_owned_callback_return_string.rs");
    #[test]
    fn returned_callback_owns_prefix_and_expands_replacement_tokens() {
        let action = keep(make("é🦀".to_owned()));
        assert_eq!(action("x😀"), "é🦀😀");
        assert_eq!(action("a\0bx"), "a\0bé🦀");
        assert_eq!(replace_x("axb", "$$:$&:$`:$'"), "a$:x:a:bb");
        assert_eq!(replace_x("xx", "$1"), "$1$1");
    }
}
#[allow(dead_code)]
mod captured {
    include!("higher_captured_factory_string.rs");
    #[test]
    fn repeated_factories_return_independent_owned_captures() {
        let factory = carry(make("é🦀".to_owned()));
        let first = factory();
        let second = factory();
        drop(factory);
        assert_eq!(first("xé"), "é🦀é");
        assert_eq!(second("xx"), "é🦀é🦀");
        assert_eq!(replace_x("xéx", "$`"), "éxé");
    }
}
#[allow(dead_code)]
mod read {
    include!("higher_read_string.rs");
    #[test]
    fn borrowed_text_produces_independently_owned_result() {
        let mut input = "é🦀".to_owned();
        let result = drive(&mut copy, &input);
        input.push('!');
        assert_eq!(result, "é🦀");
        assert_eq!(input, "é🦀!");
    }
}
#[allow(dead_code)]
mod consume {
    include!("higher_consume_string.rs");
    #[test]
    fn owned_text_moves_without_copying_its_allocation() {
        let input = "owned é🦀".to_owned();
        let pointer = input.as_ptr();
        let result = transfer(take, input);
        assert_eq!(result.as_ptr(), pointer);
        assert_eq!(result, "owned é🦀");
    }
}
#[allow(dead_code)]
mod mutate {
    include!("higher_mutate_array.rs");
    #[test]
    fn mutable_input_and_returned_copy_are_independent() {
        let mut values = vec![7, 2, 3];
        let mut result = drive(&mut change, &mut values);
        assert_eq!(values, vec![8, 2, 3]);
        result[0] = 99;
        assert_eq!(values[0], 8);
        assert_eq!(result[0], 99);
    }
}
#[allow(dead_code)]
mod record {
    include!("higher_owned_record.rs");
    #[test]
    fn record_result_deep_copies_owned_text_and_items() {
        let mut source = Record0 {
            field0: 7,
            field1: "é🦀".to_owned().into(),
            field2: vec![1, 2],
        };
        let result = drive(&mut copy, &source);
        source.field1.to_mut().push('!');
        source.field2[0] = 99;
        assert_eq!(result.field1, "é🦀");
        assert_eq!(result.field2, vec![1, 2]);
    }
}
#[allow(dead_code)]
mod arrays {
    include!("higher_deep_array.rs");
    #[test]
    fn nested_copy_keeps_empty_rows_and_separate_inner_owners() {
        let mut source = vec![vec![7, 1], vec![], vec![0, 2, 3]];
        let result = copying(&mut duplicate, &source);
        source[0][0] = 99;
        source[1].push(4);
        assert_eq!(result, vec![vec![7, 1], vec![], vec![0, 2, 3]]);
    }
}
#[allow(dead_code)]
mod tree {
    include!("higher_recursive_tree.rs");
    #[test]
    fn recursive_result_deep_copies_all_descendants() {
        let mut result = function0(1, 7, &"é🦀".into());
        let saved = copying(&mut duplicate, &result[0]);
        result[0].field1.to_mut().push('!');
        result[0].field2[1].field2[0].field0 = 99;
        assert_eq!(saved.field1, "é🦀");
        assert_eq!(saved.field2[0].field1, "a\0b");
        assert_eq!(saved.field2[1].field2[0].field0, 7);
    }
}
