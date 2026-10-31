mod order {
    include!("array_literal_spread_source_order.rs");
    #[test]
    fn each_spread_observes_the_receiver_at_its_own_evaluation_point() {
        let rows=function0(1,4,&String::Borrowed(""));
        assert_eq!(rows.0[0].0,vec![4,5,3,4,5,9]);
        assert_eq!(rows.0[1].0,vec![4,5,9]);
    }
}
mod optional {
    include!("call_spread_optional_default_counts.rs");
    #[test]
    fn absent_calls_skip_arguments_and_defaults_but_empty_present_calls_use_fallback() {
        let rows=function0(5,0,&String::Borrowed(""));
        assert!(rows.0[0].0.is_empty());
        assert_eq!(rows.0[1].0,vec![0,0]);
        assert_eq!(rows.0[3].0,vec![1,0]);
        assert_eq!(rows.0[6].0,vec![10]);
        assert_eq!(rows.0[7].0,vec![1,1]);
        assert_eq!(rows.0[9].0,vec![0,0]);
    }
}
mod once {
    include!("call_spread_owned_once_optional.rs");
    #[test]
    fn once_calls_move_owned_text_and_skip_missing_arguments() {
        let input=String::Owned("x".to_owned());
        let rows=function0(2,0,&input);
        drop(input);
        assert_eq!(rows[0].0,vec![String::Borrowed("missing")]);
        assert_eq!(rows[1].0.iter().map(|s|s.as_ref()).collect::<Vec<_>>(),vec!["0","0"]);
        assert_eq!(rows[2].0.iter().map(|s|s.as_ref()).collect::<Vec<_>>(),vec!["xx","13"]);
        assert_eq!(rows[3].0.iter().map(|s|s.as_ref()).collect::<Vec<_>>(),vec!["1","1"]);
    }
}
mod mutable {
    include!("call_spread_mutate_callback.rs");
    #[test]
    fn repeated_spread_calls_keep_mutable_capture_state() {
        let rows=function0(2,0,&String::Borrowed(""));
        assert_eq!(rows.0[0].0,vec![1]);
        assert_eq!(rows.0[2].0,vec![2,13]);
    }
}
mod math {
    include!("call_spread_math_number.rs");
    #[test]
    fn empty_max_nan_propagation_and_signed_zero_match_the_contract() {
        assert_eq!(maximum(&[]),f64::NEG_INFINITY);
        assert!(maximum(&[1.0,f64::NAN]).is_nan());
        assert_eq!(maximum(&[-0.0]).to_bits(),(-0.0_f64).to_bits());
        assert_eq!(maximum(&[-0.0,0.0]).to_bits(),0.0_f64.to_bits());
    }
}
mod nullish {
    include!("array_literal_spread_nullish_packs.rs");
    #[test]
    fn null_and_undefined_packs_keep_distinct_type_and_value_text() {
        let rows=function0(3,0,&String::Borrowed(""));
        assert!(rows[0].0.is_empty());
        assert!(rows[1].0.is_empty());
        assert_eq!(rows[2].0[0],"object:null");
        assert_eq!(rows[3].0[0],"undefined:undefined");
        assert_eq!(rows[4].0.len(),5);
        assert_eq!(rows[5].0.len(),5);
    }
}
mod callable {
    include!("array_literal_spread_callable.rs");
    #[test]
    fn spread_preserves_captured_callable_order() {
        assert_eq!(function0(1,96,&String::Borrowed("")).0[0].0,vec![96,97,98]);
    }
}
mod iterable_calls {
    include!("iterable_call_spread.rs");
    #[test]
    fn iterable_calls_keep_unicode_order_and_raw_template_rules() {
        let rows=function0(1,0,&String::Borrowed("A😀"));
        assert_eq!(rows[0].0.iter().map(|s|s.as_ref()).collect::<Vec<_>>(),vec!["2","A|😀","6","2","A😀:0","2","<a:b>"]);
        assert_eq!(raw(&["[","]"],&["x","ignored"]),"[x]");
        assert_eq!(raw(&["<",":",">"],&["x"]),"<x:>");
    }
}
mod iterable_arrays {
    include!("iterable_array_spread.rs");
    #[test]
    fn iterable_arrays_keep_order_and_handle_colliding_map_keys() {
        let rows=function0(1,0,&String::Borrowed("tail"));
        assert_eq!(rows[0].0.iter().map(|s|s.as_ref()).collect::<Vec<_>>(),vec!["0,2,0,3","t|a|i|l|é|🙂","1","tail:1","tail:0"]);
    }
}
