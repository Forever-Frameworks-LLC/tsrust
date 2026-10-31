mod indices {
    include!("array_at_int.rs");
    #[test]
    fn distinguishes_relative_indices_from_missing_positions() {
        assert_eq!(relative_index(-1,3),Some(2));
        assert_eq!(relative_index(-3,3),Some(0));
        assert_eq!(relative_index(-4,3),None);
        assert_eq!(relative_index(i64::MIN,3),None);
        assert_eq!(relative_index(i64::MAX,3),None);
        assert_eq!(relative_index(-1,0),None);
    }
}
mod booleans {
    include!("array_pop_boolean.rs");
    #[test]
    fn removed_false_remains_present_and_empty_pop_is_missing() {
        let values=function0(1,1).0;
        assert_eq!(values[0],"boolean");
        assert_eq!(values[1],"false");
        assert_eq!(values[5],"undefined");
        assert_eq!(values[6],"undefined");
    }
}
mod forward {
    include!("array_find_int.rs");
    #[test]
    fn stops_at_first_match_with_exact_callback_trace() {
        let values=function0(1,3).0;
        let texts:Vec<_>=values.iter().map(|s|s.as_ref()).collect();
        assert_eq!(texts,vec!["number","4","3","2","12","6","3","4","5"]);
    }
}
mod reverse_numbers {
    include!("array_find_last_number.rs");
    #[test]
    fn distinguishes_nan_match_from_missing_and_retains_signed_zero() {
        let matched=function0(1,3);
        assert_eq!(matched[0],1.0);
        assert!(matched[1].is_nan());
        assert_eq!(&matched[3..6],&[1.0,3.0,3.0]);
        let missing=function0(1,0);
        assert_eq!(missing[0],0.0);
        assert!(missing[1].is_nan());
        assert_eq!(function0(1,1)[1].to_bits(),(-0.0_f64).to_bits());
    }
}
mod nulls {
    include!("array_find_null.rs");
    #[test]
    fn found_null_is_distinct_from_missing() {
        assert_eq!(&function0(1,1).0[..2],&[String::Borrowed("object"),String::Borrowed("null")]);
        assert_eq!(&function0(1,0).0[..2],&[String::Borrowed("undefined"),String::Borrowed("undefined")]);
    }
}
mod owned {
    include!("array_shift_string.rs");
    #[test]
    fn removed_and_remaining_owned_text_survive_input_drop() {
        let input=String::Owned("é😀".to_owned());
        let values=function0(1,3,&input).0;
        drop(input);
        assert_eq!(values[1],"é😀");
        assert_eq!(values[3],"é😀!");
        assert_eq!(values[6],"é😀!");
        assert_eq!(values[9],"<absent>");
    }
}
