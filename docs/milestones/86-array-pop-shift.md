# Milestone 86: optional array removal results

Status: complete. Focused implementation and documentation, the retained checkpoint 86 regression, and matched runtime/resource comparisons are accepted.

## Behavior and native representation

Both methods take no arguments and mutate a supported dense homogeneous array. `pop()` removes its last element; `shift()` removes its first element and shifts the remaining elements toward index zero. An empty receiver returns undefined and remains empty. A nonempty receiver returns the removed value, preserving scalar bits and transferring owned elements without cloning them. The return type includes undefined, with canonical duplicate alternatives removed.

Use the existing native array representations: standard Rust Vec operations for heap storage and standard Option/take or element movement for bounded inline storage. Return values use ordinary checked nullable tags. Length-changing use prevents fixed-array lowering. Preserve element order, evaluate the receiver once, and retain the receiver for subsequent use. Nullable array receivers require a valid payload guard. This gate does not add sparse arrays, heterogeneous/nullable array elements, indexed borrowing results or optional chaining.

## Focused acceptance

Cover every currently supported dense element family and supported nested/record storage, empty and singleton receivers, repeated removal, aliases and temporary receiver evaluation. Verify native execution at optimization levels 0 and 3, negative arity/type/access diagnostics, mutable receiver contracts, argument loan conflicts and existing guarded field proofs. Verify exact Number bits, owned element pointer transfer, absence of removal-only allocations, and normal/unwind disposal. Preserve unrelated editor and compiler changes.

Add complete `pop` and `shift` entries to `crates/tsrust-lsp/src/builtins.json`: observable behavior, no-argument rules and errors, return and receiver mutation, concise expected examples, empty-receiver behavior, relevant native limits and a short ownership note. Completion, hover and signature help must agree. Verify the actual catalog examples with the current compiler and generated native execution, and include the results in gate evidence.

Register accurately matched source comparisons for the implemented element families, disclosing construction, removal and output-retention costs. Retain old source rows and measured tables. Once focused implementation, documentation and source acceptance are complete, run the single combined checkpoint 86 regression and accumulated performance comparisons. Retain passed checks and repair only failures; update `BENCHMARKS.md` from actual records. Compact closed evidence into SQLite only after exact restore verification.

## Focused evidence

[Accepted implementation evidence](../../benchmarks/runtime/diagnostics/2026-10-03T12-38-17-000Z-array-pop-shift-implementation.json) retains four compiler groups, one editor group, six exact catalog examples with eight output lines at optimization levels 0 and 3, and sixteen source comparisons with 352 correctness cases. Empty and repeated removal, receiver order, exact Number bits, owned pointer transfer, allocation and normal/unwind disposal witnesses pass. Original failed fixture attempts remain preserved. The checkpoint retains successful targets and repairs observed failures without repeating passed suites.

## Checkpoint 86 measurements

[Completed checkpoint evidence](../../benchmarks/runtime/diagnostics/2026-10-03T14-02-43-000Z-checkpoint86-completed-proof.json) accepts 806 named checks on the tested revision, retaining successful targets and repairing failures without a full restart. [Runtime/resource evidence](../../benchmarks/runtime/diagnostics/2026-10-03T13-57-02-000Z-checkpoint86-runtime-proof.json) accepts 16 comparisons for this milestone across eight runtimes. Native leads compiled Node on 2/16 and the fastest scripting runtime on 0/16. Results measure complete setup, selection, operation and retained output; they do not establish isolated method latency or allocation parity.

Across gates 82–86, forty new rows and one existing-row refresh produce 30,504 timed batches and 123 native profiles, all with zero final live bytes. The report now contains 554 individually dated measured rows. The other 513 report sections, nine primitive memory tables and auxiliary baselines remain byte-exact. Concurrent project/editor revisions outside the tested checkpoint are explicitly excluded from its regression claim.
