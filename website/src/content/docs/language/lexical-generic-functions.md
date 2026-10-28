---
title: Use enclosing generic types
description: Inherit lexical type parameters and preserve callable types through factories.
---

A nested function can use its enclosing function's type parameters. Each concrete
outer invocation supplies those types to the helper. The helper's own explicit
type parameters remain separate.

After [native setup](/tsrust/tools/rust-setup/), save each complete example separately
as `main.tsr` and run `tsrust run main.tsr`. Comments show expected output.

## Inherit an outer type without repeating it

```tsrust
function outer<T>(value: T): T {
  function echo<U>(other: T, unused: U): T {
    return other;
  }

  return echo<boolean>(value, true);
}

function main() {
  console.log(outer<int>(7), outer<string>("owned")); // 7 owned
}
```

`echo` inherits `T`, but declares only `U`. Its explicit call therefore supplies
one type argument. A helper with no type parameters of its own takes none, even
when its parameters mention the outer `T`.

Explicit and inferred calls follow this rule, as do recursive peers, saved
function values and imported outer functions. An extra explicit type argument
is an error; inheritance does not change the helper's public type-argument count.

## Shadow an outer type parameter deliberately

An inner declaration can introduce its own `T`. That name refers to the inner
parameter inside that declaration, independently of the enclosing function's `T`.

```tsrust
function outer<T>(value: T): string {
  function identity<T>(other: T): T {
    return other;
  }

  const number = identity<int>(7);
  const text = identity<string>("inner");
  return String(number) + ":" + text;
}

function main() {
  console.log(outer<boolean>(true)); // 7:inner
}
```

Use distinct names when shadowing would make the type relationship harder to read.

## Retain typed captures when returning a helper

The returned callback keeps the lexical types and captures of its factory call.
Separate calls create independent environments.

```tsrust
function factory<T>(value: T) {
  function take(): T {
    return value;
  }

  return take;
}

function main() {
  const number = factory<int>(7);
  const text = factory<string>("owned");
  console.log(number(), text()); // 7 owned
}
```

Returning the string transfers its captured owner. Calling `text()` again is a
consumed-callback error. Preserving a lexical type does not clone its value or
relax capture ownership. Callback failure also cleans up its original owners.
See [nested-function captures](/tsrust/language/nested-function-captures/).

## Pass a generic callable through another factory

A generic function can retain a callable without erasing that callable's own
type parameters. Here the selector transfers its captured callback to the caller.

```tsrust
function identity<T>(value: T): T {
  return value;
}

function retain<A>(action: A) {
  function select() {
    return action;
  }

  return select;
}

function main() {
  const select = retain(identity);
  const saved = select();
  console.log(saved<int>(7), saved<string>("Pat")); // 7 Pat
}
```

The selector cannot return the same owned callable twice. The returned `saved`
value can still use different type arguments when its capture access permits
reuse. A callable that consumes a captured owner remains single-use, and a
borrowed environment cannot escape as an owner through this factory.

The original parameter counts, type relationships and argument access remain
checked. Separate factory calls retain independent environments. See
[generic function values](/tsrust/language/generic-function-values/).

## Read source types and concrete call help

Editor help inside the generic declaration preserves source types such as `T`.
Concrete outer results have their checked types, and definitions navigate to the
nested declaration.

A callable passed through a generic factory keeps concrete signature help for each invocation, including its original argument count.
