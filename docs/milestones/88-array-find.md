# Milestone 88: Array.find and Array.findLast

Status: implementation, executable documentation, combined regression and 16 matched comparisons accepted at checkpoint 91.

## Behavior

`find(callback): Element | undefined` visits forward and returns the first matching element. `findLast(callback): Element | undefined` visits backward and returns the last matching element. Both stop immediately when the reusable predicate returns true, without changing receiver length or contents. Callback indices refer to the original receiver, and its third parameter observes the complete source array.

Each method takes exactly one reusable callback with a strict boolean result and no default. It may declare zero to three parameters: the element, an int index and a read-only source array. Missing or extra arguments, incompatible parameter types, truthiness coercion, once-only callbacks and consuming or mutating element/source parameters are rejected. Receiver and predicate expressions evaluate once, in that order, even for an empty receiver; an empty receiver invokes no callbacks. The predicate may mutate unrelated captured state under the existing callback contracts.

Dense homogeneous scalar, nested-array and record families use canonical optional results, with duplicate undefined alternatives removed. Undefined elements and no match both produce undefined. Number elements retain their original IEEE bits. Sparse arrays, heterogeneous or nullable elements, prototype hooks and unproved nullable receiver access remain unsupported.

## Ownership and checked views

Copyable selected elements return copied optional values. Named owned receivers produce the same scoped read-only optional views accepted for Array.at. They support whole-value observations, guarded payload reads, aliases and Read nullable consumers. Conflicting owner mutation or moves are prohibited while a view is live; its payload cannot mutate, consume, return or escape. Explicit `clone(view)` constructs an independent owning optional value. Temporary owned receivers can transfer the chosen payload without an implicit clone; unselected owners retain ordinary disposal.

Nullable receivers and selected payloads need compiler-proved strict null/undefined guards. Cached compiler projection metadata supplies editor refinements, with no separate textual flow checker. Existing findIndex/findLastIndex integer results and Array.at optional lookup remain distinct operations.

## Focused evidence

Four compiler groups and the affected literal-context supplement are accepted, covering O0/O3 content and callback order, original indices, short-circuit visits, empty receivers, canonical types, scoped views, loans, strict diagnostics, allocation and normal/unwind disposal. Current native-binding peer work is preserved.

One focused editor group verifies both signatures and parameter help, completion, inferred optional results across nine concrete type witnesses, checked guarded member access, strict predicate diagnostics and earlier array/int method contexts. All six exact catalog examples produce ten complete stdout lines at optimization levels 0 and 3, with empty stderr. The initial fixture failure for a legacy inferred findIndex hover is retained; its explicit int annotation was repaired without widening unrelated inference, and only the same group reran.

Sixteen counterpart pairs are registered across eight supported families for each method. Source checks pass 592 cases, 128 hand cells, 256 negative guards and 48 semantic mutants, with ten source-order lines. Source registration and independent source checks remain distinct from the checkpoint timing record. Accepted measurements include receiver/query and callback setup, visits, result observation encoding and complete retained output. SQLite is the canonical analytics record, while Markdown is an optional historical export. There is no isolated-method latency, allocation-parity or universal native-speed claim.

[Accepted implementation evidence](../../benchmarks/runtime/diagnostics/2026-10-03T15-50-48-000Z-array-find-implementation.json) binds the actual compiler, editor, native documentation and source records. No full regression or timing suite repeats in this gate.

[Completed checkpoint 91](../../benchmarks/runtime/diagnostics/2026-10-03T18-25-41-000Z-checkpoint91-completed-proof.json) accepts 890 distinct checks without a full restart, 68 matched native/compiled-Node comparisons and 204 fully disposed profiles. Native is faster on 34/68; there is no universal speed claim. SQLite preserves exact original records and restoration. Across history, 622 workloads are measured among 623 registered sources; the separate peer path workload remains unmeasured.
