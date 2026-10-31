// Semantic witnesses for the independently authored Rust replacement references.
include!("string_replace_all_tokens.rs");
#[test]fn expands_only_supported_substitution_tokens(){
    assert_eq!(replace("abc","b","$$|$&|$`|$'|$1|$<x>|$",false),"a$|b|a|c|$1|$<x>|$c");
    assert_eq!(replace("é😀Z","😀","$`+$&+$'",false),"éé+😀+ZZ");
    assert_eq!(replace("aba","a","$`/$'",true),"/babab/");
    assert_eq!(replace("abc","missing","$&",true),"abc");
}
#[test]fn handles_empty_patterns_and_nonoverlapping_matches(){
    assert_eq!(replace("aaaa","aa","X",true),"XX");
    assert_eq!(replace("aaaa","aa","X",false),"Xaa");
    assert_eq!(replace("é😀","","X",true),"XéX😀X");
    assert_eq!(replace("é😀","","<$&>",true),"<>é<>😀<>");
    assert_eq!(replace("","","$$",true),"$");
}
