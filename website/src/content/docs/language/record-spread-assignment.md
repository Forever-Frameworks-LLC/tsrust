---
title: Record spread and pattern assignment
description: Copy records, use shorthand fields and assign patterns to existing places.
---

Each complete example below runs separately as `main.tsr` with `tsrust run main.tsr`
after [native setup](/tsrust/tools/rust-setup/). Examples with `strictEqual` finish
without output; console examples show their output in comments.

## Object spread copies and merges

`const copied = { ...row }` creates an independent record with the source's fields. Sources must be fixed records whose values can be cloned; array, string, primitive, nullable and union sources are unsupported. Opaque native resources cannot be projected with spread or record rest. 

### Override fields in order

Ordinary fields may surround multiple spreads. Parts evaluate once, left to right. Later values replace earlier values of the same name, while each key keeps its first insertion position. 

An annotation checks the final fields and types, including supported union widening. Without an annotation, the final fixed shape is inferred. Empty records and inline record sources are supported.

```tsrust
import { strictEqual } from "std:testing";

type Row = {
  name: string;
  count: int;
};

function main() {
  const row: Row = {
    count: 2, name: "ada"
  };
  const copied = {
    ...row, count: 3
  };
  strictEqual(copied.count, 3);
  strictEqual(row.count, 2);
  strictEqual(Object.keys(copied).join(","), "count,name");
}
```

### Source access

A spread source is borrowed while copying; copying finishes before evaluating the next part. Sources remain available, including readonly parameters. A partially transferred source cannot be copied whole. 

Nested owners copy independently under native clone semantics; JavaScript shares nested object references. Ordinary fields retain their usual access rules and do not gain implicit cloning.

## Shorthand record properties

In a record literal, a name alone stands for a field of that name with the value of that variable: `{ x, y, label }` means `{ x: x, y: y, label: label }`. The variable is used under the ordinary rules, so an owned value moves into the record.

```tsrust
import { strictEqual } from "std:testing";

type Point = {
  x: int;
  y: int;
  label: string;
};

function main() {
  const x = 3, y = 4, label = "p".repeat(2);
  const point: Point = {
    x, y, label
  };
  strictEqual(point.x + point.y, 7);
  strictEqual(point.label, "pp");
}
```

## Assignment to existing places

A pattern on the left of `=` assigns to names and places that already exist. No new name is declared.

```tsrust
import { strictEqual } from "std:testing";

type Point = {
  x: int;
  y: int;
  label: string;
};

function main() {
  let a = "x".repeat(1), b = "y".repeat(1);
  [a, b] = [b, a];
  strictEqual(a + b, "yx");
  const point: Point = {
    x: 1, y: 2, label: "p"
  };
  let x = 0, text = "";
  ({ x, label: text } = point);
  strictEqual(x, 1);
  strictEqual(text, "p");
}
```

- A target is anything a plain assignment accepts: a `let` name, a record field such as `point.x`, or an array element such as `values[0]`. A `const` name is rejected.
- A record pattern at the start of a statement needs parentheses: `({ x, y } = point);`. In a record pattern, `{ x }` assigns the field `x` to the name `x`, and `{ label: text }` assigns the field `label` to `text`.
- Patterns nest, and an array pattern may end with a rest target: `[first, ...rest] = values;` assigns an independent copy of the remaining elements.
- Default values and record rest targets are not available in an assignment. Declare with `const` or `let` to use them.

### Evaluation and ownership

A computed right side is evaluated once. A named source is read in place as targets are assigned in order.

When both sides are written arrays without spread, each right-side value is held first and then assigned in order, so `[a, b] = [b, a]` swaps two values without building an array. Owned values move through the swap; nothing is cloned. A written array with fewer values than targets is a `bounds` error.

For any other right side, each target takes its position or field under the ordinary rules: copied values are copied, a tuple or record source gives up the owned positions that are taken, and an owned array element cannot be taken by position. A named source keeps the positions that are not taken.

### Use assignment as a statement

A destructuring assignment is a statement. It can also be one of the update expressions of a classic `for` loop: `for (let i = 0; i < n; [a, b] = [b, a + b], i++)`. It has no value, so `const t = ([a, b] = [b, a]);` is an error.

## Related guides

- [Destructure values](/tsrust/language/destructuring/)
- [Pattern defaults and rest](/tsrust/language/pattern-defaults-rest/)
