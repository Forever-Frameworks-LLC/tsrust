---
title: Destructure values
description: Select tuple positions, array elements and record fields with explicit ownership.
---

Each complete example below runs separately as `main.tsr` with `tsrust run main.tsr`
after [native setup](/tsrust/tools/rust-setup/). Examples with `strictEqual` finish
without output; console examples show their output in comments.

## Take positions and fields by name

A `const` or `let` declaration can name the positions of a tuple or array, or the fields of a record.

```tsrust
type Point = {
  x: int;
  y: int;
  label: string;
};

function pair(): [int, string] {
  return [7, "seven"];
}

function main() {
  const [count, word] = pair();
  console.log(count + 1, word); // 8 seven
    const point: Point = {
    x: 1, y: 2, label: "p"
  };
  const { x, label: text } = point;
  console.log(x, text); // 1 p
}
```

- `[a, b]` takes positions 0 and 1. `[, b]` skips a position.
- `{ x, y }` takes the fields `x` and `y`. `{ label: text }` takes the field `label` under the name `text`.
- Patterns nest: `const [first, [inner, flag]] = value;` and `const { from: { label } } = line;`.
- `let` makes each name reassignable.

## Ownership follows the source

Ordinary pattern positions behave like taking each position from the value by hand. Rest syntax explicitly requests a copy of the remaining array elements.

- **A copied position** (int, number, boolean) is copied. The source stays usable.
- **An owned position of an owned source** moves out. After `const { label } = point;` the field `point.label` is transferred, as after `const label = point.label;`. The other fields stay usable. Write `clone(point)` to keep the whole source.
- **A call result or other temporary** is held once, and each name takes its position from it.
- **A `ReadOnly` parameter** lends its positions. The names are readonly borrows and cannot be transferred.

This fragment uses the `Point` type from the first example:

```tsrust
function describe(point: ReadOnly<Point>): string {
    const { x, y, label } = point;
    return label + String(x + y);
}
```

## In parameters

A function or arrow parameter can be a pattern. Its names are available in the function body.

```tsrust
type Point = {
  x: int;
  y: int;
  label: string;
};

function describe({ x, y, label }: Point): string {
  return label + String(x + y);
}

function main() {
  const point: Point = {
    x: 1, y: 2, label: "p"
  };
  console.log(describe(point), point.label); // p3 p
    const pairs: [string, int][] = [["a", 1], ["b", 2]];
  console.log(pairs.map(([name, amount]) => name.repeat(amount)).join("")); // abb
}
```

A destructured parameter with no stated access is read: its names borrow the argument, and the caller keeps the value. State `Owned<Point>` to let the names move out; the call then consumes its argument.

### Take elements from an owned array parameter

An `Owned<T[]>` parameter pattern can transfer selected elements, including nested
array elements, without cloning them. The caller gives up the array. A plain
`T[]` or `ReadOnly<T[]>` parameter pattern still borrows its owned elements.

After native setup, save this complete example as `main.tsr` and run
`tsrust run main.tsr`. It prints `owner`:

```tsrust
function head([first]: Owned<string[]>): string {
  return first;
}

function main() {
  console.log(head(["owner", "extra"]));
}
```

Skipped positions and defaults keep their original positions. A rest binding
still explicitly copies the remaining elements; ordinary selected elements move.

## Arrays

In a local declaration, array positions follow indexing. A copied element is copied. An owned element is borrowed for reading, and the array cannot be changed while the name is in scope. The explicit `Owned<T[]>` parameter pattern above is different: it can transfer its selected elements.

```tsrust
function main() {
  const words = ["alpha", "be"];
  {
    const [first, second] = words;
    console.log(first, second.length); // alpha 2
    }

  words.push("gamma");
}
```

A position past the end of an array stops the program at run time, as indexing does. The same borrow applies to a plain `const word = words[0];`.

## In for...of

A loop can destructure each element. The names borrow the element for reading, as the loop variable does.

```tsrust
type Row = {
  id: int;
  name: string;
};

function main() {
  const row: Row = {
    name: "ada", id: 7
  };
  for (const [key, value] of Object.entries(row)) {
    console.log(key, String(value));
  }
    // name ada
    // id 7
    const rows: Row[] = [row];
  for (const { id, name } of rows) {
    console.log(id, name.toUpperCase());
  } // 7 ADA
}
```

Pass a borrowed name to a `ReadOnly` parameter, read it, or `clone()` it. Transferring it is a `borrowed-place` error.

The same rule now applies to a plain `const` inside a loop: `const name = item.name;` borrows the field for reading.

## In classic for loops

The initializer of a classic `for` loop can be a pattern declaration. The source expression is evaluated once, before the first condition check. The names are in scope for the condition, the update clause and the body, and end with the loop.

```tsrust
import { strictEqual } from "std:testing";

type Bounds = {
  start: int;
  end: int;
};

function bounds(): Bounds {
  return {
    start: 1, end: 4
  };
}

function main() {
  const seen: int[] = [];
  for (let { start: i, end } = bounds(); i < end; i++) {
    if (i === 2) {
      continue;
    }

    seen.push(i);
  }

  strictEqual(seen.join(","), "1,3");
}
```

Use `let` for names that the update clause or body changes. Defaults, rest and nested patterns work as in other declarations: `for (let [i, end = 3] = [0]; i < end; i++)` runs three times. `continue` runs the update clause and `break` leaves the loop as usual.

Access follows the source. A named source keeps the positions that were copied or not taken. A rest name is an independent copy, so `for (const [first, ...tail] = words; tail.length < 2; tail.push("c"))` changes only `tail`. A name that borrows an array element keeps the array loaned until the loop ends; the array can be changed again after the loop.

The names share one binding for the whole loop. They are not created again for each iteration. A name cannot repeat a name of the enclosing scope. A declaration list can mix patterns and ordinary names under one `const` or `let`
keyword. Initializers run left to right. Run this complete example separately;
it prints `3`:

```tsrust
function main() {
  let total = 0;
  for (let [i, end] = [0, 3], step = 1; i < end; i += step) {
    total += i;
  }

  console.log(total);
}
```

See [declarations](/tsrust/language/declarations/) for
initialization, scope and declaration-list errors.

## Errors

- A tuple position outside the declared tuple or an undeclared record field without a default is a `bounds` or `type` error respectively.
- Transferring a borrowed name is a `borrowed-place` error. Changing or transferring the source while a borrowed name is in scope is a `view-conflict` error; put the borrow in an inner block, or `clone()` the part.
- An array rest name must be last and cannot have a default or trailing comma. A pattern declaration is not accepted in the update clause of a classic `for` loop.

Borrowed optional string fields can use a string literal fallback without cloning: `const label = item?.label ?? "none";`. See [borrowed text with a fallback](/tsrust/language/optional-values/#borrowed-text-with-a-fallback) for ownership and lexical source loans.

## Related guides

- [Pattern defaults and rest](/tsrust/language/pattern-defaults-rest/)
- [Record spread and pattern assignment](/tsrust/language/record-spread-assignment/)
