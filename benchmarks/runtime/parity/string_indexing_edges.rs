// Semantic witnesses for the independent Rust indexing references.
mod relative {
    include!("string_at_characters.rs");
    #[test]fn checks_relative_scalar_positions(){
        assert_eq!(at("A😀e\u{301}",-3),Some('😀'));
        assert_eq!(at("A😀e\u{301}",3),Some('\u{301}'));
        assert_eq!(at("",0),None);
        assert_eq!(at("A😀",i64::MIN),None);
    }
}
mod bmp {
    include!("string_code_at_positions.rs");
    #[test]fn checks_bmp_codes_and_missing_positions(){
        assert_eq!(code("é€",0),Some(233));
        assert_eq!(code("é€",1),Some(8364));
        assert_eq!(code("é€",-1),None);
        assert_eq!(code("é€",2),None);
    }
}
mod checked {
    include!("string_indexing.rs");
    #[test]fn checks_ascii_error_results(){
        assert_eq!(at("abc",2),Ok("c"));
        assert_eq!(at("abc",3),Err(()));
        assert_eq!(at("abc",-1),Err(()));
        assert_eq!(at("abc",i64::MAX),Err(()));
    }
}
