# Milestone 74: Number decimal formatting

Status: complete. 653 compiler groups, 670 workspace groups, six matched comparisons and eighteen disposed native profiles pass.

## Scope

Implement `number.toFixed(fractionDigits?: int)`, `number.toExponential(fractionDigits?: int)` and `number.toPrecision(precision?: int)`, returning standard UTF-8 strings. Receivers must have the native `number` type; precision arguments must be strict `int` values. Each method accepts zero or one argument. Fixed and exponential precision ranges are 0 through 100; significant precision is 1 through 100. Omitted fixed precision means zero decimal places. Omitted exponential and significant precision use Rust's shortest formatter.

Rust's standard `f64` formatting supplies every rounded digit. Exact halfway values round to even, a documented native difference from JavaScript. For example, a number receiver holding 2.5 returns `"2"` from `toFixed(0)`, while JavaScript returns `"3"`. Shortest-digit tie choices also follow the pinned Rust formatter. These methods are native variants, not claims of complete JavaScript decimal conversion parity. See [Rust formatting precision](https://doc.rust-lang.org/std/fmt/#precision) and the [ECMAScript Number methods](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-number.prototype.tofixed).

Small text adapters preserve familiar notation: exact negative zero formats as positive zero; special values spell `NaN`, `Infinity` and `-Infinity`; scientific exponents have an explicit sign and no leading zeros. Shortest decimal notation switches to scientific notation below magnitude 1e-6 or at magnitude 1e21. Fixed formatting at magnitude 1e21 or above uses shortest decimal formatting. Significant formatting rounds once through the standard scientific formatter, then selects scientific notation when the rounded exponent is below -6 or at least the precision; otherwise it moves the decimal point in that formatted text.

The receiver and optional argument evaluate once, left to right. The receiver is copied before argument effects. Fixed precision bounds are checked before nonfinite handling. Exponential and significant methods return nonfinite spellings before checking runtime precision bounds, after evaluating the argument. Runtime precision failures retain source locations. Existing borrowing, inferred contracts, captures, temporary lifetimes and fixed-array storage remain valid.

## Acceptance

Focused compiler tests cover all precision bounds, signs, zero, nonfinite values, subnormal and extreme finite values, notation thresholds, rounding carries, explicit native halfway differences, diagnostics, effect order and disposal at O0/O3. Editor completion, hover, optional signatures and inferred string chains are included before one full regression.

Six matched workloads cover omitted and explicit precision for all three methods. Complete string results must agree on the independently established shared benchmark domain. Native differences remain visible in separate semantic evidence. Each operation includes pool setup, selection, actual formatting and complete output retention; observation stays outside timing. Original run records provide throughput min/average/max across the existing eight runtimes. Three allocation samples per row provide normalized native resource counters and complete disposal. Earlier 472 workload rows remain unchanged. Completed evidence uses the existing SQLite archive.

Full floating `number.toString(radix)` remains pending: Rust has no standard arbitrary-radix float formatter. This milestone does not silently implement only its decimal branch or claim its completion.

## Executed acceptance

Four focused compiler groups pass at O0/O3 across precision ranges, native rounding, notation boundaries, strict types and arity, effects, inferred contracts, captures, loans, temporary lifetimes, fixed storage, source diagnostics and normal/unwind disposal. Editor discovery and optional signatures pass. One full regression passes 653 compiler groups and 670 workspace groups. A subsequent isolated-call witness exposes missing runtime inclusion; seven isolated programs then pass at O0/O3 after the narrow dependency repair. A standard String buffer change reruns only the four affected groups at O0/O3. Unaffected full-regression passes remain accepted. Formatting, Clippy, debug/release builds and doctests pass; fresh emitters match accepted native fixtures. No full regression restarts.

Source checks cover 210 complete cases, 144 independent literal hands, 122 negatives including six executed wrong-precision mutants, and fourteen effect-order outputs. Native O0/O3 reproduces all 144 hands and fourteen order outputs at both levels. Shared benchmark values have independently established complete-string agreement; halfway and shortest-tie differences remain explicit native semantic evidence.

Baseline and optimized runs each retain 144 process cells, 4,464 timed batches and 48 published rate cells across eight engines. Standard String buffers improve native throughput by 27–49% over the baseline. Native leads compiled Node on 3/6 rows and the fastest scripting runtime on 0/6. Rates include pool setup, selection, formatting and complete result retention. Each phase has eighteen profiles across three 2,048-operation samples per row. Final profiles use 1,793 allocations and eight reallocations per sample, with zero live bytes after output and final disposal. Counters retain normalized min/average/max values; peak is retained-batch peak divided by operations. Input adaptation is separate, and JavaScript allocation parity is not claimed. Reserving standard buffers increases retained/requested bytes on several rows; this memory tradeoff remains visible in the resource proof. Every digit still comes from the pinned Rust formatter, with no custom float encoding or rounding engine. Remaining formatter performance gaps are documented rather than treated as solved.

Earlier 472 runtime sections, nine memory tables and auxiliary tables remain exact and are not retimed. The report now contains 478 separately dated workload rows, without claiming one combined run.

The [compiler proof](../../benchmarks/runtime/diagnostics/2026-10-03T03-29-54-000Z-number-format-buffer-compiler-proof.json) retains original commands, inputs and outputs.

The [source proof](../../benchmarks/runtime/diagnostics/2026-10-03T02-54-36-000Z-number-format-source-validation.json) retains original commands, inputs and outputs.

The [runtime and resource proof](../../benchmarks/runtime/diagnostics/2026-10-03T03-38-36-000Z-number-format-buffer-runtime-proof.json) retains original commands, inputs and outputs.

The [baseline runtime proof](../../benchmarks/runtime/diagnostics/2026-10-03T03-18-37-000Z-number-format-baseline-runtime-proof.json) retains original commands, inputs and outputs.

