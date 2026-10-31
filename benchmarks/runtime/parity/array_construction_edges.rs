mod characters {
    include!("array_from_string_characters.rs");
    #[test]
    fn from_string_splits_scalars_without_losing_unicode_or_nul() {
        let input=String::Borrowed("A😀e\u{301}\0");
        let rows=function0(1,1,&input);
        assert_eq!(rows[0].0.iter().map(|s|s.as_ref()).collect::<Vec<_>>(),vec!["A","😀","e","\u{301}","\0"]);
    }
}
mod character_indices {
    include!("array_from_map_string.rs");
    #[test]
    fn mapped_string_indices_count_scalars_instead_of_bytes() {
        let input=String::Borrowed("A😀e\u{301}\0");
        let rows=function0(1,1,&input);
        assert_eq!(rows[0].0.iter().map(|s|s.as_ref()).collect::<Vec<_>>(),vec!["A0","😀1","e2","\u{301}3","\u{0}4"]);
    }
}
mod mapped_ints {
    include!("array_from_map_int.rs");
    #[test]
    fn mapping_keeps_empty_single_and_full_result_shapes() {
        let input=String::Borrowed("");
        let rows=function0(3,0,&input);
        assert!(rows.0[0].0.is_empty());
        assert_eq!(rows.0[1].0,vec![26]);
        assert_eq!(rows.0[2].0,vec![52,55,58,61,64]);
    }
}
mod record_entries {
    include!("array_entries_records.rs");
    #[test]
    fn mutating_copied_record_entries_does_not_mutate_source() {
        let input=String::Owned("x".to_owned());
        let rows=function0(1,2,&input);
        drop(input);
        assert_eq!(rows[0].0.iter().map(|s|s.as_ref()).collect::<Vec<_>>(),vec!["0","1","xxx","2,3","x","11,3","2"]);
    }
}
mod string_entries {
    include!("array_entries_string.rs");
    #[test]
    fn copied_pair_index_and_text_can_change_independently() {
        let input=String::Borrowed("x");
        let rows=function0(1,2,&input);
        assert_eq!(rows[0].0.iter().map(|s|s.as_ref()).collect::<Vec<_>>(),vec!["7","x","xxx","1","x","2","2","true"]);
    }
}
mod nested {
    include!("nested_arrays.rs");
    #[test]
    fn saved_nested_values_survive_receiver_mutation() {
        assert_eq!(function0(1,7),27);
        assert_eq!(function0(2,7),50);
    }
}
mod scalar_churn {
    include!("array_churn.rs");
    #[test]
    fn empty_and_single_element_removals_preserve_saved_values() {
        assert_eq!(function0(1,0),0);
        assert_eq!(function0(1,1),3);
    }
}
mod record_churn {
    include!("record_array_churn.rs");
    #[test]
    fn saved_records_have_independent_inner_vectors() {
        assert_eq!(function0(1,0),0);
        assert_eq!(function0(1,1),6);
    }
}
mod collections {
    include!("array_from_collections.rs");
    #[test]
    fn keeps_insertion_order_deduplicates_and_updates_existing_map_keys() {
        assert_eq!(ordered_unique([3,1,3,2,1]),vec![3,1,2]);
        let input=String::Borrowed("tail");
        let rows=function0(1,0,&input);
        assert_eq!(rows[0].0.iter().map(|s|s.as_ref()).collect::<Vec<_>>(),vec!["2,0","20,1","1","tail:1","tail:0"]);
    }
}
