mod numbers {
    include!("array_to_sorted_compare_number.rs");
    #[test]
    fn keeps_equal_zero_and_nan_entries_stable() {
        let first_nan=f64::from_bits(0x7ff8000000000001);
        let second_nan=f64::from_bits(0x7ff8000000000002);
        let mut values=[first_nan,0.0,-0.0,second_nan,-1.0];
        values.sort_by(compare_number);
        assert_eq!(values[0],-1.0);
        assert_eq!(values[1].to_bits(),0.0_f64.to_bits());
        assert_eq!(values[2].to_bits(),(-0.0_f64).to_bits());
        assert_eq!(values[3].to_bits(),first_nan.to_bits());
        assert_eq!(values[4].to_bits(),second_nan.to_bits());
        let rows=function0(3,0);
        assert_eq!(rows[4][0].to_bits(),(-0.0_f64).to_bits());
        assert_eq!(rows[5][0],-2.5);
        assert!(rows[5][3].is_nan());
    }
}
mod integers {
    include!("array_sort_default_int.rs");
    #[test]
    fn default_integer_sort_is_numeric() {
        assert_eq!(function0(3,0).0[2].0,vec![1,2,10,11]);
    }
}
mod records {
    include!("array_to_sorted_compare_record.rs");
    #[test]
    fn preserves_original_and_equal_key_label_order() {
        let input=String::Owned("x".to_owned());
        let rows=function0(5,0,&input);
        drop(input);
        assert_eq!(rows[8].0[1],"1:x:1");
        let first_group:Vec<_>=rows[9].0[..6].iter().map(|s|s.as_ref()).collect();
        assert_eq!(first_group,vec!["0:x:0","0:x:3","0:x:6","0:x:9","0:x:12","0:x:15"]);
    }
}
mod nested {
    include!("array_to_sorted_compare_nested.rs");
    #[test]
    fn sorted_nested_values_own_independent_inner_vectors() {
        let mut rows=function0(3,0);
        assert!(rows[0].0.is_empty());
        assert!(rows[1].0.is_empty());
        rows[4].0[0].0[0]=999;
        assert_eq!(rows[5].0[2].0,vec![0,1]);
        assert_eq!(rows[5].0[3].0,vec![1,2]);
    }
}
