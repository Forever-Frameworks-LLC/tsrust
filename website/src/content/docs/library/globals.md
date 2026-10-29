---
title: Global functions and types
description: The names available in every file without an import — clone, Result, unwrap, Task, Function and the ownership wrappers.
---

These names are available in every module without an import. Most Tsrust files
use several of them. Each section explains when to reach for the name; the
[globals reference](../../reference/globals/functions/) lists every global with its
full contract.

| Name | Use it to |
| --- | --- |
| `clone(value)` | Make an independent copy of an owned value. |
| `Result<T, E>`, `Ok`, `Err` | Return success or a typed error the caller must handle. |
| `unwrap(result)` | Take the success value, or throw the error. |
| `Task<T>` | Type an async function's result. |
| `Function<R, [A]>` | Type a function value. |
| `ReadOnly`, `Mutable`, `Owned`, `Once` | Say how a function uses an argument or callback. |
| `String`, `Number`, `Boolean` | Convert a value explicitly. |
| `console` | Write to standard output and standard error. |

To run an example, save it as `main.tsr` after [native setup](../../tools/rust-setup/)
and run `tsrust run main.tsr`.

## clone

Assigning or passing an owned value (a string, array, record or `Map`) moves it;
the old name can no longer be used. `clone(value)` reads the value without
consuming it and returns an independent copy of the same type. Later changes to
either copy do not affect the other.

```tsrust
function main() {
  const first = [1, 2];
  const second = clone(first);
  second.push(3);
  console.log(first.join(","), second.join(","));
}
// prints: 1,2 1,2,3
```

Copies are never made implicitly, so `clone` marks every place a program pays for
one. Scalars such as `int`, `number` and `boolean` are already copied by
assignment and are not valid `clone` arguments. Plain callable values are not
cloneable either.

## Result, Ok and Err

`Result<T, E>` holds either a success value of type `T` or an error of type `E`.
Use it for an operation whose caller must handle failure. Build one with `Ok(value)`
or `Err(error)`, and inspect it with `match`:

```tsrust
function half(value: int): Result<int, string> {
  if (value % 2 !== 0) {
    return Err("odd");
  }
  return Ok(value / 2);
}

function main() {
  match (half(8)) {
    Ok(value) => {
      console.log(value);
    },
    Err(error) => {
      console.log(error);
    }
  }
  const text = match (half(3)) {
    Ok(value) => String(value),
    Err(error) => error,
  };
  console.log(text);
}
// prints: 4
// prints: odd
```

- A `match` must handle both `Ok` and `Err`, or end with `_`. A missing variant is
  a compile error.
- Each arm sees only its own payload.
- `Ok` and `Err` need to know both payload types. A return type, annotation or
  call argument usually supplies them; otherwise write `Ok<int, string>(5)`.
- Many standard-library operations return `Result`, for example
  [`std:fs`](../../reference/library/fs/) and [`std:env`](../../reference/library/env/).

## unwrap

`unwrap(result)` returns the success value. For an `Err`, it throws the original
error, which ordinary `try`/`catch`/`finally` receive with its type intact.
It does not panic or turn the error into text.

```tsrust
function parse(text: string): Result<int, string> {
  if (text === "") {
    return Err("empty");
  }
  return Ok(text.length);
}

function main() {
  console.log(unwrap(parse("abc")));
  try {
    unwrap(parse(""));
  } catch (error) {
    console.log(error);
  }
}
// prints: 3
// prints: empty
```

Use `unwrap` when failure should propagate like an exception; use `match` when the
caller recovers locally. `unwrap` consumes the `Result` and must be called
directly; it cannot be stored as a function value.

Older code may import it with `import { unwrap } from 'std:result'`. That import
still works and refers to the same function.

## Task

Calling an `async function` creates a `Task<T>`, where `T` is its return type.
The body runs when the task is awaited, or concurrently after `task.start()`.
Errors from the body reach `try`/`catch` at the `await`.

```tsrust
async function answer(): Task<int> {
  return 42;
}

async function main(): Task<void> {
  const task = answer();
  console.log(await task);
}
// prints: 42
```

A task can be awaited once and cannot be cloned. See [async tasks](../../language/async/)
for the full rules and [`std:futures`](../../reference/library/futures/) for timers,
races and cancellation.

## Function

`Function<Result, [Arguments]>` is the type of a function value. `Result` is the
return type (`void` for none). The optional list gives argument types in call
order; arguments can be labelled, optional or a final rest argument.

```tsrust
function main() {
  const double: Function<int, [int]> = (value: int): int => {
    return value * 2;
  };
  console.log(double(4));
}
// prints: 8
```

The arrow form `(value: int) => int` is also accepted as a type.

## Ownership wrappers

A function that receives a string, array, record or callback must say how it uses
it. Four wrappers express this. They appear in parameter types, in `Function`
argument lists, and around callback types.

| Wrapper | Caller keeps the value? | Function may change it? | Typical use |
| --- | --- | --- | --- |
| `ReadOnly<T>` | Yes | No | Read a value or call a callback many times. |
| `Mutable<T>` | Yes | Yes | Change the caller's value in place, or call a stateful callback. |
| `Owned<T>` | No — it moves | Yes | Keep, return or store the value. |
| `Once<Function<...>>` | No — it moves | Called exactly once | A callback that hands its captured values out. |

```tsrust
function total(values: ReadOnly<int[]>): int {
  let sum = 0;
  for (const value of values) {
    sum += value;
  }
  return sum;
}

function append(values: Mutable<int[]>, value: int) {
  values.push(value);
}

function keep(text: Owned<string>): string {
  return text;
}

function main() {
  const values = [1, 2];
  append(values, 3);
  console.log(total(values), values.length);
  const label = "moved".repeat(1);
  console.log(keep(clone(label)), label);
}
// prints: 6 3
// prints: moved moved
```

- After passing a value as `Owned`, the caller cannot use it again; that is a
  `transferred` error. Pass `clone(value)` to keep the original, as above.
- A borrowed (`ReadOnly` or `Mutable`) callback cannot be returned or stored; that is
  a `callback-escape` error.
- Calling a `Once` callback twice is a compile error.
- The wrappers apply to owned data and callbacks. Around a scalar such as `int` they
  are rejected.

## Conversions

Conversions are explicit. Conditions require a `boolean`, arithmetic does not
coerce strings, and equality uses `===` and `!==`; coercive `==` and `!=` are
rejected.

| Call | Result |
| --- | --- |
| `String(value)` | Text for a scalar, array or record. |
| `Number(value)` | A `number`; invalid text gives `NaN`, empty text and `null` give `0`. |
| `Boolean(value)` | Truthiness: `false` for `null`, `undefined`, `0`, `NaN` and `""`. |
| `parseInt(text, radix?)`, `parseFloat(text)` | JavaScript-style parsing to `number`. |

```tsrust
function main() {
  console.log(String(42), Number("12.5"), Boolean(""), Boolean([]));
}
// prints: 42 12.5 false true
```

For checked conversion that reports errors instead of `NaN`, use
[`std:numeric`](../../reference/library/numeric/).

## console

`console.log`, `console.info` and `console.debug` write to standard output;
`console.warn` and `console.error` write to standard error. Arguments are separated
by spaces. See [console](../../reference/globals/console/) for the other
methods.

## More globals

The reference lists every global: [functions](../../reference/globals/functions/),
[types](../../reference/globals/types/), [values and operators](../../reference/globals/values-and-operators/)
such as `typeof` and `NaN`, and the [`console`](../../reference/globals/console/),
[`JSON`](../../reference/globals/json/), [`Math`](../../reference/globals/math/),
[`Number`](../../reference/globals/number/) and [`Object`](../../reference/globals/object/) namespaces.
Built-in methods on strings, arrays, numbers, maps and sets are in the
[built-in types](../../reference/builtins/strings/) reference.
