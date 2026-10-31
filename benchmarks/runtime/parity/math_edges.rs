include!("math_round.rs");
#[test]
fn rounds_half_toward_positive_and_keeps_negative_zero() {
    assert_eq!(round_up_ties(-1.5), -1.0);
    assert_eq!(round_up_ties(1.5), 2.0);
    assert_eq!(round_up_ties(-0.5).to_bits(), (-0.0_f64).to_bits());
    assert_eq!(round_up_ties(-0.0).to_bits(), (-0.0_f64).to_bits());
    assert_eq!(round_up_ties(f64::MAX), f64::MAX);
    assert_eq!(round_up_ties(f64::INFINITY), f64::INFINITY);
    assert!(round_up_ties(f64::NAN).is_nan());
}
#[test]
fn sign_keeps_nan_and_both_zeros() {
    assert!(sign(f64::NAN).is_nan());
    assert_eq!(sign(-0.0).to_bits(), (-0.0_f64).to_bits());
    assert_eq!(sign(0.0).to_bits(), 0.0_f64.to_bits());
    assert_eq!(sign(f64::NEG_INFINITY), -1.0);
    assert_eq!(sign(f64::from_bits(1)), 1.0);
}
#[test]
fn extrema_propagate_nan_and_choose_the_correct_zero() {
    assert!(min_pair(1.0, f64::NAN).is_nan());
    assert!(max_pair(f64::NAN, 1.0).is_nan());
    for (a,b) in [(-0.0,0.0),(0.0,-0.0)] {
        assert_eq!(min_pair(a,b).to_bits(), (-0.0_f64).to_bits());
        assert_eq!(max_pair(a,b).to_bits(), 0.0_f64.to_bits());
    }
}
#[test]
fn uint32_truncates_wraps_and_maps_nonfinite_to_zero() {
    assert_eq!(uint32(-1.75), u32::MAX);
    assert_eq!(uint32(4294967297.75), 1);
    assert_eq!(uint32(f64::NAN), 0);
    assert_eq!(uint32(f64::INFINITY), 0);
    assert_eq!(uint32(-0.5), 0);
    assert_eq!(uint32(-1.0).wrapping_mul(uint32(5.0)) as i32, -5);
}
