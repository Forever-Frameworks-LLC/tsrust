// Boundary witnesses for the independently authored Rust operator baselines.
macro_rules! shift {
    ($module:ident,$source:literal,$expected:expr)=>{mod $module{
        include!($source);
        #[test] fn rejects_invalid_counts(){
            assert_eq!(operation(8,1),$expected);
            assert!(std::panic::catch_unwind(||operation(1,64)).is_err());
            assert!(std::panic::catch_unwind(||operation(1,-1)).is_err());
        }
    }};
}
macro_rules! power {
    ($module:ident,$source:literal)=>{mod $module{
        include!($source);
        #[test] fn checks_power_boundaries(){
            assert_eq!(operation(0,0),1);
            assert_eq!(operation(-2,3),-8);
            assert!(std::panic::catch_unwind(||operation(2,63)).is_err());
            assert!(std::panic::catch_unwind(||operation(2,-1)).is_err());
        }
    }};
}
shift!(left,"int_native_shift_left.rs",16);
shift!(right,"int_native_shift_right.rs",4);
shift!(unsigned,"int_native_shift_right_unsigned.rs",4);
shift!(assign_left,"int_native_compound_shift_left.rs",16);
shift!(assign_right,"int_native_compound_shift_right.rs",4);
shift!(assign_unsigned,"int_native_compound_shift_right_unsigned.rs",4);
power!(exponent,"int_native_power.rs");
power!(assign_exponent,"int_native_compound_power.rs");
