---
title: Borrow callables in record fields
description: Retain scoped callbacks while preserving capture lifetimes and field access.
---

Declare a field `ReadOnly<Function<...>>` for shared invocation or
`Mutable<Function<...>>` for a reusable callback that may change captured state.
These contracts control callback access. Actual captures determine which owner
loans the record retains and how long it may live.

After [native setup](/tsrust/tools/rust-setup/), save each complete example
separately as `main.tsr` and run `tsrust run main.tsr`. Comments show expected output.

## Call and replace fields

A readonly field cannot change what it captures. Replacing a field needs mutable
access to the record. Scalars captured by a replacement copy into its environment.

```tsrust
type Holder = {
  run: ReadOnly<Function<int>>;
};

function call(holder: ReadOnly<Holder>): int {
  return holder.run();
}

function replace(holder: Mutable<Holder>, amount: int): void {
  holder.run = (): int => amount;
}

function main() {
  const base = [3, 4];
  {
    const holder: Holder = {
      run: (): int => base[0]
    };
    console.log(call(holder)); // 3
    replace(holder, 10);
    console.log(holder.run()); // 10
    holder.run = (): int => base[1] * 2;
    const run = holder.run;
    console.log(run()); // 8
  }

  base[0] = 0;
  console.log(base[0]); // 0
}
```

While a callback still borrows `base`, changing or transferring `base` is rejected.
Leaving the block releases those callbacks before the final mutation. A moved-out
callback retains its own capture loans until it is disposed.

## Update captured state through a mutable field

Invoking a mutable field needs exclusive access to that field. Its captured
variables become available again after the callback's scope ends.

```tsrust
type Step = {
  next: Mutable<Function<int>>;
};

function main() {
  let count = 0;
  {
    const step: Step = {
      next: (): int => {
        count += 1;
        return count;
      }
    };
    step.next();
    console.log(step.next()); // 2
  }

  console.log(count); // 2
}
```

## Release each capture when it is no longer reachable

Each field keeps its own borrow. Sibling fields and nested record fields can be
passed to separate parameters of one call, replaced separately and moved out
separately. Moving a field out moves its callback and its borrow to the new binding.

Replacing a callback ends its old capture loan when the old callback is no longer
reachable. A replacement in only one branch, or a loop body that may not run, keeps
any borrow that can still be retained by the old callback. Replacing one field does
not release sibling captures.

## Store a caller-owned callback

A function may accept a record as `ReadOnly<Holder>` to invoke readonly fields, or
as `Mutable<Holder>` to invoke mutable fields and replace callbacks. Eligible
caller-owned callback parameters can be stored directly or wrapped in a field.
Their original caller loans remain active while the stored callback is reachable.

```tsrust
type Holder = {
  run: ReadOnly<Function<int>>;
};

function install(holder: Mutable<Holder>, action: ReadOnly<Function<int>>): void {
  holder.run = (): int => action();
}

function main() {
  const values = [3];
  {
    const action = (): int => values[0];
    const holder: Holder = {
      run: (): int => 0
    };
    install(holder, action);
    console.log(holder.run()); // 3
  }

  values[0] = 4;
  console.log(values[0]); // 4
}
```

The stored callback must not retain a callee-local owner after that call returns.
A shorter-lived input cannot be installed in an outer holder. Wrapping a borrowed
input does not make it an owning result. These escaping captures report
`callback-escape`.

## Use checked local containers

Concrete borrowing records can be stored in local arrays, Map values, tuples and
nullable record values. The container retains their capture loans. The records
remain noncloneable; a Map constructed by copying a pair array cannot duplicate
them. Populate a local Map with `set` to transfer an eligible record instead.

```tsrust
type Holder = {
  run: ReadOnly<Function<int>>;
};

function main() {
  const values = [3];
  const first: Holder = {
    run: (): int => values[0]
  };
  const rows: Holder[] = [first];
  console.log(rows[0].run()); // 3
  const second: Holder = {
    run: (): int => values[0]
  };
  const map = new Map<int, Holder>();
  map.set(1, second);
  const found = map.get(1);
  if (found !== undefined) {
    console.log(found.run()); // 3
  }

  const third: Holder = {
    run: (): int => values[0]
  };
  const maybe: Holder | null = third;
  if (maybe !== null) {
    console.log(maybe.run()); // 3
  }
}
```

This support depends on checked concrete environments; a declaration alone does
not establish a usable container layout. Set keys keep their ordinary supported-key
restrictions. Nullable or optional callback fields also support checked replacement;
see [nullable callables](/tsrust/language/nullable-callables/).

## Return only self-contained captures

A record cannot be returned, thrown or stored where its captured borrows would
outlive their owners. A readonly field declaration does not itself require a
borrow: capture-free callbacks and copied scalars can be self-contained.

```tsrust
type Holder = {
  run: ReadOnly<Function<int>>;
};

function make(amount: int): Holder {
  return {
    run: (): int => amount
  };
}

function main() {
  const holder = make(7);
  console.log(holder.run()); // 7
}
```

Returning the same kind of record while its callback borrows a local array is a
`callback-escape` error. Use an [owned field](/tsrust/language/callable-fields/) when
the environment must take ownership of captured data.

## Handle callback failures

A field callback can throw or encounter a checked runtime failure. The error
propagates to the function invoking the field and can be handled there or by its
callers.

```tsrust
type Holder = {
  run: ReadOnly<Function<int>>;
};

function guarded(holder: ReadOnly<Holder>): int {
  try {
    return holder.run();
  } catch (error) {
    if (typeof error === "string") {
      return error.length;
    }

    if (typeof error === "number") {
      return error + 1000;
    }

    return 0;
  }
}

function main() {
  const text: Holder = {
    run: (): int => {
      throw "four";
    }
  };
  const coded: Holder = {
    run: (): int => {
      throw 7;
    }
  };
  console.log(guarded(text), guarded(coded)); // 4 1007
}
```

The handler's error type includes failures from every callback used to construct a
record of that type in the program, including constructions in later functions.
Both string and number errors can reach `guarded`, so it narrows with `typeof`.
If no such record is constructed, the callback's failures are unknown: a handler
that binds the error reports `error-effect` rather than inventing a payload type.
