---
title: "Errors and matching"
description: "throw, try, catch and finally; run-time failures; Result values and match expressions with Result, literal and record patterns."
---

Tsrust has two ways to report failure. `throw` raises an error that unwinds to the
nearest `catch`, like JavaScript. A `Result<T, E>` value carries success or failure
as data that the caller inspects with `match`. Standard-library operations that
can fail usually return a `Result`; [`unwrap`](../../library/globals/#unwrap)
turns one into a thrown error when you want it to propagate.

To run an example, save it as `main.tsr` after [native setup](../../tools/rust-setup/)
and run `tsrust run main.tsr`.

## try, catch and finally

A `try` statement needs a `catch` clause, a `finally` clause or both. `catch` may
omit its binding when the error is not needed.

```tsrust
function value(): int {
  try {
    throw "failed";
  } catch (error) {
    console.log(error);
    return 4;
  } finally {
    console.log("cleanup");
  }
}

function main() {
  console.log(value());
}
// prints: failed
// prints: cleanup
// prints: 4
```

- `catch` handles failures from its `try` body, including errors thrown by called
  functions and by array callbacks such as `map`, `filter` and `sort`. A failing
  sort comparator leaves the array in its original order.
- `finally` runs after normal completion and before a pending `return`, `throw`,
  `break` or `continue` takes effect. A returned value is evaluated before
  `finally` runs. If `finally` itself returns or throws, that replaces the pending
  completion.
- An error thrown in `catch` runs `finally` and then goes to an enclosing handler.
  An error thrown in `finally` is not handled by the same statement's `catch`.
- Nested `finally` blocks run from the inner region outward.
- A `catch` handler can `break` or `continue` its enclosing loop; `finally` runs
  first.

## Thrown values

`throw` accepts any checked value. The compiler infers which error types can reach
each handler, so the catch binding has a precise type. That type also includes the
[run-time failure](#run-time-failures) record when a checked operation in the `try`
body can fail. Narrow the union with `typeof` before using type-specific
operations:

```tsrust
function main() {
  try {
    if (Math.random() > 2) {
      throw "text";
    }

    throw 7;
  } catch (error) {
    if (typeof error === "string") {
      console.log(error.length);
    } else if (typeof error === "number") {
      console.log(error + 1);
    } else {
      console.log(error.message);
    }
  }
}
// prints: 8
```

Throwing an owned value transfers it; `catch` receives that value and rethrowing
transfers it again. Error handling never clones a value or extends a borrow. An
error whose type cannot be determined, for example from an unknown callable
contract, is a compile-time diagnostic rather than a hidden failure path.

## Run-time failures

Checked arithmetic, array bounds and failed non-null assertions throw structured
errors with `name`, `code`, `message`, `source`, `start` and `end` fields. `start`
and `end` are byte offsets of the failing expression in `source`.

```tsrust
function divide(a: int, b: int): int {
  return a / b;
}

function main() {
  try {
    console.log(divide(1, 0));
  } catch (error) {
    console.log(error.name, error.code);
  }

  try {
    const values = [1, 2];
    console.log(values[5]);
  } catch (error) {
    console.log(error.name, error.code);
  }
}
// prints: RangeError arithmetic
// prints: RangeError bounds
```

An uncaught error runs the applicable `finally` blocks, prints the error and exits
with a failure status. Allocation failure, process termination, defects in native
code and failures of native bindings without an audited error contract are not
converted into catchable errors. Splitting a string inside a UTF-8 character also
stops the program; see [strings](../strings/).

## Result values

`Result<T, E>` holds either `Ok(value)` or `Err(error)`. Returning `Err` does not
throw; the caller decides what to do.

```tsrust
function divide(a: int, b: int): Result<int, string> {
  if (b === 0) {
    return Err("Cannot divide by zero");
  }

  return Ok(a / b);
}

function doubled(a: int, b: int): Result<int, string> {
  match (divide(a, b)) {
    Ok(value) => {
      return Ok(value * 2);
    },
    Err(error) => {
      return Err(error);
    }
  }
}

function main() {
  match (doubled(10, 2)) {
    Ok(value) => {
      console.log(value);
    },
    Err(error) => {
      console.log(error);
    }
  }

  console.log(unwrap(divide(9, 3)));
}
// prints: 10
// prints: 3
```

- `Ok` and `Err` need both payload types. A return type or annotation usually
  supplies them; otherwise write `Ok<int, string>(5)`.
- `Result<void, E>` succeeds with `Ok()`; match it with `Ok(_)`.
- A `Result` holds its payload by value: constructing one moves the payload in, and
  matching moves the `Result` into the match unless it is borrowed (below).
- An `Err` is ordinary data. It does not trigger `catch` unless you `unwrap` it or
  throw its payload.
- A `Result` can hold an owned callable (`Function<...>` or `Once<Function<...>>`)
  but not a borrowed `ReadOnly<Function<...>>` or `Mutable<Function<...>>`.

## match

`match (value) { pattern => arm, ... }` evaluates `value` once and runs the first
arm whose pattern matches. Arms are checked for coverage: a `match` must handle
every alternative or end with `_`, and duplicate arms or arms after complete
coverage are compile errors.

### Patterns

| Pattern | Matches |
| --- | --- |
| `Ok(name)`, `Err(name)` | A `Result` variant; `name` binds the payload inside the arm. Use `_` to ignore it. |
| `200`, `"text"`, `true`, `null`, `undefined` | A literal value of a string, `int`, `number`, `boolean`, `null` or `undefined` scrutinee. |
| `User(user)` | One named record type of a union of named records; `user` is that record. |
| `_` | Everything not matched by earlier arms. |

An open scalar type such as `int` or `string` needs a final `_`. A `boolean` or a
finite literal union can list every value instead.

```tsrust
type Admin = {
  kind: "admin";
  level: int;
};

type Guest = {
  kind: "guest";
  name: string;
};

type User = Admin | Guest;

function describe(code: int): string {
  return match (code) {
    200 => "ok",
    404 => "missing",
    _ => "other",
  };
}

function label(user: User): string {
  return match (user) {
    Admin(admin) => `admin ${admin.level}`,
    Guest(guest) => guest.name,
  };
}

function main() {
  const flag = true;
  const admin: Admin = {
    kind: "admin", level: 2
  };
  console.log(describe(404), describe(500), match (flag) {
    true => "yes",
    false => "no"
  }, label(admin));
}
// prints: missing other yes admin 2
```

Named record patterns select by the record's declared type. They do not create
constructors: build the record and pass it where the union is expected.

### Statements and values

As a statement, each arm is a block, and commas between arms are optional. As an
expression, a concise arm (`pattern => expression`) supplies the value; a block arm
in an expression must leave with `return`, `throw`, `break` or `continue`. Only the
selected arm runs. Payload bindings exist only inside their arm, and `_` is a
discard, not a variable.

### Borrowed matches

Matching a `ReadOnly<Result<T, E>>` parameter lends the payload for the arm without
consuming the caller's `Result`. Matching a `Mutable<Result<T, E>>` lends it
exclusively, so changes update the caller's payload:

```tsrust
function append(result: Mutable<Result<int[], string>>): void {
  match (result) {
    Ok(values) => {
      values.push(9);
    },
    Err(_) => {}
  }
}

function count(result: ReadOnly<Result<int[], string>>): int {
  return match (result) {
    Ok(values) => values.length,
    Err(_) => 0,
  };
}

function main() {
  const editable: Result<int[], string> = Ok([1]);
  append(editable);
  append(editable);
  console.log(count(editable));
}
// prints: 3
```

A borrowed payload cannot be moved out or captured by an escaping closure; use
`clone(...)` for an independent copy. Scalar payloads are copied into their
bindings. While a mutable match holds the payload, use the binding rather than the
original `Result`.

## Testing errors

`std:testing` reports an error that escapes a registered test as that test's
failure, runs its teardown and continues with later tests. See
[testing](../../tools/testing/).
