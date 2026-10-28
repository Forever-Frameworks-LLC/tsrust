---
title: Traverse and reorder callable arrays
description: Preserve stored environments while mapping, searching, reducing and reordering.
---

[Callable arrays](/tsrust/language/callable-arrays/) own their environments.
Use methods that borrow or reorder those owners. A method that needs to clone a
stored callable is rejected.

After [native setup](/tsrust/tools/rust-setup/), save each complete example
separately as `main.tsr` and run `tsrust run main.tsr`. Comments show expected output.

## Reverse and sort in place

`reverse()` reorders environments without cloning them or resetting captured state.
It accepts no arguments and returns `void`. Once elements remain owned by the
array until extracted.

Callable arrays require an explicit comparator for `sort()`, including empty
arrays. A zero-parameter comparator returning zero preserves order. A `number`
result of `Number.NaN` also treats compared elements as equal. Both reusable and
once elements remain in the receiver. `sort()` returns `void`.

```tsrust
function main() {
  const actions: (() => int)[] = [(): int => {
    return 7;
  }, (): int => {
    return 9;
  }];
  actions.reverse();
  actions.sort((): int => {
    return 0;
  });
  console.log(actions[0](), actions[1]()); // 9 7
}
```

`toSorted()` rejects these elements because it would need to clone them.
Comparators that borrow callable element parameters are not supported; the
zero-parameter comparator does not inspect or invoke stored elements.

## Visit without borrowing an element

Zero-parameter callbacks work with `forEach`, `some`, `every` and `map`. They
observe traversal without consuming or cloning stored environments. `map`
transfers each produced result into a separate output array. Once elements remain
available for later extraction and invocation.

```tsrust
function main() {
  const actions: (Once<Function<int>>)[] = [(): int => {
    return 7;
  }];
  let visits = 0;
  actions.forEach((): void => {
    visits += 1;
  });
  const values = actions.map((): int => {
    return 3;
  });
  const finish = actions.remove(0);
  console.log(visits, values[0], finish()); // 1 3 7
}
```

## Invoke reusable elements during mapping

An arrow visitor can take an explicit `Mutable<Function<...>>` element followed by
an optional `int` index. Each element is lent exclusively for the invocation; its
captured state remains in the array. This form works with `forEach`, `map` and
`flatMap`, and with the predicates below.

```tsrust
function main() {
  const actions: (() => int)[] = [(): int => {
    return 7;
  }];
  const values = actions.map((action: Mutable<Function<int>>, index: int): int => {
    return action() + index;
  });
  console.log(values[0], actions[0]()); // 7 7
}
```

The traversal requires mutable access to the receiver. It rejects once elements,
simultaneous access to the source array and escaping the borrowed environment.
The visitor must have a concrete arrow environment: inline arrows, local arrow
bindings and arrows returned by factories are supported. Forwarded function
declarations and abstract callback parameters cannot use this mutable element form.
The visitor itself must be reusable.

[`Array.from(source, mapper)`](/tsrust/language/array-from/#invoke-reusable-callable-elements)
also supports borrowing a reusable stored element for invocation.

## Search with short-circuit predicates

`some`, `every`, `findIndex` and `findLastIndex` accept the same mutable arrow
visitor and optional index. Predicates return `boolean`. Only visited environments
are invoked or changed; the method stops when its result is determined.
`findLastIndex` visits from the end and supplies original indices.

| Method | Result | Empty input |
| --- | --- | --- |
| `some` | Whether any predicate succeeds | `false` |
| `every` | Whether every predicate succeeds | `true` |
| `findIndex` | First matching original index | `-1` |
| `findLastIndex` | Last matching original index | `-1` |

```tsrust
function main() {
  const actions: (() => int)[] = [(): int => {
    return 7;
  }, (): int => {
    return 9;
  }];
  let visited = 0;
  const found = actions.some((action: Mutable<Function<int>>): boolean => {
    visited += 1;
    return action() === 7;
  });
  console.log(found, visited); // true 1
  const last = actions.findLastIndex((action: Mutable<Function<int>>): boolean => {
    return action() > 5;
  });
  console.log(last, actions[1]()); // 1 9
}
```

Empty inputs still evaluate the visitor expression once, but never invoke it.
The environments remain in the source even when traversal stops early.

## Flatten independent results

`flatMap` transfers independently produced values into its output and flattens
returned arrays by one level. The result may contain newly created owned
callbacks, but cannot contain the borrowed input environment.

```tsrust
function main() {
  const actions: (() => int)[] = [(): int => {
    return 7;
  }];
  const result = actions.flatMap((action: Mutable<Function<int>>): (() => int)[] => {
    console.log(action()); // 7
    return [(): int => {
      return 42;
    }];
  });
  console.log(result[0](), actions[0]()); // 42 7
}
```

## Reduce with an independent accumulator

`reduce` and `reduceRight` can lend reusable elements to an arrow visitor. Supply an
independent initial accumulator and declare the element parameter
`Mutable<Function<...>>`; an optional `int` index follows it. The receiver stays
exclusively borrowed throughout traversal. Each callback result becomes the next
accumulator.

```tsrust
function main() {
  const actions: (() => int)[] = [(): int => {
    return 7;
  }, (): int => {
    return 9;
  }];
  const forward = actions.reduce((sum: int, action: Mutable<Function<int>>): int => {
    return sum * 10 + action();
  }, 0);
  const backward = actions.reduceRight((sum: int, action: Mutable<Function<int>>): int => {
    return sum * 10 + action();
  }, 0);
  console.log(forward, backward, actions[0]()); // 79 97 7
}
```

Empty input returns the seed. Seedless callable accumulators and once elements are
unsupported. The seed expression cannot mutate or consume the array while its
traversal loan is active. The same concrete-arrow and source-alias restrictions
apply as for mutable mapping.
