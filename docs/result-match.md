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
Imported Result payloads retain nominal aliases visible in the current module.
Inferred Result hover uses the public type rather than private storage names.
Nested matches keep shadowed payload bindings separate. During an unfinished edit,
a complete pattern header still provides binding completion and navigation inside
its arm. Payload types remain unknown until checked facts are available; sibling
arms and code after the match do not acquire that binding.

Owned construction transfers the payload without cloning. Matching transfers the
owned result into the match scope; an arm can transfer its selected payload onward.
The inactive payload does not exist. Normal scope cleanup handles retained values.
Returning, breaking or continuing out of an arm releases its scoped borrow and
drops any remaining owners. Replacing an owned Result drops its previous payload.
Callable payloads follow the same cleanup rules when they return or throw; surrounding
`finally` blocks still run. Tagged `Err` values themselves do not trigger `catch`.

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

A match on a `ReadOnly<Result<T, E>>` parameter lends the selected payload for
the arm. It does not consume the caller's Result. Record fields, strings and arrays
remain readonly, and owned payloads cannot escape the borrow. Explicit `clone()`
can produce an independent value when the payload supports cloning. Matching a
borrowed Result during array iteration also leaves the array elements owned by
the array. Arm borrows end with their scope.

A `Mutable<Result<T, E>>` parameter lends exclusive access during the match.
Changes to record fields or array contents update the caller's selected payload:

```ts
function append(result: Mutable<Result<int[], string>>): void {
    match (result) {
        Ok(values) => { values.push(9); },
        Err(_) => {},
    }
}

const editable: Result<int[], string> = Ok([1]);
append(editable);
append(editable);
match (editable) {
    Ok(values) => { console.log(values.length); },
    Err(_) => {},
}
// 3
```

The exclusive payload borrow cannot transfer ownership or escape in a closure.
Access the payload rather than its original Result while the match holds that
borrow. After the match returns, the caller can borrow or consume the Result again.
Scalar payloads copy into pattern bindings, following the ordinary scalar rules;
pattern bindings themselves are not reassignable variables. Nested match statements
preserve exclusive access to nested owned payloads. Explicit cloning remains
available for cloneable payloads.

Explicit propagation uses a return in the failure arm:

```ts
function doubledQuotient(a: int, b: int): Result<int, string> {
    match (divide(a, b)) {
        Ok(value) => { return Ok(value * 2); },
        Err(error) => { return Err(error); },
    }
}
```

## Value matches, literals and named variants

A match can produce a value: `const name = match (result) { Ok(value) => value,
Err(error) => { throw error; }, };`. A concise arm supplies its expression's value.
A block arm must exit with `return`, `throw`, `break` or `continue`; `return` exits
the enclosing function. Concise arrow functions likewise return their expression.
The scrutinee runs once, and only the selected arm runs. Ordinary argument order,
short-circuiting, lexical cleanup and ownership rules apply.

Literal patterns accept strings, integers, numbers, booleans, null and undefined.
An open scalar such as `int` needs a final `_` arm. Boolean and finite literal unions
can enumerate every alternative. Duplicate patterns and arms after complete
coverage are rejected.

Named record unions support `User(user)` patterns. The binding is the selected
record, with its original nominal identity. These patterns do not add enum-style
constructors: construct the existing named record and pass it into the union.
Result patterns keep their special payload binding behavior.

Readable examples: [values](../examples/match-values/main.tsr),
[literals](../examples/match-literals/main.tsr), and
[named variants](../examples/match-variants/main.tsr). Acceptance is pending.
