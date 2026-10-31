// Unicode scalar and out-of-range witnesses for independent Rust charAt.
include!("string_char_at.rs");
#[test]fn returns_complete_scalars_and_empty_out_of_range(){
    let text="A😀e\u{301}\0Z";
    assert_eq!(character(text,1),"😀");
    assert_eq!(character(text,3),"\u{301}");
    assert_eq!(character(text,4),"\0");
    assert_eq!(character(text,-1),"");
    assert_eq!(character(text,6),"");
    assert_eq!(character(text,i64::MAX),"");
}
