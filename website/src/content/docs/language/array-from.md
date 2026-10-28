---
title: Copy or map sources with Array.from
description: Create an independent array or transform borrowed source elements.
---

`Array.from(source)` creates an independent owning array. Add a mapper as the
second argument to transform each item instead. Supported sources are dense
arrays, strings, typed Sets, typed Maps and `OrderedMap` from `std:collections`.

After [native setup](/tsrust/tools/rust-setup/), save each complete example
separately as `main.tsr` and run `tsrust run main.tsr`. Comments show expected output.

## Make an independent copy

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
[array-literal spread](/tsrust/language/array-literal-spread/).

## Transform items and indices

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

## Preserve collection order

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
[spread support](/tsrust/language/array-literal-spread/#native-collection-order)
has separate ownership rules; an array made with spread can then be mapped.

## Split text into Unicode scalars

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

## Use defaults and rest parameters

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

## Handle callback failures

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

## Invoke reusable callable elements

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

## Supported call shape

Supply one source and at most one mapper. Arbitrary iterator objects, array-like
records, spread arguments and JavaScript's third `thisArg` argument are unsupported.
Narrow a nullable source before passing it. Empty sources produce empty arrays and
do not invoke the mapper. Existing array element, rank and allocation limits apply.
