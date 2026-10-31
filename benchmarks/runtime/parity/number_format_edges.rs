include!("number_to_string_radix.rs");
#[test]
fn normalizes_zero_nonfinite_and_exponent_signs() {
    assert_eq!(number_text(-0.0), "0");
    assert_eq!(fixed(-0.0, 2), "0.00");
    assert_eq!(exponential(-0.0, Some(2)), "0.00e+0");
    assert_eq!(precision(-0.0, 5), "0.0000");
    assert_eq!(fixed(f64::INFINITY, 2), "Infinity");
    assert_eq!(exponential(f64::NEG_INFINITY, None), "-Infinity");
    assert_eq!(precision(f64::NAN, 5), "NaN");
}
#[test]
fn handles_corpus_notation_thresholds_and_subnormals() {
    assert_eq!(number_text(1e-6), "0.000001");
    assert_eq!(number_text(1e-7), "1e-7");
    assert_eq!(number_text(1e21), "1e+21");
    assert_eq!(exponential(f64::from_bits(1), None), "5e-324");
    assert_eq!(precision(f64::from_bits(1), 5), "4.9407e-324");
    assert_eq!(fixed(9.999, 2), "10.00");
    assert_eq!(precision(12345.678, 5), "12346");
    assert_eq!(precision(1e20, 5), "1.0000e+20");
}
#[test]
fn writes_exact_eighths_in_even_radices() {
    assert_eq!(radix_text(0.125, 2), "0.001");
    assert_eq!(radix_text(0.125, 16), "0.2");
    assert_eq!(radix_text(0.125, 36), "0.4i");
    assert_eq!(radix_text(-35.875, 8), "-43.7");
    assert_eq!(radix_text(-0.0, 8), "0");
}
