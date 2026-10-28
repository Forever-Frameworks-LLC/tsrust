---
title: Adapt an existing generic callback
description: Borrow or transfer one generic environment into a concrete callable contract.
---

An existing [generic function value](/tsrust/language/generic-function-values/)
can supply a concrete callable contract. This adapter uses the existing captured
environment; it does not call the factory again or reset its state.

After [native setup](/tsrust/tools/rust-setup/), save each complete example separately
as `main.tsr` and run `tsrust run main.tsr`. Comments show expected output.

## Borrow, resume and then transfer an environment

A `Mutable<Function<...>>` view lends the environment exclusively until its scope
ends. After that, the original generic value can be called again. An owned
`Function<...>` adapter transfers the environment, leaving the original unavailable.

```tsrust
function make(start: int) {
  let calls = start;
  function select<T>(value: T): T {
    calls += 1;
    console.log(calls);
    return value;
  }

  return select;
}

function main() {
  const action = make(10);
  {
    const integer: Mutable<Function<int, [value: int]>> = action;
    integer(7); // 11
  }

  action("generic"); // 12
  const text: Function<string, [value: Owned<string>]> = action;
  text("owned"); // 13
}
```

Calling `action` while its mutable adapter is live conflicts with that adapter's
exclusive access. Calling it after the owning assignment is a transferred-value
error. A single-use adapter cannot be called twice.

The argument/result relationship still comes from the generic declaration.
This identity function cannot adapt to an integer argument with a string result.
Returning a string argument also needs owned access; a read-only string contract
cannot authorize that transfer.

## Supply a generic value to an array callback

Supported array methods infer a concrete callback from their element and argument
contracts. In this example, the adapter borrows the existing environment for the
traversal; later calls continue with the same state.

```tsrust
function make(start: int) {
  let calls = start;
  function select<T>(value: T): T {
    calls += 1;
    console.log(calls);
    return value;
  }

  return select;
}

function main() {
  const action = make(10);
  const mapped = [3, 4].map(action); // 11, then 12
  console.log(mapped.join(",")); // 3,4
  action("after"); // 13
}
```

Other supported methods include reductions, sorting and searching when their
callback requirements match the source function. Each method retains its ordinary
parameter access, result and invocation constraints. See
[callable array methods](/tsrust/language/callable-array-methods/).

## Store concrete adapters in records and arrays

A typed field or callable array can select a concrete contract from an existing
generic value. Owning storage takes the corresponding environment. Separate
factory calls still create independent state.

```tsrust
type Holder = {
  run: Function<int, [value: int]>;
};

function make(start: int) {
  let calls = start;
  function select<T>(value: T): T {
    calls += 1;
    console.log(calls);
    return value;
  }

  return select;
}

function main() {
  const first = make(10);
  const holder: Holder = {
    run: first
  };
  holder.run(7); // 11
  const second = make(20);
  const actions: Function<string, [value: Owned<string>]>[] = [second];
  actions[0]("a"); // 21
  actions[0]("b"); // 22
}
```

Field borrowing and callable-array extraction rules still apply. Storing a
consuming environment does not make it reusable. See
[callable fields](/tsrust/language/callable-fields/) and
[callable arrays](/tsrust/language/callable-arrays/).

## Preserve parameters and editor help

Adapters retain default side effects and rest packs. Defaults run in the original
function when omitted or `undefined`; supplying a non-undefined value skips that
default.
Ownership of arguments and captures is not implicitly cloned. See
[captured parameters](/tsrust/language/captured-function-parameters/).

Hover and signature help show the selected concrete signature and parameter
access. Definitions on an adapter's source alias navigate to that alias's binding.
This differs from creating a new specialization directly from the
[original generic declaration](/tsrust/language/generic-function-contexts/).
