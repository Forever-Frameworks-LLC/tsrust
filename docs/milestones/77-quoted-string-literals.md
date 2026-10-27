# Milestone 77: ordinary quoted String literals

Status: implementation and checkpoint 81 accepted. Focused behavior and complete IntelliSense examples pass; combined regression and selected comparisons are sealed.

## Contract

Single and double quote delimiters produce the same native UTF-8 String values. Quotes inside the opposite delimiter are ordinary characters. The lexer decodes escapes once at compile time; emitted runtime strings retain standard Rust Cow storage.

Strict ordinary escapes include escaped quotes/backslashes, backspace, form feed, newline, carriage return, tab, vertical tab, NUL with no following decimal digit, two hexadecimal digits, four hexadecimal digits and braced Unicode code points. Identity escapes preserve the following Unicode scalar. Backslash followed by LF, CR, CRLF, U+2028 or U+2029 contributes no content. Raw LF/CR inside quotes are syntax errors; raw U+2028/U+2029 and other source scalars are retained.

Native UTF-8 cannot represent lone surrogates. Escaped pairs decode into their scalar; lone high/low surrogate escapes remain source errors. Legacy octal and non-octal decimal escapes are rejected. Invalid, truncated or out-of-range Unicode escapes report source errors without panicking. Leading zeroes in braced escapes do not introduce a length limit or recursive parsing.

The reference is [ECMAScript ordinary String literals](https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html#sec-literals-string-literals). Native surrogate rejection follows [the String contract](../guides/language/string-semantics.md).

## Acceptance

Prove equivalent emitted Rust for decoded values across both quote delimiters, scalar-width boundaries, controls and identity escapes. Exercise empty and long literals, all line continuations, strict malformed escapes, source byte offsets, calls, captures, arrays, methods and static/owned result lifetimes. Editor discovery must treat quoted content as one literal without creating comments, declarations or argument separators from its contents.

Run focused compiler/editor checks and verify documentation examples before implementation acceptance. The full regression already running is retained. Under the updated [five-gate workflow](../development-workflow.md), performance comparisons and accumulated regression acceptance are scheduled at gate 81. Keep actual command evidence; pending comparisons are not measured coverage. Two matched ASCII/Unicode workloads retain complete String arrays. Their runtime measurements describe pool setup, dynamic selection and retention; quote decoding happens at compile time and is not a runtime throughput claim. Preserve earlier measured rows and retain actual raw records and normalized resource counts.

## IntelliSense documentation and checkpoint evidence

This gate adds ordinary quoted literal syntax without adding a built-in method or property. Existing editor changes are preserved. All 114 current catalog examples compile through the current compiler, and generated native execution matches complete expected stdout. The [documentation example evidence](../../benchmarks/runtime/diagnostics/2026-10-03T05-42-17-000Z-builtin-documentation-examples.json) retains catalog identities, sources, commands and outputs. Changes to editor documentation during the already running regression require affected protocol/example checks afterward; the original full regression is retained. New built-ins must satisfy all six documentation requirements before future implementation gates close.

The two quoted-literal workload pairs are correctness-checked and registered, with timing deferred to the gate 81 checkpoint. They are not measured coverage yet.

## Focused implementation acceptance

Four compiler groups pass, including O0/O3 calls, captures, arrays, methods and native Cow lifetimes. An expanded continuation group and the existing Unicode escape test also pass. Nineteen current editor groups verify quoted literal inference, completion, hover, signature argument counting and unfinished-literal suppression. Native auxiliary fixtures produce 70 exact output lines at O0/O3. Original fixture and staged ownership failures remain recorded alongside their focused repairs.

The [implementation evidence](../../benchmarks/runtime/diagnostics/2026-10-03T05-56-16-000Z-quoted-string-literals-implementation.json) binds accepted commands, sources and documentation examples. The original full regression passes without restart. Affected checks verify current editor documentation changes separately. The two accurate benchmark pairs remain pending at checkpoint 81; no timing is claimed here.

## Checkpoint 81 acceptance

The combined checkpoint accepts 700 compiler integration groups and 735 workspace groups with retained passes and no full restart. [Runtime and resource evidence](../../benchmarks/runtime/diagnostics/2026-10-03T08-44-00-000Z-checkpoint81-runtime-proof.json) binds 29 selected workloads across eight runtimes and 87 fully disposed allocation samples. [BENCHMARKS.md](../../BENCHMARKS.md) contains actual min/avg/max ops/s. Earlier pending statements describe the implementation-stage queue; this checkpoint closes that queue.
