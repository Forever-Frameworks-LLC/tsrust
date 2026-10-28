---
title: Use assignment values
description: Read the committed value of an assignment and understand when ownership moves.
---

An ordinary assignment (`=`) is an expression: it stores a value and lets the
surrounding expression use that value. The result's ownership depends on its type.

## Store and use a value

After [setting up native execution](/tsrust/tools/rust-setup/), save this as
`main.tsr` and run `tsrust run main.tsr`. All assertions pass without output.

```tsrust
import { strictEqual } from "std:testing";

type Point = {
  x: int;
  label: string;
};

function main() {
  let count = 0;
  let other = 0;
  count = other = 4;
  strictEqual(count + other, 8);

  let text = "a";
  strictEqual((text = "xyz").length, 3);
  strictEqual(text, "xyz");

  let point: Point = {
    x: 1, label: "p"
  };
  strictEqual((point = {
    x: 5, label: "q"
  }).x, 5);
  strictEqual((point.label = "longer").length, 6);

  let current = "";
  let rounds = 0;
  while ((current = rounds < 2 ? "go" : "") !== "") {
    rounds += 1;
  }

  strictEqual(rounds, 2);
}
```

Assignments associate from right to left. In `count = other = 4`, the inner
assignment stores `4` in `other`; the outer assignment stores its result in `count`.
Parentheses make an assignment's role clear inside a larger expression.

## Copy scalar results

For `int`, `number` and `boolean`, the assignment result is a copy of the committed
value. Chaining assignments leaves each scalar target usable. Scalar array
elements can also supply assignment values.

## Read an owned target in place

For a string, array or record, an assignment value refers to the target after the
store. `(text = "xyz").length` stores the string, then reads its length. It does
not create a second owned string.

A read-only member access, a non-consuming method call or a read-only parameter
leaves the target usable. Consuming uses still transfer ownership. Binding the result to another variable moves ownership,
as reading the target in an owning context normally would.

This complete example prints `new`:

```tsrust
function main() {
  let text = "old";
  const moved = (text = "new");
  console.log(moved);
}
```

Using `text` after this move is a `transferred` error. Similarly,
`first = second = "same"` leaves only `first` usable. To retain both strings,
assign each variable separately or explicitly clone the value.

## Choose a supported target

An owned assignment value needs a writable variable or a record field with a fixed
path, such as `holder.point.label`. An owned array element or other computed place
cannot supply an assignment value. Store it in a statement and read it afterwards:

```tsrust
function main() {
  const words = ["short"];
  words[0] = "longer";
  console.log(words[0].length);
}
```

This prints `6`. Writing `console.log((words[0] = "longer").length)` instead is
unsupported. Ordinary target permissions and type checks still apply: an
assignment value does not make a `const` binding reassignable.

## Distinguish replacement from compound assignment

These rules describe ordinary `=` replacement. Do not infer that every compound
operation returns an owned result. In particular, string `+=` can be used when its
result is discarded, but cannot provide an owning value for another binding.
See [comma expressions](/tsrust/language/comma-expressions/) for ordered discarded
operations and selected results.
