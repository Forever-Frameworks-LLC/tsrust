# Grouping values with Map.groupBy

`Map.groupBy(source, callback)` creates an insertion-ordered Map of independently
owned arrays. The callback receives a readonly element and its zero-based int
position. Its key determines the group. Groups retain first-key order, and each
array retains source order. Empty input creates no groups and calls no callback.

Sources can be dense arrays, strings, Sets or Maps. Strings supply Unicode scalar
character strings, Sets supply keys, and Maps supply key/value tuples. A Map key
must be a supported scalar, enum or compatible union. Number keys use SameValueZero:
NaN keys group together, and positive/negative zero are the same key. Object identity
keys and arbitrary iterator objects are unsupported.

```tsrust
function main() {
    const groups = Map.groupBy([4, 1, 6, 3], (value: int): int => {
        return value % 2;
    });
    const even = groups.get(0);
    if (even !== undefined) { console.log(even.join(",")); } // 4,6
}
```

The source expression evaluates once before the callback expression. The callback
runs once for each element and may mutate unrelated captured state. It cannot
mutate or consume the source or its owning elements. Callback defaults and compatible
rest arrays follow the two supplied positions; extra required positions and consuming
callbacks are rejected. A callback failure stops traversal and propagates through
ordinary catch/finally completion.

Grouping explicitly copies values into independent output arrays. The source stays
usable and later source mutation cannot change copied owning values. Noncloneable
values are rejected. This differs from JavaScript, which retains object references
inside groups. Returning an owning borrowed element as a key requires `clone()` or
a freshly constructed key; ordinary bindings and access never implicitly clone.
String positions count scalar characters, while native string length/indexing retain
UTF-8 byte behavior. Exactly two arguments are required; spread arguments are not
supported for this static operation.
