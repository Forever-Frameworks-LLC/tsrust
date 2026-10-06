# Map collections

Map stores typed key/value entries in insertion order. Replacing an existing key
changes its value without moving its insertion position. Deleting and reinserting
a key places it last. Keys may be int, number, boolean, string, null, undefined,
enum values or unions of these types. Object, array and callable identity keys
are unsupported. Number keys use SameValueZero: NaN equals NaN and negative zero
shares the positive-zero key.

## Construction and types

`new Map<K, V>()` creates an empty Map. Its optional source is a dense array of
key/value tuples, another Map, null or undefined. Construction explicitly copies
source entries and leaves the source usable. Values must support explicit copying
when a source is supplied. Ordinary bindings and access never implicitly clone.

A Map annotation supplies context for an empty constructor. A typed tuple array
or Map source supplies inferred key/value types. Literal tuple entries infer
column types when their expressions provide type information. Use explicit types
when empty arrays, untyped record literals or ambiguous expressions need context.
Compatible source types can widen into unions; int keys do not implicitly become
number keys.

```tsrust
const counts = new Map<string, int>([["red", 1], ["blue", 2], ["red", 3]]);
console.log(counts.size, counts.get("red") ?? 0); // 2 3
const empty: Map<boolean, int> = new Map();
empty.set(true, 1);
```

## Reading and mutation

`size` is a readonly int. `get(key)` returns the stored value or undefined.
`has(key)` returns a boolean. Both operations read the receiver and borrow owned
query keys. `delete(key)` removes an entry and reports whether it existed.
`clear()` removes all entries and returns void. `set(key, value)` inserts or
replaces an entry, transfers owned inputs and returns void. It does not support
chaining. These methods require keys and values compatible with the Map type.

An owned value read from a named Map is a readonly borrow. Mutation or transfer of
the source Map is rejected while that borrow remains in scope. Scalar values copy.
A get on a temporary Map consumes the temporary and moves the selected owned value
out; other entries are disposed. Missing values still return undefined.

```tsrust
const names = new Map<int, string>();
names.set(1, String("one"));
{
    const name = names.get(1);
    console.log(name ?? "missing"); // one
}
names.clear();
console.log(names.size); // 0
```

`clone(map)` explicitly copies a cloneable Map. Maps may be stored in records,
arrays and unions, passed to generic functions and returned from functions.
Console inspection displays Map entries; String conversion and array join use
`[object Map]`. Console inspection requires inspectable key and value types.

## Independent snapshots

`keys()` returns Key[], `values()` returns Value[], and `entries()` returns
[Key, Value][]. They take no arguments. All three return independent arrays in
insertion order and leave the Map usable. Empty Maps produce empty arrays.
Replacement preserves a key's position; deleting and reinserting moves it last.
Later receiver changes do not change the snapshots.

```tsrust
const values = new Map<string, int>([["b", 2], ["a", 1]]);
const pairs = values.entries();
values.clear();
console.log(pairs[0][0], pairs[0][1]); // b 2
```

Snapshots explicitly copy owned elements, including nested owned storage. Values
must support cloning for values() and entries(); keys() needs no value copies and
works when values cannot be cloned. Snapshot arrays and their owned elements can
be mutated independently. These operations return arrays rather than live
iterators. This does not claim JavaScript shallow alias identity for owned values.

This scope includes construction, size, set, get, has, delete, clear, keys, values
and entries. Map supports borrowed for-of and callbacks. Live iterator objects remain unsupported.
Collection measurements remain pending; implementation acceptance is recorded
separately in the milestone.

## Borrowed callbacks

`map.forEach(callback)` visits entries once in insertion order and supplies
`(value, key, readonlyMap)`. Callbacks may accept zero through three supplied
arguments, including ordinary default and rest parameters. Named functions and
reusable borrowed callbacks are supported. There is no thisArg argument.

Scalar keys and values copy into callback parameters; owned inputs borrow
readonly. The receiver remains borrowed during traversal. Values do not need to
support cloning. A callback may change unrelated captured state, but cannot
mutate or transfer the receiver or move an owned key/value out.

The method returns void. Callback results are evaluated and discarded after each
call. Empty maps do not call the callback. A checked callback failure stops
iteration and reaches surrounding catch/finally.

```tsrust
const values = new Map<string, int>([["b", 2], ["a", 1]]); let text = ""; values.forEach((value, key) => { text = text + key + String(value) + "|"; }); console.log(text);
```

This prints `b2|a1|`.

## Borrowed iteration

`for (const entry of map)` visits readonly `[K, V]` entries in insertion order.
`for (const [key, value] of map)` destructures the same borrowed entry. Replacing a
value keeps its position; deleted entries are skipped. The iterable expression
runs once, and an empty Map runs no loop body.

```tsrust
const values = new Map<string, int>([["a", 1], ["b", 2], ["a", 3]]);
let text = "";
for (const [key, value] of values) {
    text = text + key + String(value) + "|";
}
console.log(text); // a3|b2|
```

Entries also work with readonly tuple parameters. Iteration borrows the Map until
the loop exits. Mutation or transfer of the Map inside the loop is rejected.
Owned keys and values cannot move out of the borrowed entry. Use an explicit
`clone(value)` when an independent cloneable value is needed. Noncloneable values can
be read without copying. Native iteration exposes stored entries without creating
JavaScript's temporary entry arrays.

## Grouping inputs

Use `Map.groupBy(source, callback)` to create insertion-ordered groups with
independently owned value arrays. See [Map.groupBy](./map-group-by.md) for source
forms, callback contracts, key equality and explicit-copy behavior.
