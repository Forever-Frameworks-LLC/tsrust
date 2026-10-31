mod mixed {
    include!("array_concat_int_variadic.rs");
    #[test]
    fn preserves_scalar_array_and_empty_argument_order() {
        assert_eq!(function0(1,3)[0],vec![3,4,2,0,7,3,4,1,3]);
        assert_eq!(function0(1,0)[0],vec![0,1,1]);
    }
}
mod nested {
    include!("array_concat_nested_int_zero.rs");
    #[test]
    fn zero_argument_results_own_independent_nested_copies() {
        let mut rows=function0(5,0);
        assert!(rows[0].is_empty());
        assert!(rows[4].is_empty());
        rows[1][0][0]=999;
        assert_eq!(rows[2][0],vec![0,1]);
    }
}
mod recursive {
    include!("array_concat_recursive_record_zero.rs");
    #[test]
    fn copies_recursive_children_and_owned_text_independently() {
        let input=String::Owned("é😀".to_owned());
        let mut rows=function0(3,1,&input);
        drop(input);
        rows[0][0].field1.to_mut().push('x');
        rows[0][0].field2[0].field0=999;
        assert_eq!(rows[1][0].field1,"é😀");
        assert_eq!(rows[1][0].field2[0].field0,1);
    }
}
mod ordered {
    include!("array_concat_ordered_trace.rs");
    #[test]
    fn evaluates_receiver_then_array_then_scalar_once() {
        assert_eq!(function0(1,0),17415);
        assert_eq!(function0(0,7),7);
    }
}
mod floating {
    include!("array_concat_number_variadic.rs");
    #[test]
    fn preserves_ieee_values_across_scalar_and_array_arguments() {
        let rows=function0(4,0);
        assert_eq!(rows[0][1].to_bits(),(-0.0_f64).to_bits());
        assert_eq!(rows[0][2].to_bits(),(-0.0_f64).to_bits());
        assert_eq!(rows[3][2],f64::INFINITY);
        assert_eq!(rows[3][3],f64::NEG_INFINITY);
        assert!(rows[3][4].is_nan());
        assert_eq!(rows[3][7].to_bits(),(-0.0_f64).to_bits());
    }
}
