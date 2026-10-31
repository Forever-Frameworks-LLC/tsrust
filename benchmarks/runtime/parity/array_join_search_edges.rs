mod nullish {
    include!("array_to_string_nullish.rs");
    #[test]
    fn missing_values_are_empty_fields_with_separators_retained() {
        let rows=function0(4,0).0;
        assert_eq!(rows.iter().map(|s|s.as_ref()).collect::<Vec<_>>(),vec!["","",",",",,"]);
        assert_eq!(join_display(&[Empty;3],"é\0"),"é\0é\0");
    }
}
mod nested {
    include!("array_join_nested.rs");
    #[test]
    fn inner_arrays_keep_commas_and_empty_segments() {
        let values=[Nested(vec![1,2]),Nested(vec![]),Nested(vec![3])];
        assert_eq!(join_display(&values,"|"),"1,2||3");
        assert_eq!(join_display(&values,","),"1,2,,3");
    }
}
mod numbers {
    include!("array_locale_string_values.rs");
    #[test]
    fn ordinary_join_normalizes_zero_and_spells_nonfinite_values() {
        let values=[Number(-0.0),Number(f64::NAN),Number(f64::INFINITY),Number(f64::NEG_INFINITY)];
        assert_eq!(join_display(&values,","),"0,NaN,Infinity,-Infinity");
        assert_eq!(locale(-0.0),"-0");
    }
    #[test]
    fn locale_arrays_keep_grouping_nullish_and_nested_separator_rules() {
        let input=String::Borrowed("x");
        let rows=function0(1,0,&input);
        let values=rows[0].0.iter().map(|s|s.as_ref()).collect::<Vec<_>>();
        assert_eq!(values,vec!["0,0","1,000.5,-0,1.235","x,é😀,","true,false",",",",","[object Object],[object Object]","1,000.5,-0,,2,000.235"]);
    }
}
mod bounds {
    include!("array_last_index_of_bounded_int.rs");
    #[test]
    fn bounded_reverse_search_distinguishes_default_zero_and_negative_extremes() {
        assert_eq!(last_start(None,129),Some(128));
        assert_eq!(last_start(Some(0),129),Some(0));
        assert_eq!(last_start(Some(-129),129),Some(0));
        assert_eq!(last_start(Some(-128),129),Some(1));
        assert_eq!(last_start(Some(-130),129),None);
        assert_eq!(last_start(Some(i64::MIN),129),None);
        assert_eq!(last_start(Some(i64::MAX),129),Some(128));
        assert_eq!(last_start(None,0),None);
    }
}
mod equality {
    include!("array_last_index_of_bounded_number.rs");
    #[test]
    fn nan_is_not_found_and_zero_search_uses_numeric_equality() {
        assert_eq!(function0(1,4).0,vec![0]);
        assert_eq!(function0(1,1).0,vec![129]);
        assert_eq!(function0(8,1).0[7],65);
    }
}
mod ownership {
    include!("array_join_string.rs");
    #[test]
    fn joined_dynamic_text_survives_input_drop() {
        let input=String::Owned("é😀".to_owned());
        let rows=function0(1,1,&input);
        drop(input);
        assert_eq!(rows.0[0],"é😀");
    }
}
