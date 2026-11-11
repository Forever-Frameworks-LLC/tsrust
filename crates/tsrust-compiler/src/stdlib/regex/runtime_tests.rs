use tsrust_native_regex::*;
fn span(start: i64, end: i64) -> Match { Match { start, end } }

#[test]
fn reusable_regex_observes_unicode_byte_ranges_and_no_match() {
    let value = compile("(é)(🦀)?", None).unwrap();
    for _ in 0..2 {
        assert!(is_match(&value, "xé🦀 é"));
        assert_eq!(find(&value, "xé🦀 é"), Some(span(1, 7)));
        assert_eq!(find_all(&value, "xé🦀 é"), [span(1, 7), span(8, 10)]);
    }
    assert!(!is_match(&value, "missing"));
    assert_eq!(find(&value, "missing"), None);
    assert!(find_all(&value, "missing").is_empty());
    assert_eq!(captures(&value, "missing"), None);
    assert!(captures_all(&value, "missing").is_empty());
    assert_eq!(string_search("missing", &value), -1);
    assert_eq!(string_search("xé🦀 é", &value), 1);
}

#[test]
fn capture_slots_distinguish_missing_empty_and_full_match() {
    let value = compile("(a)?()b", None).unwrap();
    let expected = vec![
        Captures { matched: span(0, 1), groups: vec![None, Some(span(0, 0))] },
        Captures { matched: span(2, 4), groups: vec![Some(span(2, 3)), Some(span(3, 3))] },
    ];
    assert_eq!(captures(&value, "b ab"), Some(expected[0].clone()));
    assert_eq!(captures_all(&value, "b ab"), expected);
    assert_eq!(string_match("b ab", &value), captures(&value, "b ab"));
    assert_eq!(string_match_all("b ab", &value), captures_all(&value, "b ab"));
    let plain = compile("b", None).unwrap();
    assert!(captures(&plain, "b").unwrap().groups.is_empty());
}

#[test]
fn snapshots_own_ranges_and_survive_regex_and_text_disposal() {
    let (mut found, mut captures_snapshot, mut split_snapshot) = {
        let text = std::string::String::from("é🦀 é");
        let value = compile("(é)(🦀)?", None).unwrap();
        (find_all(&value, &text), captures_all(&value, &text), split(&value, &text))
    };
    assert_eq!(found, [span(0, 6), span(7, 9)]);
    assert_eq!(captures_snapshot[0].groups, [Some(span(0, 2)), Some(span(2, 6))]);
    assert_eq!(split_snapshot, [span(0, 0), span(6, 7), span(9, 9)]);
    found[0].start = 9;
    captures_snapshot[0].groups.clear();
    split_snapshot.clear();
    let value = compile("(é)(🦀)?", None).unwrap();
    assert_eq!(find_all(&value, "é🦀 é")[0], span(0, 6));
    assert_eq!(captures_all(&value, "é🦀 é")[0].groups.len(), 2);
}

#[test]
fn empty_matches_progress_at_unicode_boundaries_and_skip_adjacent_empty() {
    let empty = compile("", None).unwrap();
    assert_eq!(find_all(&empty, "é🦀"), [span(0, 0), span(2, 2), span(6, 6)]);
    assert_eq!(find_all(&empty, ""), [span(0, 0)]);
    assert_eq!(split(&empty, "é🦀"), [span(0, 0), span(0, 2), span(2, 6), span(6, 6)]);
    assert_eq!(replace_all(&empty, "é🦀", "x"), "xéx🦀x");
    let adjacent = compile("a*", None).unwrap();
    assert_eq!(find_all(&adjacent, "ab"), [span(0, 1), span(2, 2)]);
    assert_eq!(captures_all(&adjacent, "ab").len(), 2);
    assert_eq!(replace_all(&adjacent, "ab", "x"), "xbx");
}

#[test]
fn template_replacements_keep_rust_capture_and_literal_dollar_rules() {
    let value = compile("(?P<word>[a-z]+)", None).unwrap();
    let template = "${word}:$word:$0:$$:${missing}";
    assert_eq!(replace(&value, "cat dog", template), "cat:cat:cat:$: dog");
    assert_eq!(replace_all(&value, "cat dog", template), "cat:cat:cat:$: dog:dog:dog:$:");
    assert_eq!(replace(&value, "123", template), "123");
    let numbers = compile("([a-z]+)=([0-9]+)", None).unwrap();
    assert_eq!(replace_all(&numbers, "a=12 b=3", "${2}:$1 $$"), "12:a $ 3:b $");
    assert_eq!(replace(&numbers, "a=12", "$1x"), "");
    assert_eq!(replace(&numbers, "a=12", "${1}x"), "ax");
}

#[test]
fn split_ranges_preserve_boundary_and_repeated_empty_segments() {
    let value = compile(",", None).unwrap();
    let text = ",a,,b,";
    let snapshot = split(&value, text);
    assert_eq!(snapshot, [span(0, 0), span(1, 2), span(3, 3), span(4, 5), span(6, 6)]);
    assert_eq!(snapshot.iter().map(|value| &text[value.start as usize..value.end as usize]).collect::<Vec<_>>(), regex::Regex::new(",").unwrap().split(text).collect::<Vec<_>>());
    assert_eq!(split(&value, "é"), [span(0, 2)]);
    assert_eq!(split(&value, ""), [span(0, 0)]);
}

#[test]
fn escaping_is_literal_and_compile_errors_keep_owned_identity() {
    let text = "a+b[é].*";
    let escaped = escape(text);
    assert_eq!(escaped, r"a\+b\[é\]\.\*");
    assert!(is_match(&compile(&format!("^{escaped}$"), None).unwrap(), text));
    for pattern in ["(", r"(?=x)", r"(a)\1"] {
        let error = compile(pattern, None).unwrap_err();
        assert_eq!(error.code, "Syntax");
        assert!(!error.message.is_empty());
    }
    let error = compile(r"\w", Some(&Options { size_limit: Some(1), ..Options::default() })).unwrap_err();
    assert_eq!(error.code, "SizeLimit");
    assert!(!error.message.is_empty());
}

#[test]
fn optional_compile_limits_preserve_defaults_and_validate_before_build() {
    let default = compile(r"\w+", None).unwrap();
    let omitted = compile(r"\w+", Some(&Options::default())).unwrap();
    assert_eq!(find_all(&default, "é abc"), find_all(&omitted, "é abc"));
    let no_cache = compile(r"\w+", Some(&Options { dfa_size_limit: Some(0), ..Options::default() })).unwrap();
    assert_eq!(find_all(&default, "é abc"), find_all(&no_cache, "é abc"));
    assert!(compile("a", Some(&Options { nest_limit: Some(0), ..Options::default() })).is_ok());
    assert_eq!(compile("(a)", Some(&Options { nest_limit: Some(0), ..Options::default() })).unwrap_err().code, "Syntax");
    for options in [
        Options { size_limit: Some(-1), ..Options::default() },
        Options { dfa_size_limit: Some(-1), ..Options::default() },
        Options { nest_limit: Some(-1), ..Options::default() },
        Options { nest_limit: Some(i64::from(u32::MAX) + 1), ..Options::default() },
    ] {
        assert_eq!(compile("(", Some(&options)).unwrap_err().code, "Range");
    }
    #[cfg(target_pointer_width = "32")]
    assert_eq!(compile("a", Some(&Options { size_limit: Some(i64::MAX), ..Options::default() })).unwrap_err().code, "Range");
}
