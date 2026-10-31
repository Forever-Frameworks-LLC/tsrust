// UTF-8 byte-position witnesses for independent Rust search references.
macro_rules! reference {
    ($module:ident,$source:literal)=>{mod $module{
        include!($source);
        #[test]fn checks_clamping_and_rejects_interior_bytes(){
            assert!(search("éA","éA",None));
            assert!(search("éA","",Some(-99)));
            assert!(search("éA","",Some(i64::MAX)));
            assert!(std::panic::catch_unwind(||search("éA","",Some(1))).is_err());
        }
    }};
}
reference!(prefix,"string_starts_with_unicode.rs");
reference!(suffix,"string_ends_with_unicode.rs");
reference!(contains,"string_includes_unicode.rs");
