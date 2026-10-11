# Imported collections API audit

Status: 33 imported type/member rows reviewed. This accepts this audit slice,
not the complete globals/imports audit or a new performance milestone.

## Placement and common contracts

Keep `Deque`, `Heap`, `OrderedSet` and `OrderedMap` as explicit `std:collections`
imports, including aliases. Their ordering and ownership choices are substantial
API choices; adding global synonyms would increase namespace size without solving
a compatibility problem. Existing global Set/Map remain a separate review slice.
No public renaming or migration is proposed for these 33 rows.

Each constructor takes zero arguments and requires explicit generic arguments.
Construction produces independent owned storage; no explicit close is needed.
Wrong arity, incompatible types, unsupported keys, readonly mutation and live-borrow
invalidation are static errors. Methods do not return a recoverable allocation
error. Empty lookups/removals use `undefined`, not an exception. The exception is
user callback failure, which propagates through `forEach`.

Ordered keys are int, number, boolean, string or enum, not mixed unions or object
identity keys. Numeric zeros normalize to positive zero; NaNs compare equal and
sort after positive infinity. Strings use scalar lexicographic order without
normalization or locale collation; false sorts before true; enums use backing
values. Heap retains duplicates; OrderedSet and OrderedMap deduplicate keys.

All four types support explicit independent `clone` only when contained types
support cloning. Views remain non-cloneable and retain real owner lifetimes when
stored. Read traversal prevents receiver mutation/transfer. Passing an owned
collection by value transfers it. There is no hidden shared mutable owner.

## Constructors

| Symbol | Signature and behavior | Ownership and backend |
|---|---|---|
| `Deque` | `new Deque<T>()`; empty double-ended queue; front-to-back iteration | Owned VecDeque payloads; borrowed payloads retain owners |
| `Heap` | `new Heap<T>()`; empty max heap over ordered keys; iteration order unspecified; repeated pop descends | Owned BinaryHeap with checked key wrapper; strings borrow during traversal |
| `OrderedSet` | `new OrderedSet<T>()`; empty unique ordered keys; ascending iteration | Owned BTreeSet with checked key wrapper |
| `OrderedMap` | `new OrderedMap<K,V>()`; empty map; ascending key iteration | Owned BTreeMap; arbitrary supported value payloads, including borrowed views |

## Deque members

Here `T` is the declared element type. Owned read results borrow from a stored
receiver; scalar reads copy. Removal transfers the element, preserving any loans
inside it. Negative `at` indices do not count from the end.

| Symbol | Signature | Observable behavior and ownership |
|---|---|---|
| `size` | `int` readonly | Current element count; borrows receiver |
| `pushFront` | `(value:T):void` | Moves value to front; mutates receiver |
| `pushBack` | `(value:T):void` | Moves value to back; mutates receiver |
| `popFront` | `():T \| undefined` | Removes front, transferring payload; empty returns undefined |
| `popBack` | `():T \| undefined` | Removes back, transferring payload; empty returns undefined |
| `front` | `():T \| undefined` | Reads front without removal; empty returns undefined |
| `back` | `():T \| undefined` | Reads back without removal; empty returns undefined |
| `at` | `(index:int):T \| undefined` | Reads zero-based position; negative/out-of-range returns undefined |
| `clear` | `():void` | Releases all stored elements; mutates receiver to empty |

## Heap members

| Symbol | Signature | Observable behavior and ownership |
|---|---|---|
| `size` | `int` readonly | Counts all entries including duplicates; borrows receiver |
| `push` | `(value:T):void` | Transfers value into heap; mutates receiver |
| `peek` | `():T \| undefined` | Greatest value without removal; empty returns undefined; scalar copies/string borrows |
| `pop` | `():T \| undefined` | Removes greatest value and transfers it; empty returns undefined |
| `clear` | `():void` | Releases all entries; mutates receiver to empty |

## OrderedSet members

| Symbol | Signature | Observable behavior and ownership |
|---|---|---|
| `size` | `int` readonly | Number of unique keys; borrows receiver |
| `add` | `(value:T):void` | Transfers key; duplicate leaves size unchanged; mutates receiver |
| `has` | `(value:T):boolean` | Tests membership; borrows receiver and owned query key |
| `delete` | `(value:T):boolean` | Removes stored key if present; returns whether found; borrows query key |
| `clear` | `():void` | Releases all keys; mutates receiver to empty |

## OrderedMap members

| Symbol | Signature | Observable behavior and ownership |
|---|---|---|
| `size` | `int` readonly | Number of distinct keys; borrows receiver |
| `set` | `(key:K,value:V):void` | Transfers arguments; replaces existing value without increasing size; releases old value |
| `get` | `(key:K):V \| undefined` | Borrows query; absent returns undefined; stored owned value borrows readonly, scalar copies; consumed temporary transfers selected value and releases other entries |
| `has` | `(key:K):boolean` | Tests key presence; borrows receiver and query |
| `delete` | `(key:K):boolean` | Removes/releases entry; reports presence; borrows query, mutates receiver |
| `clear` | `():void` | Releases every entry; mutates receiver to empty |
| `keys` | `():K[]` | Independent copied/cloned keys in ascending order; receiver unchanged |
| `values` | `():V[]` | Independent deep-cloned values in key order; rejects non-cloneable payloads |
| `entries` | `():[K,V][]` | Independent deep-cloned pairs in key order; rejects non-cloneable payloads |
| `forEach` | `(callback:(value:V,key:K,source:OrderedMap<K,V>)=>Result):void` | Calls once per entry in key order; trailing callback parameters may be omitted; ignores callback result; borrows payloads/source and prevents receiver mutation/transfer; callback failures propagate |

`Result` in the callback signature is the catalog's callback return placeholder,
not a promise that the callback must return the checked Result enum.

## Source reconciliation and dependencies

Anchors below are under `crates/tsrust-compiler/src/`:

- `native_types.rs`: all four opaque identities, generic parameters and conditional
  independent-clone capability; thread transfer is not declared.
- `analysis.rs::native_collection_constructor`: zero-argument construction and
  native identity validation. Import aliases retain identity.
- `stdlib/collections/check.rs`, `heap_check.rs`, `ordered_set_check.rs`: exhaustive
  recognized members, argument counts/types, mutation modes and borrowed results.
  Their member sets agree with the corresponding catalog groups.
- `maps.rs::map_method_type`, `map_method_call`, `map_snapshot_call`: all ten
  OrderedMap members, readonly access, snapshot clone checks and callback dispatch.
- `stdlib/collections/emit.rs`, `heap_emit.rs`, `ordered_set_emit.rs`,
  `map_emit.rs::map_method_expression`: move/borrow emission, optional results,
  ordered queries and explicitly cloned snapshots.
- `stdlib/collections/deque_runtime.rs`, `ordered_runtime.rs`,
  `ordered_key_runtime.rs`: concrete storage, key comparison, normalization and
  borrowed query implementations.
- `emit.rs`: selected program record types trigger collection support. Any native
  element record includes deque support; Heap/OrderedSet/OrderedMap also include
  the ordered support family. This is family-level generated-source selection,
  not proof of per-method dependency elimination. No external collection package
  or user-supplied linker flag is required. Binary elimination is not measured here.

The 33 catalog rows in `crates/tsrust-lsp/src/builtins.json` agree with reviewed
signatures, including readonly size, void insertion, borrowed reads and snapshot
copy costs. Constructors and members remain imports/receiver operations exactly
as listed. Their executable examples are checked through the editor workspace,
formatted to stability, compiled natively and checked against expected output.

## Focused acceptance

Evidence is retained in SQLite under `tsrust-api-collections-*.log`:

- Three public ordered-collection tests cover construction, aliases, sorted values,
  query access, family separation and invalid keys/ownership.
- Focused heap and deque tests cover move/borrow behavior, empty/bounds results,
  ordering and invalidation rejection.
- The editor catalog test executes examples for all 33 rows.
- Whole-project formatting verifies token preservation and idempotence before the
  checkpoint. No new benchmark timing or full-family regression is claimed.
