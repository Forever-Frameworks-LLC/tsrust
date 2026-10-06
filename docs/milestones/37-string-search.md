# Milestone 37: standard string search

[Roadmap](../roadmap.md) · [Native string contract](../string-semantics.md)

## Implementation contract

Add `startsWith`, `endsWith`, `includes`, and `indexOf` through Rust's standard `str` methods. Each accepts a string needle and an optional integer position. Receiver, needle, and position evaluate once, left to right. Both string inputs remain borrowed while later arguments evaluate; no implicit clone or transfer occurs.

Positions use UTF-8 byte coordinates. Negative positions clamp to zero and positions beyond length clamp to length. `startsWith`, `includes`, and `indexOf` search a suffix starting at that position; the default is zero. `endsWith` examines a prefix ending at its position; the default is the full byte length. This differs from `slice`, whose negative offsets count backward from the end.

Selected prefix and suffix boundaries must be valid UTF-8 boundaries, including searches for an empty needle. Invalid boundaries fail explicitly. `indexOf` returns the absolute UTF-8 byte offset of the first match, or `-1`. The other methods return a boolean. Empty needles follow the standard Rust substring methods after boundary validation.

Array `includes` and `indexOf` retain their existing element and offset contracts. Receiver type determines the method implementation without duplicate receiver evaluation.

## Acceptance

Cover empty strings and needles, absent and repeated matches, ASCII, multi-byte scalars, combining marks, NUL, clamped positions, invalid boundaries, and integer extremes. Verify source reuse, temporary results, fields, indexed strings, captures, single evaluation, overlapping argument mutations, and consumed values at optimization levels 0 and 3.

Use one full regression after the four-method implementation settles. Match observable byte-search behavior in runtime counterparts, validate results independently, and preserve true timing and allocation records. Unicode representation conversion costs must be disclosed rather than presented as JavaScript's native UTF-16 search cost.

## Status

The gate is complete. All four search methods are integrated, and four focused groups pass at optimization levels 0 and 3. All 495 compiler groups pass, together with doctests, formatting, Clippy, and the release build. The single regression exposed an obsolete test that rejected string `includes`; that fixture was repaired, with only its failed group rerun and previously unrun binaries resumed. No full regression restart was needed.

The [compiler acceptance record](../../benchmarks/runtime/diagnostics/2026-10-02T00-32-06-449Z-native-string-search-compiler-acceptance.json) preserves exact commands, logs, source hashes, the fixture repair, and the continuation inventory across 47 integration binaries. Production compiler logic did not change during verification.

Eight ASCII/Unicode comparisons pass across all eight runtime variants, with 320 correctness checks and 5,952 timed batches in the [execution record](../../benchmarks/runtime/results/2026-10-02T00-34-53-278Z.json). Existing startup and compilation baselines were retained. Native averages range from 55.268 to 253.779 million searches per second; all eight exceed Node after `tsc`, while ASCII `indexOf` remains below Deno in this run. These are matched workload measurements with digest and query-selection costs, not universal engine rankings.

Unicode counterparts translate byte positions and returned indices to and from JavaScript's UTF-16 coordinates. That mapping setup is included in each batch. The timed seed selects seven explicit positions (0, 3, 8, 11, 16, 22, and 999), alongside default positions; negative positions and other boundary cases remain covered by correctness tests. No negative-position throughput claim is made.

The [allocation record](../../benchmarks/runtime/diagnostics/2026-10-02T00-35-15-345Z-native-allocations.json) contains three samples per workload. All eight rows report zero operation allocations, reallocations, retained bytes, and peak requested bytes, with zero live bytes after full disposal. Input buffers are accounted for separately. The prior 57 measured rows and immutable raw history remain preserved.

The [runtime acceptance proof](../../benchmarks/runtime/diagnostics/2026-10-02T00-37-25-000Z-native-string-search-runtime-proof.json) records 164 frozen inputs and 290 unchanged historical records, with exact timed/profile emission identities verified before the next gate changed compiler sources.
