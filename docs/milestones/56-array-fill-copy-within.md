# Milestone 56: array fill and copyWithin

Status: complete. One 581-group regression, required Rust checks, fresh O0/O3 release checks, 1,836 TypeScript counterpart cases and native adapter checks pass. Sixteen comparisons across eight runtimes and 48 allocation profiles are recorded; all profiles fully dispose storage.

## Operations

Support `array.fill(value, start?, end?)` and `array.copyWithin(target, start, end?)` across every current dense array element type and rank, including recursive records. Bounds are `int`, use existing relative-index normalization, and clamp safely at both ends. Both operations return `void`, consistent with existing native mutators. JavaScript's returned receiver is an explicit language difference.

Receiver selections and arguments evaluate once, in source order. Normalize ranges against the receiver length after arguments finish. Empty or identical ranges still evaluate arguments and enforce ownership and access rules. Preserve existing selected-parent loans and early index failure order.

## Native implementation

Use standard Rust mutable-slice `fill` and `copy_within` for Copy elements. Owned fill values transfer into the operation; explicit `clone` preserves a source owner. Standard fill clones owned values into independent positions.

For owned `copyWithin`, snapshot the clipped original source with `to_vec`, then move its elements into the destination with `iter_mut().zip(snapshot)`. This preserves both overlap directions without cloning the snapshot again. Deep copies and scratch allocations are intentional costs, and differ from JavaScript reference sharing.

Existing compact arrays operate only on their initialized prefix. Numeric arrays and ordinary vectors use their live slices. Recursive record arrays reuse existing Clone and Drop implementations. Preserve fixed primitive arrays when width stays unchanged; retain conservative mutable fallback for owned fixed storage. No new storage representation or unsafe code is required.

## Acceptance and comparison

Focused native fixtures cover empty and singleton arrays, omitted and extreme bounds, reversed ranges, overlap in both directions, exact Number bits, nested independence, recursive records, argument effects, ownership diagnostics, and cleanup. Batch compatible cases into native programs. Run one full regression after focused checks settle; retain passes and resume only failed or unrun checks after a repair.

Compare matching TypeScript operations over the same dense inputs and integer ranges. Observe complete resulting arrays outside timing. Include receiver reset, range normalization, copies and retention consistently. Disclose owned-copy differences; record throughput separately from allocations and final disposal. Preserve earlier measurements and add only new or affected comparisons.

Prepare sixteen comparisons: both methods over `int[]`, `number[]`, `boolean[]`, `string[]`, `int[][]`, ordinary record arrays, `int[][][]`, and recursive record arrays. Use bounded, varying ranges and seeds to exercise both overlap directions. Independent oracles describe complete natural results, including Number bits and every record field. These drafts are not measured results.

Compiler evidence: [581-group regression and release checks](../../benchmarks/runtime/diagnostics/2026-10-02T12-40-00-000Z-array-fill-copy-within-compiler-proof.json). Counterpart evidence: [actual TypeScript and independent-result checks](../../benchmarks/runtime/diagnostics/2026-10-02T12-42-00-000Z-array-fill-copy-within-source-validation.json). These checks do not claim measured performance.

Runtime evidence: [timing, allocation and preservation proof](../../benchmarks/runtime/diagnostics/2026-10-02T12-46-47-000Z-array-fill-copy-within-runtime-proof.json). All 298 earlier runtime tables, nine primitive memory tables and auxiliary tables remain unchanged. Native leads Node after tsc on 10 of 16 rows and the fastest compared runtime on eight. Scalar array rows lead Node by roughly 8–16×; owned-copy costs remain visible in strings, records and recursive data.
