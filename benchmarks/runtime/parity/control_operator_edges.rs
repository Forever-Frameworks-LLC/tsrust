#[allow(dead_code)]
mod scalar {
    include!("scalar_control.rs");
    #[test]
    fn state_branching_preserves_checked_scalar_results() {
        assert_eq!(function0(0, 7), 7);
        assert_eq!(function0(1, 7), 139);
        assert_eq!(function0(2, 7), 138);
    }
}
#[allow(dead_code)]
mod classic {
    include!("classic_for_control.rs");
    #[test]
    fn zero_bound_skips_body_and_preserves_index() {
        assert_eq!(function0(0, 7), 7);
        assert_eq!(function0(1, 0), 0);
        assert_eq!(function0(1, 1), 2265384);
    }
}
#[allow(dead_code)]
mod do_while {
    include!("do_while_control.rs");
    #[test]
    fn zero_bound_still_runs_body_and_increments_index() {
        assert_eq!(function0(0, 7), 7);
        assert_eq!(function0(1, 0), 1);
        assert_eq!(function0(1, 1), 2265384);
    }
}
#[allow(dead_code)]
mod branches {
    include!("else_if_control.rs");
    #[test]
    fn first_matching_branch_selects_state_update() {
        assert_eq!(function0(1, 0), 11);
        assert_eq!(function0(1, 3), 470);
    }
}
#[allow(dead_code)]
mod short {
    include!("boolean_short_circuit.rs");
    #[test]
    fn exactly_one_right_operand_is_evaluated_in_each_truth_case() {
        for (left, right, bits, tag) in [
            (false, false, 0, 1),
            (false, true, 1, 1),
            (true, false, 1, 2),
            (true, true, 3, 2),
        ] {
            let mut probe = Probe::default();
            assert_eq!(evaluate(&mut probe, left, right), bits);
            assert_eq!((probe.calls, probe.trace), (1, tag));
        }
    }
}
#[allow(dead_code)]
mod and {
    include!("number_and.rs");
    #[test]
    fn wrapping_word_conversion_keeps_low_bits_after_truncation() {
        assert_eq!(operation(4294967297.75, 3.0), 1.0);
        assert_eq!(operation(-4294967297.75, 3.0), 3.0);
        assert_eq!(operation(-0.75, 3.0), 0.0);
        assert_eq!(operation(2147483648.0, -1.0), -2147483648.0);
    }
}
#[allow(dead_code)]
mod or {
    include!("number_or.rs");
    #[test]
    fn or_returns_signed_32_bit_result() {
        assert_eq!(operation(2147483648.0, 1.0), -2147483647.0);
        assert_eq!(operation(4294967296.75, 3.0), 3.0);
    }
}
#[allow(dead_code)]
mod xor {
    include!("number_xor.rs");
    #[test]
    fn xor_handles_high_bit_and_identical_values() {
        assert_eq!(operation(4294967295.0, 2147483648.0), 2147483647.0);
        assert_eq!(operation(-1.0, -1.0), 0.0);
    }
}
#[allow(dead_code)]
mod not {
    include!("number_not.rs");
    #[test]
    fn not_truncates_fraction_before_complement() {
        assert_eq!(operation(-0.75), -1.0);
        assert_eq!(operation(-1.75), 0.0);
        assert_eq!(operation(2147483648.0), 2147483647.0);
    }
}
#[allow(dead_code)]
mod shl {
    include!("number_shift_left.rs");
    #[test]
    fn shift_counts_mask_and_high_bits_wrap() {
        assert_eq!(operation(1.0, 31.0), -2147483648.0);
        assert_eq!(operation(1.0, 32.0), 1.0);
        assert_eq!(operation(1.0, -1.0), -2147483648.0);
        assert_eq!(operation(2147483648.0, 1.0), 0.0);
    }
}
#[allow(dead_code)]
mod shr {
    include!("number_shift_right.rs");
    #[test]
    fn signed_shift_extends_sign_and_masks_count() {
        assert_eq!(operation(2147483648.0, 1.0), -1073741824.0);
        assert_eq!(operation(-1.0, 33.0), -1.0);
        assert_eq!(operation(2147483647.0, -1.0), 0.0);
    }
}
#[allow(dead_code)]
mod unsigned {
    include!("number_shift_right_unsigned.rs");
    #[test]
    fn unsigned_shift_zero_extends_and_preserves_full_range() {
        assert_eq!(operation(-1.0, 0.0), 4294967295.0);
        assert_eq!(operation(-1.0, 1.0), 2147483647.0);
        assert_eq!(operation(-1.0, -1.0), 1.0);
        assert_eq!(operation(-1.0, 32.0), 4294967295.0);
    }
}
