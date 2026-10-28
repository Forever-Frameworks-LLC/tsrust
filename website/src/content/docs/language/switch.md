---
title: Select a switch branch
description: Match ordered case expressions, share labels and handle unmatched values.
---

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

## Match cases in order

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

### Compare compatible values

A case matches by strict equality (`===`). The value and each case must be
compatible with `===`. Numeric, boolean and string cases are supported, along with
enum values and supported unions. Cases may also check `null` or `undefined`;
there is no coercion between those two values.

When the value is a union variable and has not changed during matching, a matching
case narrows it inside the clause. For example, `case "go": value.toUpperCase();` can use the string methods.

### Share a body between labels

Adjacent labels share one clause: `case 2: case 3:` runs the same statements for
both values. A group may include `default`; its case expressions still participate
in matching, and the shared body also handles values that match no case.

### Choose a fallback

`default` may appear anywhere. It runs only after every case has failed, whatever
its position. A switch can have one `default`.

## Handle missing values

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

## End a clause explicitly

A nonempty clause before the last clause must end with `break`, `return` or `continue`. The last clause may finish without an explicit terminator. `break` leaves the switch. `continue` continues the enclosing loop. A clause that is one block may end the block with `break`.

Two JavaScript forms are not available, and each has a message that says what to write:

- **Running into the next clause.** A clause with statements cannot fall into the clause after it. End it explicitly, or leave it empty to share the next clause.
- **A `break` inside a nested statement of a clause**, such as `if (done) break;`. Make `break` the last statement of the clause, or restructure with `if` and `else`. A `break` inside a loop in the clause belongs to that loop, as usual.

## Scope and return values

Each clause is its own block. A name declared in one clause is not visible in another. To return from every branch of the switch, provide a returning `default` clause, as in the example. Without one, handle the unmatched path after the switch.

A switch supports at most 120 clauses.
