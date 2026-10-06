# Gate 111: String codePointAt and charCodeAt

Status: focused implementation, editor support, documentation and benchmark support accepted. Full regressions and timing wait for the string-methods feature-family milestone.

## Behavior

`text.codePointAt(position?)` returns the Unicode code point of the character at a position. `text.charCodeAt(position?)` returns its UTF-16 code unit.

- `position` is an optional int, default `0`. Positions count whole Unicode characters, as `charAt` and `at` do.
- `codePointAt` has the result type `int | undefined`. A negative position or one at or past the end gives `undefined`; positions do not count from the end.
- `charCodeAt` has the result type `number` and gives `NaN` for a position that names no character. For a character above U+FFFF it gives the first (high surrogate) unit.
- More than one argument is an `arity` error; a position that is not an int is a `type` error.
- The string is only read.

For text whose characters are all below U+10000, results equal Node's; the first three lines of the compiler test were produced by Node 24.11.1. Node counts positions in UTF-16 units, so positions after a character above U+FFFF differ there.

## Implementation

`Analyzer::string_code_point_at` runs before call checking and rewrites `text.codePointAt(p)` to an internal slot call followed by `.at(0)`. The slot call (`CallTarget::StringCodePointSlot`, internal member name `codePointAt#slot`, which source text cannot spell) yields an int array that holds the code when the position names a character and is empty otherwise. The existing array `at` contract then supplies the checked `int | undefined` result. `CallTarget::StringCharCodeAt` lowers inline to a character lookup and returns a number.

Both lookups walk the text from the start, so cost grows with the position. The slot array is a short-lived allocation of at most one element. Neither cost has been measured.

## Evidence

- `crates/tsrust-compiler/tests/string_code_at.rs`: two native tests written before the implementation.
- `crates/tsrust-lsp/tests/intelligence.rs`: `string_code_at_methods_have_hover_help_completion_and_result_types`; three catalog examples execute natively. All 161 editor tests passed.
- Focused compiler suites passed: `string_code_at`, `string_at`, `array_at`, `string_from_codes`, `strings`, `string_substring`, `string_substr`, `call_spread`, `root_builtin_spread`, `value_types`, `fixed_storage`, `generic_default_builtins`.
- Benchmark support: `string_code_at_positions` with an independent table oracle, source contract and registration test. The matched pair always passes a position, because TypeScript requires the argument. Untimed native and compiled-Node verification passed; all 304 harness tests passed. No timing was run.
