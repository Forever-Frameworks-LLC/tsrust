# Array.from

Array.from accepts arrays, strings, typed Sets and typed Maps. Without a mapper it
creates an independent owned array. Set values retain insertion order, and Map
items are fixed key/value tuples in insertion order. Owned descendants are copied,
so native copies do not preserve JavaScript shallow object aliases.

With a mapper, each input is borrowed and the returned value transfers into the
result. Map tuples can borrow noncloneable values; there is no intermediate
entries() snapshot. The mapper receives exactly its item and zero-based int index.
Normal defaults and rest packs apply; no third collection position is supplied.
The source expression evaluates once. Checked callback failures stop traversal
and propagate through surrounding catch/finally.

```tsrust
const values = new Map<string, int>([["a", 2], ["b", 3]]);
const labels = Array.from(values, ([key, value], index) => {
    return key + String(value + index);
});
console.log(labels.join("|")); // a2|b4
```

The source and its owned items remain readonly during mapping. Mutating the
receiver or moving a borrowed tuple/value out of the mapper is rejected. Explicit
clone(item) creates independent cloneable ownership. Unmapped source descendants
must support copying. Unsupported iterator objects, array-like records, spread
arguments and thisArg are rejected. Strings split into Unicode scalar values;
ordinary native string lengths continue to count UTF-8 bytes.
