---
title: "Function values"
description: "Store callables in locals, record fields and arrays, and handle nullable callables."
---

A function value carries its captured environment. Where it is stored decides who owns that environment: an owned local, field or array element keeps it; a `ReadOnly` or `Mutable` borrow uses it for a limited scope and cannot escape; a `Once` value can be called a single time. Nullable callables must be guarded or called with `?.()`. The type of a function value is written `Function<R, [A]>` or as an arrow type; see [Function](../../library/globals/#function).

To run an example, save it as `main.tsr` after [native setup](../../tools/rust-setup/) and run `tsrust run main.tsr`. Examples that use `strictEqual` finish without output when their assertions pass; others show expected output in comments.

## Local function values

Callable annotations own environments by default. Use `ReadOnly<Function<..., [...]>>` or `Mutable<Function<..., [...]>>` for scoped borrowing; `Once<Function<..., [...]>>` owns an environment consumed by invocation. `Owned<Function<...>>` means the same as plain `Function<...>`. Arrow type spelling such as `(value: int) => int` remains accepted and the wrappers compose with it.

A local `const` or `let` can declare a callable signature. The compiler checks argument types, access modes and the result, then retains the concrete function or closure environment. Parentheses around the signature are permitted.

In this table, `F` stands for `Function<int, [value: int]>` (a callable taking an
`int` and returning an `int`).

| Annotation | Behavior |
| --- | --- |
| `F` | Owns its environment; repeated calls borrow it temporarily. |
| `Once<F>` | The environment moves with the binding and can be invoked at most once. |
| `ReadOnly<F>` | Borrows an environment whose captured state stays unchanged. |
| `Mutable<F>` | Borrows a reusable environment whose captured state may change. |

### Borrow a callable for a temporary call

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

### Separate environment and argument ownership

The callback environment and its arguments have separate ownership contracts. `Mutable<Function<int, [text: Owned<string>]>>` borrows the callback while transferring each supplied string into its call; consuming an argument does not itself consume the callback environment.

Scalar arguments copy. Signature arguments containing owned data require `ReadOnly<T>`, `Mutable<T>` or `Owned<T>`, such as `(text: ReadOnly<string>) => string`. An arrow parameter may state its access the same way; the body may use the value less than stated, never more. Unstated access is inferred from the body and must match the signature. Argument names aid editor help and do not change structural type identity. Invocation uses the ordinary call syntax and argument count/type checks.

### Own and move captured state

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

### Restrict invocation to once

`Once` applies even to a capture-free function: a second invocation produces `closure-consumed`. An environment that consumes its captures requires a once signature; labeling it reusable produces `callback-once`. Returning an owned captured payload from a generic local, for example `const action: Once<Function<T>> = (): T => { return value; };`, retains its concrete nominal result type.

### End a borrowing scope

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

### Return an owning callable

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

### Replace and select environments

These local annotations validate a concrete origin; they do not introduce arbitrary replacement between different local closure origins. [Owned callable record fields](/tsrust/language/function-values/#owned-callable-fields) provide storage and replacement between compatible environments. [Callable arrays](/tsrust/language/function-values/#callable-arrays) support construction, indexed reusable calls, replacement and explicit removal. [Nullable callable storage](/tsrust/language/function-values/#nullable-callables) also has checked ownership and invocation rules. Callable values do not acquire implicit clone or shared ownership behavior.

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

## Owned callable fields

A record field can store a `Function<...>` or a `Once<Function<...>>`. Its signature defines argument types and access, return type and whether it may be called repeatedly. Creating the field transfers an owned environment into the record. Different compatible environments can replace the field without changing the record type.

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

### Check parameter access and generic types

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

### Restore a consumed field

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

### Select a compatible environment

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

### Move owners and use record arrays

Moving a field into another record transfers its ownership. Records containing callable fields cannot be cloned, including through enclosing records, arrays or copy-producing array methods. Record arrays support ordinary push/removal and reusable indexed invocation. To invoke a single-use field in an array, remove its owning record first; partial moves out of indexed records remain unsupported.

Bare callable field signatures own their environments. A borrowed local environment cannot be converted into an owned field merely by giving it a matching signature; use an owned environment with independent captures, or declare a [borrowing field](/tsrust/language/function-values/#borrowed-callable-fields). A stored function can move from one record field into another record.

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

### Current layout limits

Owned stored callable signatures with callable-valued parameters or results are
rejected with `callable-layout`. Recursive by-value record/environment cycles also
report `callable-layout`; compatible ordinary environment replacement remains
supported.

Unused callable record declarations compile without constructing an environment.
Imported records retain private nominal parameter and result types; importing a
record does not make those private type names visible in the caller.

### Use field documentation and editor help

Hover, completion details and signature help retain field argument names and access.
A field's documentation comment supplies its description and `@param` help. Calls
returning supported data keep chained member help, including generic and imported
fields. Unsaved imported parameter-name changes update the help.

See [local callable annotations](/tsrust/language/function-values/#local-function-values) for local-origin
replacement rules and [borrowing fields](/tsrust/language/function-values/#borrowed-callable-fields)
for callbacks that retain scoped captures.

## Borrowed callable fields

Declare a field `ReadOnly<Function<...>>` for shared invocation or
`Mutable<Function<...>>` for a reusable callback that may change captured state.
These contracts control callback access. Actual captures determine which owner
loans the record retains and how long it may live.

### Call and replace fields

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

### Update captured state through a mutable field

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

### Release each capture when it is no longer reachable

Each field keeps its own borrow. Sibling fields and nested record fields can be
passed to separate parameters of one call, replaced separately and moved out
separately. Moving a field out moves its callback and its borrow to the new binding.

Replacing a callback ends its old capture loan when the old callback is no longer
reachable. A replacement in only one branch, or a loop body that may not run, keeps
any borrow that can still be retained by the old callback. Replacing one field does
not release sibling captures.

### Store a caller-owned callback

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

### Use checked local containers

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
see [nullable callables](/tsrust/language/function-values/#nullable-callables).

### Return only self-contained captures

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
`callback-escape` error. Use an [owned field](/tsrust/language/function-values/#owned-callable-fields) when
the environment must take ownership of captured data.

### Handle callback failures

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

## Callable arrays

An array can own callable environments with one compatible signature. Each element
retains its own captured state. Arrays can appear in local declarations, nested
arrays, record fields, function parameters and results.

### Declare and invoke elements

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

### Insert and replace owners

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

### Lend a reusable element

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

### Extract a once callable before invoking it

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

### Pass whole arrays and use editor help

Passing the array follows the function's checked access. A function that removes
an element borrows the array mutably. Returning the array as an owning result
transfers ownership. Generic record fields retain concrete argument types, for example
`((input: ReadOnly<User>) => int)[]` when `User` is declared in scope.

Signature help for indexed local arrays and record fields preserves declared
parameter names. Field documentation, argument descriptions and chained result
help remain available. Imported record fields preserve their nominal parameter
types, declaration navigation and unsaved declaration updates.

### Choose methods that preserve ownership

Callable environments cannot be cloned. Copying operations including `slice`,
`toReversed`, `concat`, `filter` and unmapped `Array.from` reject owning callable
elements. Reordering and traversal have separate rules; see
[callable array methods](/tsrust/language/function-values/#array-methods-on-callables).

## Array methods on callables

[Callable arrays](/tsrust/language/function-values/#callable-arrays) own their environments.
Use methods that borrow or reorder those owners. A method that needs to clone a
stored callable is rejected.

### Reverse and sort in place

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

### Visit without borrowing an element

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

### Invoke reusable elements during mapping

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

[`Array.from(source, mapper)`](/tsrust/language/arrays/#invoke-reusable-callable-elements)
also supports borrowing a reusable stored element for invocation.

### Search with short-circuit predicates

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

### Flatten independent results

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

### Reduce with an independent accumulator

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

## Nullable callables

A callable type can include `null`, `undefined`, or both. Parenthesize the callable before adding the absent alternatives. `(value: int) => int | null` describes a callable returning a nullable integer; `((value: int) => int) | null` describes a nullable callable.

### Guard ordinary invocation

Use a strict guard before ordinary invocation. The guard checks the tag without copying the environment. A callable retains its argument access and invocation contracts when narrowed.

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

### Keep the callable access contract

`Owned<Function<...>>` means the same as the default owning `Function<...>`. A `ReadOnly<Function<...>>` or `Mutable<Function<...>>` borrows its environment; `Once<Function<...>>` owns an environment that can be invoked once. A callback type may take a nullable function argument, such as `ReadOnly<Function<int, [inner: ReadOnly<Function<int>> | null]>>`. Adding an absent alternative does not make any environment cloneable, permit a borrowed environment to escape, or permit a consumed environment to be used again.

### Skip absent calls

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

### Initialize an absent callable

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

### Convert absent tags explicitly

`String` rejects an entire nullable type containing a callable payload, even if its current tag is absent. A strict guard can narrow the value to an accepted absent alternative first.

```tsrust
function main() {
  const action: (Function<int, [value: int]>) | null = null;
  if (action === null) {
    console.log(String(action)); // null
    }
}
```

### Replace a borrowed callback

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
