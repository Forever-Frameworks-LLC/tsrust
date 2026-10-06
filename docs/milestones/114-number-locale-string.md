# Gate 114: numeric toLocaleString

Status: focused implementation, editor support, documentation and benchmark support accepted. Full regressions and timing remain deferred under the current implementation-first direction.

## Behavior

`value.toLocaleString()` accepts an `int` or `number` receiver and no arguments. It returns independent decimal text with comma groups of three integer digits and a period decimal separator. Formatting is fixed; it does not inspect the system locale.

- Numbers round their shortest decimal representation to at most three fractional digits, with decimal ties away from zero. Trailing fractional zeros are removed.
- Negative zero stays `-0`, including negative values rounded to zero. NaN is `NaN`; infinities are `∞` and `-∞`.
- Large finite numbers expand to full decimal notation before grouping rather than using exponents.
- Integers retain their complete signed 64-bit range exactly. Floating-point Number cannot represent every integer in that range; the native integer path never converts to floating point.
- The receiver evaluates once and is unchanged. Its value is copied; the caller owns the resulting text.
- Locale/options arguments produce an `unsupported` diagnostic at the first argument. Other receiver types produce a `type` diagnostic. Currency, alternate digits, custom precision and other locale rules are not supported.

Reference defaults and rounding terminology are described in the [ECMA-402 NumberFormat specification](https://tc39.es/ecma402/#numberformat-objects). This gate implements the documented fixed decimal display, not general Intl locale negotiation.

## Implementation

Two native call targets select exact integer grouping or decimal number formatting. Number formatting reuses the existing shortest decimal scientific digits and tie correction, expands the exponent, rounds decimal text with carry and groups the integer portion. Tiny magnitudes round directly to signed zero. The generated string-runtime inclusion guard recognizes both new helper calls, including numeric-only source programs.

The editor infers string results on both primitive types. Separate `number.toLocaleString` and `int.toLocaleString` catalog entries supply consistent zero-argument hover, completion and signature help, with four executable examples.

## Evidence

- `crates/tsrust-compiler/tests/number_locale_string.rs`: three failing tests before implementation; native formatting, once-only receiver evaluation, integer extremes, locale/options rejection and receiver errors pass.
- `tests/fixtures/number-locale-string/values.tsr` and `.stdout`: 342 explicitly Node-checked values, including carries, decimal ties, random IEEE values, signed zero, smallest subnormal and maximum finite values. `target/editor/number-locale-string-node-check.log` records Node v24.11.1 with explicit en-US and current fixture hashes. Integer endpoints match BigInt decimal grouping, avoiding Number precision loss.
- `target/editor/number-locale-string-editor.log`: two focused tests pass; hover, completion, parameter help and result types agree, and all four catalog examples execute natively.
- `target/editor/number-locale-string-compiler.log`: the new suite plus `number_to_string`, `number_formatting` and `number_text_ties` focused suites pass.
- Debug compiler/LSP and release compiler rebuilt; workspace and editor consumer binaries refreshed. Build logs are retained.
- `number_locale_string_decimal`: matched native/TypeScript workload with an independent digit/grouping oracle and written rounding/special-value cells, AST and hash source guards, registration tests, ordinary string-array ABI/driver support and allocation-profile hooks.
- `target/editor/number-locale-string-workload-verify.log` and `benchmarks/runtime/results/2026-10-04T19-18-30-390Z-verify.json`: five untimed correctness cases pass on native and Node after tsc. The Node workload uses the host default locale; the independent oracle rejects a host whose formatting differs from the fixed native rules. Workload integers remain exactly representable on both sides.
- `target/editor/number-locale-string-harness.log`: all 310 benchmark harness tests pass on current consumer binaries.

## Pending measurements

No full compiler/LSP regression, timing or allocation measurement was run. Numeric method source acceptance and future feature-family measurements remain separate. Shortest decimal conversion, text expansion, rounding and grouping allocate intermediate storage; no performance claim is made. SQLite measurement records are unchanged. Exact prior files and binaries are retained in `archive/number-locale-string-start`.
