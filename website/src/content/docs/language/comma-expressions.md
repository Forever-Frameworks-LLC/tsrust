---
title: Sequence comma expressions
description: Evaluate ordered effects, select a final value and preserve ownership and narrowing.
---

A comma expression evaluates each operand once, from left to right, and selects
the last operand as its result. Earlier results are discarded before the next
operand runs. Prefer separate statements when they make the sequence easier to read.

## Evaluate in order

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

## Separate expressions from lists

Comma has the lowest expression precedence. Parenthesize a comma expression used
as an argument, array element, initializer or arrow body. Otherwise, commas in
argument, parameter, declaration, array and record lists remain separators.

For example, `console.log(1, 2)` passes two arguments, while
`console.log((1, 2))` passes only the selected value `2`.

Return and throw expressions, conditions and computed indexes also accept comma
expressions. A declaration cannot be a comma operand. If an operand throws, later
operands do not run; normal `catch` and `finally` handling applies.

## Preserve ownership and borrows

The final operand determines the result type and access. Discarding an owned name
does not clone or transfer it. An explicit call with an `Owned` parameter still
transfers its argument, even if its return value is discarded.

Borrowed results keep their source loans. Selecting a borrowed array does not
permit mutation. Selecting an indexed array keeps its containing source loaned
while the method needs it. Temporary prefix results are discarded; named borrows
keep their normal lexical scope.

Ordinary owned replacement follows the rules for [assignment values](/tsrust/language/assignment-values/).
String `+=` is allowed in a discarded position, such as a prefix operand or a
`for` update, but does not supply an owning result. Destructuring assignment keeps
its statement form.

## Narrow a selected value

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

### Do not narrow a source that changed

A comparison observes its left value before evaluating its right value. If the
right operand changes the source of that observed value, the comparison cannot
narrow the current source. Check the current value again before using its payload.

Read-only calls preserve the proof. A possible write on either branch of a
right-hand conditional invalidates it. Equality still uses `===` and `!==`;
comma expressions do not add coercion.
