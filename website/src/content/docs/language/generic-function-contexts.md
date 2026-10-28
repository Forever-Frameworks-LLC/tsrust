---
title: Specialize a generic function value
description: Select concrete argument, result and capture-access contracts.
---

A concrete callable annotation selects one specialization of a generic declaration.
The checker uses the annotation's parameter and result types while preserving the
relationships declared by the generic function. The resulting value accepts that
concrete contract, rather than arbitrary future argument types.

After [native setup](/tsrust/tools/rust-setup/), save each complete example separately
as `main.tsr` and run `tsrust run main.tsr`. Comments show expected output.

## Choose concrete arguments and results

```tsrust
function identity<T>(value: T): T {
  return value;
}

function main() {
  const numberValue: (value: int) => int = identity;
  const stringValue: (value: Owned<string>) => string = identity;
  console.log(numberValue(7)); // 7
  console.log(stringValue("text")); // text
}
```

Returning a string argument transfers it, so its contract uses `Owned<string>`.
An annotation such as `(value: int) => string` cannot specialize this identity
function: its argument and result must have the same type. Passing a string to
`numberValue` is also an error.

## Infer from a result or preserve mutation

Result context can select a type even without an argument. A mutating generic
operation needs a contract that permits its argument mutation.

```tsrust
function empty<T>(): T[] {
  return [];
}

function takeFirst<T>(values: T[]): T {
  return values.remove(0);
}

function main() {
  const fresh: () => int[] = empty;
  console.log(fresh().length); // 0
  const head: (values: Mutable<int[]>) => int = takeFirst;
  const values = [7, 8];
  console.log(head(values), values.length); // 7 1
}
```

The caller observes the removal. Specialization does not add a clone or weaken
argument access. An owning argument remains transferred after its call.

## Choose borrowed or owned captured state

The callable's access contract also controls its environment. A mutable borrowed
view can update an enclosing scalar; an owned callable retains its own scalar
capture. These are different environments even when both come from one declaration.

```tsrust
function main() {
  let calls = 0;
  function select<T>(value: T): T {
    calls += 1;
    return value;
  }

  {
    const borrowed: Mutable<Function<int, [value: int]>> = select;
    borrowed(1);
    borrowed(2);
  }

  console.log(calls); // 2
  {
    const owned: (value: int) => int = select;
    owned(3);
    owned(4);
  }

  console.log(calls); // 2
}
```

A borrowed view keeps its capture loans until its scope ends. Owned captures use
normal transfer rules for owner types; a callback that consumes a capture is
single-use. See [captured state](/tsrust/language/nested-function-captures/) and
[local callable contracts](/tsrust/language/callable-locals/).

## Preserve default and rest parameters

```tsrust
function offset<T>(value: T, index: int = 0): T {
  return value;
}

function first<T>(...values: T[]): T {
  return values.remove(0);
}

function main() {
  const choose: (value: int, index?: int) => int = offset;
  console.log(choose(7, undefined)); // 7
  const pick: (...values: int[]) => int = first;
  console.log(pick(3, 4)); // 3
}
```

Omission or `undefined` evaluates a default in the original function once. A
supplied `null` must satisfy the declared type. Every rest element must satisfy
the selected element type, and the rest pack belongs to that invocation. See
[captured parameter behavior](/tsrust/language/captured-function-parameters/).

To keep one value generic across calls, omit its concrete callable annotation;
continue with [generic function values](/tsrust/language/generic-function-values/).
