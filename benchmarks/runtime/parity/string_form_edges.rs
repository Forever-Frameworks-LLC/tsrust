#[allow(dead_code)]
mod identity {
    include!("string_to_string_unicode.rs");
    #[test]
    fn string_identity_retains_independent_owned_unicode_outputs() {
        let input: String = "é🦀".into();
        let mut result = function0(9, 0, &input);
        drop(input);
        result.0[0].to_mut().push('!');
        assert_eq!(result.0[0], "é🦀!");
        assert_eq!(result.0[8], "é🦀");
    }
}
#[allow(dead_code)]
mod valid {
    include!("string_is_well_formed_unicode.rs");
    #[test]
    fn valid_utf8_includes_nul_combining_marks_and_noncharacters() {
        assert!(well_formed("\0e\u{301}\u{fdd0}\u{ffff}\u{10ffff}🦀"));
        assert!(function0(16, 0, &"é🦀".into()).0.into_iter().all(|x| x));
    }
}
#[allow(dead_code)]
mod repaired {
    include!("string_to_well_formed_unicode.rs");
    #[test]
    fn well_formed_conversion_keeps_noncharacters_and_independent_owner() {
        assert_eq!(
            function0(1, 7, &"x".into()).0[0],
            "\u{fdd0}\u{ffff}\u{10ffff}"
        );
        assert_eq!(function0(1, 0, &"é🦀".into()).0[0], "é🦀");
    }
}
#[allow(dead_code)]
mod ascii_literals {
    include!("quoted_string_literals_ascii.rs");
    #[test]
    fn decoded_literals_keep_escapes_unknown_escapes_and_continuations() {
        for (at, expected) in [
            (3, "A\0Z"),
            (4, "B\0Y"),
            (5, "C\0X"),
            (6, "quote' slash\\"),
            (7, "quote\" slash\\"),
            (8, "q$_"),
            (9, "\u{8}\u{c}\n\r\t\u{b}\0"),
            (10, "joined"),
            (13, "joined"),
            (15, ""),
        ] {
            assert_eq!(function0(1, at, &"x".into()).0[0], expected);
        }
    }
}
#[allow(dead_code)]
mod unicode_literals {
    include!("quoted_string_literals_unicode.rs");
    #[test]
    fn decoded_unicode_literals_keep_scalar_values_and_combining_sequence() {
        for (at, expected) in [
            (2, "BOM\u{feff}中"),
            (3, "é"),
            (5, "😀\u{10ffff}"),
            (6, "😀"),
            (7, "e\u{301}"),
            (8, "qé$_"),
            (9, "\0é😀"),
            (10, "é😀"),
            (14, "é😀"),
        ] {
            assert_eq!(function0(1, at, &"x".into()).0[0], expected);
        }
    }
}
#[allow(dead_code)]
mod primitive_template {
    include!("template_primitive_ascii.rs");
    #[test]
    fn template_formats_exact_fraction_boolean_and_integer() {
        assert_eq!(function0(1, 0, &"x".into()).0[0], "head:x|0|0.5|true:tail");
        assert_eq!(function0(1, 1, &"x".into()).0[0], "head:|1|1.25|false:tail");
    }
}
#[allow(dead_code)]
mod unicode_template {
    include!("template_string_unicode.rs");
    #[test]
    fn escaped_interpolation_marker_remains_literal() {
        assert_eq!(
            function0(1, 0, &"x".into()).0[0],
            "é😀:${literal}|x|a\0é:e\u{301}"
        );
    }
}
macro_rules! basic_wrapper {
    ($module:ident,$file:literal,$expected:literal) => {
        #[allow(dead_code)]
        mod $module {
            include!($file);
            #[test]
            fn wrapper_keeps_raw_body_text() {
                assert_eq!(wrap("<&\0é"), $expected);
            }
        }
    };
}
macro_rules! attribute_wrapper {
    ($module:ident,$file:literal,$expected:literal) => {
        #[allow(dead_code)]
        mod $module {
            include!($file);
            #[test]
            fn wrapper_escapes_only_attribute_double_quotes() {
                assert_eq!(wrap("<&\0é", "a\"b&<>'\0é"), $expected);
                assert!(wrap("x", "&quot;").contains("&quot;"));
                assert!(!wrap("x", "&quot;").contains("&amp;"));
            }
        }
    };
}
basic_wrapper!(big, "string_html_big.rs", "<big><&\0é</big>");
basic_wrapper!(blink, "string_html_blink.rs", "<blink><&\0é</blink>");
basic_wrapper!(bold, "string_html_bold.rs", "<b><&\0é</b>");
basic_wrapper!(fixed, "string_html_fixed.rs", "<tt><&\0é</tt>");
basic_wrapper!(italics, "string_html_italics.rs", "<i><&\0é</i>");
basic_wrapper!(small, "string_html_small.rs", "<small><&\0é</small>");
basic_wrapper!(strike, "string_html_strike.rs", "<strike><&\0é</strike>");
basic_wrapper!(sub, "string_html_sub.rs", "<sub><&\0é</sub>");
basic_wrapper!(sup, "string_html_sup.rs", "<sup><&\0é</sup>");
attribute_wrapper!(
    anchor,
    "string_html_anchor.rs",
    "<a name=\"a&quot;b&<>'\0é\"><&\0é</a>"
);
attribute_wrapper!(
    color,
    "string_html_fontcolor.rs",
    "<font color=\"a&quot;b&<>'\0é\"><&\0é</font>"
);
attribute_wrapper!(
    size,
    "string_html_fontsize.rs",
    "<font size=\"a&quot;b&<>'\0é\"><&\0é</font>"
);
attribute_wrapper!(
    link,
    "string_html_link.rs",
    "<a href=\"a&quot;b&<>'\0é\"><&\0é</a>"
);
