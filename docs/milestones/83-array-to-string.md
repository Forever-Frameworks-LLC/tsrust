# Milestone 83: Array.toString and direct array conversion

Status: complete. Focused implementation and documentation, the retained checkpoint 86 regression, and matched runtime/resource comparisons are accepted.

## Contract

`toString(): string` converts a dense homogeneous array into comma-separated text without changing its elements. No arguments are accepted. Its text matches `join()` with the default comma. Nested arrays use internal commas; null and undefined positions contribute empty text, records contribute `[object Object]`, and supported scalar values retain the conversion rules documented for join. Native signed integers retain full 64-bit decimal values. Number zero becomes `0`; NaN and infinities retain their names. Finite decimal and scientific text follows the native formatter. Custom prototype overrides and coercion hooks are unsupported.

The existing `String(array)` operation must produce the same text through direct borrowed iteration into a standard Rust String. Extra arguments still evaluate once, left to right, before their values are ignored. Preserve existing loans and temporary lifetimes. Primitive String conversions and console inspection retain their existing behavior.

Ownership: receiver and argument arrays remain borrowed and reusable; the text result owns independent storage. Preserve fixed arrays, borrowed slices, Vec and inline owning array layouts. Avoid diagnostic trees, per-element cloned strings and materialized intermediate string arrays.

## Acceptance

Focused fixtures cover every supported leaf and nested rank, empty literals, Unicode/NUL, numeric boundaries, captures, conditional receivers, temporary values, ignored-argument effects, loans and disposal. Verify complete leaf text at the existing maximum array rank of 128 and rejection of rank 129. The old diagnostic cutoff does not truncate valid array-only content within this bound; this change preserves text while removing diagnostic traversal. Execute complete catalog examples at O0/O3 and keep parameter help, signature, completion and hover consistent.

Register six actual JavaScript `Array.toString()` counterparts with independent complete-output expectations. Refresh the existing `string_array_conversion` measurement at checkpoint 86; scalar `string_explicit_int` is unaffected. Preserve all existing source rows and report records. Allocation assertions describe focused construction and disposal evidence, not throughput measurements.

## Focused evidence

[Implementation evidence](../../benchmarks/runtime/diagnostics/2026-10-03T09-36-55-000Z-array-to-string-implementation.json) accepts four compiler groups and one affected editor group. Seventeen catalog examples match nineteen complete output lines at each native optimization level, O0 and O3. Compiler fixtures cover all supported leaves, records and nesting, ignored argument effects, loans, captured/temporary storage and normal/unwind disposal. Both array text paths execute the maximum rank128 with complete leaf text; rank129 rejects.

Six source counterparts pass 111 independent cases, twelve literal hands, 101 negative guards and six executed semantic mutants. Current compiler check/emit accepts all six programs, and pool/order witnesses execute against exact expected output at O0/O3. The authored deep Rust wrapper needed one storage-ABI correction; its original failure remains preserved and only that group repeated. No full regression or timing run occurs. Earlier 514 report rows remain unchanged; the previous 520 manifest entries are exact.

## Checkpoint 86 measurements

[Completed checkpoint evidence](../../benchmarks/runtime/diagnostics/2026-10-03T14-02-43-000Z-checkpoint86-completed-proof.json) accepts 806 named checks on the tested revision, retaining successful targets and repairing failures without a full restart. [Runtime/resource evidence](../../benchmarks/runtime/diagnostics/2026-10-03T13-57-02-000Z-checkpoint86-runtime-proof.json) accepts 6 comparisons for this milestone across eight runtimes. Native leads compiled Node on 5/6 and the fastest scripting runtime on 3/6. The affected existing `string_array_conversion` row is also refreshed; its native/Node ratio is 2.861× and native/fastest scripting ratio is 0.736×. Results measure complete setup, selection, operation and retained output; they do not establish isolated method latency or allocation parity.

Across gates 82–86, forty new rows and one existing-row refresh produce 30,504 timed batches and 123 native profiles, all with zero final live bytes. The report now contains 554 individually dated measured rows. The other 513 report sections, nine primitive memory tables and auxiliary baselines remain byte-exact. Concurrent project/editor revisions outside the tested checkpoint are explicitly excluded from its regression claim.
