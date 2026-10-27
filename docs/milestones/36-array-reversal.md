# Milestone 36: native array reversal

[Roadmap](../roadmap.md) · [Primitive and collection scope](../guides/language/primitives-and-collections.md)

## Contract

`values.reverse()` reverses a dense array in place using Rust's standard slice reversal. It takes no arguments and returns `void`, consistent with the existing mutation-oriented `push()` contract. JavaScript returns its array receiver; using that return value is a deliberate language difference.

Named arrays, record fields, and indexed nested arrays are mutable receivers. Receiver selections evaluate once and preserve existing source bounds checks, ownership rules, and root loans. Borrowed iteration elements cannot be mutated.

Reversal preserves length, capacity, ownership, and element identity. Empty and singleton arrays are valid. Proven fixed primitive arrays retain their Rust array representation. Growable primitive arrays reverse their live slice; owned-element arrays reverse only their live prefix or vector. Read-only literal specializations fall back to mutable storage when reversal is used. Recursive record arrays continue to use their existing vector storage.

## Temporary array length

The same array gate also removes the existing restriction that permits `.length` on temporary strings but rejects temporary arrays. Array-returning calls and typed array literals must expose read-only length, evaluate once, and dispose of temporary owned elements normally. Non-array/non-string receivers remain errors.

## Acceptance

- Verify empty, singleton, inline-boundary, and spilled arrays without reallocating or dropping elements during reversal.
- Cover integers, booleans, strings, nested arrays, records, and recursive records, including independent clones.
- Preserve mutable parameter and capture contracts, single evaluation of indexed receivers, and source failure order.
- Reject invalid receiver types, arguments, consumed values, conflicting loans, and uses of the `void` result.
- Verify temporary array length, single evaluation, read-only diagnostics, empty values, and cleanup.
- Run focused native tests at optimization levels 0 and 3, then one full compiler regression after implementation settles.
- Compare equivalent in-place reversal with TypeScript/Node and the pinned runners; retain independently checked results and true timing/allocation records.

## Completed acceptance

All seven focused groups pass at optimization levels 0 and 3. The single full regression passes 491 groups across 46 integration binaries, including doctests; formatting, Clippy with warnings denied, and the release build also pass. The [compiler proof](../../benchmarks/runtime/diagnostics/2026-10-02T00-11-32-972Z-native-array-reverse-compiler-acceptance.json) preserves exact commands, source identities, and focused fixture repairs. No full regression was repeated.

The [three-workload execution record](../../benchmarks/runtime/results/2026-10-02T00-13-46-688Z.json) contains 120 independent correctness checks and 2,232 timing batches across eight runtimes. Native average reversals/s are 30,601,169 for fixed integers, 21,674,548 for growable integers, and 52,800,394 for owned strings. Each workload observes resulting order; numeric construction/growth and string equality/digest costs are included. These are matched complete workloads, not isolated reversal instruction costs.

The [allocation profiles](../../benchmarks/runtime/diagnostics/2026-10-02T00-14-04-697Z-native-allocations.json) contain nine samples with zero retained bytes and zero live owned bytes after disposal. Fixed integers allocate nothing. Growable construction allocates 20,000 times with a 256-byte peak; owned string setup allocates six times with a 147-byte peak, excluding the separately recorded 11-byte input. Reversal itself preserves capacity and drops no elements, as verified by native witness tests.

The [runtime proof](../../benchmarks/runtime/diagnostics/2026-10-02T00-15-08-000Z-native-array-reverse-runtime-proof.json) preserves 148 inputs, 284 previous raw records, and all 54 earlier source pairs. A failed preparation due to unsupported else-if fixture syntax emitted no timing record and is recorded truthfully. Execution-only measurements reuse dated startup/compilation baselines. Temporary array length receives correctness and cleanup acceptance; no artificial isolated throughput row is claimed.

This gate is complete. [BENCHMARKS.md](../../BENCHMARKS.md) has 57 measured workload rows, grouped by operation family.
