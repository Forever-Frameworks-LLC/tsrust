# Gate 108: Math.f16round

Status: focused implementation, editor support, documentation and benchmark support accepted. Full regressions and timing wait for the math-functions feature-family milestone.

## Behavior

`Math.f16round(value)` rounds a number to the nearest value that 16-bit (half precision) floating point can hold and returns it as a `number`.

- A value exactly halfway between two candidates goes to the one with an even last bit: `Math.f16round(2049.0)` is `2048`.
- Magnitudes of 65520 and above give `Infinity` with the sign of the input.
- Magnitudes too small for half precision give zero with the sign of the input. Results below `2^-14` use the fixed step `2^-24`.
- `NaN`, infinities and zeros are returned unchanged.
- One number argument is required (`arity`, `type`). An int needs `Number(...)` first, as for the other `Math` functions.

The compiler fixture holds 341 values (grid edges, ties, small results, overflow and 300 generated values) with output produced by Node 24.11.1.

## Implementation

`CallTarget::MathF16round` joins the exact `Math` group and lowers to the runtime function `tsr_math_f16round` in `runtime_math.rs`. The function finds the half-precision step from the exponent bits and rounds once with ties to even, so no double rounding through single precision occurs. Nothing is allocated.

## Defects fixed in this gate

Both were found by the reference fixture and are covered by `crates/tsrust-compiler/tests/number_text_ties.rs`.

1. **Tied shortest digits.** Some numbers sit exactly halfway between two shortest digit strings that both read back as the same value, for example `0.000233173370361328125`. Number text printed the upper string (`…813`); the reference prints the string with an even last digit (`…812`). A new runtime helper, `tsr_number_tie` in `runtime_number_tie.rs`, detects the tie from the exact decimal expansion and corrects console output, `String()`, `toString()`, `join()` and array display. Most values leave the helper after a few integer operations. The fixture checks 339 tied values through every path, and a separate run of 19,996 generated values matched Node.
2. **Numbers in template literals.** A number inside a template used a different formatter and printed `inf`, `-inf`, `-0`, and long digit runs where exponent form is required. Templates now use the same number text as `String()`.

Stale expectations updated: `templates.rs` (expected `inf`, `-inf` and `-0`) and `fixed_number_storage.rs` (expected `includes(NaN)` to be `false`, already corrected in gate 106).

## Editor

`Math.f16round` has catalog documentation with a natively verified example. The editor now shows `number` as the result of every `Math` call and constant, so `const value = Math.fround(1.5)` hovers as `const value: number` and member completion works on the result.

## Evidence

- `crates/tsrust-compiler/tests/math_f16round.rs`: two native tests written before the implementation.
- `crates/tsrust-compiler/tests/number_text_ties.rs`: two native tests for the defect fixes.
- `crates/tsrust-lsp/tests/intelligence.rs`: `math_f16round_has_hover_help_completion_and_number_result`. All 158 editor tests passed.
- Focused compiler suites passed: `math_f16round`, `math_exact`, `math_approx`, `math_unary`, `number_text_ties`, `number_formatting`, `number_operators`, `number_parse_int`, `number_predicates`, `number_prefix_parsing`, `number_to_string`, `numbers`, `templates`, `array_join`, `console_output`, `fixed_number_storage`, `int_to_string`, `value_types`, `fixed_storage`, `generic_default_builtins`.
- Benchmark support: `math_f16round_values` with an independent arithmetic oracle, source contract and registration test. The TypeScript counterpart declares the function type because the comparison library set does not include it. Untimed native and compiled-Node verification passed for this workload and for 16 number-text and template workloads affected by the fixes; all 298 harness tests passed. No timing was run.
