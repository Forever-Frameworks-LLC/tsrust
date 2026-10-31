// Semantic witnesses for independent Rust split/pad references.
mod splitting {
    include!("string_split_limits.rs");
    fn pieces(text:&str,separator:Option<&str>,limit:Option<i64>)->Vec<std::string::String>{split(text,separator,limit).0.into_iter().map(|s|s.into_owned()).collect()}
    #[test]fn preserves_empty_fields_and_explicit_limit_conversion(){
        assert_eq!(pieces("::a::::",Some("::"),None),["","a","",""]);
        assert_eq!(pieces("::a::::",Some("::"),Some(2)),["","a"]);
        assert!(pieces("a::b",Some("::"),Some(4294967296)).is_empty());
        assert_eq!(pieces("a::b",Some("::"),Some(4294967297)),["a"]);
        assert_eq!(pieces("é😀e\u{301}",Some(""),Some(-1)),["é","😀","e","\u{301}"]);
        assert_eq!(pieces("",None,None),[""]);
    }
}
mod padding {
    include!("string_pad_start_unicode.rs");
    #[test]fn checks_byte_targets_and_utf8_prefixes(){
        assert_eq!(pad("Z",5,"é",true),"ééZ");
        assert_eq!(pad("Z",3,"é",false),"Zé");
        assert_eq!(pad("Z",100,"",true),"Z");
        assert_eq!(pad("Z",-1,"é",true),"Z");
        assert!(std::panic::catch_unwind(||pad("Z",2,"é",true)).is_err());
    }
}
