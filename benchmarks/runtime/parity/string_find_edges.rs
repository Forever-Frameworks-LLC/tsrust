// Semantic witnesses for independent Rust byte-coordinate search references.
mod forward {
    include!("string_index_of_unicode.rs");
    #[test]fn returns_byte_offsets_and_checks_boundaries(){
        assert_eq!(index_of("é😀é","é",Some(2)),6);
        assert_eq!(index_of("é😀é","",Some(i64::MAX)),8);
        assert_eq!(index_of("é😀é","é",Some(i64::MIN)),0);
        assert!(std::panic::catch_unwind(||index_of("é😀é","",Some(3))).is_err());
    }
}
mod reverse {
    include!("string_last_index_of_bounded_unicode.rs");
    #[test]fn preserves_overlap_and_start_position_bounds(){
        assert_eq!(last_index_of("aaaa","aa",Some(1)),1);
        assert_eq!(last_index_of("é😀é","😀",Some(2)),2);
        assert_eq!(last_index_of("é😀é","é",Some(2)),0);
        assert_eq!(last_index_of("é😀é","é",Some(i64::MAX)),6);
        assert_eq!(last_index_of("é😀é","é",Some(i64::MIN)),0);
        assert!(std::panic::catch_unwind(||last_index_of("é😀é","",Some(3))).is_err());
    }
}
