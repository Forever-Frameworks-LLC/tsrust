# Milestone 85: nullable flow narrowing and payload projection

Status: complete. Focused implementation and documentation, the retained checkpoint 86 regression, and matched runtime/resource comparisons are accepted.

## Supported guard scope

Supports strict comparison against null or undefined with `===` and `!==`, in either operand order. Facts apply to stable named locals, parameters and direct named record-field paths. A comparison removes or selects only the tested alternative: excluding null from `T | null | undefined` does not also exclude undefined. Facts compose through nested conditions, logical negation, short-circuit boolean operators, conditional expressions, loop bodies and surviving early-return paths.

Calls and dynamic indexed expressions do not establish a reusable place identity. Bind their result once to a local before narrowing. Truthiness, loose equality, typeof guards, nullable callable values, nullable array elements, optional chaining and nullish coalescing remain separate work.

## Storage and ownership

Declared types and physical Rust tagged storage remain unchanged. A refinement supplies a checked view of the active payload; it does not silently replace a binding's storage type. Generates ordinary Rust matches for copied, borrowed, mutable or transferred payloads, with no unsafe code, hidden owned clone or shared mutable graph.

Writes and transfers invalidate overlapping facts. Calls must account for mutable or consumed roots and callback effects. Read-only observations may preserve facts only when their contracts prove this safe. Closures must prove their own guards rather than inherit facts that mutable captures could invalidate.

Supports borrowed and mutable payload operations, whole-payload moves, explicit clone and original-tag assignment with sound branch availability joins. Moving an owned subfield from a projected nullable record requires moving the whole payload into a local first; otherwise report a precise diagnostic until the compiler supports that partial-move representation. Do not discard sibling fields or conceal their ownership.

## Focused acceptance

Four retained compiler groups verify guard composition and scalar bits; borrowed/mutable and transferred owned payloads; stale facts, loans, unsupported guards and depth limits; and allocation, lifetime and normal/unwind disposal. Generated native witnesses execute at optimization levels 0 and 3.

Editor completion, signature help and hover must agree with supported refined views and stop exposing payload methods when facts are invalidated. Expose checked projection spans and canonical view types from compiler analysis; cache these with the editor source index for each document revision. Do not implement a second flow checker or recompile on every editor request. Incomplete completion may use a narrowly reconstructed compiler-checked receiver probe with precise span mapping; remain conservative when that probe cannot establish a valid view. Verify concise language examples with the current compiler and native execution. Any new built-in method or property still requires complete `builtins.json` documentation before acceptance. Preserve unrelated editor quick fixes and formatting support.

Register equivalent source workloads with exact retained outputs and disclosed selection, guard, operation and retention costs. Record pending measurements separately. Keep existing measured report rows unchanged until checkpoint 86.

## Source comparison registration

Six matched workloads are registered, covering int, Number, boolean, String, array and record payload operations after strict guards. Focused source checks pass 126 cases, 22 literal hands, 118 negative guards, 13 executed semantic mutants and a four-line source-order example. All 532 previous manifest objects and 514 measured report rows remain unchanged. Current source registration totals 538; compiler/editor/native acceptance is complete; performance measurements are accepted at checkpoint 86.

## Verified edge cases

- After excluding null from `string | null | undefined`, contextual `clone(value)` into `string | undefined` must preserve the source and the excluded alternative. Explicit clone authorizes copying the payload; ordinary borrowed observations must not reconstruct an owning tag.
- Guarded int/Number compound assignment and increment must update the payload through the existing physical tag. Plain assignment of an absent alternative invalidates the proof.
- Mutating a projected array or a field inside a projected record cannot by itself change the outer tag. Preserve its presence proof while invalidating any nullable field actually replaced. Account separately for captured calls that can replace the enclosing tag.
- A guard on the left of a logical expression must not survive a right-side write to the same tag merely because the checker later revisits the original condition syntax.
- Read-only argument projection must retain its loan through later arguments. A subsequent argument cannot mutate or transfer the same owner while the borrowed payload is live.

Focused acceptance verifies these cases. Passed groups and original failed attempts are retained with their executed source identities; the complete checkpoint regression is accepted with retained passes and targeted repairs.

## Accepted evidence

[Focused compiler, editor, native example and matched source evidence](../../benchmarks/runtime/diagnostics/2026-10-03T11-59-10-000Z-nullable-flow-narrowing-implementation.json) preserves all actual attempts, repaired source snapshots and observed output. Assignment evaluates the right-hand side using incoming facts, then invalidates the replaced tag. Copying an absent alternative from owned storage borrows the original owner; present and absent source reuse is verified. Editor inferred field-chain types use cached compiler projections.

This gate adds no built-in method or property. The changed `ArrayConstructor.isArray` catalog entry retains its complete contract; both executable catalog examples produce their four expected lines with the current compiler. Seven additional language/example lines verify guarded payload operations. Existing editor quick fixes, formatting and CLI output changes are preserved.

## Checkpoint 86 measurements

[Completed checkpoint evidence](../../benchmarks/runtime/diagnostics/2026-10-03T14-02-43-000Z-checkpoint86-completed-proof.json) accepts 806 named checks on the tested revision, retaining successful targets and repairing failures without a full restart. [Runtime/resource evidence](../../benchmarks/runtime/diagnostics/2026-10-03T13-57-02-000Z-checkpoint86-runtime-proof.json) accepts 6 comparisons for this milestone across eight runtimes. Native leads compiled Node on 4/6 and the fastest scripting runtime on 4/6. Results measure complete setup, selection, operation and retained output; they do not establish isolated method latency or allocation parity.

Across gates 82–86, forty new rows and one existing-row refresh produce 30,504 timed batches and 123 native profiles, all with zero final live bytes. The report now contains 554 individually dated measured rows. The other 513 report sections, nine primitive memory tables and auxiliary baselines remain byte-exact. Concurrent project/editor revisions outside the tested checkpoint are explicitly excluded from its regression claim.
