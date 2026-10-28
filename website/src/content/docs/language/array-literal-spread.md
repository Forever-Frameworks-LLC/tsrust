---
title: Build arrays with spread
description: Combine array, string and collection elements into an owning array.
---

Use `...values` inside an array literal to append every source element in order.
Ordinary elements can appear before, between and after spreads. Each source must
be a supported dense array, string, Set, Map, Deque, Heap, OrderedSet or OrderedMap. The result owns its elements and can be
mutated independently.

After [native setup](/tsrust/tools/rust-setup/), save each complete example separately as `main.tsr` and run
`tsrust run main.tsr`. Comments show expected output.

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

## Copy or transfer elements

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

## Choose the result element type

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


## String and collection sources

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

## Native collection order

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

See [call spread](/tsrust/language/call-spread/) for passing elements to fixed, optional or rest parameters.
