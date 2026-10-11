# Global Set and Map API audit

Status: 29 rows reviewed: two constructors, sixteen Set members, ten Map members
and Map.groupBy. The complete API audit remains open.

## Placement, errors and dependencies

Keep familiar global Set/Map constructors and their receiver/static methods. No
imported duplicates or public renaming are proposed. Unlike specialized ordered
collections, these spellings already identify the ordinary insertion-order
collections. Their supported semantics must remain visible in signatures and
IntelliSense: add/set return void, snapshots return arrays, and traversal borrows
rather than permitting arbitrary mutation. No compatibility claim follows from
matching JavaScript spelling.

Keys support int, number, boolean, string, null, undefined, enums and supported
unions; arrays, records and function identity keys are rejected. Number keys use
SameValueZero, including NaN and normalized positive zero. Cross-type Set algebra
and relations retain enum identity; large int/number comparisons can lose
precision. A non-Set argument, including a set-like record, is rejected for these
operations. Mixed-key unions remain checked types, never dynamically typed data.

All rows enforce static argument count/type/access rules. No recoverable allocation
Result is promised. Callback failures propagate through ordinary catch/finally;
empty traversal invokes no callback. Explicit clone and snapshot operations make
independent storage; binding/passing an owned collection transfers it. Views in
Map values retain their source loans and reject independent cloning. Storage drops
automatically; no close operation exists.

The compiler recognizes constructors through set_syntax.rs and validates shadowed
Set/Map names in sets.rs::set_constructor and maps.rs::map_constructor. Local,
pending lexical and function declarations prevent built-in construction. Static
groupBy dispatch checks Map shadowing too. This is not general user-defined class
construction. Imported aliases remain available for specialized collections.

Generated runtime dependencies are selected from checked program record metadata:
set_emit.rs::set_types includes runtime_sets.rs for Set or Map definitions;
map_emit.rs::map_types includes runtime_maps.rs for Map definitions. These are
family-level helpers, not evidence of per-member binary elimination. They use
ordered slots and hashed lookup buckets, with no extra user linker flag or external
collection package. Merely preserving global names does not add a separate API
implementation. Binary size/dead-code elimination is not measured in this slice.

## Constructors

| Symbol | Signature and behavior | Ownership |
|---|---|---|
| Set | new Set<T>(source?:T[] or Set<T> or string or null or undefined); no-source needs explicit/contextual type; string yields Unicode scalar characters; duplicates keep first position | Borrows source, clones owned keys into independent storage; null/undefined creates empty |
| Map | new Map<K,V>(source?:[K,V][] or Map<K,V> or null or undefined); no-source needs explicit/contextual types; duplicate replaces value at first position | Borrows source and clones entries; rejects non-cloneable values; null/undefined creates empty |

Both constructors also infer types from supported nonempty sources. Deleting and
reinserting a key places it last. Owned arguments to add/set transfer, unlike the
explicit independent copy requested by a source constructor.

## Set members

| Symbol | Signature | Behavior, errors and ownership |
|---|---|---|
| `Set.size` | `size: int` | Readonly distinct count; borrows receiver. |
| `Set.add` | `add(value: Element): void` | Transfers owned key; duplicate preserves position; returns void (no chaining). |
| `Set.has` | `has(value: Element): boolean` | Borrows receiver/query; reports membership. |
| `Set.delete` | `delete(value: Element): boolean` | Borrows query; mutates receiver, releases stored key; true iff present. |
| `Set.clear` | `clear(): void` | Releases all keys; receiver becomes empty. |
| `Set.values` | `values(): Element[]` | Borrows receiver; independent copied/cloned keys in insertion order. |
| `Set.keys` | `keys(): Element[]` | Same independent snapshot contract as values. |
| `Set.entries` | `entries(): [Element, Element][]` | Independent copied/cloned [key,key] tuples in insertion order. |
| `Set.forEach` | `forEach(callback: (value: Element, key: Element, source: Set<Element>) => Result): void` | Reusable callback receives value, repeated key and readonly source in insertion order; results discarded; failure propagates; receiver mutation/transfer rejected. |
| `Set.isSubsetOf` | `isSubsetOf(other: Set<OtherKey>): boolean` | Borrows both sets; true iff every receiver key occurs in other; empty receiver true. |
| `Set.isSupersetOf` | `isSupersetOf(other: Set<OtherKey>): boolean` | Borrows both sets; true iff every other key occurs in receiver; empty other true. |
| `Set.isDisjointFrom` | `isDisjointFrom(other: Set<OtherKey>): boolean` | Borrows both sets; true iff no shared key; either empty true. |
| `Set.union` | `union(other: Set<OtherKey>): Set<Element \| OtherKey>` | Independent cloned result: receiver keys then unseen other keys; joined key type. |
| `Set.intersection` | `intersection(other: Set<OtherKey>): Set<Element>` | Independent cloned shared keys; follows smaller set order, receiver on equal size; retains receiver key type. |
| `Set.difference` | `difference(other: Set<OtherKey>): Set<Element>` | Independent cloned receiver keys absent from other, in receiver order; retains receiver key type. |
| `Set.symmetricDifference` | `symmetricDifference(other: Set<OtherKey>): Set<Element \| OtherKey>` | Independent cloned unshared receiver keys then unshared other keys; joined key type. |

## Map members

| Symbol | Signature | Behavior, errors and ownership |
|---|---|---|
| `Map.size` | `size: int` | Readonly distinct key count; borrows receiver. |
| `Map.set` | `set(key: Key, value: Value): void` | Transfers owned key/value; replaces and releases old value while preserving key position; void (no chaining). |
| `Map.get` | `get(key: Key): Value \| undefined` | Borrows query; absent returns undefined. Stored receiver: scalar copy/readonly owned borrow. Consumed temporary: transfers selected owned value and releases other entries. |
| `Map.has` | `has(key: Key): boolean` | Borrows receiver/query; distinguishes present undefined from absent key. |
| `Map.delete` | `delete(key: Key): boolean` | Borrows query; removes/releases entry, returns presence; remaining order unchanged. |
| `Map.clear` | `clear(): void` | Releases all entries; receiver becomes empty. |
| `Map.keys` | `keys(): Key[]` | Independent copied/cloned keys in insertion order; receiver unchanged. |
| `Map.values` | `values(): Value[]` | Independent deep-cloned values in key order; requires cloneable values. |
| `Map.entries` | `entries(): [Key, Value][]` | Independent deep-cloned key/value tuples in key order; requires cloneable values. |
| `Map.forEach` | `forEach(callback: (value: Value, key: Key, source: Map<Key, Value>) => Result): void` | Reusable callback receives readonly value, key and source in insertion order; results discarded; failure propagates; receiver mutation/transfer rejected. |

Callback `Result` in catalog signatures denotes the callback return placeholder,
not a requirement to return the checked Result enum. forEach allows omitted
trailing parameters and compatible defaults/rest; it has no thisArg parameter.
Snapshot copying includes owned strings. Receiver mutation or transfer while its
borrowed values remain live is rejected.

## Map.groupBy

`groupBy<Element,Key>(source:Element[] | string | Set<Element> |
Map<unknown,unknown>, callback:(value:ReadOnly<Element>,index:int)=>Key):
Map<Key,Element[]>` stays a static member on global Map. Source is evaluated before
the callback expression. It borrows source elements during callback execution,
then creates independently owned grouped arrays; non-cloneable source values are
rejected. String elements are Unicode scalar strings, Set elements keys, Map
elements entry tuples. Group keys follow first occurrence; elements within each
group retain source order. Callback errors stop traversal and propagate. Empty
input produces an empty typed map. Invalid key types, mutable/consuming callback
access, spread arguments and arbitrary iterator objects are rejected.

## Source reconciliation

All anchors are under crates/tsrust-compiler/src unless otherwise stated:

- sets.rs::set_method_type/set_method_call/set_algebra_call/set_relation_call:
  recognized Set members, result typing, arity and receiver/argument loans.
- set_emit.rs::set_method_expression/set_algebra_expression/set_relation_expression:
  concrete copies, order, cross-key comparisons and mutation.
- maps.rs::map_method_type/map_method_call/map_snapshot_call: all Map members,
  clone restrictions, borrowed get and consuming temporary get.
- maps.rs::map_group_by and analysis.rs::array_transform: static dispatch,
  callback checking, source element checking and cloneable grouped payloads.
- map_emit.rs::map_method_expression/map_constructor_expression and the MapGroupBy
  target in emit.rs: concrete runtime operations and copying.
- runtime_sets.rs/runtime_maps.rs: insertion order, key equality, lookup,
  replacement, removal and release behavior.
- crates/tsrust-lsp/src/builtins.json groups globals, Set, Map and MapConstructor:
  reconciled signatures and behavior. Source constructors and snapshots explicitly
  copy; ordinary reads do not implicitly clone.

## Focused acceptance

SQLite evidence `tsrust-api-set-map-*.log` records focused constructor/method,
Set algebra/relation, grouping, ownership and editor checks. Existing documented
Set/Map fixtures run natively; editor tests reconcile examples against fixtures
and verify hover, completion and parameter help. Whole-project formatting runs
before the checkpoint. No new timing or complete family regression is claimed.
