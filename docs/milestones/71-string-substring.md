# Milestone 71: string substring selection

Status: complete. String.substring passes compiler, editor, source, native and runtime/resource acceptance: 641 compiler groups, 656 workspace groups, six matched comparisons and eighteen fully disposed profiles.

The [source proof](../../benchmarks/runtime/diagnostics/2026-10-03T00-40-49-000Z-string-substring-source-validation.json), [compiler proof](../../benchmarks/runtime/diagnostics/2026-10-03T00-49-43-000Z-string-substring-compiler-proof.json) and [runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-03T00-56-53-000Z-string-substring-runtime-proof.json) retain actual source, command and result identities. Earlier 454 measured sections remain unchanged.

## Scope

Implement `String.substring(start: int, end?: int): string` using standard Rust UTF-8 storage and slicing. The required start and optional end follow the TypeScript method signature. Clamp each supplied bound to `[0, byteLength]`; omitted end means byteLength. Select the range between the smaller and larger clamped bounds, so reversed bounds swap rather than producing an empty slice. Negative bounds clamp to zero, unlike String.slice relative offsets. Number arguments, coercion, omitted start and explicit undefined remain unsupported under current strict typing.

Receiver, start and end evaluate once in source order, including empty or full-range results. Borrow the receiver through argument evaluation and result construction; preserve inferred parameter/capture loans, fields and selected temporary lifetimes. Keep current Array.slice, String.slice and charAt behavior unchanged.

Use the established standard Cow slicing policy: an empty range returns borrowed static empty text without accessing a boundary, including equal indices inside a scalar. Full range uses standard Cow cloning, preserving static borrowed storage and independent owned copies. A valid partial slice of static text stays borrowed; a valid partial slice of owned text copies independently with standard Rust `to_owned`. A nonempty range with an endpoint inside a UTF-8 scalar fails with a source-byte diagnostic. Combining-scalar boundaries remain valid; no grapheme indexing or UTF-16 representation is introduced.

The [ECMAScript substring algorithm](https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.substring) supplies clamping, swapping and end-default behavior. Byte indexing and valid-scalar boundaries remain deliberate native differences from JavaScript UTF-16 code units.

## Acceptance

Four focused O0/O3 groups pass on the first command. They cover strict bounds/arity, clamp/swap/default behavior, nested expression contexts, earlier borrowed roots, once-only effects, captures/callbacks, selected temporary lifetimes and normal-stack depth diagnostics. Unicode evidence includes two-, three- and four-byte scalars, combining text, NUL and FEFF, valid swapped selections, invalid nonempty endpoints and equal interior-byte ranges that return empty without boundary access. Static partial/full results stay borrowed with zero result allocations; nonempty owned partial/full results copy independently with one allocation. Normal and unwind disposal pass, preserving existing String.slice, Array.slice, charAt, concat, padding and addition.

The sole full workspace regression passes 641 compiler groups and fifteen additional workspace groups without a restart. Eight existing editor groups exercise the catalog, completion, hover, signature help and current compiler witnesses with substring included before that full run. Doctests, formatting, all-target Clippy, debug/release builds and fresh byte-identical CLI emissions pass. The saved Gate70 release rejects substring. An independent 28-output hand fixture matches Node and native O0/O3. Its original native emission failed on an authored NUL escape; only the escape spelling was repaired, preserving the failed command and before bytes. This was a fixture correction, not a compiler regression or full-run restart.

Source acceptance passes six TypeScript programs, 822 complete-output cases, 48 literal hand cells, 125 negative checks including ten executed semantic mutants, two Node groups and twenty-one order lines. Native O0/O3 preflight accepts twenty-six builds, sixty complete outputs, twelve disposal checks and two order executions. Original command records and complete observations remain preserved.

Six comparisons cover default, bounded and swapped substring in ASCII and Unicode. Runtime input/pool construction, bound-tuple setup, dynamic selection, the actual method, loops and complete String[] retention are timed. Unicode JavaScript workloads initialize valid byte/UTF-16 endpoint tuples during timed setup and select corresponding endpoints per operation; dynamic input receivers use whole-value boundaries. Mapping/setup costs are included, with no per-operation encoder allocation, full mapping scan or substring reimplementation. Invalid native byte boundaries remain separate correctness evidence. Complete observation stays outside timing. Natural static borrowing and independent owned copying differ from possible JavaScript immutable sharing; neither allocation parity nor isolated method latency is claimed.

The execution-only run records 144 process cells, 4,464 timed batches and forty-eight min/average/max rate cells across eight runtimes. Native leads compiled Node on 6/6 rows and the fastest scripting runtime on 6/6. Average-rate ratios span 1.595–1.648× versus Node and 1.310–1.589× versus the fastest scripting runtime. These are whole-workload results, not a universal native-speed claim. Complete per-row rates remain in [BENCHMARKS.md](../../BENCHMARKS.md).

All eighteen profiles return output and final owners to absolute zero. Raw counters and normalized min/average/max allocations, reallocations and requested/retained bytes per complete operation are retained across three 2,048-operation samples. Input adaptation stays separate. Peak means retained-batch peak divided by operations, not isolated-call peak. Static subslice sharing, owned receiver copies, partial-result construction and complete String[] storage have their natural measured costs; JavaScript allocation parity is not claimed.

The report contains 460 separately dated comparisons. Earlier 454 measured sections, nine primitive memory tables and auxiliary baselines remain byte-exact and were not retimed. No combined 460-row run is claimed. The closed certificate binds the actual proofs, release, source, manifest and report without changing historical acceptance records.

## Remaining separate work

Number bounds, omitted start, coercion and explicit undefined remain unsupported. Native bounds count UTF-8 bytes rather than JavaScript UTF-16 units, and nonempty endpoints must fall on Unicode scalar boundaries; combining sequences are not indivisible graphemes.
