---
title: Annotate and borrow local callables
description: Control callable ownership, captured state and single-use invocation.
---

Callable annotations own environments by default. Use `ReadOnly<Function<..., [...]>>` or `Mutable<Function<..., [...]>>` for scoped borrowing; `Once<Function<..., [...]>>` owns an environment consumed by invocation. `Owned<Function<...>>` means the same as plain `Function<...>`. Arrow type spelling such as `(value: int) => int` remains accepted and the wrappers compose with it.

After [native setup](/tsrust/tools/rust-setup/), save each complete example separately
as `main.tsr` and run `tsrust run main.tsr`. Comments show expected output.

A local `const` or `let` can declare a callable signature. The compiler checks argument types, access modes and the result, then retains the concrete function or closure environment. Parentheses around the signature are permitted.

In this table, `F` stands for `Function<int, [value: int]>` (a callable taking an
`int` and returning an `int`).

| Annotation | Behavior |
| --- | --- |
| `F` | Owns its environment; repeated calls borrow it temporarily. |
| `Once<F>` | The environment moves with the binding and can be invoked at most once. |
| `ReadOnly<F>` | Borrows an environment whose captured state stays unchanged. |
| `Mutable<F>` | Borrows a reusable environment whose captured state may change. |

## Borrow a callable for a temporary call

An owned reusable callable parameter can be passed temporarily to a `Mutable<Function<...>>` parameter, then invoked again or transferred afterward. This reborrowing does not copy or move its environment. 

Overlapping mutable argument loans and borrowing while transferring the same callable remain errors. Borrowed parameters cannot become owning parameters, and `once` parameters cannot satisfy reusable borrowed contracts. 

An abstract owned parameter permits captured-state mutation, so it does not promise the stronger `read` contract; readonly borrowing requires a proven readonly environment.

Callback parameters follow the same default. A bare parameter takes ownership, so the caller cannot reuse the transferred callback. Use `Mutable<Function<...>>` when an operation only needs to call it temporarily:

```tsrust
function twice(action: Mutable<Function<int>>): int {
  return action() + action();
}

function take(action: () => int): int {
  return action();
}

function main() {
  let value = 0;
  const next = (): int => {
    value = value + 1;
    return value;
  };
  console.log(twice(next)); // 3: the calls return 1 and 2
    console.log(next());     // 3: borrowing leaves next available
    console.log(take((): int => {
    return 7;
  })); // 7
}
```

## Separate environment and argument ownership

The callback environment and its arguments have separate ownership contracts. `Mutable<Function<int, [text: Owned<string>]>>` borrows the callback while transferring each supplied string into its call; consuming an argument does not itself consume the callback environment.

Scalar arguments copy. Signature arguments containing owned data require `ReadOnly<T>`, `Mutable<T>` or `Owned<T>`, such as `(text: ReadOnly<string>) => string`. An arrow parameter may state its access the same way; the body may use the value less than stated, never more. Unstated access is inferred from the body and must match the signature. Argument names aid editor help and do not change structural type identity. Invocation uses the ordinary call syntax and argument count/type checks.

## Own and move captured state

```tsrust
type State = {
  value: int;
};

function increment(value: int): int {
  return value + 1;
}

function main() {
  const state: State = {
    value: 0
  };
  const counter: ((() => int)) = (): int => {
    state.value = state.value + 1;
    return state.value;
  };
  console.log(counter(), counter());
  const moved: () => int = counter;
  console.log(moved());
  const action: (value: int) => int = increment;
  console.log(action(4));
}
```

This prints `1 2`, `3` and `5` on separate lines. Each counter call changes its private captured state. Creating the owned counter transfers `state` into that environment; moving `counter` into `moved` transfers the callable too. Using either original binding afterward is rejected.

## Restrict invocation to once

`Once` applies even to a capture-free function: a second invocation produces `closure-consumed`. An environment that consumes its captures requires a once signature; labeling it reusable produces `callback-once`. Returning an owned captured payload from a generic local, for example `const action: Once<Function<T>> = (): T => { return value; };`, retains its concrete nominal result type.

## End a borrowing scope

Borrowed captures keep the existing enclosing-block loan lifetime. Calling a borrowed local does not shorten that compiler lifetime. Put the borrowed callable in a smaller block when its owner must be mutated afterward. An owned annotation cannot retain borrowed storage from an external scope or a borrowed array element without an explicit ownership operation.

A shorter block releases the borrowed environment before later owner mutation:

```tsrust
function main() {
  const values = [3];
  {
    const read: ReadOnly<Function<int>> = (): int => values[0];
    console.log(read()); // 3
  }

  values[0] = 4;
  console.log(values[0]); // 4
}
```

## Return an owning callable

Factories can explicitly return `Once<Function<...>>` callables:

```tsrust
function make(value: int): Once<Function<int>> {
  return (): int => {
    return value;
  };
}

function main() {
  const action = make(7);
  console.log(action()); // prints 7
}
```

A second `action()` is rejected. An explicit `Once` result restricts even a reusable or capture-free environment to one invocation. Grouped annotations and generic payload results are supported.

Bare factory results continue to infer invocation from the returned environment; a factory can therefore return a naturally consuming environment without an explicit modifier. 

Returning `ReadOnly` or `Mutable` callable types is rejected because their borrowed environment cannot escape. The older `owned`/`owned once` result spellings are accepted aliases.

## Replace and select environments

These local annotations validate a concrete origin; they do not introduce arbitrary replacement between different local closure origins. [Owned callable record fields](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/docs/guides/language/callable-fields.md) provide storage and replacement between compatible environments. [Callable arrays](/tsrust/language/callable-arrays/) support construction, indexed reusable calls, replacement and explicit removal. [Nullable callable storage](/tsrust/language/nullable-callables/) also has checked ownership and invocation rules. Callable values do not acquire implicit clone or shared ownership behavior.


A conditional expression can select compatible owned environments into one checked
callable value. This selection is distinct from reassigning an annotated local to
a different concrete origin:

```tsrust
function main() {
  const flag = true;
  const action = flag ? () : int => 7: (): int => 9;
  console.log(action()); // 7
}
```
