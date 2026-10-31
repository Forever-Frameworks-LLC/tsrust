mod some {
    include!("array_some_int.rs");
    #[test]
    fn empty_some_skips_callbacks_and_first_match_stops_visits() {
        assert_eq!(function0(1,0).0[0].0,vec![0,0,0]);
        assert_eq!(function0(1,2).0[0].0,vec![1,270,1]);
    }
}
mod every {
    include!("array_every_int.rs");
    #[test]
    fn empty_every_is_true_and_first_false_stops_visits() {
        assert_eq!(function0(1,0).0[0].0,vec![1,0,0]);
        assert_eq!(function0(1,2).0[0].0,vec![0,270,1]);
    }
}
mod reverse {
    include!("array_find_last_index_int.rs");
    #[test]
    fn reverse_search_reports_original_index_and_callback_order() {
        assert_eq!(function0(1,3).0[0].0,vec![8,408,1]);
        assert_eq!(function0(1,0).0[0].0,vec![0,0,0]);
    }
}
mod nan {
    include!("array_some_number.rs");
    #[test]
    fn qualifying_nan_runs_through_the_real_predicate() {
        assert_eq!(function0(1,8).0[0].0,vec![1,138346,2]);
    }
}
mod mapped {
    include!("array_map_int.rs");
    #[test]
    fn maps_every_source_position_and_keeps_zero_iteration_state() {
        assert_eq!(function0(0,7).0[0].0,vec![7,0]);
        let rows=function0(1,0);
        assert_eq!(rows.0[0].0,vec![8,10,12,14,16,18,20,22]);
        assert_eq!(rows.0[1].0[1],8);
    }
}
mod nested {
    include!("array_filter_nested.rs");
    #[test]
    fn selected_nested_values_have_independent_owners() {
        let mut rows=function0(2,0);
        rows[0].0[0].0[0]=999;
        assert_eq!(rows[1].0[0].0,vec![2,3,4]);
        assert_eq!(rows[2].0[0].0[1],16);
    }
}
mod strings {
    include!("array_map_string.rs");
    #[test]
    fn mapped_text_survives_input_drop_and_retains_unicode_and_nul() {
        let input=String::Owned("  é😀  ".to_owned());
        let rows=function0(1,0,&input);
        drop(input);
        assert_eq!(rows[0].0[0],"é😀");
        assert_eq!(rows[0].0[1],"Pat");
        assert_eq!(rows[0].0[7],"a\0b");
        assert_eq!(rows[1].0[0],"trace");
        assert_eq!(rows[1].0.len(),49);
    }
}
