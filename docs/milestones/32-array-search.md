# Milestone 32: read-only primitive array search

[Roadmap](../roadmap.md) · [Array removal](./30-array-removal.md) · [Foundation audit](../plans/foundation-audit.md)

**Status:** Complete. Source contracts, edge acceptance, resource checks, and all current comparisons have recorded evidence. Remaining runner performance gaps are tracked in the foundation audit before complex features.

## Methods and supported values

`includes(value, fromIndex?)` returns `boolean`. `indexOf(value, fromIndex?)` returns the first matching zero-based `int` index, or `-1`. Both read dense `int[]`, `boolean[]`, and `string[]` arrays without transferring the receiver or a string needle. The needle has the exact element type; the optional offset is an `int` and defaults to zero. Empty arrays return `false` or `-1`.

Offsets follow the supported integer subset of [ECMAScript array search](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.includes). A nonnegative offset starts at that position, clamped to length. A negative offset starts that far from the end, clamped to zero. Offsets at or beyond the end find nothing. Normalization safely handles signed 64-bit minimum and maximum values. Repeated elements return their first match at or after the normalized start.

Integer and boolean equality is exact. String equality compares content without trimming, case folding, or Unicode normalization. For the implemented integer/boolean/valid-Unicode-string types, `includes` and `indexOf` have the same match relation. Floating-point NaN and signed-zero distinctions, sparse arrays, coercion, and owned object/reference identity are not part of this source subset. Arrays of records and arrays are rejected rather than silently introducing deep equality or aliases.

```typescript
function main() {
    const values = [7, 9, 7];
    console.log(values.includes(7));       // true
    console.log(values.indexOf(7, 1));     // 2
    console.log(values.indexOf(7, -1));    // 2
    console.log(values.includes(7, 3));    // false
}
```

## Borrowing and evaluation

Named, field, indexed, literal, cloned, and returned receivers are supported. Receiver selection runs once, followed by needle evaluation and then optional offset evaluation. Selected-index bounds failures occur before search arguments; arguments still run when the receiver is empty or the offset excludes every element. Existing source arithmetic failures remain checked at all backend optimization settings.

A named or selected receiver holds a shared loan through both arguments. A string needle holds its source-root shared loan through offset evaluation. Mutation, replacement, or transfer through a conflicting root is rejected with a related source location. This is stricter than JavaScript's aliasing model and preserves the language's unique mutable ownership. The loans end with the call; later mutation and transfer remain valid.

Temporary receivers own independent storage. For example, `clone(values).includes(change(values))` can search the clone selected before `change` mutates the original. Search itself does not clone elements, grow containers, increment shared string references, or create a source view. Partial field availability, shared iteration, recursive access inference, local capture loans, callback contracts, and retained owned factories retain their existing behavior.

## Representation and acceptance

Shared private helpers search logical live elements. Copied numeric leaves use slices; string arrays use their inline occupied slots or heap slice. Unused/default slots are unobservable. Integer first-match search compares bounded groups of four live elements, selects the first matching lane, and handles the remainder separately; boolean search retains scalar early exit. Heap string search unrolls four comparisons with an immediate exit after each; it does not evaluate later lanes after a match. No vector instruction, alignment, or padding guarantee enters the source contract. The emitter adds search support when source methods or selected private representations require it. No per-value method allocation, public Rust ABI, container-layout guarantee, unsafe support, or tracing garbage collector is introduced.

Fifteen regression groups cover signatures and source diagnostics, duplicates and signed offsets, exact Unicode/embedded-NUL equality, receiver and needle loans, temporary ownership, mixed selection order, clone/removal/growth/emptying, partial field moves, read-only literal specialization, closures/callbacks/factories, and source failure order. A differential model checks integer, boolean, and dynamic-string search across widths zero through 80, both storage variants, removal, signed extremes, and independent clones. An additional model records every comparison across widths zero through 80, signed offsets, duplicates, removal, and both storage forms, proving immediate exit and precise comparison order. Shared string reference counts remain unchanged. Debug and optimized executables pass; failure-order cases also run with native overflow checks on and off.

Current string equality uses standard UTF-8 content comparison. The earlier interned-descriptor and cache policy was superseded by the native Rust string migration; historical gate-32 measurements preserve that old policy at their recorded date.

## Comparison requirement

Six basic matched workloads measure both methods on each supported primitive array type. Each builds 129 dense elements, varies positive/negative/beyond-end offsets, and observes every result in a checksum. Integer data includes duplicates and misses; boolean data has rare false values; string data uses distinct Unicode labels and an external string equal to one label. Independent algebraic oracles locate results without calling JavaScript search or constructing the measured arrays; nine oracle tests compare them with a separate dense traversal across multiple seeds and sizes.

Two further matched workloads construct every array element from external runtime-owned Unicode input. They mix common lowercase values and rare mixed-case values, then query both matching contents and a same-length Unicode miss. Literal interning cannot determine those results.

The [full 35-workload record](../../benchmarks/runtime/results/2026-10-01T06-16-41-224Z.json) and [two dynamic-string comparisons](../../benchmarks/runtime/results/2026-10-01T06-20-37-081Z.json) validate all eight runtime modes on five inputs per workload and retain 27,528 measured batches. All 37 native mean rates exceed Node after tsc. Literal string `indexOf` remains 0.77× Bun and dynamic normalization 0.98× Deno; the [foundation audit](../plans/foundation-audit.md) reviews every comparison and rejected alternative.

The [37-workload resource diagnosis](../../benchmarks/runtime/diagnostics/2026-10-01T06-30-12-232Z-native-allocations.json) verifies complete cleanup without increasing any earlier allocation count or byte total. The [primitive memory record](../../benchmarks/runtime/results/2026-10-01T06-09-54-430Z-allocations.json) and [final source/emission proof](../../benchmarks/runtime/diagnostics/2026-10-01T06-41-29-972Z-search-final-proof.json) preserve separate measurement provenance. All 443 workspace regression groups and 18 benchmark/oracle checks pass. JavaScript does not enforce the source loan and transfer diagnostics, so those guarantees have no equivalent runtime throughput ratio.

## Deferred capabilities

Owned record/array identity, floating-point values, sparse arrays, coercive offsets, string substring methods, reverse search, predicate search, callback traversal, mutable/consuming iteration, public slices/views, sorting, and collection fusion remain separately scoped. These deferrals do not change the completed method contracts above.

## Subsequent helper optimization

The [full 37-workload retiming](../../benchmarks/runtime/results/2026-10-01T08-14-04-568Z.json) follows shared normalization and immediate-exit string-loop improvements. All native means exceed Node after tsc. Dynamic normalization exceeds Deno 1.17×; literal string `indexOf` remains 0.79× Bun. The [foundation audit](../plans/foundation-audit.md) preserves the earlier snapshots, current allocation evidence, and remaining performance scope. The implementation gate stays complete; its benchmark evidence is updated as the foundation improves.

## Subsequent proven literal storage

Read-only local arrays containing only string literals can use private static reference slices after complete-use and budget proofs. Named read-only forwarding preserves the specialization. Indexed reads and iteration expose borrowed string values; explicit clones restore independently mutable owned arrays. Mutated, moved, captured, dynamic, oversized, or unproved arrays retain ordinary storage. The earlier canonical-domain optimization is retired by milestone 33; current search uses standard UTF-8 content equality.

The [fresh 37-workload record](../../benchmarks/runtime/results/2026-10-01T08-50-08-509Z.json) preserves every source pair, driver, and oracle. All native means exceed Node after tsc; literal string `indexOf` exceeds Bun 1.24×. The paired experiment improves both literal search methods 1.53× and removes their only heap allocation. All 449 workspace regression groups and 18 benchmark checks pass. The foundation audit retains current recursive growth/clone comparisons that remain near or below the fastest runner; the source-method gate stays complete.
