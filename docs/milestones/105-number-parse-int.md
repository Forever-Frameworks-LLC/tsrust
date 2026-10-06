# Gate 105: Number.parseInt

Status: focused implementation, editor support, documentation and benchmark support accepted. Full regressions and timing wait for the number-methods feature-family milestone.

## Behavior

`Number.parseInt(text, radix?)` reads a whole number from the start of a string and returns a `number`.

- Leading whitespace is skipped; one `+` or `-` is accepted; reading stops at the first character that is not a digit of the radix.
- `radix` is an optional int from 2 through 36. Omitted or `0` means base 10, or base 16 when the digits start with `0x` or `0X`. Radix 16 also accepts that prefix.
- The result is `NaN` when no digit is read or when the radix is outside 2 through 36. `"-0"` gives negative zero.
- One or two arguments are required (`arity`); the text must be a string and the radix an int (`type`). The result is not an `int`.

Every case in the compiler test was checked against Node 24.11.1, including prefixes, signs, radix limits, fullwidth digits and values beyond 2^53.

## Implementation

A new call target, `CallTarget::NumberParseInt`, is checked by `Analyzer::number_parse_int` and lowered to the runtime function `tsr_number_parse_int` in `runtime_strings.rs`. Base 10 digits use standard `f64` parsing for correct rounding; other radices accumulate in `f64`. The text is borrowed and nothing is allocated.

## Evidence

- `crates/tsrust-compiler/tests/number_parse_int.rs`: two native tests written before the implementation.
- `crates/tsrust-lsp/tests/intelligence.rs`: `number_parse_int_has_hover_help_completion_and_number_result`; catalog examples for `Number.parseInt` execute natively.
- `target/editor/number-parse-int-compiler.log`: eleven focused compiler suites and the unit tests.
- `target/editor/number-parse-int-editor.log`: all editor tests.
- Benchmark support: `number_parse_int_radix` with an independent arithmetic oracle, source contract and registration test. `target/editor/number-parse-int-workload-verify.log` records untimed native and compiled-Node correctness; `target/editor/number-parse-int-harness.log` records all 292 harness tests passing. No timing was run.
