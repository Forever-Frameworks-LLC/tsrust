# S08 collections

S07 is accepted on macOS. S08 is in progress; its family acceptance and measurements
remain pending. This plan preserves the six collection kinds required by the
[standard-library roadmap](../standard-library-roadmap.md).

## Scope and order

1. Audit existing Map and Set against key equality, insertion order, explicit deep
   clone, borrowed access/iteration and invalidation. Include native-owned and
   borrowed payload integration after S07, not only primitive values.
2. Implement deque storage and its public generic surface. Use Rust VecDeque;
   push/pop at both ends transfer values, reads borrow, empty pops return undefined,
   and iteration borrows the container. No implicit element clones.
3. Implement key-ordered map and set using Rust tree collections. Preserve the
   existing insertion-ordered Map and Set behavior. Specify admissible key types,
   comparison and number edge cases before implementation acceptance.
4. Implement heap using Rust BinaryHeap, with an explicit ordering contract and
   checked generic bounds. Peek borrows; pop transfers; empty results are undefined.
5. Complete all six kinds across compiler, native execution, editor, catalog,
   executable examples and matched workload support. Then perform family regression,
   measurement, native-reference review and installed-consumer acceptance.

Names, supported generic arguments and error boundaries are finalized with each
public surface. The roadmap does not require JavaScript object identity keys,
shared ownership or live JavaScript iterator objects. Borrowing iteration and
invalidation are required. Do not replace the missing collections with array
wrappers or describe existing Map/Set as full S08 acceptance.

## Initial integration findings

The assertion fixture `crates/tsrust-compiler/tests/fixtures/collections/borrowed-values.tsr`
combines map insertion/replacement/deletion, actual borrowed byte views, recursive
view records, deep cloning and independent Set order. It exposed three defects:

- Generated array-join trait registrations omitted native-view record lifetimes.
- The callback-storage deletion planner treated an untracked value Map as empty,
  dropping survivors after deleting one key. Limit this planning to tracked scoped
  callback payloads.
- Map.set did not retain native view loans from its stored value. Retain those loans
  in the collection just as array insertion does, so owner transfer is rejected.

These are focused repairs, not complete generic Map/view transport acceptance.
Returned Maps, parameter mutation, borrowed lookup, iteration, captured Maps and
views nested in payload records now have focused native and ownership witnesses in
`returned-views.tsr`. Multiple-input selection preserves independent lifetimes: the
unselected owner can end before the returned Map is read. Map payloads participate
in both source provenance and emitted lifetime enumeration.

Escaping readonly projections and recursive Map payload cycles now have focused
positive and negative witnesses. Readonly views forward the original slice lifetime;
reborrows of exclusive views retain their container borrow and reject local escape.
The `native_byte_views_maps` workload includes returned Maps, parameter insertion,
borrowed traversal and a readonly projection that outlives its intermediate Map.
Its arithmetic oracle and matched Node source are ready for family measurements.

Precise per-key loan release is deferred. The documented conservative scope rule
preserves ownership guarantees and does not prevent the six required collection
kinds from proceeding to S08 acceptance.
The current element-level loan retention is conservative across collection mutation.
No lifetime erasure or hidden owned copy is permitted to make these cases compile.

Verification lives in the `collections` integration target and nearby Map tests.
Original failing and repaired diagnostics are retained in SQLite. Timing is deferred
until the complete collection family is ready.

## Deque backend progress

The owned `VecDeque` backend has a focused native witness at optimization levels
0 and 3. A separate `Vec` sequence model checks 8,192 mixed operations, indexed
reads, both iteration directions, empty behavior and clearing. Additional checks
cover independent deep clones, exact payload destruction, moving non-clonable
payloads and preservation of borrowed slice addresses. Five Rust rejection cases
cover mutation during a read or iteration, owner movement, cloning non-clonable
views and escaping a local byte owner.

The native signature layer represents indexed type parameters, applied native types
and optional results. Substitution preserves declaration identity, validates arity
and rejects missing or cyclic arguments. Specializations retain checked element
identities without exposing implementation fields.

Deque payloads participate in carried element-owner paths, recursive view
components, environment lifetime counts and native signature rendering. Generated
record wrappers contain the backend directly. Native witnesses check ordinary and
recursive payloads at optimization levels 0 and 3, unchanged borrowed slice
addresses and rejection of a shorter-lived owner.

The initial public surface exports `Deque<T>` from `std:collections` and supports
`new Deque<T>()`, including import aliases and generic factories. Construction takes
no arguments. Empty deques of views can be returned with an independently quantified
lifetime; no byte owner or static lifetime is fabricated. Non-native record types
and other opaque native types cannot use this constructor. Constructor documentation,
its executable example, alias hover, inferred types and signature help have focused
coverage.

The initial method surface now supports size, pushFront, pushBack, popFront,
popBack, front, back, at and clear. Focused checks cover scalar and owned values,
view transport, receiver mutation conflicts, moved inputs and escaping reborrows.
Aliases have typed completion, hover and signature help, including fallback
inference. All nine catalog examples execute after formatting and idempotence
checks. A local deque containing exclusive views cannot return a readonly reborrow
of that storage; the compiler now rejects it before native emission.

The matched `native_byte_views_deque` workload lives in
`benchmarks/runtime/workloads/collections/`. Five native/Node correctness cases
pass against an independent arithmetic oracle. Runtime and allocation drivers use
manifest source paths so active workloads can be grouped by feature. Original
measurement records and their historical paths remain unchanged.

Borrowed `for...of` iteration now visits the front-to-back sequence without
removing elements. Focused checks cover owned strings, empty traversal, mutation
rejection, native view payloads and early returns through finally. Checked and
fallback editor hover resolve the element type through import aliases. The deque
workload also observes the borrowed traversal sum independently of its pop sum.

Scoped callback payload integration, conditional cloning and
thread-transfer capabilities remain pending. Initial methods do not establish
complete collection acceptance. S08 remains open and no deque timing has been
performed.
