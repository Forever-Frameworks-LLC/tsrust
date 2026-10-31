mod random {
    include!("math_random_range.rs");
    #[test]
    fn actual_draws_cover_the_unit_interval_without_constants() {
        let mut smallest = 1.0_f64;
        let mut largest = 0.0_f64;
        let mut total = 0.0;
        let mut bits = std::collections::HashSet::new();
        for _ in 0..65536 {
            let value = draw();
            assert!(value.is_finite() && (0.0..1.0).contains(&value));
            smallest = smallest.min(value);
            largest = largest.max(value);
            total += value;
            bits.insert(value.to_bits());
        }
        assert!(smallest < 0.1 && largest > 0.9);
        assert!((0.48..0.52).contains(&(total / 65536.0)));
        assert!(bits.len() > 65000);
    }
    #[test]
    fn workload_retains_each_draw_and_scaled_roll_check() {
        let input = String::Borrowed("ignored");
        assert_eq!(function0(0, 0, &input).0.len(), 0);
        for seed in [0, 96, 1_000_000] {
            let rows = function0(2048, seed, &input).0;
            assert_eq!(rows.len(), 2048);
            assert!(rows.iter().all(|row| row.0 == [true, true, true, true]));
        }
    }
}
mod normalization {
    include!("string_normalize_forms.rs");
    #[test]
    fn canonical_compatibility_hangul_and_reordered_marks() {
        for (input, nfc, nfd, nfkc, nfkd) in [
            ("", "", "", "", ""),
            ("e\u{301}", "é", "e\u{301}", "é", "e\u{301}"),
            ("Åﬁ①", "Åﬁ①", "A\u{30a}ﬁ①", "Åfi1", "A\u{30a}fi1"),
            ("각", "각", "각", "각", "각"),
            ("a\u{315}\u{300}", "à\u{315}", "a\u{300}\u{315}", "à\u{315}", "a\u{300}\u{315}"),
            ("\u{1e69}", "\u{1e69}", "s\u{323}\u{307}", "\u{1e69}", "s\u{323}\u{307}"),
            ("ﬃ\0😀", "ﬃ\0😀", "ﬃ\0😀", "ffi\0😀", "ffi\0😀"),
        ] {
            for (form, expected) in [("NFC",nfc),("NFD",nfd),("NFKC",nfkc),("NFKD",nfkd)] {
                let actual = normalize(input, form);
                assert_eq!(actual, expected);
                assert_eq!(normalize(&actual, form), actual);
            }
        }
    }
    #[test]
    fn default_and_dynamic_paths_remain_complete_owned_outputs() {
        let rows = function0(20, 17, &String::Borrowed("ignored"));
        assert_eq!(rows.len(), 20);
        for (i, row) in rows.iter().enumerate() {
            assert_eq!(row.0.len(), 7);
            assert_eq!(row.0[0], row.0[4]);
            assert_eq!(row.0[0], row.0[5]);
            assert_eq!(row.0[i%4], row.0[6]);
            assert!(row.0.iter().all(|value| matches!(value, String::Owned(_))));
        }
        assert!(function0(0, 0, &String::Borrowed("")).is_empty());
    }
    #[test]
    #[should_panic(expected="invalid normalization form")]
    fn unsupported_forms_are_rejected() { normalize("é", "nfc"); }
}
