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

Public integration remains pending for OrderedMap. OrderedSet and Heap source
integration follows below. Backend checks alone are not family acceptance.

Internal native identities now retain ordered collection type arguments without
exposing implementation fields. OrderedMap has separate key/value parameters;
OrderedSet and Heap each have one element parameter. Specialization validates the
ordered key restriction, including nullable/union and object rejection. Focused
metadata checks preserve ByteView as a map value and reject wrong arity without
changing the record arena. OrderedMap remains an internal identity;
OrderedSet and Heap are now module exports.
Metadata evidence: `tsrust-s08-ordered-metadata-acceptance.json` in SQLite.

## Heap source integration

`std:collections` now exports `Heap<T>` with alias-aware construction, size, push,
peek, pop and clear. The compiler checks scalar/enum ordering constraints and
receiver ownership. String reads borrow; pops transfer values. Explicit cloning
creates independent storage. Borrowed iteration visits all elements without removal; order is unspecified.

Native witnesses cover empty results, duplicates, independent clones, owned strings,
boolean and numeric enums, NaN/infinity/signed-zero boundaries, invalid element
shapes and mutation/transfer errors. Checked and fallback editor indexes resolve
aliased methods, typed arguments, results and size. Constructor and five member
catalog examples execute after format/idempotence checks.

The `native_heap` workload compares nine inserts, peek, independent clone and drain
against an explicitly sorted Node array. An independent arithmetic oracle checks
sizes, maximum and drain sum, including modulo wrap boundaries. Timing remains
pending. Evidence: `tsrust-s08-heap-acceptance.json` in SQLite.

Heap borrowed iteration now has native witnesses for scalar and owned string
payloads, empty traversal, early return through finally, and rejection of mutation
or transfer during traversal. Checked and fallback editor indexes resolve aliased
loop element types. The matched heap workload independently observes traversal and
removal sums. No timing run was performed. Evidence:
`tsrust-s08-heap-iteration-acceptance.json` in SQLite.

Call and array spread now support Deque and Heap. The previous checker rejected
these sources with `spread-source`; it did not permit invalid native code. Scalar
spread copies values, readonly arguments borrow owned elements, and owning arguments
transfer the collection. Mutable Deque elements use exclusive borrows; mutable Heap
elements are rejected to preserve ordering. Heap spread order remains unspecified.
Array and call results preserve byte-view owners, including returned fixed and
rest arguments. Focused checks cover native output, arity failures, left-to-right
scalar snapshots, invalid transfers, conflicting borrows, Heap mutation rejection,
and checked/fallback editor hover. Thirteen catalog examples execute. The matched
heap workload adds call/array spread observations; five native/Node cases pass.
Evidence: `tsrust-s08-collection-spread-acceptance.json` in SQLite. Timing remains
pending.

A separate nested payload witness exposed a native emission defect in
`nested.front()!.length` for `Deque<int[]>`: a scalar observation selected an
owning union pattern despite borrowing the array payload. Non-null projection now
uses readonly access for this observation. Focused execution covers front, back,
indexed reads and readonly parameters; data-union regression coverage is retained.
Evidence: `tsrust-s08-deque-projection-acceptance.json` in SQLite.

Readonly nested-element indexing now recognizes that `values.front()![0]` borrows
the stored array. Scalar elements copy and owned elements remain borrowed. Focused
checks reject mutation, transfer of owned elements, invalidating a retained element
borrow, and receiver mutation while computing an index. The matched deep-clone
workload observes a nested scalar through the readonly front record before cloning.
Catalog examples exercise this syntax. Evidence:
`tsrust-s08-deque-index-acceptance.json` in SQLite; timing remains pending.


## OrderedSet source integration

`std:collections` exports `OrderedSet<T>` with explicit scalar/enum type arguments,
no-argument construction, add, has, delete, clear and readonly size. Method results
match existing Set: add/clear return void, has/delete return boolean. Duplicate
keys follow the ordered equality contract above. Iteration and spread visit keys
in ascending order; owned string iteration borrows storage. Deep cloning creates
independent storage. Mutable key access is rejected.

String queries borrow their arguments through BTreeSet's borrowed str lookup;
there is no implicit string copy or linear scan. Scalar/enum queries use copied
comparison keys. Native checks cover aliases, duplicate keys, Unicode scalar order,
NaNs, signed zero, infinities, booleans, enums, independent clones and invalid
ownership. Checked/fallback editor help and executable catalog examples agree.

The matched `native_ordered_set` workload compares nine unique inserts plus a
duplicate, ordered traversal, cloning, lookup and deletion against Node Set with
explicit numeric sorting. An independent arithmetic oracle observes a position-
weighted traversal sum and independent sizes, including modulo-wrap boundaries.
Thirty-five collection tests, ten call-spread tests, three native identity tests,
typed editor help and twenty executable catalog examples pass. The workload passes
five correctness cases in each engine. Evidence:
`tsrust-s08-ordered-set-acceptance.json` in SQLite. No timing run was performed.
OrderedMap and remaining family integration still precede S08 regression and
measurement acceptance.

## OrderedMap integration foundation

Internal OrderedMap specializations now use shared map analysis while retaining
their native family identity. Value specialization and scoped map joins cannot
collapse an ordered map into an ordinary Map with identical key/value types.
Resolved generic arguments and diagnostic names preserve that identity.

The tree backend supports borrowed string queries without requiring Clone, copied
scalar queries with canonical number equality, borrowed source traversal and
consuming entry/value extraction. Native witnesses verify view pointer retention,
mutable value access, selected-value transfer and exact disposal of other values
at optimization levels 0 and 3. Four rejection witnesses cover invalidating reads,
invalidating traversal and escaping a map borrow. Thirty-seven collection tests
and nine focused library checks pass. Evidence:
`tsrust-s08-ordered-map-foundation-acceptance.json` in SQLite.

At the foundation checkpoint OrderedMap remained internal. Borrowed entries represent keys and values occupying
separate tree fields, including passing complete entries to functions. They must
not introduce implicit key copies,
shared ownership or erased lifetimes. Editor/catalog and matched workloads follow
that integration; no OrderedMap timing or public acceptance is claimed here.

The emitter now selects tree storage for internal OrderedMap identities. Set/get,
has/delete, clear/size, clone, sorted keys/values/entries snapshots and forEach use
that backend. String queries borrow; a get on a consumed temporary transfers the
selected owned value. A formatted native witness checks these paths, duplicate
replacement, independent clone mutation and number-key equality. The witness
selects internal identities after ordinary Map analysis; it does not establish
public OrderedMap construction, complete ownership integration or editor support.
The ordinary collection regression suite remains green. Evidence:
`tsrust-s08-ordered-map-emission-acceptance.json` in SQLite. That checkpoint did
not include borrowed for-of entries or complete callback/view payload integration.

Internal OrderedMap for-of now builds readonly record views over the tree's
separate key and value references. It reuses record borrowing bridges for complete
entry parameters, nested record fields, captured reads and explicit deep clones.
The native witness checks sorted traversal, direct reads, readonly helper calls,
nested fields, independent cloned data and early return through finally. Iteration
views acquire their field borrows inside each loop iteration, not the function
prologue. Ordinary Map and array entries in a borrowing family use the same field
transport. No implicit key/value copies or shared owners are introduced.

Evidence: `tsrust-s08-ordered-map-entry-acceptance.json` in SQLite.

ByteView entry payloads now retain named input lifetimes in readonly record bounds,
including closure parameters and helpers returning a subview. Opaque native values
remain leaves of a borrowed record; they do not require nonexistent record-field
traits. A second native witness stores distinct callbacks capturing a view, visits
them in key order, reads by key and verifies the surviving callback after deletion.
Callback storage transitions consume and rebuild tree entries without cloning keys
or payloads. Evidence: `tsrust-s08-ordered-map-payload-acceptance.json` in SQLite.

Recursive records now form whole borrowed fields within entry views. Their
existing recursive representation and shared lifetime remain intact instead of
being split into independently typed field parameters. Native witnesses cover
nullable view chains, array-recursive view trees, readonly whole-entry helpers,
and independent clones of both entries and selected owned recursive values.
Evidence: `tsrust-s08-ordered-map-recursive-acceptance.json` in SQLite.

The source now exports OrderedMap with explicit key/value types and no constructor
arguments. Checked and fallback editor help includes aliases, typed entries and all
ten members. Thirty-one collection catalog examples execute successfully. Existing
OrderedMap backend witnesses now run through public imports and project analysis.
Callback storage epochs recognize native constructors. Copied map keys can also
appear in the value expression without creating a false ownership conflict.

The matched `native_ordered_map` workload passes five untimed correctness cases in
each engine, using an independent arithmetic oracle for sorted traversal and
replacement. Source acceptance is recorded separately from full implementation
acceptance. Conversion to ordinary Map preserves source keys and independent
storage; its focused regression exercises owned string keys and values.

Public acceptance remains pending broader spread, parameter and branch
integration. Installed consumers remain at b53e9065 until those paths are accepted.
No OrderedMap timing or S08 milestone acceptance is claimed. Source integration
evidence: `tsrust-s08-ordered-map-public-integration-acceptance.json` in SQLite.
The next milestone after S08 remains project-wide, multiple-error diagnostics;
see [project diagnostics](project-diagnostics-plan.md).

OrderedMap Array.from mapping now constructs borrowed entry views over tree storage.
Mapping retains key order and zero-based indices, supports records containing
noncloneable ByteView values, and rejects receiver mutation. The no-mapper form
creates an independent snapshot only for cloneable entries. Closure signatures
accept borrowed record transports even when the entry has no native-view payload.
The same adapter serves for-of traversal. Evidence:
`tsrust-s08-ordered-map-array-from-acceptance.json` in SQLite.

Owning OrderedMap spread now transfers entries in ascending key order into array
literals and owning rest parameters. The adapter unwraps tree keys and constructs
entry records directly, without cloning keys or payloads. ByteView entries keep
their real owner lifetimes. Reusing a consumed map or moving a live view owner is
rejected. Evidence: `tsrust-s08-ordered-map-owned-spread-acceptance.json` in SQLite.
Readonly spread was excluded from the owning-spread checkpoint; subsequent
carrier work is recorded below.

Readonly spread now stages entry views and transports their borrowed fields through
fixed argument slots. Slots can hold ordinary owned records, ordinary references,
or tree entry fields, so calls may mix explicit tuple arguments with OrderedMap
spread in either order. The shared record bridge reads every form without cloning
or erased lifetimes. View payloads retain their owners, and ordinary tuple-array
spread remains supported in the same borrowing family. The collection and
call-spread regression targets pass. Mixed union sources and broader branch integration still require focused coverage
before public acceptance. Optional parameters and returned subviews are covered by
the subsequent checkpoint below. Evidence: `tsrust-s08-ordered-map-read-spread-acceptance.json` in SQLite.

Optional readonly tuple parameters now accept empty or one-entry OrderedMap spread.
Nullable record signatures use the existing borrowing family, and absent arguments
name a concrete payload type when Rust cannot infer it from a value. Spread loan
transport adds the entry value field after selecting a Map storage element.
Consequently a returned explicit subview can outlive a local map while retaining
its original byte owner. The same provenance repair applies to ordinary Map.
Owner transfer with a live returned view remains rejected. Evidence:
`tsrust-s08-ordered-map-optional-projection-acceptance.json` in SQLite.
