---
title: Use optional and default parameters
description: Accept omitted values and initialize defaults in parameter order.
---

## Optional arguments

`value?: T` accepts a value of type `T`, omission, or explicit `undefined`. Inside the function, the value has type `T | undefined`. Narrow it or use a nullish operation before an operation that requires `T`.

After [native setup](/tsrust/tools/rust-setup/), save each complete example separately as `main.tsr` and run
`tsrust run main.tsr`. Comments show the expected output.

```tsrust
function show(value?: int): string {
  return String(value ?? 7);
}

function main() {
  console.log(show(), show(undefined), show(0)); // 7 7 0
}
```

An optional marker does not permit `null` unless the declared type includes it. Optional parameters cannot precede required parameters. Extra arguments remain errors.

## Defaults and null

`value: T = expression` evaluates the default only when the argument is omitted or explicitly `undefined`. A supplied `null`, zero, false, or empty string does not trigger a default. The initializer must satisfy the declared type.

```tsrust
function pick(value: int | null = 5): int | null {
  return value;
}

function main() {
  console.log(String(pick()), String(pick(undefined)), String(pick(null))); // 5 5 null
}
```

The accepted call type includes `undefined`, but the initialized body type excludes it when neither the declared type nor the default retains it. An explicit `T | undefined` declaration retains that union; flow analysis may narrow an initialized occurrence when its value is known to be present. Defaults can call functions and construct strings, arrays, records, or supported callable values. Existing access and origin checks still apply.

Callable type annotations describe omission with `?`. Initializer expressions belong to executable function or arrow declarations, not callable type annotations.

## Evaluation order and scope

The callee and supplied arguments evaluate once, in source order, before defaults. Reached defaults then evaluate once, in parameter order. Earlier initialized parameters are available to later defaults. The current parameter, later parameters, and body-local declarations are unavailable in a default. Existing duplicate and reserved binding-name restrictions still apply.

```tsrust
function mark(trace: int[], value: int): int {
  trace.push(value);
  return value;
}

function add(trace: int[], left: int = mark(trace, 1), right: int = mark(trace, left + 1)): int {
  return left + right;
}

function main() {
  const trace: int[] = [];
  console.log(add(trace)); // 3
  console.log(add(trace, 10)); // 21
  console.log(trace.join(",")); // 1,2,11
}
```

The second call skips the `left` initializer. Its `right` initializer sees the supplied value `10`.

## Defaults before required arguments

A default may precede a required parameter, but its position still needs an argument. Pass `undefined` to select that default.

```tsrust
function choose(first: int = 5, required: int): int {
  return first + required;
}

function main() {
  console.log(choose(undefined, 2)); // 7
}
```

`choose(2)` is an arity error. A callable value representing this function has a required first slot accepting `int | undefined`, rather than an optional slot before a required slot. Forwarding invokes the selected function's default; the wrapper does not initialize that default itself.

## Callbacks and ownership

Array methods retain their existing callback arguments. For data-element callbacks, `map` supplies the value, index, and source array. An additional optional or defaulted trailing callback parameter receives `undefined`; a required extra parameter is rejected. Mutable callable-element visitors retain their existing slot limits and source-array loan restrictions. Initializers run only when the callback is actually invoked, so an empty traversal creates no default value.

Defaults preserve ordinary ownership rules. A skipped initializer creates no owner. A reached owned default is local to that invocation and is disposed when its owner leaves scope, including unwinding. Borrowed defaults do not permit an escaping borrow, duplicate mutable loans, or consumption through a read contract. Once callables still require consuming invocation, and existing optional-view restrictions remain in force.

## Current boundaries

Defaults do not add JavaScript coercions or change native string, integer, or collection behavior. Strings retain UTF-8 semantics and ordinary native integer limits apply. Parameter rebinding remains prohibited; mutable access permits supported receiver mutation, not assignment to the parameter binding. Rest parameters, destructuring patterns and nullable array elements are supported under their own type and ownership rules. Defaults do not relax those rules. See [rest parameters](/tsrust/language/rest-parameters/), [pattern defaults](/tsrust/language/pattern-defaults-rest/) and [nullable assignment](/tsrust/language/nullish-assignment/).

## Related guides

- [Use optional and default parameters](/tsrust/language/optional-default-parameters/)
- [Collect rest arguments](/tsrust/language/rest-parameters/)
- [Pattern defaults and rest](/tsrust/language/pattern-defaults-rest/)
