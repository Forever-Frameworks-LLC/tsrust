# Callable arrays

An annotated array can own callable environments with one compatible signature. Arrays can appear in local declarations, record fields and function parameters or results. Each element retains its own captured state. Indexing a reusable callable temporarily borrows it for invocation; replacing an element releases its previous environment.

```tsrust
function main() {
    const actions: (() => int)[] = [
        (): int => { return 7; },
        (): int => { return 9; }
    ];
    actions[0] = (): int => { return 11; };
    console.log(actions[0]()); // 11
    const moved = actions.remove(0);
    console.log(moved());      // 11
    console.log(actions[0]()); // 9
    console.log(actions.length); // 1
}
```

Nested annotated arrays support indexed calls as well. `push` and `unshift` transfer eligible environments into an array; `unshift` keeps argument order and returns the new length. Captures borrowed from another scope cannot be inserted into owning storage.

`splice` also transfers inserted callbacks and returns the removed callbacks in an independent owning array. Their captured state is preserved; no callback is cloned. For example:

```tsrust
const actions: (() => int)[] = [(): int => { return 7; }];
const removed = actions.splice(0, 1, (): int => { return 11; });
console.log(removed[0](), actions[0]()); // 7 11
```

An indexed reusable callback can be lent to a `Mutable<Function<...>>` parameter. The function may invoke it repeatedly; the environment stays in the array and remains available afterward. A removed reusable callback can be lent in the same way.

```tsrust
function invoke(action: Mutable<Function<int>>): int { return action(); }
function main() {
    const actions: (() => int)[] = [(): int => { return 7; }];
    console.log(invoke(actions[0]), actions[0]()); // 7 7
}
```

The stored and borrowed signatures must agree on argument types, argument access and result type. A once element cannot satisfy a reusable parameter. A reusable owning storage signature permits mutation, so it cannot promise a `ReadOnly<Function<...>>` contract. Overlapping mutable loans and array mutation during an element loan are rejected.

A `Once<Function<...>>` element must be removed before invoking it: indexing borrows an element, while a consuming call needs ownership. Removal shifts subsequent elements and returns the owned environment. A second invocation of that removed once callable is rejected.

```tsrust
function main() {
    const text = "Pat";
    const actions: (Once<Function<string>>)[] = [
        (): string => { return text; }
    ];
    const moved = actions.remove(0);
    console.log(moved()); // Pat
    console.log(actions.length); // 0
}
```

Callable environments cannot be cloned. Copying operations such as `slice`, `toReversed`, `concat` and `filter` reject these elements. Mutating an array while an indexed callable receiver is active is also rejected, including mutations inside its call arguments.

`reverse()` reorders callable environments in place without copying them or resetting captured state. It also supports once callbacks, which remain owned by the array until removed. The operation takes no arguments and returns `void`.

Callable arrays require an explicit comparator for `sort()`, even when empty. A zero-argument comparator returning zero preserves element order; returning `Number.NaN` also treats compared elements as equal. Owners stay in the receiver, including once callbacks. `toSorted()` rejects callable elements because it would need to clone them. Comparator contracts accepting callable elements are still being integrated.

Callbacks with no parameters work with `forEach`, `some`, `every` and `map` over callable arrays. They borrow the array without cloning or consuming its environments; `map` transfers each callback result into a separate output array. Once elements remain available for removal and invocation afterward.

`forEach`, `map` and `flatMap` also support a visitor built from an arrow function that accepts a `Mutable<Function<...>>` element, followed by an optional `int` index. Each element is lent exclusively for the invocation, and its captured state remains in the array. The traversal requires mutable receiver access and rejects a simultaneous source-array parameter, once elements and escaping the borrowed environment. The arrow can be used directly, held in a local binding or returned by a factory. Forwarded function declarations and abstract callback parameters remain in progress.

`flatMap` transfers independently produced values into its output and flattens returned arrays by one level. Those arrays may contain newly created owned callbacks; they cannot contain the borrowed input environment.

```tsrust
const actions: (() => int)[] = [(): int => { return 7; }];
const values = actions.map((action: Mutable<Function<int>>, index: int): int => {
    return action() + index;
});
console.log(values[0], actions[0]()); // 7 7
```

`some`, `every`, `findIndex` and `findLastIndex` support the same mutable arrow visitors and optional index. Their predicates return `boolean`. They stop as soon as the result is determined; `findLastIndex` visits from the end using the original indices. Only visited environments are invoked and changed. Empty arrays return `false`, `true`, `-1` and `-1`, respectively, while still evaluating the visitor expression once. The environment stays in the array even when traversal stops early.

Generic record fields retain their concrete argument types, such as `((input: ReadOnly<User>) => int)[]`. Passing a whole array follows the function's inferred access: a function that removes an element borrows the array mutably, while a function that forwards the array transfers its ownership.

Signature help preserves declared parameter names for indexed local arrays and record fields, including generic fields. Field documentation and argument descriptions remain available, and result types support chained method help. Imported record fields preserve private nominal parameter names, field navigation and unsaved declaration updates. Broader imported array signatures, complete array-operation coverage and other container integration paths remain in progress. The implementation ledger tracks their separate acceptance requirements in [callable container integration](../../milestones/callable-containers-integration.md).

`reduce` and `reduceRight` can also lend reusable callable elements to an arrow visitor. Supply an independent initial accumulator and declare the element parameter `Mutable<Function<...>>`; an optional `int` index follows it. The receiver stays exclusively borrowed during traversal. Each callback result becomes the next accumulator. Empty input returns the seed; seedless callable accumulators and once elements are unsupported.
