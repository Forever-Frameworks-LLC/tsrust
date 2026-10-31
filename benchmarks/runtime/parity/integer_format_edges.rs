// Sign/magnitude witnesses for independent Rust integer formatting references.
macro_rules! reference {
    ($module:ident,$source:literal,$radix:expr,$negative:literal)=>{mod $module{
        include!($source);
        #[test]fn formats_signed_magnitudes_and_extremes(){
            assert_eq!(format_integer(-35),$negative);
            assert_eq!(format_integer(0),"0");
            for value in [i64::MIN,i64::MAX,-1,0,1,35,9007199254740991]{
                let text=format_integer(value);
                assert_eq!(i64::from_str_radix(&text,$radix).unwrap(),value);
            }
        }
    }};
}
reference!(decimal,"int_to_string_decimal.rs",10,"-35");
reference!(binary,"int_to_string_binary.rs",2,"-100011");
reference!(hex,"int_to_string_hex.rs",16,"-23");
reference!(base36,"int_to_string_base36.rs",36,"-z");
