---
title: Store callables in arrays
description: Own, borrow, replace and extract callable environments safely.
---

An array can own callable environments with one compatible signature. Each element
retains its own captured state. Arrays can appear in local declarations, nested
arrays, record fields, function parameters and results.

After [native setup](/tsrust/tools/rust-setup/), save each complete example
separately as `main.tsr` and run `tsrust run main.tsr`. Comments show expected output.

## Declare and invoke elements

Annotate the element signature when the array is empty or its contract needs to be
explicit. Indexing a reusable callable borrows its stored environment for the call.
Replacing an element releases its previous environment.

```tsrust
function main() {
  const actions: (() => int)[] = [(): int => {
    return 7;
  }, (): int => {
    return 9;
  }];
  actions[0] = (): int => {
    return 11;
  };
  console.log(actions[0]()); // 11
  const moved = actions.remove(0);
  console.log(moved(), actions[0](), actions.length); // 11 9 1
}
```

The compiler can infer owning callable storage from a checked first element.
Remaining elements must satisfy the same argument, access and result contract.
This does not make incompatible callable signatures interchangeable.

```tsrust
function main() {
  const actions = [(value: int): int => {
    return value + 1;
  }, (value: int): int => {
    return value + 2;
  }];
  console.log(actions[0](3), actions[1](3)); // 4 5
}
```

## Insert and replace owners

`push` and `unshift` transfer eligible environments into the array. `unshift` keeps
supplied argument order and returns the new length. `splice` transfers inserted
callbacks and returns removed callbacks in an independent owning array. No callback
is cloned, and removed environments retain their captured state.

```tsrust
function main() {
  const actions: (() => int)[] = [(): int => {
    return 7;
  }];
  const removed = actions.splice(0, 1, (): int => {
    return 11;
  });
  console.log(removed[0](), actions[0]()); // 7 11
  actions.push((): int => {
    return 13;
  });
  console.log(actions.unshift((): int => {
    return 5;
  })); // 3
  console.log(actions[0](), actions[2]()); // 5 13
}
```

An environment that still borrows another scope's owner cannot be moved into owning
array storage. Capture and lifetime checks apply when constructing or inserting an
element; storing a callback does not extend the lifetime of its borrowed captures.

## Lend a reusable element

An indexed reusable callback can be lent to a `Mutable<Function<...>>` parameter.
The callee may invoke it repeatedly, and its environment stays in the array.
A removed reusable callback can be lent in the same way.

```tsrust
function invoke(action: Mutable<Function<int>>): int {
  return action();
}

function main() {
  const actions: (() => int)[] = [(): int => {
    return 7;
  }];
  console.log(invoke(actions[0]), actions[0]()); // 7 7
}
```

The stored and borrowed signatures must agree on argument types, argument access
and result type. A once element cannot satisfy a reusable parameter. An owning
reusable storage signature permits mutation, so it cannot promise a
`ReadOnly<Function<...>>` contract.

Overlapping mutable loans are rejected. The source array cannot be mutated while
an indexed callable receiver is active, including inside its call arguments.

## Extract a once callable before invoking it

A `Once<Function<...>>` call needs ownership; indexing only borrows the element.
Use `remove(index)` to obtain the owned environment. Removal shifts later elements
left. A second invocation of the removed once callable is rejected.

```tsrust
function main() {
  const text = "Pat";
  const actions: (Once<Function<string>>)[] = [(): string => {
    return text;
  }];
  const moved = actions.remove(0);
  console.log(moved(), actions.length); // Pat 0
}
```

## Pass whole arrays and use editor help

Passing the array follows the function's checked access. A function that removes
an element borrows the array mutably. Returning the array as an owning result
transfers ownership. Generic record fields retain concrete argument types, for example
`((input: ReadOnly<User>) => int)[]` when `User` is declared in scope.

Signature help for indexed local arrays and record fields preserves declared
parameter names. Field documentation, argument descriptions and chained result
help remain available. Imported record fields preserve their nominal parameter
types, declaration navigation and unsaved declaration updates.

## Choose methods that preserve ownership

Callable environments cannot be cloned. Copying operations including `slice`,
`toReversed`, `concat`, `filter` and unmapped `Array.from` reject owning callable
elements. Reordering and traversal have separate rules; see
[callable array methods](/tsrust/language/callable-array-methods/).
