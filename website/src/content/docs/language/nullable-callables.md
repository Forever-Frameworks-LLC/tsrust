---
title: Work with nullable callables
description: Guard, invoke and store callable values that may be absent.
---

A callable type can include `null`, `undefined`, or both. Parenthesize the callable before adding the absent alternatives. `(value: int) => int | null` describes a callable returning a nullable integer; `((value: int) => int) | null` describes a nullable callable.

## Guard ordinary invocation

Use a strict guard before ordinary invocation. The guard checks the tag without copying the environment. A callable retains its argument access and invocation contracts when narrowed.

After [native setup](/tsrust/tools/rust-setup/), run each complete example separately as `main.tsr` with
`tsrust run main.tsr`. Console comments show expected output.

```tsrust
function main() {
  const action: (Function<int, [input: int]>) | null = (value: int): int => {
    return value + 1;
  };
  if (action !== null) {
    console.log(action(4)); // 5
    }
}
```

## Keep the callable access contract

`Owned<Function<...>>` means the same as the default owning `Function<...>`. A `ReadOnly<Function<...>>` or `Mutable<Function<...>>` borrows its environment; `Once<Function<...>>` owns an environment that can be invoked once. A callback type may take a nullable function argument, such as `ReadOnly<Function<int, [inner: ReadOnly<Function<int>> | null]>>`. Adding an absent alternative does not make any environment cloneable, permit a borrowed environment to escape, or permit a consumed environment to be used again.

## Skip absent calls

Optional invocation skips argument evaluation when the callable is absent and returns `undefined`. A present callable returns its ordinary result, including any existing absent alternatives.

```tsrust
function main() {
  const action: (Function<int, [value: int]>) | undefined = (value: int): int => {
    return value + 1;
  };
  const missing: (Function<int, [value: int]>) | undefined = undefined;
  console.log(String(action?.(2)), String(missing?.(2))); // 3 undefined
}
```

## Initialize an absent callable

Nullish assignment evaluates its fallback only when the destination is absent. Its expression result is a readonly alias. Assign an owning reusable environment, then invoke the narrowed destination when the call needs mutable access to that environment.

```tsrust
function main() {
  let action: (Function<int, [value: int]>) | null = null;
  action ??= (value: int): int => {
    return value + 2;
  };
  if (action !== null) {
    console.log(action(5)); // 7
    }
}
```

## Convert absent tags explicitly

`String` rejects an entire nullable type containing a callable payload, even if its current tag is absent. A strict guard can narrow the value to an accepted absent alternative first.

```tsrust
function main() {
  const action: (Function<int, [value: int]>) | null = null;
  if (action === null) {
    console.log(String(action)); // null
    }
}
```

## Replace a borrowed callback

A nullable readonly callback field can retain different compatible callback
environments. Replacing the field changes which callback it invokes without
changing the original callback variables. This complete example finishes without
output:

```tsrust
import { strictEqual } from "std:testing";

type Holder = {
  run: ReadOnly<Function<int>> | undefined;
};

function install(holder: Mutable<Holder>, action: ReadOnly<Function<int>>): void {
  holder.run = action;
}

function main() {
  const firstValues = [3], secondValues = [10];
  const first = (): int => firstValues[0];
  const second = (): int => secondValues[0];
  const holder: Holder = {
    run: undefined
  };
  install(holder, first);
  strictEqual(holder.run!(), 3);
  install(holder, second);
  strictEqual(first(), 3);
  strictEqual(holder.run!(), 10);
}
```

The `!` assertions above follow successful installation in this example. For an
unknown field, guard its absence or use optional invocation.

Storage must preserve the callback's checked argument access, invocation kind and
borrowed lifetime. A callback that captures callee-local data cannot escape by
being stored in a caller-owned holder. Matching signatures alone do not authorize
all replacements: concrete origins and checked storage compatibility still matter.

## Related guides

- [Optional values: chaining, fallbacks and assignment](/tsrust/language/optional-values/)
