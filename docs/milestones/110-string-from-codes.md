# Gate 110: String.fromCodePoint and String.fromCharCode

Status: focused implementation, editor support, documentation and benchmark support accepted. Full regressions and timing wait for the string-methods feature-family milestone.

## Behavior

`String.fromCodePoint(...codes)` builds a string from Unicode code points. `String.fromCharCode(...codes)` builds a string from UTF-16 code units.

- Arguments are ints, in any number, and may include spread `int[]` arrays in any position: `String.fromCodePoint(62, ...codes, 60)`. A `number` or other type is a `type` error.
- No arguments, or an empty spread array, gives `""`.
- `fromCodePoint` accepts 0 through 1114111. A code outside that range stops the program with `String.fromCodePoint code point must be between 0 and 1114111` and the source byte range of the call, after every argument has evaluated.
- `fromCharCode` uses only the low 16 bits of each value, so 65601 gives `A`. No value causes an error.
- In both functions a high surrogate followed by its low surrogate forms one character. A surrogate with no partner becomes U+FFFD, because a string always holds valid Unicode. JavaScript keeps the lone unit; printed output is the same in both.
- The codes are only read. A spread array stays usable, and a single spread source is read in place without a copy.

The compiler fixture output was produced by Node 24.11.1 for the same program.

## Implementation

`Analyzer::string_from_codes` runs before call checking and gathers the arguments into one int array expression, so ordinary and spread arguments share the existing array-literal checks and lowering. The call targets `CallTarget::StringFromCodePoint` and `CallTarget::StringFromCharCode` lower to `tsr_string_from_code_points` and `tsr_string_from_code_units` in `runtime_strings.rs`, which decode a stream of UTF-16 units without an intermediate buffer. The gathered array has the span of the argument list, so editor type lookups for the call and its arguments stay distinct.

## Editor

A new catalog family, `StringConstructor`, documents both functions with natively verified examples. `String.` now offers member completion, hover and parameter help, and both calls show a `string` result. Hover on `String`, `Object` and `Array` now ends with a `Static members:` line.

The `String()` documentation no longer says that shortest digits can differ at rounding boundaries; gate 108 removed that difference.

## Related repairs

- `Math.random(...[])` has a zero-argument spread contract, so a spread that supplies an argument is an `arity` error.
- The string runtime text now includes the number text tie helper from gate 108, so tests that compile the string runtime alone still build.

## Evidence

- `crates/tsrust-compiler/tests/string_from_codes.rs`: three native tests written before the implementation.
- `crates/tsrust-lsp/tests/intelligence.rs`: `string_from_codes_have_namespace_hover_help_completion_and_string_result`. All 160 editor tests passed.
- Focused compiler suites passed: `string_from_codes`, `math_random`, `array_literal_spread`, `call_spread`, `root_builtin_spread`, `string_at`, `string_construction`, `strings`, `primitive_storage`, `number_text_ties`, `console_output`, `array_join`, `templates`, `array_from`, `object_is`, `value_types`, `fixed_storage`, `generic_default_builtins`.
- Benchmark support: `string_from_codes_text` with an independent UTF-8 byte oracle, source contract and registration test. Untimed native and compiled-Node verification passed; all 302 harness tests passed. No timing was run.
