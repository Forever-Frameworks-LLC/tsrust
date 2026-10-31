// Conversion and operation witnesses for the 32-bit number-reference family.
macro_rules! reference {
    ($module:ident,$source:literal,$expected:expr)=>{mod $module{
        include!($source);
        #[test]fn converts_words_and_retains_signed_results(){
            assert_eq!(word(-4294967297.0),u32::MAX);
            assert_eq!(word(4294967296.0),0);
            assert_eq!(word(4294967297.75),1);
            assert_eq!(word(-0.5),0);
            assert_eq!(word(f64::NAN),0);
            assert_eq!(word(f64::INFINITY),0);
            assert_eq!(operation(-1.0,1.0),$expected);
        }
    }};
}
reference!(and,"compound_number_and.rs",1.0);
reference!(or,"compound_number_or.rs",-1.0);
reference!(xor,"compound_number_xor.rs",-2.0);
reference!(left,"compound_number_shift_left.rs",-2.0);
reference!(right,"compound_number_shift_right.rs",-1.0);
reference!(unsigned,"compound_number_shift_right_unsigned.rs",2147483647.0);
