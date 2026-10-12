// Ordinary f64 operations with the exact static Math edge contracts.
fn tsr_math_round(value: f64) -> f64 {
    if !value.is_finite() || value == 0.0 || value.fract() == 0.0 {
        return value;
    }
    // Rust rounds halfway away from zero; Math chooses the positive tie.
    let mut result = value.round();
    if value < 0.0 && value.fract() == -0.5 {
        result += 1.0;
    }
    if result == 0.0 && value < 0.0 {
        -0.0
    } else {
        result
    }
}
fn tsr_math_sign(value: f64) -> f64 {
    if value == 0.0 || value.is_nan() {
        value
    } else {
        value.signum()
    }
}
// Nearest IEEE half-precision value, ties to even, returned as f64.
#[allow(dead_code)]
fn tsr_math_f16round(value: f64) -> f64 {
    if !value.is_finite() || value == 0.0 {
        return value;
    }
    let magnitude = value.abs();
    // Halfway between the largest half-precision value and 2^16 rounds away.
    if magnitude >= 65520.0 {
        return f64::INFINITY.copysign(value);
    }
    // Half precision keeps 11 significant bits; below 2^-14 the step stays 2^-24.
    let exponent = (((magnitude.to_bits() >> 52) as i32) - 1023).max(-14);
    let step = 2.0f64.powi(exponent - 10);
    ((magnitude / step).round_ties_even() * step).copysign(value)
}
// Uniform number in [0, 1). Each thread seeds one xorshift128+ state from the
// operating system's hash randomization on first use. Not for security use.
#[allow(dead_code)]
fn tsr_math_random() -> f64 {
    use std::hash::{BuildHasher, Hasher};
    thread_local! {
        static TSR_RANDOM_STATE: std::cell::Cell<(u64, u64)> = {
            let seed = |salt: u64| {
                let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
                hasher.write_u64(salt);
                hasher.finish()
            };
            // An all-zero state would stay zero.
            std::cell::Cell::new((seed(1) | 1, seed(2)))
        };
    }
    TSR_RANDOM_STATE.with(|state| {
        let (mut first, second) = state.get();
        first ^= first << 23;
        first ^= first >> 17;
        first ^= second ^ (second >> 26);
        state.set((second, first));
        // The top 53 bits give every multiple of 2^-53 below one.
        (first.wrapping_add(second) >> 11) as f64 / 9007199254740992.0
    })
}
fn tsr_math_uint32(value: f64) -> u32 {
    if !value.is_finite() || value == 0.0 {
        return 0;
    }
    // Saturating numeric casts alone do not implement ToUint32.
    let remainder = value.trunc() % 4294967296.0;
    let unsigned = if remainder < 0.0 {
        remainder + 4294967296.0
    } else {
        remainder
    };
    unsigned as u32
}
// The emitter binds every argument before this reduction can short-circuit.
fn tsr_math_extreme(values: &[f64], maximum: bool) -> f64 {
    tsr_math_extreme_iter(values.iter().copied(), maximum)
}
fn tsr_math_extreme_iter(values: impl IntoIterator<Item = f64>, maximum: bool) -> f64 {
    let mut result = if maximum {
        f64::NEG_INFINITY
    } else {
        f64::INFINITY
    };
    for value in values {
        if value.is_nan() {
            return f64::NAN;
        }
        if (maximum && value > result) || (!maximum && value < result) {
            result = value;
        } else if value == 0.0 && result == 0.0 {
            if maximum && !value.is_sign_negative() {
                result = value;
            }
            if !maximum && value.is_sign_negative() {
                result = value;
            }
        }
    }
    result
}

// The emitter evaluates both operands before these static Math edge guards.
// Finite general cases intentionally use the platform's ordinary powf.
fn tsr_math_pow(base: f64, exponent: f64) -> f64 {
    if exponent == 0.0 {
        return 1.0;
    }
    if exponent.is_nan() || (base.abs() == 1.0 && exponent.is_infinite()) {
        return f64::NAN;
    }
    base.powf(exponent)
}
// All arguments already exist in a standard stack array. Infinity takes
// precedence over NaN regardless of argument order; empty input yields +0.
// Sequential standard hypot avoids intermediate square overflow/underflow,
// but its rounding can differ from engines using a different reduction order.
fn tsr_math_hypot(values: &[f64]) -> f64 {
    tsr_math_hypot_iter(values.iter().copied())
}
fn tsr_math_hypot_iter<I: Iterator<Item = f64> + Clone>(values: I) -> f64 {
    if values.clone().any(|value| value.is_infinite()) {
        return f64::INFINITY;
    }
    values.fold(0.0f64, f64::hypot)
}
