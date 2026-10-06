# Gate 104: String.at

Status: focused implementation, editor support, documentation and benchmark support accepted. Full regressions and timing wait for the string-methods feature-family milestone.

## Behavior

- `text.at(index)` returns the character at an int position, counted from the start, or from the end when negative. The result type is `string | undefined`; a position outside the string gives `undefined`.
- Positions count Unicode characters, the same unit `charAt` uses. Node's `String.at` counts UTF-16 code units, so results differ for characters outside the Basic Multilingual Plane.
- The receiver is evaluated once, borrowed and unchanged. One int argument is required (`arity`, `type`).

## Implementation

`Analyzer::string_at` rewrites the receiver before call checking to the string's characters (`split("")`), so the existing checked array `at` supplies bounds, negative positions, the nullable result and its ownership rules. No new runtime code or call target. This allocates a character array per call; a direct character selection is a candidate optimization at the string-methods milestone.

The diagnostic for a receiver that is neither an array nor a string now reads `at() requires an array or string receiver.`

## Evidence

- `crates/tsrust-compiler/tests/string_at.rs`: two native tests written before the implementation. `array_at.rs` no longer expects a string receiver to be rejected.
- `crates/tsrust-lsp/tests/intelligence.rs`: `string_at_has_hover_help_completion_and_nullable_result`; catalog examples for `string.at` execute natively.
- `target/editor/string-at-compiler.log` and the `array_at` rerun: ten focused compiler suites pass.
- `target/editor/string-at-editor.log`: all 154 editor tests passed.
- Benchmark support: `string_at_characters` with an independent oracle, source contract and registration test. The Node counterpart selects from the string's code points so both sides retain the same characters. `target/editor/string-at-workload-verify.log` records untimed native and compiled-Node correctness; `target/editor/string-at-harness.log` records all 290 harness tests passing. No timing was run.
