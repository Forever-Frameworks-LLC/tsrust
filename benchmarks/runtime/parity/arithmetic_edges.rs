// Semantic boundary checks for the standard Rust compound-arithmetic references.
macro_rules! integer {
    ($module:ident,$source:literal,$expected:expr)=>{mod $module{
        include!($source);
        #[test]fn checks_results_and_overflow(){
            assert_eq!(operation(7,3),$expected);
            assert!(std::panic::catch_unwind(||operation(i64::MAX,2)).is_err());
        }
    }};
}
integer!(int_add,"compound_int_add.rs",13);
integer!(int_subtract,"compound_int_subtract.rs",7);
integer!(int_multiply,"compound_int_multiply.rs",21);
integer!(int_divide,"compound_int_divide.rs",7);
integer!(int_remainder,"compound_int_remainder.rs",1);
mod divide {
    include!("compound_number_divide.rs");
    #[test]fn preserves_ieee_zero_and_infinity(){
        assert_eq!(operation(-0.0,2.0).to_bits(),(-0.0_f64).to_bits());
        assert_eq!(operation(1.0,0.0),f64::INFINITY);
        assert!(operation(0.0,0.0).is_nan());
    }
}
mod remainder {
    include!("compound_number_remainder.rs");
    #[test]fn preserves_signed_remainders(){
        assert_eq!(operation(-3.5,2.0),-1.5);
        assert_eq!(operation(-0.0,2.0).to_bits(),(-0.0_f64).to_bits());
        assert!(operation(1.0,0.0).is_nan());
    }
}
mod power {
    include!("compound_number_power.rs");
    #[test]fn preserves_power_special_cases(){
        assert_eq!(operation(0.0,0.0),1.0);
        assert_eq!(operation(-2.0,3.0),-8.0);
        assert_eq!(operation(4.0,0.5),2.0);
    }
}
