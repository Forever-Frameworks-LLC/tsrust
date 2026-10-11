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

Initial callback-record integration covers borrowed captures, distinct callback
bodies, view captures, traversal and removal. A returned consuming callback that
moves a view retains its parameter owner. Escaping callbacks that borrow a local
view wrapper are rejected before native compilation. Nested callback invocation
also retains its result type when selecting native argument alternatives.

Broader callback storage integration remains pending: nested containers, parameter
stores, branch joins and direct callback elements need dedicated acceptance.
A consuming integer callback reading a captured view remains rejected by the
existing callback capture rules. Conditional cloning now follows the complete element structure, including nested
records and recursive view rejection. Native execution verifies independent nested
array mutation and empty queues. Thread-transfer capabilities remain pending. Initial methods do not establish
complete collection acceptance. S08 remains open and no deque timing has been
performed.

The matched `native_byte_views_deque_callbacks` workload keeps three callback
origins through indexed reads, traversal, removal and clearing. Five native and
Node-after-tsc cases pass with an independent oracle; no timing was performed.
The original diagnostics and focused acceptance are stored in SQLite as
`tsrust-s08-deque-callback-acceptance.json`.

The `native_deque_clone` workload compares explicit deep deque cloning against
Node arrays with explicit record/array copies, then mutates the copied payload.
Its independent arithmetic oracle observes both originals and copies. Acceptance
evidence is stored as `tsrust-s08-deque-clone-acceptance.json`; timing remains pending.

## Ordered collection contract and backend progress

The planned public names are `OrderedMap<K,V>`, `OrderedSet<T>` and `Heap<T>` in
`std:collections`. These are new types; existing Map/Set retain insertion order.
The first ordered key surface uses homogeneous scalar keys: int, number, string,
boolean and enums with an ordered scalar representation. Object identity keys,
record keys and user comparators are outside this initial contract.

Tree traversal is ascending by key, independent of insertion order. Integers use
numeric order, booleans place false before true, and strings use Unicode scalar
lexicographic order without locale collation or normalization. Number keys preserve
the existing collection equality rule: signed zeros compare equal and all NaNs
compare equal. Zeros are stored as positive zero; all NaNs form one key ordered
after positive infinity. No subtraction-based comparator is used. Heap is a max
heap under the same order: peek borrows the greatest value and pop transfers it;
empty results are undefined. Repeated equal heap values remain separate entries.
Heap storage order is not a sorted traversal contract.

The backend now uses BTreeMap, BTreeSet and BinaryHeap. Focused Rust witnesses at
optimization levels 0 and 3 cover 2,048 map operations against an independent
sequence model, ordered Unicode traversal, duplicate replacement, independent deep
cloning, empty behavior, exact drops, non-clonable borrowed payloads and retained
slice addresses. Number witnesses cover signed zeros, infinities, multiple NaN
representations, comparison transitivity and heap removal order. Six Rust rejection
cases cover borrowed read/iterator invalidation, non-clonable values and escaping
owners. Evidence is retained as `tsrust-s08-ordered-backend-acceptance.json`.

Public signatures, compiler integration, lifetime/callback transport, editor help,
executable catalog examples and matched workloads remain pending for these three
types. These backend checks are not source-language or family acceptance.
