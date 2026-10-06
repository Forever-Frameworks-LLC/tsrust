# Milestone 58: array toSpliced

Status: complete. Four focused groups, one 589-group regression, required Rust checks and fresh O0/O3 release checks pass. Eight matched comparisons across eight runtimes, 918 TypeScript observations, eighteen unique Node harness groups and 24 native allocation profiles complete acceptance. Every output and final disposal check reaches zero live bytes.

## Operation and ownership

Support `array.toSpliced(start?, deleteCount?, ...items)` over every current dense array leaf and rank. No arguments copy the whole array. A start argument alone removes the suffix. With a delete count, clamp negative counts to zero and oversized counts to the remaining suffix. Negative start indices count from the end; normalize signed integer extremes safely. Start and delete count are `int`; inserted values match the element type.

Return a fresh owned array and keep the source available. Clone retained descendants independently, skip deleted descendants before cloning, and transfer inserted owners into the result. Explicit `clone` retains inserted owners for their original bindings. This follows the existing native ownership contract; JavaScript shallow reference sharing remains a documented difference.

Evaluate the receiver and every argument once, from left to right, before constructing the result. Retain receiver and selected-parent read loans during argument evaluation. A selected receiver's bounds failure precedes argument evaluation. Preserve contextual typing for empty arrays and infer inserted elements when the receiver is an empty literal.

## Native implementation

Reuse current array containers and ordinary vectors. Build the result with standard Rust prefix and suffix iterators, `.take().cloned()` and `.skip().cloned()`, and move insertions between them. Reserve the checked final capacity once. Literal string storage uses the existing borrowed literal representation; no new runtime representation or unsafe implementation is introduced.

## Acceptance and comparisons

Focused checks cover every arity, signed bounds, exact Number bits, all current leaves at ranks one through three, independent descendants, literal and selected receivers, argument effects, ownership and loan diagnostics, and normal and unwind disposal. Allocation instrumentation proves deleted payloads are never cloned and insertion pointers transfer unchanged.

The single full regression completed once. Eight comparisons cover int, Number, boolean, string, nested, record, deep and recursive arrays across the eight existing runtimes. Ten runtime-selected forms cover omitted arguments, signed starts, bounded deletion and zero through three inserted values. Each iteration retains its whole returned array through normal ownership transfer, with no extra retention clone. The outer result adds one array rank; complete observations preserve variable-length result boundaries, every descendant and record field, Unicode scalars and exact Number bits. Input setup, insertion construction and natural result retention are timed; complete observer traversal runs outside timing. Native independent owned copies and JavaScript shallow sharing remain disclosed.

TypeScript library declarations omit the legal zero-argument toSpliced overload. Counterparts add only `interface Array<T>{toSpliced():T[];}`; the JavaScript call and behavior remain unchanged. The initial type-check failure stopped before any observations and is preserved. Existing Node passes were retained; only the affected source-binding group was resumed before the corrected 918-case check. Independent checks include ten hand cells, eight signed-start boundary cells, fifteen mutation/domain negatives and complete evaluation-order output.

Actual native preflight covers 34 O0/O3 builds, 80 complete outputs, sixteen untimed allocation/disposal checks and two complete order outputs. Timing records 192 process cells and 5,952 measured batches, publishing 64 min/average/max rate cells. Twenty-four allocation profiles retain raw counters and report min/average/max allocations, reallocations and bytes per operation across three samples. Input adaptation remains separate. Peak memory is the retained-batch peak divided by 128 operations, not an isolated-call peak. These normalized resource measurements remain separate from throughput. Whole retained output remains live until disposal; every output-drop and final-drop check reaches zero. All 330 earlier runtime tables, nine primitive memory tables and auxiliary startup/compiler baselines remain byte-exact. No earlier measurements were repeated.

Compiler evidence: [589-group regression and fresh release checks](../../benchmarks/runtime/diagnostics/2026-10-02T13-25-52-000Z-array-to-spliced-compiler-proof.json). Source evidence: [actual TypeScript, independent complete-output checks and retained failed attempt](../../benchmarks/runtime/diagnostics/2026-10-02T13-40-00-000Z-array-to-spliced-source-validation.json). Runtime evidence: [timing, complete native output and allocation acceptance](../../benchmarks/runtime/diagnostics/2026-10-02T13-50-12-000Z-array-to-spliced-runtime-proof.json).

Native leads Node after tsc on 3 of eight rows and the fastest scripting runtime on 3. Complete owned copying costs remain part of the comparison.
