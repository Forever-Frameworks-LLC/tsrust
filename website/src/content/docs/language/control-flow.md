---
title: "Control flow"
description: "Statement bodies, else branches, loop scope, switch, for...in and labels."
---

`if`, `else` and loops accept a braced block or a single statement, and an `else` belongs to the nearest unmatched `if`. Names declared in a loop header are scoped to the loop. `switch` evaluates its subject once, compares it with each `case` in source order using `===`, and runs the first matching clause; adjacent labels share one clause, and `default` runs only when no case matches.

To run an example, save it as `main.tsr` after [native setup](../../tools/rust-setup/) and run `tsrust run main.tsr`. Examples that use `strictEqual` finish without output when their assertions pass; others show expected output in comments.

## Statement bodies

The body of `if`, `else`, `while`, `do`, classic `for` and `for...of` can be one statement without braces. The statement forms a block of its own.

```tsrust
import { strictEqual } from "std:testing";

function sign(value: int): string {
  if (value < 0) return "negative";
  else if (value === 0) return "zero";
  else return "positive";
}

function main() {
  strictEqual(sign(-2), "negative");
  let total = 0;
  for (let i = 0; i < 4; i++) total += i;
  for (const value of [10, 20]) total += value;
  while (total > 30) total--;
  strictEqual(total, 30);
}
```

### Choose a single statement

The body can be an expression or assignment, `return`, `break`, `continue`, or another `if` or loop. Bodies nest: `for (...) for (...) if (...) work();`.

### Attach an else branch

An `else` belongs to the nearest `if` that has no `else` yet. `if (a) if (b) x(); else y();` runs `y()` when `a` is true and `b` is false. Use braces to attach the `else` to the outer `if`.

### Use braces for declarations

A `const`, `let` or `function` declaration cannot be a body without braces. Put it in a block.

### Write an empty body

An empty statement is not a body. `if (ready) ;` is a `syntax` error; write `{}` for an empty body.

### Scope and nesting

A body without braces counts toward the nesting limit of 128, like a block.

Names declared in a loop header are scoped to the loop whether or not its body has braces. A destructuring assignment can be a body: `if (a < b) [a, b] = [b, a];`.

## switch

`switch` compares one value with each `case` in source order and runs the first clause that matches. `default` runs when no case matches.

```tsrust
import { strictEqual } from "std:testing";

function name(code: int): string {
  switch (code) {
    case 1:
      return "one";
    case 2:
    case 3:
      return "few";
    default:
      return "many";
  }
}

function main() {
  strictEqual(name(1) + name(3) + name(9), "onefewmany");
  let total = 0;
  for (let i = 0; i < 4; i++) {
    switch (i % 2) {
      case 0:
        continue;
      default:
        total += i;
        break;
    }

    total += 100;
  }

  strictEqual(total, 204);
}
```

### Match cases in order

The subject is selected before case expressions run. Case expressions run in
source order and stop at the first match. Later case expressions are not evaluated.
Changing a copyable subject variable from a case expression does not change the
value being matched:

```tsrust
import { strictEqual } from "std:testing";

function main() {
  let value = 1;
  let selected = "none";
  switch (value) {
    case(value = 2):
      selected = "two";
      break;
    case 1:
      selected = "one";
      break;
  }

  strictEqual(value, 2);
  strictEqual(selected, "one");
}
```

Prefer simple case expressions when side effects would make the order hard to
follow. Owned values, such as strings, are read without an implicit copy or
ownership transfer. A case expression cannot mutate or replace an owned subject
while comparing it. The selected body may mutate or consume it under the normal
ownership rules.

#### Compare compatible values

A case matches by strict equality (`===`). The value and each case must be
compatible with `===`. Numeric, boolean and string cases are supported, along with
enum values and supported unions. Cases may also check `null` or `undefined`;
there is no coercion between those two values.

When the value is a union variable and has not changed during matching, a matching
case narrows it inside the clause. For example, `case "go": value.toUpperCase();` can use the string methods.

#### Share a body between labels

Adjacent labels share one clause: `case 2: case 3:` runs the same statements for
both values. A group may include `default`; its case expressions still participate
in matching, and the shared body also handles values that match no case.

#### Choose a fallback

`default` may appear anywhere. It runs only after every case has failed, whatever
its position. A switch can have one `default`.

### Handle missing values

Use separate labels for `null` and `undefined`. Group them when both should take
the same path:

```tsrust
import { strictEqual } from "std:testing";

function describe(value: int | null | undefined): string {
  switch (value) {
    case null:
    case undefined:
      return "missing";
    default:
      return "present";
  }
}

function main() {
  strictEqual(describe(null), "missing");
  strictEqual(describe(undefined), "missing");
  strictEqual(describe(0), "present");
}
```

### End a clause explicitly

A nonempty clause before the last clause must end with `break`, `return` or `continue`. The last clause may finish without an explicit terminator. `break` leaves the switch. `continue` continues the enclosing loop. A clause that is one block may end the block with `break`.

Two JavaScript forms are not available, and each has a message that says what to write:

- **Running into the next clause.** A clause with statements cannot fall into the clause after it. End it explicitly, or leave it empty to share the next clause.
- **A `break` inside a nested statement of a clause**, such as `if (done) break;`. Make `break` the last statement of the clause, or restructure with `if` and `else`. A `break` inside a loop in the clause belongs to that loop, as usual.

### Scope and return values

Each clause is its own block. A name declared in one clause is not visible in another. To return from every branch of the switch, provide a returning `default` clause, as in the example. Without one, handle the unmatched path after the switch.

A switch supports at most 120 clauses.

## for...in

`for (const key in value)` visits the keys of a record, tuple, array or string as
strings, in the order `Object.keys(value)` returns them: field names in declaration
order for a record, and `"0"`, `"1"`, ... for the others.

```tsrust
type Point = {
  x: int;
  y: int;
};

function main() {
  const point: Point = {
    x: 1, y: 2
  };
  const keys: string[] = [];
  for (const key in point) {
    keys.push(clone(key));
  }

  let indices = "";
  for (const index in[5, 6, 7]) {
    indices += index;
  }

  console.log(keys.join(","), indices);
}
// prints: x,y 012
```

- Declare the key with `const` or `let`. A destructuring pattern or an existing
  variable in that place is `unsupported`.
- The keys are collected once before the first iteration; changing the source
  during the loop does not change them.
- Each key borrows the collected string and cannot be reassigned. Use `clone(key)`
  to keep an owned copy.
- To read array elements, use `for...of` or an `int` index instead.
- Numbers and booleans have no keys. [`in`](../operators/#the-in-operator) tests
  a single key.

## Labels

A label names a statement so that a nested `break` can leave it. A label on a loop
also lets a nested `continue` start that loop's next iteration.

```tsrust
function main() {
  let total = 0;
  let updates = 0;
  outer: for (let i = 0; i < 3; i++, updates++) {
    for (let j = 0; j < 2; j++) {
      if (i === 1) continue outer;
      if (i === 2) break outer;
      total += 1;
    }
  }

  let chosen: int;
  done: {
    chosen = 7;
    break done;
  }

  console.log(total, updates, chosen);
}
// prints: 2 2 7
```

`continue outer` skips the rest of the outer body and runs the outer loop's update
(`i++, updates++`) before its condition. Updates of the loops it leaves do not
run. `break outer` leaves the loop without running its update.

- Blocks, `if`, loops and other statements can be labeled, and one statement can
  carry several labels (`first: second: while (...)`), up to 32.
- Without a name, `break` and `continue` target the nearest loop; a labeled block
  does not capture them.
- A label is visible only inside its statement and the same function. An unknown
  target, or `continue` naming a block, is `loop-control`. Reusing an active label
  is `duplicate`. A label directly before `let`, `const`, `type` or a function
  declaration is `syntax`.
- Labels have their own namespace; a variable can share a label's name.
- A deferred variable must be assigned on every path that reaches its use,
  including paths that leave a labeled block. Moving an outer owner before
  `continue` is rejected unless every continuing path reinitializes it.
