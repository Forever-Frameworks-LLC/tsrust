# Gate 106: number.toString

Status: focused implementation, editor support, documentation and benchmark support accepted. Full regressions and timing wait for the number-methods feature-family milestone.

## Behavior

`value.toString(radix?)` returns a number as text.

- With no argument, or radix 10, the result is the shortest decimal text that reads back as the same number, including exponent form such as `1e+21`.
- With an int radix from 2 through 36, the whole part and the fraction are written in that base. The fraction uses as many digits as are needed to read back the same number. Digits above 9 are lowercase letters.
- `NaN`, `Infinity` and `-Infinity` print as those words in every base; negative zero prints as `0`.
- More than one argument is an `arity` error and a non-int radix is a `type` error. A radix outside 2 through 36 stops the program with a message and source position, as `int.toString` does. Node throws a `RangeError` there.

## Reference comparison

Output matches Node 24.11.1 exactly on the fixture `fixtures/number-to-string/radix.tsr` (29 values in 7 radices, including the largest and smallest numbers and values above 2^53) and on `rand.tsr` (400 generated large, fractional, tiny and negative values in nine radices).

For values above 2^53, digits the number cannot represent print as zeros. The rule was established against Node on 40,000 generated values: keep dividing by the radix, emitting `0`, while the quotient is still at least 2^53.

## Implementation

A new call target, `CallTarget::NumberToString`, shares the `toString` arity and radix checks with `int.toString` in `analysis.rs` and lowers to `tsr_number_to_string` in `runtime_strings.rs`. The fraction loop rounds its last digit to nearest and carries. The receiver is copied; one string is allocated for the result.

## Evidence

- `crates/tsrust-compiler/tests/number_to_string.rs`: three native tests; the fixtures were generated from Node before the implementation.
- Four older tests expected `number.toString()` to be rejected (`int_to_string`, `number_formatting`, `primitive_identity`, and one editor test). They now check the supported behavior or a still-invalid call.
- `crates/tsrust-lsp/tests/intelligence.rs`: `number_to_string_has_hover_help_completion_and_string_result`; catalog examples execute natively.
- `target/editor/number-to-string-compiler.log` plus the reruns of the three updated suites; `target/editor/number-to-string-editor.log`: all editor tests.
- Benchmark support: `number_to_string_radix` with an independent digit oracle (repeated division and exact scaled fractions, cross-checked against the platform formatter on all 2048 cases), source contract and registration test. `target/editor/number-to-string-workload-verify.log` records untimed native and compiled-Node correctness; `target/editor/number-to-string-harness.log` records all 294 harness tests passing. No timing was run.
