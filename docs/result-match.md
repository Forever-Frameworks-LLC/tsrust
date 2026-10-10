# Results and matching

`Result<T, E>` represents either a successful value of type `T` or an error of type
`E`. Returning an error does not throw an exception. The caller handles it explicitly.

```ts
function divide(a: int, b: int): Result<int, string> {
    if (b === 0) {
        return Err("Cannot divide by zero");
    }
    return Ok(a / b);
}

function main() {
    match (divide(10, 2)) {
        Ok(value) => { console.log(value); },
        Err(error) => { console.error(error); },
    }
}
// 5
```

The result annotation supplies both constructor types. A local constructor can use
an annotation, such as `const value: Result<int, string> = Ok(5)`, or explicit type
arguments: `const value = Ok<int, string>(5)`. `Err<int, string>("failed")` creates
the corresponding failure. A constructor without enough context is a compile error.
A void alternative has no payload:
`const done: Result<void, string> = Ok()`; match success with `Ok(_)`.

The scrutinee is evaluated once. A match must handle both variants, or use a final
`_` arm for the remaining case. Duplicate arms and arms after complete coverage are
compile errors. Payload bindings exist only inside their arm. Use `Ok(_)` or
`Err(_)` to ignore a payload. Block arms accept optional separating commas.

Payload bindings have checked hover types, member completion and navigation to
their own pattern declaration. The same name in another arm is a separate binding.
An unused payload binding retains its checked type; `_` is a discard, not a variable.

Owned construction transfers the payload without cloning. Matching transfers the
owned result into the match scope; an arm can transfer its selected payload onward.
The inactive payload does not exist. Normal scope cleanup handles retained values.

Owned callable payloads retain captured state and invocation rules:

```ts
function counter(start: int): Result<Function<int>, string> {
    let value = start;
    return Ok((): int => { value += 1; return value; });
}
```

Matching its success arm binds a reusable owned callable. `Once<Function<...>>`
instead permits one invocation. A borrowed `ReadOnly<Function<...>>` or
`Mutable<Function<...>>` cannot be stored directly in a Result. Generic wrappers
can take explicit callable arguments, such as `wrap<Function<int>>(callback)`;
an inline callback alone may not supply enough information for generic inference.

Explicit propagation uses a return in the failure arm:

```ts
function doubledQuotient(a: int, b: int): Result<int, string> {
    match (divide(a, b)) {
        Ok(value) => { return Ok(value * 2); },
        Err(error) => { return Err(error); },
    }
}
```

## Implementation status

The first construction/matching increment has focused compiler and native evidence.
B10 is not accepted. Borrowed matching, project-wide pattern-binding assistance, matched benchmarks and
family acceptance remain open. Match expressions, guards, nested destructuring
patterns, `?` propagation and general discriminated unions are not implemented.
Nested match statements are supported.
