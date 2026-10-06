# Gate 112: String toLocaleLowerCase and toLocaleUpperCase

Status: focused implementation, editor support, documentation and benchmark support accepted. Full regressions and timing wait for the string-methods feature-family milestone.

## Behavior

`text.toLocaleLowerCase()` and `text.toLocaleUpperCase()` return the same strings as `toLowerCase()` and `toUpperCase()`.

- They take no arguments. A locale argument is an `unsupported` error at the argument, with a message that says to remove it.
- The conversion is the general Unicode mapping and does not depend on the system language. A character can change length, for example `"ß"` to `"SS"`.
- Text in Turkish, Azerbaijani or Lithuanian can need rules that this conversion does not apply.
- The receiver is only read; the result is a new string.

The compiler test output was produced by Node 24.11.1 with the `"en-US"` locale.

## Implementation

`Analyzer::string_locale_case` runs before call checking and renames the call to the plain case method when the receiver is a string and no argument is present. No new call target or runtime code is needed.

## Evidence

- `crates/tsrust-compiler/tests/string_locale_case.rs`: two native tests written before the implementation.
- `crates/tsrust-lsp/tests/intelligence.rs`: `string_locale_case_methods_have_hover_completion_and_string_result`; both catalog examples execute natively. All 162 editor tests passed.
- Focused compiler suites passed: `string_locale_case`, `strings`, `string_code_at`, `string_at`, `generic_default_builtins`, `value_types`.
- Benchmark support: `string_locale_case_text` with an independent written-forms oracle, source contract and registration test. The Node side uses the host default locale; the chosen letters map the same way under every locale without special rules for them. Untimed native and compiled-Node verification passed; all 306 harness tests passed. No timing was run.
