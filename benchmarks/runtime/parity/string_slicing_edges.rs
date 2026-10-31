// UTF-8 range witnesses for the independently authored Rust slicing references.
mod substring {
    include!("string_substring_bounded_unicode.rs");
    #[test]fn swaps_bounds_and_short_circuits_empty_ranges(){
        assert_eq!(substring("éA",2,Some(0)),"é");
        assert_eq!(substring("éA",1,Some(1)),"");
        assert_eq!(substring("éA",-99,Some(99)),"éA");
        assert!(std::panic::catch_unwind(||substring("éA",1,None)).is_err());
    }
}
mod substr {
    include!("string_substr_bounded_unicode.rs");
    #[test]fn checks_negative_offsets_and_nonpositive_lengths(){
        assert_eq!(substr("éA",-1,99),"A");
        assert_eq!(substr("éA",1,0),"");
        assert_eq!(substr("éA",1,-1),"");
        assert!(std::panic::catch_unwind(||substr("éA",1,1)).is_err());
    }
}
mod slice {
    include!("string_slice_unicode.rs");
    #[test]fn checks_signed_bounds_and_empty_order(){
        assert_eq!(slice("éA",-1,99),"A");
        assert_eq!(slice("éA",2,0),"");
        assert!(std::panic::catch_unwind(||slice("éA",1,2)).is_err());
    }
}
