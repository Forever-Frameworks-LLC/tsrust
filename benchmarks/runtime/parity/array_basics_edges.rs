mod indexing {
    include!("int_array_index_of.rs");
    #[test]
    fn clamps_relative_indices_without_overflow() {
        assert_eq!(bound(-1,8),7);
        assert_eq!(bound(-8,8),0);
        assert_eq!(bound(-9,8),0);
        assert_eq!(bound(i64::MIN,8),0);
        assert_eq!(bound(i64::MAX,8),8);
        assert_eq!(bound(-1,0),0);
    }
}
mod slices {
    include!("array_slice_nested.rs");
    #[test]
    fn copied_nested_slices_own_independent_inner_vectors() {
        let mut results=function0(16,0);
        assert_eq!(results[0].0[0].0[0],0);
        assert_eq!(results[5].0[0].0[0],0);
        results[0].0[0].0[0]=999;
        assert_eq!(results[5].0[0].0[0],0);
        assert!(results[8].0.is_empty());
    }
}
mod units {
    include!("unit_string_array_includes.rs");
    #[test]
    fn indexes_unicode_scalars_and_retains_combining_marks() {
        assert_eq!(char_at("A😀e\u{301}Z",1),"😀");
        assert_eq!(char_at("A😀e\u{301}Z",3),"\u{301}");
        assert_eq!(char_at("A😀e\u{301}Z",5),"");
    }
}
mod conditionals {
    include!("conditional_array_int.rs");
    #[test]
    fn evaluates_only_the_selected_branch_and_retains_empty_rows() {
        let rows=function0(2,0);
        assert!(rows.0[0].0.is_empty());
        assert_eq!(rows.0[1].0,vec![104,105,106]);
        assert_eq!(rows.0[2].0,vec![2,1,1]);
        assert_eq!(function0(0,0).0[0].0,vec![0,0,0]);
    }
}
mod temporaries {
    include!("array_temporary_index_string.rs");
    #[test]
    fn selected_owned_strings_survive_input_and_other_element_drops() {
        let input=String::Owned("é😀".to_owned());
        let rows=function0(4,0,&input);
        drop(input);
        let texts:Vec<_>=rows.0.iter().map(|value|value.as_ref()).collect();
        assert_eq!(texts,vec!["é😀","a\0😀","é😀","Pat"]);
    }
}
