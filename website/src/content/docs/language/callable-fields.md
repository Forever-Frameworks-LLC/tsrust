---
title: Own callables in record fields
description: Store, replace, consume and restore owned callable environments.
---

A record field can store a `Function<...>` or a `Once<Function<...>>`. Its signature defines argument types and access, return type and whether it may be called repeatedly. Creating the field transfers an owned environment into the record. Different compatible environments can replace the field without changing the record type.

After [native setup](/tsrust/tools/rust-setup/), save each complete example separately
as `main.tsr` and run `tsrust run main.tsr`. Expected output is shown with each example.

```tsrust
type Handler = {
  run: (delta: int) => int;
};

function main() {
  const handler: Handler = {
    run: (delta: int): int => {
      return delta + 1;
    }
  };
  console.log(handler.run(2));
  handler.run = (delta: int): int => {
    return delta * 2;
  };
  console.log(handler.run(3));
}
```

This prints `3` and `6`. Reusable calls borrow the field mutably and can update captured state. Replacing a field disposes of its previous environment. Captured owned data moves into the environment; scalar captures copy. The record binding can be `const`: replacing a field changes its contents without rebinding the record.

## Check parameter access and generic types

Owned-data parameters require `ReadOnly<T>`, `Mutable<T>` or `Owned<T>` access in
the signature. An arrow body can infer its argument access, which must match the
field signature. Generic records can use type parameters in those signatures;
concrete specialization preserves nominal types and checks access.

Here the field borrows a `User` readonly, and the arrow only reads its name:

```tsrust
type User = {
  name: string;
};

type Handler<T> = {
  run: Function<int, [value: ReadOnly<T>]>;
};

function main() {
  const handler: Handler<User> = {
    run: (user: User): int => {
      return user.name.length;
    }
  };
  console.log(handler.run({
    name: "Pat"
  })); // 3
}
```

## Restore a consumed field

A `Once` field transfers its environment when called. Another call or whole-record use is rejected until that field is restored. Assigning a new compatible environment restores only that field; other consumed fields remain unavailable. A reusable environment can initialize a single-use field, but its stored contract still permits only one call.

```tsrust
type Handler = {
  run: Once<Function<string>>;
};

function main() {
  const text = "Pat";
  const handler: Handler = {
    run: (): string => {
      return text;
    }
  };
  console.log(handler.run()); // Pat
  handler.run = (): string => {
    return "Sam";
  };
  console.log(handler.run()); // Sam
}
```

## Select a compatible environment

Conditional field construction and replacement accept different compatible environments. Only the selected branch creates its environment at runtime. Both branches must match the field signature. Owned captures moved on either continuing branch are unavailable afterward; nested conditionals follow the same rule.

```tsrust
type Handler = {
  run: (value: int) => int;
};

function main() {
  const flag = true;
  const handler: Handler = {
    run: flag ? (value: int) : int => value + 1: (value: int): int => value * 2
  };
  console.log(handler.run(2)); // 3
}
```

## Move owners and use record arrays

Moving a field into another record transfers its ownership. Records containing callable fields cannot be cloned, including through enclosing records, arrays or copy-producing array methods. Record arrays support ordinary push/removal and reusable indexed invocation. To invoke a single-use field in an array, remove its owning record first; partial moves out of indexed records remain unsupported.

Bare callable field signatures own their environments. A borrowed local environment cannot be converted into an owned field merely by giving it a matching signature; use an owned environment with independent captures, or declare a [borrowing field](/tsrust/language/borrowed-callable-fields/). A stored function can move from one record field into another record.


```tsrust
type Handler = {
  run: Once<Function<int>>;
};

function main() {
  const rows: Handler[] = [{
    run: (): int => {
      return 7;
    }
  }];
  const row = rows.remove(0);
  console.log(row.run(), rows.length); // 7 0
}
```

## Current layout limits

Owned stored callable signatures with callable-valued parameters or results are
rejected with `callable-layout`. Recursive by-value record/environment cycles also
report `callable-layout`; compatible ordinary environment replacement remains
supported.

Unused callable record declarations compile without constructing an environment.
Imported records retain private nominal parameter and result types; importing a
record does not make those private type names visible in the caller.

## Use field documentation and editor help

Hover, completion details and signature help retain field argument names and access.
A field's documentation comment supplies its description and `@param` help. Calls
returning supported data keep chained member help, including generic and imported
fields. Unsaved imported parameter-name changes update the help.

See [local callable annotations](/tsrust/language/callable-locals/) for local-origin
replacement rules and [borrowing fields](/tsrust/language/borrowed-callable-fields/)
for callbacks that retain scoped captures.
