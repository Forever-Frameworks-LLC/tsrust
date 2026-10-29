---
title: "Building arrays"
description: "Create arrays with spread and Array.from, with explicit ownership of source elements."
---

An array literal can spread elements from arrays, strings and collections into a new owning array. `Array.from` copies or maps a source into a new array. Scalar elements are copied. An owning spread transfers strings, records, nested arrays and stored functions out of its array source, which cannot be used afterwards; string and scalar collection sources stay available. `Array.from(source)` builds an independent copy, and `Array.from(source, mapper)` lends each element to the mapper without transferring it. Array methods are listed in the [arrays reference](../../reference/builtins/arrays/).

To run an example, save it as `main.tsr` after [native setup](../../tools/rust-setup/) and run `tsrust run main.tsr`. Examples that use `strictEqual` finish without output when their assertions pass; others show expected output in comments.

## Array spread

Use `...values` inside an array literal to append every source element in order.
Ordinary elements can appear before, between and after spreads. Each source must
be a supported dense array, string, Set, Map, Deque, Heap, OrderedSet or OrderedMap. The result owns its elements and can be
mutated independently.

```tsrust
function main() {
  const source = [1, 2];
  const values = [0, ...source, 3];
  console.log(values.join("|"), "length:", source.length);
// prints 0|1|2|3 length: 2
}
```

Each segment evaluates once, from left to right. A spread captures its elements
before the next segment evaluates. Later mutation of a scalar source therefore
cannot change elements already added to the result. Empty spreads add no elements;
the compiler still evaluates their source expression.

### Copy or transfer elements

Scalar elements are copied. Owned strings, records, nested arrays and stored
function environments transfer into the result. The source of an owning spread
cannot be used afterward; borrowed owners require explicit cloning before transfer.
Unlike JavaScript reference sharing, this transfer does not leave two mutable
owners for the same element.

```tsrust
function main() {
  const text = ["a", "b"];
  const moved = [...text, "c"];
  console.log(moved.join("|")); // prints a|b|c
const empty: int[] = [...[]];
  console.log(empty.length); // prints 0
}
```

### Choose the result element type

The result must have one supported declared element type. Context supplies the
empty element type and callable storage contract where needed. `number` context
accepts integer literals; typed `int` values do not implicitly become `number` values.
Homogeneous `null[]` and `undefined[]` and declared union elements work.

Sparse slots and arbitrary iterator objects are unsupported. Narrow
nullable array sources before spreading them. Existing array rank and native
capacity limits apply. Allocation can fail.

Callable array spreads preserve their invocation contracts and captured owners.
Reusable stored functions can be invoked through the resulting array. Extract a
Once function with `remove` before invoking it; existing single-call restrictions
still apply. Object spread and tuple types follow their own checked contracts.

### String and collection sources

String spread constructs one owning string for each Unicode scalar character.
The source remains available. Supplementary characters stay whole; combining
marks remain separate elements. Indexing and length on each character string
retain the native UTF-8 byte rules.

```tsrust
function main() {
  const source = String("Aé🙂");
  const characters = [...source];
  console.log(characters.join("|"), source);
// prints A|é|🙂 Aé🙂
const keys = new Set<int>([3, 1, 3]);
  console.log([...keys].join(","), keys.size);
// prints 3,1 2
}
```

Sets contribute values in insertion order. Scalar keys copy, so the source can be
used or changed after the spread is captured. Owned keys transfer and make the
source unavailable afterward. A readonly Set of owning keys cannot transfer.

Maps contribute stored key/value tuple owners in insertion order. The source
transfers even when both tuple fields are scalars, matching ordinary record-array
spread. Noncloneable values transfer without copying. Readonly Maps cannot
transfer; use explicit `Array.from(map)` when the values support copying, or map
borrowed inputs with `Array.from(map, mapper)`.

An expected array type supplies types for empty `new Set()` or `new Map()` spread
sources. Map context must provide a two-position tuple element. Contextual union
array elements accept compatible string/Set items; no implicit numeric widening
of typed values is added. `Array.of(...source)` follows these same rules. Function-call
spread accepts these sources under the target parameter access contracts. Arbitrary iterator objects are
not supported.

### Native collection order

`Deque`, `Heap`, `OrderedSet` and `OrderedMap` from `std:collections` are also accepted spread
sources. Deque, Heap and OrderedSet scalar elements copy; owned elements transfer.
OrderedMap entries transfer even when their fields are scalar. This example prints `1,2 2`:

```tsrust
import { Deque } from "std:collections";

function main() {
  const source = new Deque<int>();
  source.pushBack(2);
  source.pushFront(1);
  const values = [...source];
  console.log(values.join(","), source.size);
}
```

Deque contributes front-to-back sequence order. OrderedSet contributes ascending
keys; OrderedMap contributes entries in ascending key order. Heap spread order is
unspecified. Use repeated `pop()` when descending heap order is needed.
OrderedMap entries transfer like ordinary Map entries, including scalar pairs.

```tsrust
import { OrderedMap, OrderedSet } from "std:collections";

function main() {
  const keys = new OrderedSet<int>();
  keys.add(3);
  keys.add(1);
  console.log([...keys].join(","), keys.size); // 1,3 2
  const values = new OrderedMap<string, int>();
  values.set("z", 2);
  values.set("a", 1);
  const entries = [...values];
  console.log(entries[0][0], entries[1][0]); // a z
}
```

See [call spread](/tsrust/language/functions/#spread-arguments) for passing elements to fixed, optional or rest parameters.

## Array.from

`Array.from(source)` creates an independent owning array. Add a mapper as the
second argument to transform each item instead. Supported sources are dense
arrays, strings, typed Sets, typed Maps and `OrderedMap` from `std:collections`.

### Make an independent copy

Without a mapper, owned descendants are copied. Changing a copied record or nested
array does not change the source. This differs from JavaScript's shallow copying.
The source remains available, but its descendants must support copying: a
noncloneable stored callable cannot be duplicated this way.

```tsrust
type Row = {
  count: int;
};

function main() {
  const source: Row[] = [{
    count: 1
  }];
  const copy = Array.from(source);
  copy[0].count = 9;
  console.log(source[0].count, copy[0].count); // 1 9
}
```

For an owning array built by transferring elements, use
[array-literal spread](/tsrust/language/arrays/#array-spread).

### Transform items and indices

The mapper receives exactly its item and a zero-based `int` index. It may omit
unused parameters. Scalar items copy; ordinary owned items are borrowed readonly. The returned value
transfers into the result. The source expression evaluates once before traversal.
Items are visited in source order.

```tsrust
function main() {
  const values = new Map<string, int>([["a", 2], ["b", 3]]);
  const labels = Array.from(values, ([key, value], index) => {
    return key + String(value + index);
  });
  console.log(labels.join("|"), values.size); // a2|b4 2
}
```

Map tuples can borrow noncloneable values during mapping. The operation does not
need an intermediate `entries()` snapshot. Reading fields or producing a new value
is allowed; transferring a borrowed owner out of the mapper is rejected. Use
`clone(item)` when an independent copy is needed and the item supports cloning.

For these ordinary items, the source and its owned items remain readonly throughout
mapping. A mapper cannot mutate the source collection or its stored owners through
another alias. Reusable callable elements have the invocation contract below.

### Preserve collection order

Sets contribute values in insertion order; Maps contribute fixed key/value tuples
in insertion order. `OrderedMap` contributes tuples in ascending key order.
Copying leaves the original collection available.

```tsrust
import { OrderedMap } from "std:collections";

function main() {
  const source = new OrderedMap<string, int>();
  source.set("z", 2);
  source.set("a", 1);
  const copy = Array.from(source);
  const labels = Array.from(source, (entry, index) => {
    return entry[0] + String(entry[1] + index);
  });
  copy[0][1] = 9;
  console.log(labels.join("|"), source.get("a"), copy[0][1]); // a1|z3 1 9
}
```

`Deque`, `Heap` and `OrderedSet` are not direct `Array.from` sources. Their
[spread support](/tsrust/language/arrays/#native-collection-order)
has separate ownership rules; an array made with spread can then be mapped.

### Split text into Unicode scalars

String sources produce one owning string per Unicode scalar value. Supplementary
characters stay whole, while combining marks remain separate. Native string
lengths and indices still count UTF-8 bytes.

```tsrust
function main() {
  const source = String("Aé🙂");
  const characters = Array.from(source);
  const bytes = Array.from(source, (character, index) => {
    return character.length;
  });
  console.log(characters.join("|"), bytes.join(","), source.length);
  // A|é|🙂 1,2,4 7
  console.log(Array.from("").length, Array.from("é").length); // 0 2
}
```

### Use defaults and rest parameters

Normal callback defaults and rest packs apply. No third collection argument is
supplied. A required third parameter is an arity error; a defaulted third parameter
uses its default. A rest parameter collects only the remaining supplied positions.

```tsrust
function main() {
  const defaults = Array.from([4], (value: int, index: int, extra: int = 7) => {
    return value + index + extra;
  });
  const counts = Array.from([4], (value: int, ...indices: int[]) => {
    return indices.length;
  });
  console.log(defaults.join(","), counts.join(",")); // 11 1
}
```

### Handle callback failures

A checked callback failure stops traversal and propagates through the surrounding
`catch` and `finally`. Later items are not mapped.

```tsrust
function main() {
  let visited = 0;
  let finished = 0;
  try {
    Array.from([1, 2], (value: int) => {
      visited += 1;
      if (value === 1) {
        throw "stopped";
      }

      return value;
    });
  } catch (error) {
    console.log(visited); // 1
  } finally {
    finished += 1;
  }

  console.log(finished); // 1
}
```

### Invoke reusable callable elements

A concrete mapper can borrow a reusable stored callable through an explicit
`Mutable<Function<...>>` parameter and invoke it. The environment stays in the
source array. This mutable element loan prevents overlapping access to the source
array during the invocation. A borrowed once callable cannot be consumed here.

```tsrust
function main() {
  const actions: (() => int)[] = [(): int => {
    return 7;
  }, (): int => {
    return 9;
  }];
  const results = Array.from(actions, (action: Mutable<Function<int>>): int => {
    return action();
  });
  console.log(results.join(","), actions[0]()); // 7,9 7
}
```

The mapper itself must be reusable: a consuming mapper cannot run once per item.
Callable environments cannot be cloned with the unmapped form.

### Supported call shape

Supply one source and at most one mapper. Arbitrary iterator objects, array-like
records, spread arguments and JavaScript's third `thisArg` argument are unsupported.
Narrow a nullable source before passing it. Empty sources produce empty arrays and
do not invoke the mapper. Existing array element, rank and allocation limits apply.
