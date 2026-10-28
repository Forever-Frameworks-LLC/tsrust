---
title: Keep function values generic
description: Infer each invocation while preserving ownership and captured state.
---

Storing a generic declaration without a concrete callable annotation preserves its
generic relationship. Each invocation can select different types while checking
the original parameter, result and access requirements.

After [native setup](/tsrust/tools/rust-setup/), save each complete example separately
as `main.tsr` and run `tsrust run main.tsr`. Comments show expected output.

## Call one value with different types

```tsrust
function identity<T>(value: T): T {
  return value;
}

function size<T>(values: ReadOnly<T[]>): int {
  return values.length;
}

function main() {
  const action = identity;
  console.log(action(7)); // 7
  console.log(action("text")); // text
  const values = [3, 4];
  const length = size;
  console.log(length(values), values.length); // 2 2
  const returned = action(values);
  console.log(returned.join(",")); // 3,4
}
```

`length` borrows the array, so `values` remains available afterwards. `action`
returns its argument as an owner: after `action(values)`, use `returned` instead
of the transferred `values`. Changing the argument type does not bypass ownership.

Generic constraints remain checked. For example, a function whose two parameters
share type `T` cannot receive incompatible types for them. Required arguments must
still be supplied, and a result annotation cannot contradict the inferred type.

## Share one captured environment across calls

A returned generic callback keeps one environment across its concrete invocations.
Separate calls to its factory create independent environments.

```tsrust
function make(start: int) {
  let calls = start;
  function select<T>(value: T): T {
    calls += 1;
    console.log(calls);
    return value;
  }

  return select;
}

function main() {
  const first = make(10);
  const second = make(20);
  first(1); // 11
  second("x"); // 21
  first("y"); // 12
}
```

A new argument type does not reset the counter or clone the environment. If a
callback consumes a captured owner, it cannot be invoked again with any type.
Locally borrowed captures retain their capture loans; see
[nested-function captures](/tsrust/language/nested-function-captures/).

Default and optional parameters retain their omission behavior. Generic rest
parameters build a checked rest pack for each invocation. See
[captured parameters](/tsrust/language/captured-function-parameters/).

## Supply context for result-only types

An expected result annotation can select generic types when arguments provide no
information about them.

```tsrust
function empty<T>(): T[] {
  return [];
}

function main() {
  const action = empty;
  const numbers: int[] = action();
  const strings: string[] = action();
  numbers.push(7);
  strings.push("text");
  console.log(numbers[0], strings[0]); // 7 text
}
```

The two calls produce separately typed arrays. A result-only call without argument
or expected-result information still needs type context. This inference does not
relax parameter types or required argument counts.

## Inspect generic and concrete editor help

Hover and completion preserve the source generic signature for an alias, including
imported values and returned captured callbacks.

Signature help at a checked call shows the concrete types and access: the string identity call reports
`value: Owned<string>`, while `size` reports `values: ReadOnly<int[]>` for an integer
array. Definitions navigate back to the source declaration.

Use a [concrete callable context](/tsrust/language/generic-function-contexts/) when
a stored value should accept a fixed signature. [Concrete adapters](/tsrust/language/generic-value-adapters/) can also retain an
existing generic environment; their borrowing and transfer rules still apply.
