#[allow(dead_code)]
mod normalization {
    include!("dynamic_normalization.rs");
    #[test]
    fn rust_whitespace_trim_keeps_bom_and_lowercases_unicode() {
        let input: String = "\u{85}É🦀\u{85}".into();
        assert_eq!(function0(1, 0, &input).0[0], "é🦀");
        let bom: String = "\u{feff}É\u{feff}".into();
        assert_eq!(function0(1, 0, &bom).0[0], "\u{feff}é\u{feff}");
    }
}
#[allow(dead_code)]
mod uppercase {
    include!("dynamic_unicode_uppercase.rs");
    #[test]
    fn uppercase_keeps_expansions_combining_marks_and_nul() {
        let input: String = "Straße ﬀ e\u{301}\0🦀".into();
        assert_eq!(function0(1, 0, &input).0[0], "STRASSE FF E\u{301}\0🦀");
    }
}
#[allow(dead_code)]
mod clone {
    include!("dynamic_string_clone.rs");
    #[test]
    fn retained_clones_have_independent_owners() {
        let input: String = "é🦀".into();
        let mut output = function0(2, 0, &input);
        drop(input);
        output.0[0].to_mut().push('!');
        assert_eq!(output.0[0], "é🦀!");
        assert_eq!(output.0[1], "é🦀");
    }
}
#[allow(dead_code)]
mod byte_length {
    include!("string_length.rs");
    #[test]
    fn length_counts_utf8_bytes_including_trimmed_variant() {
        let input: String = " é🦀 ".into();
        assert_eq!(function0(1, 0, &input), 8);
        assert_eq!(function0(1, 1, &input), 137);
    }
}
#[allow(dead_code)]
mod concat {
    include!("string_add_chained_unicode.rs");
    #[test]
    fn chained_concatenation_keeps_source_and_embedded_nul() {
        let input: String = "x".into();
        let result = function0(1, 0, &input);
        assert_eq!(result.0[0], "x\u{85}\u{feff}");
        assert_eq!(input, "x");
        let result = function0(1, 4, &input);
        assert!(result.0[0].contains('\0'));
    }
}
#[allow(dead_code)]
mod conditional {
    include!("conditional_string_unicode.rs");
    #[test]
    fn only_selected_branch_updates_counts_and_builds_owned_result() {
        let input: String = "é🦀".into();
        let result = function0(3, 0, &input);
        drop(input);
        assert_eq!(result.0, vec!["Lé🦀", "Ré🦀é🦀", "Lé🦀", "PPP", "LL", "R"]);
    }
}
#[allow(dead_code)]
mod split {
    include!("string_cow_split_pieces.rs");
    #[test]
    fn delimiter_split_preserves_empty_fields_and_limits() {
        assert_eq!(
            pieces("::a::::b::", Some("::"), 7),
            vec!["", "a", "", "b", ""]
        );
        assert!(pieces("x", Some("::"), 0).is_empty());
        assert_eq!(pieces("::a::", Some("::"), 2), vec!["", "a"]);
        assert_eq!(pieces("", Some("::"), 7), vec![""]);
        assert_eq!(pieces("é🦀", None, usize::MAX), vec!["é🦀"]);
    }
}
#[allow(dead_code)]
mod scalars {
    include!("string_scalar_split_unicode.rs");
    #[test]
    fn empty_delimiter_splits_scalars_instead_of_bytes() {
        assert_eq!(pieces("é🦀e\u{301}", 7), vec!["é", "🦀", "e", "\u{301}"]);
        assert_eq!(pieces("é🦀", 1), vec!["é"]);
        assert!(pieces("", 7).is_empty());
        assert!(pieces("é🦀", 0).is_empty());
    }
}
