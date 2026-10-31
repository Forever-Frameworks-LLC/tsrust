mod forward {
    include!("array_reduce_record.rs");
    #[test]
    fn seedless_reduce_skips_first_callback_and_keeps_last_owned_value() {
        let value=function0(1,3);
        assert_eq!(value.field4,4);
        assert_eq!(value.field3,897109493);
        assert_eq!(value.field1,"last");
        assert_eq!(value.field2,vec![vec![5]]);
        assert_eq!(function0(1,0).field1,"initial");
        assert_eq!(function0(0,7).field4,0);
    }
}
mod reverse {
    include!("array_reduce_right_record.rs");
    #[test]
    fn seedless_reverse_skips_last_callback_and_keeps_original_indices() {
        let value=function0(1,3);
        assert_eq!(value.field4,4);
        assert_eq!(value.field3,901605409);
        assert_eq!(value.field1,"é");
        assert_eq!(value.field2,vec![vec![3]]);
    }
}
mod numbers {
    include!("array_reduce_number.rs");
    #[test]
    fn empty_initial_and_fractional_accumulation_keep_exact_values() {
        assert_eq!(function0(1,0),0.0);
        assert_eq!(function0(1,1),34.75);
    }
}
mod nested_numbers {
    include!("array_reduce_nested_number.rs");
    #[test]
    fn nested_results_keep_negative_zero_and_independent_storage() {
        let mut rows=function0(7,0);
        assert!(rows[0].is_empty());
        assert_eq!(rows[1][1].to_bits(),(-0.0_f64).to_bits());
        rows[1][0]=999.0;
        assert_eq!(rows[6][0],0.25);
    }
}
mod owned_record {
    include!("array_reduce_int_to_record.rs");
    #[test]
    fn record_accumulator_mutates_owned_rows_and_resets_for_empty_iteration() {
        let value=function0(1,3);
        assert_eq!(value.field1,"owned");
        assert_eq!(value.field4,5);
        assert_eq!(value.field2,vec![vec![3,0],vec![4,1],vec![2,2],vec![3,3],vec![7,4]]);
        let reset=function0(2,4);
        assert_eq!(reset.field1,"initial");
        assert!(reset.field2.is_empty());
        assert_eq!(reset.field4,0);
    }
}
mod owned_nested {
    include!("array_reduce_right_int_to_nested.rs");
    #[test]
    fn reverse_nested_accumulator_preserves_original_source_indices() {
        assert_eq!(function0(1,3)[0],vec![vec![7,4],vec![3,3],vec![2,2],vec![4,1],vec![3,0]]);
    }
}
mod text {
    include!("array_reduce_string.rs");
    #[test]
    fn string_accumulator_keeps_first_replacement_unicode_and_nul() {
        assert_eq!(function0(0,7),"MARKMARK");
        assert_eq!(function0(1,1),"MARKa");
        assert_eq!(function0(1,3),"MARKz\0😀éaMARKz\0😀éa");
    }
}
