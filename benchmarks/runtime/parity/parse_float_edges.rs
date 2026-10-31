// Semantic witnesses for the independently authored Rust decimal-prefix scanner.
include!("number_parse_float_mixed.rs");
#[test]fn scans_prefixes_and_exact_whitespace(){
    assert_eq!(parse_float("1e+tail"),1.0);
    assert_eq!(parse_float("1.2.3"),1.2);
    assert_eq!(parse_float("0x1"),0.0);
    assert_eq!(parse_float("\u{feff}\t.5tail"),0.5);
    for text in ["\u{85}12","\u{180e}12","\u{200b}12",".e1","--1",""]{assert!(parse_float(text).is_nan(),"{text:?}");}
}
#[test]fn preserves_ieee_extremes_and_signed_zero(){
    assert_eq!(parse_float("-0tail").to_bits(),(-0.0_f64).to_bits());
    assert_eq!(parse_float("-1e-9999").to_bits(),(-0.0_f64).to_bits());
    assert_eq!(parse_float("5e-324").to_bits(),1);
    assert_eq!(parse_float("1e309"),f64::INFINITY);
    assert_eq!(parse_float("-Infinity!"),f64::NEG_INFINITY);
}
