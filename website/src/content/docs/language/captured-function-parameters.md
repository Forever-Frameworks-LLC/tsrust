---
title: Preserve parameters in captured callbacks
description: Use optional, default and rest parameters when storing a nested function.
---

Storing, passing or returning a captured named function preserves its checked
parameter contract. Its defaults run in the original declaration, with access to
that declaration's captures.

After [native setup](/tsrust/tools/rust-setup/), save each complete example separately
as `main.tsr` and run `tsrust run main.tsr`. Comments show expected output.

## Evaluate defaults once per omitted argument

Omitting an argument or passing `undefined` evaluates its default once. Passing a
non-undefined value skips the initializer, including its side effects and possible
failure. An optional parameter without a default receives `undefined`.

```tsrust
function main() {
  let hits = 0;
  function bump(): int {
    hits += 1;
    return 3;
  }

  let total = 10;
  function next(step: int = bump()): int {
    total += step;
    return total;
  }

  {
    const action = next;
    console.log(action()); // 13
    console.log(action(2)); // 15
    console.log(action(undefined)); // 18
  }

  console.log(hits); // 2
}
```

Borrowed captures retain their normal loan scope. A default that consumes a
captured owner can make the callback single-use, even though some calls supply
an argument. No implicit copy of an owned capture is introduced. See
[capture ownership](/tsrust/language/nested-function-captures/).

## Account for arguments supplied by array methods

Array callbacks receive the method's arguments. A supplied index is a real second
argument, so it overrides a default on that parameter.

```tsrust
function main() {
  const base = [10];
  function pick(value: int, extra: int = base[0]): int {
    return value + extra;
  }

  console.log([1, 2].map(pick).join(",")); // 1,3

  function optional(value: int, extra?: int): int {
    return value + (extra ?? base[0]);
  }

  const action = optional;
  console.log(action(1), action(1, 2)); // 11 3
}
```

Parameter types and argument counts are still checked. See
[optional and default parameters](/tsrust/language/optional-default-parameters/)
for declaration and call rules.

## Forward empty, explicit and spread rest arguments

A stored rest function receives its arguments in order. An empty call builds an
empty rest pack. Spreading an array supplies its elements as arguments.

```tsrust
function main() {
  const base = 10;
  function sum(...values: int[]): int {
    let total = base;
    for (const value of values) {
      total += value;
    }

    return total;
  }

  const action = sum;
  console.log(action(), action(2, 3)); // 10 15
  const values = [4, 5];
  console.log(action(...values)); // 19
}
```

Every rest element must satisfy the declared element type. An array method's full
callback tuple can be incompatible: an `int[]` rest parameter cannot receive a
source-array argument along with integer values and indexes. A compatible fixed
callable contract can adapt a rest function.

Fixed parameters before the rest parameter keep their optional/default behavior.
Omitted or `undefined` fixed slots evaluate their defaults once. Returned factories
retain separate captured state; consuming a captured owner still limits invocation
to once. See [rest parameters](/tsrust/language/rest-parameters/) and
[call spread](/tsrust/language/call-spread/) for argument and ownership details.
