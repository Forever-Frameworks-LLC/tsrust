mod ranges {
    include!("array_splice_int.rs");
    #[test]
    fn clamps_extreme_starts_and_negative_deletion_without_overflow() {
        let mut values=vec![0,1,2];
        assert_eq!(apply_splice(&mut values,i64::MIN,None,vec![]),vec![0,1,2]);
        assert!(values.is_empty());
        let mut values=vec![0,1,2];
        assert!(apply_splice(&mut values,i64::MAX,Some(i64::MAX),vec![7,8]).is_empty());
        assert_eq!(values,vec![0,1,2,7,8]);
        let mut values=vec![0,1,2];
        assert!(apply_splice(&mut values,1,Some(i64::MIN),vec![7]).is_empty());
        assert_eq!(values,vec![0,7,1,2]);
    }
    #[test]
    fn omitted_and_zero_deletion_have_different_results() {
        let mut omitted=vec![0,1,2];
        let mut zero=omitted.clone();
        assert_eq!(apply_splice(&mut omitted,1,None,vec![]),vec![1,2]);
        assert!(apply_splice(&mut zero,1,Some(0),vec![]).is_empty());
        assert_eq!(omitted,vec![0]);
        assert_eq!(zero,vec![0,1,2]);
        assert_eq!(function0(1,0).0,vec![String::Owned("0".to_owned()),String::Owned("0".to_owned())]);
    }
}
mod insertion {
    include!("array_unshift_int.rs");
    #[test]
    fn keeps_argument_order_before_the_original_receiver() {
        let values=function0(1,2).0;
        let text:Vec<_>=values.iter().map(|s|s.as_ref()).collect();
        assert_eq!(text,vec!["7","7","103","104","105","2","3","4","5"]);
    }
}
mod numbers {
    include!("array_unshift_number.rs");
    #[test]
    fn retains_signed_zero_nan_and_infinity_in_inserted_and_original_values() {
        let values=function0(1,2);
        assert_eq!(&values[..2],&[7.0,7.0]);
        assert_eq!(values[2].to_bits(),(-0.0_f64).to_bits());
        assert_eq!(values[5].to_bits(),(-0.0_f64).to_bits());
        assert!(values[4].is_nan());
        assert!(values[7].is_nan());
        assert_eq!(values[8],f64::INFINITY);
    }
}
mod records {
    include!("array_splice_record.rs");
    #[test]
    fn retained_removed_and_inserted_labels_survive_input_drop() {
        let input=String::Owned("x".to_owned());
        let values=function0(1,3,&input).0;
        drop(input);
        assert_eq!(values[0],"1");
        assert_eq!(values[1],"5");
        assert_eq!(values[2],"3:x:0");
        assert_eq!(values[3],"104:x:A");
        assert_eq!(values[4],"4:x:1");
    }
}
