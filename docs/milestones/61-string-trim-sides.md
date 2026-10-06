# Milestone 61: directional string trimming

Status: complete. Four focused groups, one 601-group regression, required Rust checks and fresh O0/O3 release checks pass. Eight matched comparisons across eight runtimes and twenty-four native allocation profiles complete acceptance. Every output and final disposal check reaches zero live bytes.

## Scope and native contract

Support `string.trimStart()`, `string.trimEnd()`, and the aliases `trimLeft()` and `trimRight()`. Use standard Rust `str::trim_start()` and `str::trim_end()` with the existing valid UTF-8 Cow representation. Preserve interior whitespace and the opposite edge. Whitespace follows the pinned Rust toolchain, consistent with existing `trim()`; U+0085 is whitespace and U+FEFF is preserved.

Evaluate and borrow the receiver once. Static borrowed literals can retain borrowed static slices; trimming an owned receiver produces an independent owned result. Receiver storage remains reusable. Preserve existing access rules for parameters, captures, fields, indexed strings and temporary expressions. Add no encoding cache, custom whitespace implementation, or implicit coercion.

## Acceptance

All four methods pass focused O0/O3 checks for empty and all-whitespace strings, ASCII and Unicode whitespace, astral and combining scalars, embedded NUL, preserved interior text and exact aliases. Ownership checks cover receiver effects, chains, parameters, captures, fields, indexed strings and temporaries. Source diagnostics and normal/unwind disposal pass. Borrowed static results allocate nothing; owned results remain independent after the source is disposed. The full 601-group regression ran once.

Eight matched rows cover each method over ASCII and Unicode input. Counterparts call the actual JavaScript built-ins. Timed inputs use whitespace shared by pinned Rust and JavaScript; U+0085 and U+FEFF are excluded. Separate compiler and independent source witnesses verify the deliberate native differences: U+0085 trims and U+FEFF remains. Aliases retain separate source coverage and benchmark rows.

Each invocation sets up eight receiver variants, including two explicit runtime-input clones, three owned repeat variants and three static literals. Setup, runtime selection, one directional trim and retention of every complete string result in a String[] are timed. Complete content and shape observation runs outside timing. Rust Cow literals can return borrowed static slices; owned receivers produce independent owned results. JavaScript may reuse immutable storage. These comparisons measure complete workload iterations and do not claim identical allocation behavior or isolated-call latency.

Actual source checks cover 8 programs and 1,728 TypeScript observations, 24 mutation/domain negatives, 48 independent hand cells, 8 whitespace-divergence hand cells and complete source-order output. All 18 unique Node harness groups pass. Native preflight records thirty-four O0/O3 builds, eighty complete outputs, sixteen untimed allocation/disposal checks and two complete order outputs.

Timing records 192 process cells and 5,952 measured batches, publishing sixty-four min/average/max rate cells. Twenty-four allocation profiles retain raw counters and report three-sample min/average/max allocations, reallocations and requested, retained and peak bytes per operation. Input adaptation remains separate. Normalized peak is the retained-batch peak divided by 2,048 operations, not an isolated-call peak. Output and final disposal remain absolute zero checks. Earlier 350 runtime tables, nine primitive memory tables and auxiliary startup/compiler baselines remain byte-exact and were not measured again.

Native leads Node after tsc on 0/8 rows and the fastest scripting runtime on 0/8. The following ratios use measured average rates. Resource columns show three-sample averages per complete operation; the runtime proof retains all min/average/max statistics and raw profiles.

| Workload | Native / Node after tsc | Native / fastest scripting | Allocations / operation | Requested bytes / operation | Retained-batch peak bytes / operation |
| --- | ---: | ---: | ---: | ---: | ---: |
| `string_trim_start_ascii` | 0.789× | 0.789× | 0.6284 | 58.4180 | 34.5117 |
| `string_trim_start_unicode` | 0.729× | 0.723× | 0.6284 | 60.4258 | 36.5195 |
| `string_trim_end_ascii` | 0.827× | 0.786× | 0.6284 | 58.4180 | 34.5117 |
| `string_trim_end_unicode` | 0.793× | 0.790× | 0.6284 | 60.4258 | 36.5195 |
| `string_trim_left_ascii` | 0.769× | 0.769× | 0.6284 | 58.4180 | 34.5117 |
| `string_trim_left_unicode` | 0.752× | 0.719× | 0.6284 | 60.4258 | 36.5195 |
| `string_trim_right_ascii` | 0.821× | 0.803× | 0.6284 | 58.4180 | 34.5117 |
| `string_trim_right_unicode` | 0.790× | 0.782× | 0.6284 | 60.4258 | 36.5195 |

Compiler evidence: [601-group regression and fresh release checks](../../benchmarks/runtime/diagnostics/2026-10-02T15-18-00-000Z-string-side-trimming-compiler-proof.json). Source evidence: [actual TypeScript and independent complete-output checks](../../benchmarks/runtime/diagnostics/2026-10-02T15-25-00-000Z-string-trim-repeat-pool-source-supplement.json). Runtime evidence: [timing, native outputs and normalized resource acceptance](../../benchmarks/runtime/diagnostics/2026-10-02T15-36-57-000Z-string-trim-runtime-proof.json).
