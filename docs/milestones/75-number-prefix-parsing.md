# Milestone 75: Number decimal prefix parsing

Status: complete. 657 compiler groups, 675 workspace groups, six matched comparisons and eighteen fully disposed profiles pass.

## Scope

Implement `Number.parseFloat(text: string): number` as a strict, borrowed UTF-8 string operation. Exactly one string argument is required; implicit coercion is unsupported. The argument evaluates once, with ordinary loan and temporary lifetime rules. Existing `Number(text)` whole-string conversion remains unchanged.

A small syntax adapter selects the longest decimal prefix after ECMAScript leading whitespace. It accepts an optional sign, decimal integer/fraction syntax and a complete optional exponent. Incomplete exponents roll back to the mantissa. Case-sensitive `Infinity` prefixes produce signed infinity. Invalid starts produce NaN. Signed zero, overflow, underflow and rounding come from standard Rust `str::parse::<f64>()`; no custom float representation or decimal rounding engine is added. The scanner borrows a substring and does not allocate.

Leading whitespace follows the ECMAScript list, including BOM U+FEFF and line terminators. U+0085, U+180E and U+200B remain invalid starts. See [ECMAScript parseFloat](https://tc39.es/ecma262/multipage/global-object.html#sec-parsefloat-string), [whitespace grammar](https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html#sec-white-space), and [Rust f64 parsing](https://doc.rust-lang.org/std/primitive.f64.html#impl-FromStr-for-f64).

## Acceptance

Compiler acceptance covers independent edge expectations, incomplete exponents, UTF-8 boundaries, very long mantissas/exponents, IEEE extremes, once-only effects, captures, inferred contracts, temporary and indexed strings, fixed storage, strict types and arity, source diagnostics and disposal. Isolated generated programs compile at O0/O3. Editor completion, signature help and inferred number operations must agree. Run one full regression after focused checks; subsequent repairs use affected checks only.

Six matched workloads exercise decimal, exponent, whitespace, trailing text, invalid/nonfinite and mixed pools. Retain every parsed number and validate complete IEEE observations outside timing. Preserve original min/average/max rates across eight engines, normalized native resource counters, and true raw run records. Earlier 478 report rows remain unchanged. Compact finished evidence into SQLite only after consumers finish and exact restore checks pass.

`Number.parseInt`, global parsing aliases, locale conversion, boxed Number objects and arbitrary-radix float formatting remain separate pending operations.

## Executed acceptance

Four focused compiler groups pass at O0/O3. A diagnostic fixture was corrected to expect the existing unknown-name diagnostic for the unsupported global alias; allocation witnesses now observe all short results and hide inputs from constant folding. Only the two affected groups rerun; the unaffected passes remain accepted. One full regression passes 657 compiler groups and 675 workspace groups. Formatting, Clippy, doctests and debug/release builds pass. Editor discovery, required string signature and inferred number chains pass. Fresh debug/release emitters match accepted native fixtures byte-for-byte.

An independent standalone parser probe matches Node across 1,218 cases, including rounding midpoints, 100k-digit inputs, overflow, signed underflow, all leading whitespace and explicit exclusions. This probe is separate from generated-program acceptance. Source acceptance executes 702 complete workload cases, 160 literal IEEE hands, 149 negatives including six executed semantic mutants, and eighteen effect-order outputs. Generated native O0/O3 reproduces all 160 literal comparisons and eighteen order outputs at both levels. Invalid NaN observations canonicalize payload/sign; signed-zero bits remain distinct.

A shared Number timing wrapper initially lacks the generic external-string argument hooks. Normalizing its existing iteration name and trailing-comma call layout fixes both preparation and argument insertion. Thirty-six actual complete Rust/TypeScript observations verify old two-argument Number and Number-array behavior plus new external inputs, including empty and Unicode strings. The original failed fixture command and its narrow correction remain recorded; compiler and workload bodies do not change.

The execution-only run retains 144 process cells, 4,464 timed batches and 48 rate cells across eight engines. Native leads compiled Node on 6/6 rows and the fastest scripting runtime on 6/6. Rates measure complete pool setup, selection, parsing and Number result retention. Eighteen profiles retain three 2,048-operation samples per row, with normalized min/average/max allocations, reallocations and requested/retained bytes. Input adaptation is separate; peak is retained-batch peak divided by operations. All output and final disposal counters return to zero. Compiler witnesses separately prove zero parser allocations for observed short and 100k-digit inputs; the complete workloads also allocate input-pool and output storage. JavaScript allocation parity is not claimed. Earlier 478 runtime sections, nine memory tables and auxiliary tables remain exact and are not retimed. The report contains 484 separately dated rows without claiming one combined run.

The [compiler proof](../../benchmarks/runtime/diagnostics/2026-10-03T04-25-02-000Z-number-parse-float-compiler-proof.json) retains original inputs, commands and outputs.

The [source proof](../../benchmarks/runtime/diagnostics/2026-10-03T04-01-58-000Z-number-parse-float-source-validation.json) retains original inputs, commands and outputs.

The [runtime and resource proof](../../benchmarks/runtime/diagnostics/2026-10-03T04-29-32-000Z-number-parse-float-runtime-proof.json) retains original inputs, commands and outputs.

