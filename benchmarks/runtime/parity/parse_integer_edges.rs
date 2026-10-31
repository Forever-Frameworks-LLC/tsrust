include!("number_parse_int_radix.rs");
#[test]
fn accepts_sign_prefix_and_trailing_text() {
    assert_eq!(parse_integer("\u{feff}\t-0XfFtail", 0), -255.0);
    assert_eq!(parse_integer("+101suffix", 2), 5.0);
    assert_eq!(parse_integer("zZ!", 36), 1295.0);
    assert_eq!(parse_integer("12px", 10), 12.0);
    assert_eq!(parse_integer("0xff", 10), 0.0);
}
#[test]
fn rejects_missing_digits_and_invalid_radix() {
    for (text, radix) in [("", 0), ("0x", 0), ("-", 10), ("z12", 0), ("10", 1), ("10", 37), ("\u{85}12", 0)] {
        assert!(parse_integer(text, radix).is_nan());
    }
}
#[test]
fn preserves_negative_zero_and_exact_integer_boundaries() {
    assert_eq!(parse_integer("-0px", 0).to_bits(), (-0.0_f64).to_bits());
    assert_eq!(parse_integer("1fffffffffffff", 16), 9007199254740991.0);
    assert_eq!(parse_integer("20000000000000", 16), 9007199254740992.0);
}
