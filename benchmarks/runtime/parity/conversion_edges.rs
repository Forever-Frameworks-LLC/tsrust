mod numeric {
    include!("numeric_string_conversion.rs");
    #[test]
    fn parses_whitespace_radices_and_rounds_wide_integer_once() {
        assert_eq!(number("\u{feff}12\u{a0}"), 12.0);
        assert_eq!(number("0b101"), 5.0);
        assert_eq!(number("0o17"), 15.0);
        assert_eq!(number("0x20000000000003"), 9007199254740996.0);
        assert_eq!(number(""), 0.0);
    }
    #[test]
    fn preserves_signed_underflow_and_rejects_non_decimal_tokens() {
        assert_eq!(number("-1e-999").to_bits(), (-0.0_f64).to_bits());
        assert_eq!(number("-0").to_bits(), (-0.0_f64).to_bits());
        assert_eq!(number("1e999"), f64::INFINITY);
        for value in ["inf", "12tail", "-0x10", "0b2", "\u{85}12", "1_000"] {
            assert!(number(value).is_nan(), "{value:?}");
        }
    }
}
mod half {
    include!("math_f16round_values.rs");
    #[test]
    fn selects_even_neighbors_and_preserves_signed_underflow() {
        assert_eq!(half_round(2049.0), 2048.0);
        assert_eq!(half_round(2051.0), 2052.0);
        assert_eq!(half_round(2.0_f64.powi(-25)), 0.0);
        assert_eq!(half_round(3.0*2.0_f64.powi(-25)), 2.0_f64.powi(-23));
        assert_eq!(half_round(-2.0_f64.powi(-25)).to_bits(), (-0.0_f64).to_bits());
        assert_eq!(half_round(f64::from_bits(1)), 0.0);
    }
    #[test]
    fn handles_normal_extremes_overflow_and_nonfinite_values() {
        assert_eq!(half_round(65504.0), 65504.0);
        assert_eq!(half_round(65519.0), 65504.0);
        assert_eq!(half_round(65520.0), f64::INFINITY);
        assert_eq!(half_round(-65520.0), f64::NEG_INFINITY);
        assert_eq!(half_round(f64::INFINITY), f64::INFINITY);
        assert!(half_round(f64::NAN).is_nan());
    }
}
mod grouping {
    include!("number_locale_string_decimal.rs");
    #[test]
    fn rounds_decimal_text_carries_groups_and_keeps_negative_zero() {
        assert_eq!(locale(0.2345), "0.235");
        assert_eq!(locale(999.9995), "1,000");
        assert_eq!(locale(-0.0004), "-0");
        assert_eq!(locale(-0.0005), "-0.001");
        assert_eq!(locale(1e21), "1,000,000,000,000,000,000,000");
        assert_eq!(locale(f64::NEG_INFINITY), "-∞");
    }
}
mod defaults {
    include!("default_parameter_number_value.rs");
    #[test]
    fn evaluates_fallback_only_for_missing_value() {
        let mut trace=Trace{calls:0,defaults:0};
        assert_eq!(apply(&mut trace,0,Some(-0.0)).to_bits(),(-0.0_f64).to_bits());
        assert_eq!(trace.defaults,0);
        assert_eq!(apply(&mut trace,0,None),25.0);
        assert_eq!((trace.calls,trace.defaults),(2,1));
    }
}
