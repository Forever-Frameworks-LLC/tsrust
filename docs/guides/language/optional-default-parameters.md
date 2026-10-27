# Optional and default parameters

Status: gate 98 has focused implementation acceptance. These examples pass the integrated compiler and generated native execution at O0 and O3. Full regression and performance measurements remain queued for checkpoint 101. See [the gate evidence](../../milestones/98-optional-default-parameters.md).

## Optional arguments

`value?: T` accepts a value of type `T`, omission, or explicit `undefined`. Inside the function, the value has type `T | undefined`. Narrow it or use a nullish operation before an operation that requires `T`.

```ts
function show(value?: int): string {
    return String(value ?? 7);
}
// Inside main:
console.log(show(), show(undefined), show(0)); // 7 7 0
```

An optional marker does not permit `null` unless the declared type includes it. Optional parameters cannot precede required parameters. Extra arguments remain errors.

## Defaults and null

`value: T = expression` evaluates the default only when the argument is omitted or explicitly `undefined`. A supplied `null`, zero, false, or empty string does not trigger a default. The initializer must satisfy the declared type.

```ts
function pick(value: int | null = 5): int | null {
    return value;
}
// Inside main:
console.log(String(pick()), String(pick(undefined)), String(pick(null)));
// 5 5 null
```

The accepted call type includes `undefined`, but the initialized body type excludes it when neither the declared type nor the default retains it. An explicit `T | undefined` declaration retains that union; flow analysis may narrow an initialized occurrence when its value is known to be present. Defaults can call functions and construct strings, arrays, records, or supported callable values. Existing access and origin checks still apply.

Callable type annotations describe omission with `?`. Initializer expressions belong to executable function or arrow declarations, not callable type annotations.

## Evaluation order and scope

The callee and supplied arguments evaluate once, in source order, before defaults. Reached defaults then evaluate once, in parameter order. Earlier initialized parameters are available to later defaults. The current parameter, later parameters, and body-local declarations are unavailable in a default. Existing duplicate and reserved binding-name restrictions still apply.

```ts
function mark(trace: int[], value: int): int {
    trace.push(value);
    return value;
}
function add(trace: int[], left: int = mark(trace, 1),
             right: int = mark(trace, left + 1)): int {
    return left + right;
}
// Inside main:
const trace: int[] = [];
console.log(add(trace));     // 3
console.log(add(trace, 10)); // 21
console.log(trace.join(",")); // 1,2,11
```

The second call skips the `left` initializer. Its `right` initializer sees the supplied value `10`.

## Defaults before required arguments

A default may precede a required parameter, but its position still needs an argument. Pass `undefined` to select that default.

```ts
function choose(first: int = 5, required: int): int {
    return first + required;
}
// Inside main:
console.log(choose(undefined, 2)); // 7
```

`choose(2)` is an arity error. A callable value representing this function has a required first slot accepting `int | undefined`, rather than an optional slot before a required slot. Forwarding invokes the selected function's default; the wrapper does not initialize that default itself.

## Callbacks and ownership

Array methods retain their existing callback arguments. For data-element callbacks, `map` supplies the value, index, and source array. An additional optional or defaulted trailing callback parameter receives `undefined`; a required extra parameter is rejected. Mutable callable-element visitors retain their existing slot limits and source-array loan restrictions. Initializers run only when the callback is actually invoked, so an empty traversal creates no default value.

Defaults preserve ordinary ownership rules. A skipped initializer creates no owner. A reached owned default is local to that invocation and is disposed when its owner leaves scope, including unwinding. Borrowed defaults do not permit an escaping borrow, duplicate mutable loans, or consumption through a read contract. Once callables still require consuming invocation, and existing optional-view restrictions remain in force.

## Current boundaries

Defaults do not add JavaScript coercions or change native string, integer, or collection behavior. Strings retain UTF-8 semantics and ordinary native integer limits apply. Parameter rebinding remains prohibited; mutable access permits supported receiver mutation, not assignment to the parameter binding. Rest parameters, destructuring, heterogeneous unions, and nullable array elements belong to later gates.
