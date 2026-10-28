---
title: Adapt fresh generic factory results
description: Construct selected callback environments once and give them concrete contracts.
---

A factory can return a generic function directly into an owning callable annotation
or a supported array callback context. Construction happens once; later callback
invocations reuse the resulting environment.

After [native setup](/tsrust/tools/rust-setup/), save each complete example separately
as `main.tsr` and run `tsrust run main.tsr`. Comments show expected output.

## Construct once and infer the callback result

The returned generic function may omit its result annotation. A concrete callback
contract or traversal supplies argument types, then its checked body establishes
the result type.

```tsrust
function make(start: int) {
  console.log(start);
  let calls = start;
  function select<T>(value: T) {
    calls += 1;
    console.log(calls);
    return value;
  }

  return select;
}

function main() {
  const fixed: Function<int, [value: int]> = make(10); // 10
  fixed(7); // 11
  fixed(8); // 12
  const mapped = [3, 4].map(make(20)); // 20, then 21, then 22
  console.log(mapped.join(",")); // 3,4
}
```

Result inference cannot contradict an explicit contract. For example, this identity
callback cannot satisfy `Function<string, [value: int]>`. Adapting it does not run
its parameter defaults early or weaken argument access.

## Select a factory before evaluating its arguments

A conditional can choose between factories with different callback environments.
Only predicates on the selected path execute. The chosen factory's arguments are
then evaluated once, followed by its construction.

```tsrust
type Action = Function<int, [value: int]>;

function left(start: int) {
  console.log("left", start);
  let calls = start;
  function select<T>(value: T): T {
    calls += 1;
    console.log(calls);
    return value;
  }

  return select;
}

function right(start: int) {
  console.log("right", start);
  let calls = start;
  function choose<T>(value: T): T {
    calls += 2;
    console.log(calls);
    return value;
  }

  return choose;
}

function main() {
  let checks = 0;
  let arguments = 0;
  function check(expected: int): boolean {
    return checks++ === expected;
  }

  const fixed: Action = (check(0) ? (check(1) ? left : right) : right)(++arguments);
  // left 1
  fixed(7); // 2
  console.log(checks, arguments); // 2 1
}
```

Each possible callback must satisfy the destination's parameter, result and
invocation requirements, even when a predicate is a constant. The selected
adapter retains the chosen environment rather than constructing both alternatives.
The same selected factory-call form can supply an array traversal's callback.

Keep this selected factory call inline. Storing the selection of `left` and `right`
in a separate local first currently produces a `callable-layout` error; their
results are themselves callable environments.

## Store fresh callbacks in typed fields and arrays

Concrete aggregate destinations can supply the specialization context directly.
Each factory invocation still creates its own state.

```tsrust
type Action = Function<int, [value: int]>;

type Holder = {
  run: Action;
};

function make(start: int) {
  let calls = start;
  function select<T>(value: T) {
    calls += 1;
    console.log(calls);
    return value;
  }

  return select;
}

function main() {
  const holder: Holder = {
    run: make(10)
  };
  const actions: Action[] = [make(20)];
  holder.run(7); // 11
  actions[0](8); // 21
  actions[0](9); // 22
}
```

Storage retains its ordinary ownership rules. See
[callable fields](/tsrust/language/callable-fields/) and
[callable arrays](/tsrust/language/callable-arrays/).

## Keep nullable presence separate from captured state

A nullable owning callable can contain a fresh generic factory result. Optional
invocation checks presence and preserves the environment across successful calls.

```tsrust
type Action = Function<int, [value: int]>;

function make() {
  let calls = 0;
  function select<T>(value: T): T {
    calls += 1;
    console.log(calls);
    return value;
  }

  return select;
}

function main() {
  const present: Action | undefined = make();
  present?.(7); // 1
  present?.(8); // 2
  const absent: Action | undefined = undefined;
  console.log(absent?.(9) === undefined); // true
}
```

Optional invocation does not make a consuming callback reusable. See
[nullable callables](/tsrust/language/nullable-callables/) for presence narrowing,
argument evaluation and ownership.

## Give borrowed adapters an owner that stays in scope

A borrowed `Mutable<Function<...>>` view cannot retain a fresh temporary environment
that would end before the view. Store the generic factory result in a local first,
then borrow that local as shown in
[existing generic adapters](/tsrust/language/generic-value-adapters/).

An owned environment also cannot retain borrowed factory parameter data as an
owner. Supply owned captures or explicitly clone when an independent copy is
needed. Factory or callback failure follows the normal cleanup rules; adaptation
does not duplicate the owners.

Editor help preserves factory navigation and shows the concrete callback result,
including results inferred from a generic body and optional-call absence.
