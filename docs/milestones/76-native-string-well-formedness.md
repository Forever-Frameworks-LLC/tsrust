# Milestone 76: native String well-formedness

Status: complete. 661 compiler groups, 680 workspace groups, four matched comparisons and twelve fully disposed profiles pass.

## Scope

Implement strict zero-argument `String.isWellFormed(): boolean` and `String.toWellFormed(): string`. Every representable TSRust string is valid UTF-8, so the predicate returns true and conversion preserves the complete text. Both evaluate their receiver once and borrow it. Predicate results must not remove receiver effects, diagnostics or temporary disposal. Conversion uses existing standard Cow cloning: static text remains borrowed and owned text copies independently.

There is no extra validation scan, UTF-16 storage, primitive wrapper or method table. JavaScript can represent lone surrogates; native UTF-8 strings cannot. Surrogate repair is therefore outside the native representation, rather than a measured compatibility case. This gate does not change source decoding or accepted escape syntax. Empty text, NUL, BOM, combining marks, astral scalars, noncharacters and U+10FFFF remain ordinary valid contents.

See [Rust String representation](https://doc.rust-lang.org/std/string/struct.String.html#representation), [ECMAScript isWellFormed](https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.iswellformed) and [ECMAScript toWellFormed](https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.towellformed).

## Acceptance

Focused compiler checks cover strict arity/type diagnostics, once-only receiver evaluation, contexts, inferred contracts, captures, temporary lifetimes and fixed storage. Generated programs run at O0/O3. Allocation witnesses observe dynamic predicate results and complete conversion text, and verify normal/unwind disposal. Editor completion, signatures and inferred boolean/string chains agree. Run one full regression once implementation settles; rerun only affected checks after any repair.

Four paired ASCII/Unicode workloads retain natural Boolean[] or String[] results and compare complete outputs independently. Pool setup, selection, method execution and retention are timed; observation is outside timing. Publish min/average/max ops/s across eight engines plus normalized native resource counters. Previous 484 runtime sections and nine memory tables remain unchanged. Raw results remain available, and completed evidence enters SQLite only after exact restoration checks.

## Executed acceptance

Four focused compiler groups pass first attempt at O0/O3, including used and unused receiver effects, trapping receivers, incoming loans, captures, selected temporary consumers and fixed storage. Unicode witnesses cover all scalar widths, U+10FFFF, NUL, BOM, replacement characters, combining sequences, noncharacters and astral charAt/split outputs. Observed, black-boxed resource witnesses verify predicate zero allocations, static borrowing, independent owned Cow copies and normal/unwind disposal. The existing safe String Cow helper is reused unchanged. Strict arity/type and ordinary-stack depth diagnostics pass. Editor completion, zero-parameter signatures and Boolean/String result chains pass.

One full workspace regression accepts 661 compiler groups and 680 total workspace groups, without repairs or a restart. Doctests, formatting, Clippy, debug/release builds and fresh byte-exact emission pass. Source validation retains two Node groups, 548 complete cases, 48 literal hands, 89 negatives and five executed semantic mutants. Four native O0/O3 builds reproduce 96 complete pool/order lines. Four malformed UTF-8 CLI inputs are rejected at the existing source boundary; no decoder or surrogate policy changes.

The execution-only run retains 96 process cells, 2,976 timed batches and 32 rate cells across eight engines. Pool construction, dynamic receiver selection, the actual methods and natural Boolean[]/String[] retention are timed; complete observation is outside timing. The native predicate exploits the valid UTF-8 invariant. Conversion copies owned native strings, so these whole-workload rates do not describe JavaScript surrogate repair or isolated method latency. Native leads compiled Node on 4/4 rows and the fastest scripting runtime on 4/4. Twelve profiles retain raw counters and normalized min/average/max allocations, reallocations and requested/retained bytes across three 2,048-operation samples per row. Input adaptation is separate. Peak is retained-batch peak divided by operations, not an isolated-call peak; all output and final disposal counters return to zero. Predicate allocation witnesses observe zero method allocations; owned conversion copies naturally, while JavaScript may share string storage. No allocation parity is claimed. Earlier 484 runtime sections, nine memory tables and auxiliary tables remain exact and are not retimed. The report contains 488 separately dated rows.

The [compiler proof](../../benchmarks/runtime/diagnostics/2026-10-03T05-08-16-000Z-string-well-formed-compiler-proof.json) retains original inputs, commands and outputs.

The [source proof](../../benchmarks/runtime/diagnostics/2026-10-03T04-43-42-000Z-string-well-formed-source-validation.json) retains original inputs, commands and outputs.

The [runtime and resource proof](../../benchmarks/runtime/diagnostics/2026-10-03T05-16-49-000Z-string-well-formed-runtime-proof.json) retains original inputs, commands and outputs.

