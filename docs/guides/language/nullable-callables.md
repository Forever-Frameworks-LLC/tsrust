# Nullable callable values

A callable type can include `null`, `undefined`, or both. Parenthesize the callable before adding the absent alternatives. `(value: int) => int | null` describes a callable returning a nullable integer; `((value: int) => int) | null` describes a nullable callable.

Use a strict guard before ordinary invocation. The guard checks the tag without copying the environment. A callable retains its argument access and invocation contracts when narrowed.

```tsrust
function main() {
    const action: (Function<int, [input: int]>) | null =
        (value: int): int => { return value + 1; };
    if (action !== null) {
        console.log(action(4)); // 5
    }
}
```

`Owned<Function<...>>` means the same as the default owning `Function<...>`. A `ReadOnly<Function<...>>` or `Mutable<Function<...>>` borrows its environment; `Once<Function<...>>` owns an environment that can be invoked once. A callback type may take a nullable function argument, such as `ReadOnly<Function<int, [inner: ReadOnly<Function<int>> | null]>>`. Adding an absent alternative does not make any environment cloneable, permit a borrowed environment to escape, or permit a consumed environment to be used again.

Optional invocation skips argument evaluation when the callable is absent and returns `undefined`. A present callable returns its ordinary result, including any existing absent alternatives.

```tsrust
function main() {
    const action: (Function<int, [value: int]>) | undefined =
        (value: int): int => { return value + 1; };
    const missing: (Function<int, [value: int]>) | undefined = undefined;
    console.log(String(action?.(2)), String(missing?.(2))); // 3 undefined
}
```

Nullish assignment evaluates its fallback only when the destination is absent. Its expression result is a readonly alias. Assign an owning reusable environment, then invoke the narrowed destination when the call needs mutable access to that environment.

```tsrust
function main() {
    let action: (Function<int, [value: int]>) | null = null;
    action ??= (value: int): int => { return value + 2; };
    if (action !== null) {
        console.log(action(5)); // 7
    }
}
```

`String` rejects an entire nullable type containing a callable payload, even if its current tag is absent. A strict guard can narrow the value to an accepted absent alternative first.

```tsrust
function main() {
    const action: (Function<int, [value: int]>) | null = null;
    if (action === null) {
        console.log(String(action)); // null
    }
}
```

Concrete callable origins and borrowed lifetimes remain part of the contract. Nullable storage does not allow arbitrary replacement between distinct borrowed callable origins. Owning stored callable families retain the existing checked compatibility and layout restrictions.
