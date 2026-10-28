---
title: Declare and initialize variables
description: Introduce multiple names, scope loop variables and assign deferred values safely.
---

Use `const` for a binding that cannot be reassigned and `let` when its binding may
change. Each declared name has its own type and initialization rules.

## Declare several names

Separate declarators with commas. Initializers run left to right; a later
initializer may read a name introduced earlier in the same declaration.

After [setting up native execution](/tsrust/tools/rust-setup/), save this as
`main.tsr` and run `tsrust run main.tsr`. Its assertions pass without output.

```tsrust
import { strictEqual } from "std:testing";

function main() {
  let width = 3, height = 4, label: string = "area";
  const area = width * height, text = label + "=" + String(area);
  strictEqual(text, "area=12");
  width = 5;
  strictEqual(width * height, 20);
}
```

The keyword applies to every name in the list: all are `const` or all are `let`.
Each annotation applies only to its own name. Reading a later name before its
declaration is a `name` error; declaring the same name twice is a `duplicate` error.

`const` needs an initializer. A `let` without an initializer needs an explicit type
and must satisfy the [assignment-before-use rules](#assign-a-value-later).

## Mix names and patterns

A declarator may be a destructuring pattern. Run this as a separate `main.tsr`;
its assertion passes without output:

```tsrust
import { strictEqual } from "std:testing";

type Point = {
  x: int;
  y: int;
};

function main() {
  const origin: Point = {
    x: 7, y: 8
  };
  const first = 4, [second, third] = [5, 6], { x, y: z } = origin;
  strictEqual(first + second + third + x + z, 30);
}
```

The array pattern selects positions; the record pattern selects fields, with
`y: z` binding field `y` to the local name `z`. Each declarator keeps the ownership
rules it has as a separate declaration. A list creates no extra copies or borrows.
Destructuring patterns always need an initializer.

## Scope a classic for loop

A classic `for` initializer accepts declaration lists. The loop's names are visible
inside its condition, body and update clause, but not after the loop.

This separate example finishes without output:

```tsrust
import { strictEqual } from "std:testing";

function main() {
  let total = 0;
  for (let i = 0, j = 10; i < j; i++, j--) {
    total += j - i;
  }

  strictEqual(total, 30);
}
```

Update expressions run left to right after each iteration, including an iteration
that reaches `continue`. A `break` skips them. A declaration cannot appear in the
update clause. Neither declaration lists nor update lists accept a trailing comma.

Commas in a declaration list separate declarators. To select a value with the
comma operator elsewhere, use a [comma expression](/tsrust/language/comma-expressions/),
such as `const selected = (1, 2);`.

## Assign a value later

An annotated `let` may omit its initializer when later control flow chooses the
value. Every path that reaches a read must assign the variable first. Otherwise,
the compiler reports `unassigned`.

Run this complete example separately; its assertions finish without output:

```tsrust
import { strictEqual } from "std:testing";

function grade(score: int): string {
  let label: string;
  if (score >= 90) {
    label = "high";
  } else if (score >= 50) {
    label = "mid";
  } else {
    label = "low";
  }

  return label;
}

function main() {
  strictEqual(grade(95), "high");
  strictEqual(grade(60), "mid");
  strictEqual(grade(10), "low");
  let width: int, height: int = 3;
  width = height * 2;
  strictEqual(width, 6);
  let note: string | undefined;
  strictEqual(note === undefined, true);
}
```

### Cover every reaching branch

A plain assignment can initialize the variable in the same block or a nested block.
An `if` can establish initialization when every branch reaching the read assigns
the value. A `return`, `break` or `continue` excludes a branch only from reads that
its control flow skips. An `if` without `else` is not enough by itself.

A `switch` with a `default` can establish initialization when every reaching clause
assigns the variable. Without a default, an unmatched value leaves an unassigned path.

### Initialize before loops and reads

Assignments inside a loop body can make the value available later in that iteration.
They do not establish initialization for code after `while`, `for`, `for...of` or
`do...while`. Initialize a fallback before the loop when the value is needed afterwards.

Compound assignment such as `value += 1` reads the old value and cannot initialize
an unassigned variable. The right side of `value = value + 1` also reads too soon.
If computing an assignment throws before the store, a catch handler cannot assume
that the assignment completed.

### Initialize before creating a closure

An arrow or function expression cannot capture an unassigned value. This fragment
is rejected with `unassigned`:

```text
let total: int;
const read = (): int => total;
total = 1;
```

Assign `total` before creating the closure, or initialize it in its declaration.

## Understand omitted initializers

| Declaration | Result |
| --- | --- |
| `let value: int;` | Needs an assignment before use |
| `let note: string \| undefined;` | Starts as `undefined` |
| `let value;` | `annotation-required` error |
| `const value: int;` | `syntax` error |
| A destructuring pattern without an initializer | `syntax` error |

When the annotation includes `undefined`, the omitted initializer behaves as an
explicit `= undefined`. Other supported types remain unassigned. A non-optional
function-typed variable requires an initializer.

A deferred declaration may occur in a declaration list or a classic `for`
initializer. After initialization it follows ordinary `let` typing and ownership
rules; later assignments do not bypass those rules.
