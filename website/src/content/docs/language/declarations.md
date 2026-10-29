---
title: "Declarations and assignment"
description: "Declare variables, assign values, use assignment results and comma expressions."
---

`const` declares a binding that cannot be reassigned; `let` declares one that can. Every binding has a checked type and must be initialized before it is read. Assignment is an expression whose result follows the value's ownership: copy values are snapshots and owned values move unless cloned. Comma expressions evaluate operands left to right and produce the last one.

To run an example, save it as `main.tsr` after [native setup](../../tools/rust-setup/) and run `tsrust run main.tsr`. Examples that use `strictEqual` finish without output when their assertions pass; others show expected output in comments.

## Declarations

Use `const` for a binding that cannot be reassigned and `let` when its binding may
change. Each declared name has its own type and initialization rules.

### Declare several names

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

### Mix names and patterns

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

### Scope a classic for loop

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
comma operator elsewhere, use a [comma expression](/tsrust/language/declarations/#comma-expressions),
such as `const selected = (1, 2);`.

### Assign a value later

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

#### Cover every reaching branch

A plain assignment can initialize the variable in the same block or a nested block.
An `if` can establish initialization when every branch reaching the read assigns
the value. A `return`, `break` or `continue` excludes a branch only from reads that
its control flow skips. An `if` without `else` is not enough by itself.

A `switch` with a `default` can establish initialization when every reaching clause
assigns the variable. Without a default, an unmatched value leaves an unassigned path.

#### Initialize before loops and reads

Assignments inside a loop body can make the value available later in that iteration.
They do not establish initialization for code after `while`, `for`, `for...of` or
`do...while`. Initialize a fallback before the loop when the value is needed afterwards.

Compound assignment such as `value += 1` reads the old value and cannot initialize
an unassigned variable. The right side of `value = value + 1` also reads too soon.
If computing an assignment throws before the store, a catch handler cannot assume
that the assignment completed.

#### Initialize before creating a closure

An arrow or function expression cannot capture an unassigned value. This fragment
is rejected with `unassigned`:

```text
let total: int;
const read = (): int => total;
total = 1;
```

Assign `total` before creating the closure, or initialize it in its declaration.

### Understand omitted initializers

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

## Assignment values

An ordinary assignment (`=`) is an expression: it stores a value and lets the
surrounding expression use that value. The result's ownership depends on its type.

### Store and use a value

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

### Copy scalar results

For `int`, `number` and `boolean`, the assignment result is a copy of the committed
value. Chaining assignments leaves each scalar target usable. Scalar array
elements can also supply assignment values.

### Read an owned target in place

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

### Choose a supported target

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

### Distinguish replacement from compound assignment

These rules describe ordinary `=` replacement. Do not infer that every compound
operation returns an owned result. In particular, string `+=` can be used when its
result is discarded, but cannot provide an owning value for another binding.
See [comma expressions](/tsrust/language/declarations/#comma-expressions) for ordered discarded
operations and selected results.

## Comma expressions

A comma expression evaluates each operand once, from left to right, and selects
the last operand as its result. Earlier results are discarded before the next
operand runs. Prefer separate statements when they make the sequence easier to read.

### Evaluate in order

After [setting up native execution](/tsrust/tools/rust-setup/), save this as
`main.tsr` and run `tsrust run main.tsr`. It finishes without output when the
assertions pass.

```tsrust
import { strictEqual } from "std:testing";

function main() {
  let i = 0, j = 3;
  for (i = 0, j = 3; i < 3; i++, j++) {}

  strictEqual(i, 3);
  strictEqual(j, 6);
  const selected = (1, "last");
  strictEqual(selected, "last");
  const values = [1, 2];
  let calls = 0;
  (calls += 1, values).push(3);
  strictEqual(calls, 1);
  strictEqual((0, values).join(","), "1,2,3");
}
```

The selected array supports the same methods as the original name. Prefix effects
run before method arguments; they do not run again when the method uses its receiver.

### Separate expressions from lists

Comma has the lowest expression precedence. Parenthesize a comma expression used
as an argument, array element, initializer or arrow body. Otherwise, commas in
argument, parameter, declaration, array and record lists remain separators.

For example, `console.log(1, 2)` passes two arguments, while
`console.log((1, 2))` passes only the selected value `2`.

Return and throw expressions, conditions and computed indexes also accept comma
expressions. A declaration cannot be a comma operand. If an operand throws, later
operands do not run; normal `catch` and `finally` handling applies.

### Preserve ownership and borrows

The final operand determines the result type and access. Discarding an owned name
does not clone or transfer it. An explicit call with an `Owned` parameter still
transfers its argument, even if its return value is discarded.

Borrowed results keep their source loans. Selecting a borrowed array does not
permit mutation. Selecting an indexed array keeps its containing source loaned
while the method needs it. Temporary prefix results are discarded; named borrows
keep their normal lexical scope.

Ordinary owned replacement follows the rules for [assignment values](/tsrust/language/declarations/#assignment-values).
String `+=` is allowed in a discarded position, such as a prefix operand or a
`for` update, but does not supply an owning result. Destructuring assignment keeps
its statement form.

### Narrow a selected value

Strict equality recognizes a selected `null` or `undefined` sentinel, a selected
`typeof` expression and a selected literal type label. Run this separate example;
its assertions finish without output:

```tsrust
import { strictEqual } from "std:testing";

function lengthOrValue(value: int | string): int {
  if ((0, typeof value) === (0, "string")) {
    return value.length;
  }

  return value;
}

function main() {
  strictEqual(lengthOrValue("abc"), 3);
  strictEqual(lengthOrValue(7), 7);
}
```

#### Do not narrow a source that changed

A comparison observes its left value before evaluating its right value. If the
right operand changes the source of that observed value, the comparison cannot
narrow the current source. Check the current value again before using its payload.

Read-only calls preserve the proof. A possible write on either branch of a
right-hand conditional invalidates it. Equality still uses `===` and `!==`;
comma expressions do not add coercion.
