mod depth {
    include!("array_flat_nested.rs");
    #[test]
    fn flattens_one_level_without_flattening_inner_numeric_arrays() {
        let rows=function0(1,2);
        let values:Vec<_>=rows[0].0.iter().map(|v|v.0.clone()).collect();
        assert_eq!(values,vec![vec![2,0],vec![3,1],vec![8,6],vec![11,9],vec![12,10]]);
    }
}
mod empty_inner {
    include!("array_flat_map_nested.rs");
    #[test]
    fn flat_map_keeps_empty_inner_arrays_as_elements() {
        let rows=function0(3,0);
        assert!(rows[0].0.is_empty());
        assert!(rows[1].0.is_empty());
        assert_eq!(rows[2].0.len(),4);
        assert_eq!(rows[2].0[0].0,vec![1,1,1,4]);
        assert!(rows[2].0[1].0.is_empty());
        assert_eq!(rows[2].0[2].0,vec![2,2,2,4]);
        assert!(rows[2].0[3].0.is_empty());
    }
}
mod source_length {
    include!("array_flat_map_int.rs");
    #[test]
    fn callback_observes_original_index_and_source_length() {
        assert!(piece(5,0,7).is_empty());
        assert_eq!(piece(5,1,7),vec![5,13]);
        assert_eq!(piece(5,1,10),vec![5,16]);
    }
}
mod factory {
    include!("array_of_int.rs");
    #[test]
    fn single_numeric_argument_is_an_element_and_empty_call_is_empty() {
        let rows=function0(3,0);
        assert!(rows.0[0].0.is_empty());
        assert_eq!(rows.0[1].0,vec![1]);
        assert_eq!(rows.0[2].0,vec![2,3,4,5,6]);
    }
}
mod numbers {
    include!("array_of_number.rs");
    #[test]
    fn construction_preserves_nan_infinity_and_both_zeros() {
        let rows=function0(1,2);
        assert!(rows[0][0].is_nan());
        assert_eq!(rows[0][1],0.5);
        assert_eq!(rows[0][2].to_bits(),0.0_f64.to_bits());
        assert_eq!(rows[0][3],f64::NEG_INFINITY);
        assert_eq!(rows[0][4].to_bits(),(-0.0_f64).to_bits());
    }
}
mod ownership {
    include!("array_of_record.rs");
    #[test]
    fn constructed_record_descriptions_survive_input_drop() {
        let input=String::Owned("é😀".to_owned());
        let rows=function0(1,2,&input);
        drop(input);
        assert_eq!(rows[0].0[0],"2:é😀:0");
        assert_eq!(rows[0].0[4],"6:é😀:4");
    }
}
