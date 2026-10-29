---
title: "Generic functions"
description: "Generic function values, specialization, enclosing type parameters, adapters and factories."
---

Generic functions declare type parameters such as `<T>` and are specialized for each use. A generic function value can stay generic, so each call infers its own type arguments, or be given a concrete contract by an annotation or call context. Nested functions can use the type parameters of their enclosing function. Adapting an existing generic value borrows or transfers its environment; a fresh factory result is constructed once for its concrete contract.

To run an example, save it as `main.tsr` after [native setup](../../tools/rust-setup/) and run `tsrust run main.tsr`. Examples that use `strictEqual` finish without output when their assertions pass; others show expected output in comments.

## Generic function values

Storing a generic declaration without a concrete callable annotation preserves its
generic relationship. Each invocation can select different types while checking
the original parameter, result and access requirements.

### Call one value with different types

```tsrust
function identity<T>(value: T): T {
  return value;
}

function size<T>(values: ReadOnly<T[]>): int {
  return values.length;
}

function main() {
  const action = identity;
  console.log(action(7)); // 7
  console.log(action("text")); // text
  const values = [3, 4];
  const length = size;
  console.log(length(values), values.length); // 2 2
  const returned = action(values);
  console.log(returned.join(",")); // 3,4
}
```

`length` borrows the array, so `values` remains available afterwards. `action`
returns its argument as an owner: after `action(values)`, use `returned` instead
of the transferred `values`. Changing the argument type does not bypass ownership.

Generic constraints remain checked. For example, a function whose two parameters
share type `T` cannot receive incompatible types for them. Required arguments must
still be supplied, and a result annotation cannot contradict the inferred type.

### Replace a value with another generic declaration

A mutable local holding a capture-free generic declaration can be reassigned to
another one. Each call is checked against every declaration that can reach it, and
a saved copy keeps the declaration it held:

```tsrust
function first<T>(value: T): T {
  return value;
}

function second<T>(left: T, right: T): T {
  return right;
}

function choose(flag: boolean): int {
  let action = first;
  const saved = action;
  action = first;
  if (flag) {
    action = second;
  }

  if (flag) {
    return saved<int>(3) + action<int>(4, 7);
  }

  return saved<int>(3) + action<int>(4);
}

function main() {
  console.log(choose(false), choose(true)); // 7 10
}
```

- The checker follows assignments through branches, loops, labeled exits and
  `try`/`catch`/`finally`. A call that some reachable declaration cannot accept is
  rejected: calling `action<int>(i)` after a branch that may assign `second` reports
  `arity`.
- When one reachable declaration reads an argument (`ReadOnly<T[]>`) and another
  takes it (`Owned<T[]>`), the call moves the argument: later use of the variable
  reports `transferred`. Signature help shows the stronger requirement. Omitted
  defaults use the selected declaration's initializer.
- Callback arguments must have the same signature and ownership contract in every
  reachable declaration.
- Reassigning inside a loop so that a later iteration calls the new declaration is
  rejected with `callable-layout`. Select the declaration before the loop instead.
- Assignment never clones an owned callable. After moving a value into another
  local, assign the mutable local again before calling it.

### Share one captured environment across calls

A returned generic callback keeps one environment across its concrete invocations.
Separate calls to its factory create independent environments.

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
  const first = make(10);
  const second = make(20);
  first(1); // 11
  second("x"); // 21
  first("y"); // 12
}
```

A new argument type does not reset the counter or clone the environment. If a
callback consumes a captured owner, it cannot be invoked again with any type.
Locally borrowed captures retain their capture loans; see
[nested-function captures](/tsrust/language/functions/#captures).

Default and optional parameters retain their omission behavior. Generic rest
parameters build a checked rest pack for each invocation. See
[captured parameters](/tsrust/language/functions/#parameters-of-captured-functions).

### Supply context for result-only types

An expected result annotation can select generic types when arguments provide no
information about them.

```tsrust
function empty<T>(): T[] {
  return [];
}

function main() {
  const action = empty;
  const numbers: int[] = action();
  const strings: string[] = action();
  numbers.push(7);
  strings.push("text");
  console.log(numbers[0], strings[0]); // 7 text
}
```

The two calls produce separately typed arrays. A result-only call without argument
or expected-result information still needs type context. This inference does not
relax parameter types or required argument counts.

### Inspect generic and concrete editor help

Hover and completion preserve the source generic signature for an alias, including
imported values and returned captured callbacks.

Signature help at a checked call shows the concrete types and access: the string identity call reports
`value: Owned<string>`, while `size` reports `values: ReadOnly<int[]>` for an integer
array. Definitions navigate back to the source declaration.

Use a [concrete callable context](/tsrust/language/generics/#specialization) when
a stored value should accept a fixed signature. [Concrete adapters](/tsrust/language/generics/#adapting-existing-generic-values) can also retain an
existing generic environment; their borrowing and transfer rules still apply.

## Specialization

A concrete callable annotation selects one specialization of a generic declaration.
The checker uses the annotation's parameter and result types while preserving the
relationships declared by the generic function. The resulting value accepts that
concrete contract, rather than arbitrary future argument types.

### Choose concrete arguments and results

```tsrust
function identity<T>(value: T): T {
  return value;
}

function main() {
  const numberValue: (value: int) => int = identity;
  const stringValue: (value: Owned<string>) => string = identity;
  console.log(numberValue(7)); // 7
  console.log(stringValue("text")); // text
}
```

Returning a string argument transfers it, so its contract uses `Owned<string>`.
An annotation such as `(value: int) => string` cannot specialize this identity
function: its argument and result must have the same type. Passing a string to
`numberValue` is also an error.

### Infer from a result or preserve mutation

Result context can select a type even without an argument. A mutating generic
operation needs a contract that permits its argument mutation.

```tsrust
function empty<T>(): T[] {
  return [];
}

function takeFirst<T>(values: T[]): T {
  return values.remove(0);
}

function main() {
  const fresh: () => int[] = empty;
  console.log(fresh().length); // 0
  const head: (values: Mutable<int[]>) => int = takeFirst;
  const values = [7, 8];
  console.log(head(values), values.length); // 7 1
}
```

The caller observes the removal. Specialization does not add a clone or weaken
argument access. An owning argument remains transferred after its call.

### Choose borrowed or owned captured state

The callable's access contract also controls its environment. A mutable borrowed
view can update an enclosing scalar; an owned callable retains its own scalar
capture. These are different environments even when both come from one declaration.

```tsrust
function main() {
  let calls = 0;
  function select<T>(value: T): T {
    calls += 1;
    return value;
  }

  {
    const borrowed: Mutable<Function<int, [value: int]>> = select;
    borrowed(1);
    borrowed(2);
  }

  console.log(calls); // 2
  {
    const owned: (value: int) => int = select;
    owned(3);
    owned(4);
  }

  console.log(calls); // 2
}
```

A borrowed view keeps its capture loans until its scope ends. Owned captures use
normal transfer rules for owner types; a callback that consumes a capture is
single-use. See [captured state](/tsrust/language/functions/#captures) and
[local callable contracts](/tsrust/language/function-values/#local-function-values).

### Preserve default and rest parameters

```tsrust
function offset<T>(value: T, index: int = 0): T {
  return value;
}

function first<T>(...values: T[]): T {
  return values.remove(0);
}

function main() {
  const choose: (value: int, index?: int) => int = offset;
  console.log(choose(7, undefined)); // 7
  const pick: (...values: int[]) => int = first;
  console.log(pick(3, 4)); // 3
}
```

Omission or `undefined` evaluates a default in the original function once. A
supplied `null` must satisfy the declared type. Every rest element must satisfy
the selected element type, and the rest pack belongs to that invocation. See
[captured parameter behavior](/tsrust/language/functions/#parameters-of-captured-functions).

To keep one value generic across calls, omit its concrete callable annotation;
continue with [generic function values](/tsrust/language/generics/#generic-function-values).

## Enclosing type parameters

A nested function can use its enclosing function's type parameters. Each concrete
outer invocation supplies those types to the helper. The helper's own explicit
type parameters remain separate.

### Inherit an outer type without repeating it

```tsrust
function outer<T>(value: T): T {
  function echo<U>(other: T, unused: U): T {
    return other;
  }

  return echo<boolean>(value, true);
}

function main() {
  console.log(outer<int>(7), outer<string>("owned")); // 7 owned
}
```

`echo` inherits `T`, but declares only `U`. Its explicit call therefore supplies
one type argument. A helper with no type parameters of its own takes none, even
when its parameters mention the outer `T`.

Explicit and inferred calls follow this rule, as do recursive peers, saved
function values and imported outer functions. An extra explicit type argument
is an error; inheritance does not change the helper's public type-argument count.

### Shadow an outer type parameter deliberately

An inner declaration can introduce its own `T`. That name refers to the inner
parameter inside that declaration, independently of the enclosing function's `T`.

```tsrust
function outer<T>(value: T): string {
  function identity<T>(other: T): T {
    return other;
  }

  const number = identity<int>(7);
  const text = identity<string>("inner");
  return String(number) + ":" + text;
}

function main() {
  console.log(outer<boolean>(true)); // 7:inner
}
```

Use distinct names when shadowing would make the type relationship harder to read.

### Retain typed captures when returning a helper

The returned callback keeps the lexical types and captures of its factory call.
Separate calls create independent environments.

```tsrust
function factory<T>(value: T) {
  function take(): T {
    return value;
  }

  return take;
}

function main() {
  const number = factory<int>(7);
  const text = factory<string>("owned");
  console.log(number(), text()); // 7 owned
}
```

Returning the string transfers its captured owner. Calling `text()` again is a
consumed-callback error. Preserving a lexical type does not clone its value or
relax capture ownership. Callback failure also cleans up its original owners.
See [nested-function captures](/tsrust/language/functions/#captures).

### Pass a generic callable through another factory

A generic function can retain a callable without erasing that callable's own
type parameters. Here the selector transfers its captured callback to the caller.

```tsrust
function identity<T>(value: T): T {
  return value;
}

function retain<A>(action: A) {
  function select() {
    return action;
  }

  return select;
}

function main() {
  const select = retain(identity);
  const saved = select();
  console.log(saved<int>(7), saved<string>("Pat")); // 7 Pat
}
```

The selector cannot return the same owned callable twice. The returned `saved`
value can still use different type arguments when its capture access permits
reuse. A callable that consumes a captured owner remains single-use, and a
borrowed environment cannot escape as an owner through this factory.

The original parameter counts, type relationships and argument access remain
checked. Separate factory calls retain independent environments. See
[generic function values](/tsrust/language/generics/#generic-function-values).

### Read source types and concrete call help

Editor help inside the generic declaration preserves source types such as `T`.
Concrete outer results have their checked types, and definitions navigate to the
nested declaration.

A callable passed through a generic factory keeps concrete signature help for each invocation, including its original argument count.

## Adapting existing generic values

An existing [generic function value](/tsrust/language/generics/#generic-function-values)
can supply a concrete callable contract. This adapter uses the existing captured
environment; it does not call the factory again or reset its state.

### Borrow, resume and then transfer an environment

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

### Supply a generic value to an array callback

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
[callable array methods](/tsrust/language/function-values/#array-methods-on-callables).

### Store concrete adapters in records and arrays

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
[callable fields](/tsrust/language/function-values/#owned-callable-fields) and
[callable arrays](/tsrust/language/function-values/#callable-arrays).

### Preserve parameters and editor help

Adapters retain default side effects and rest packs. Defaults run in the original
function when omitted or `undefined`; supplying a non-undefined value skips that
default.
Ownership of arguments and captures is not implicitly cloned. See
[captured parameters](/tsrust/language/functions/#parameters-of-captured-functions).

Hover and signature help show the selected concrete signature and parameter
access. Definitions on an adapter's source alias navigate to that alias's binding.
This differs from creating a new specialization directly from the
[original generic declaration](/tsrust/language/generics/#specialization).

For a direct factory call or a selected factory expression, continue with
[fresh generic factory results](/tsrust/language/generics/#fresh-generic-factories).

## Fresh generic factories

A factory can return a generic function directly into an owning callable annotation
or a supported array callback context. Construction happens once; later callback
invocations reuse the resulting environment.

### Construct once and infer the callback result

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

### Select a factory before evaluating its arguments

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

### Store fresh callbacks in typed fields and arrays

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
[callable fields](/tsrust/language/function-values/#owned-callable-fields) and
[callable arrays](/tsrust/language/function-values/#callable-arrays).

### Keep nullable presence separate from captured state

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
[nullable callables](/tsrust/language/function-values/#nullable-callables) for presence narrowing,
argument evaluation and ownership.

### Give borrowed adapters an owner that stays in scope

A borrowed `Mutable<Function<...>>` view cannot retain a fresh temporary environment
that would end before the view. Store the generic factory result in a local first,
then borrow that local as shown in
[existing generic adapters](/tsrust/language/generics/#adapting-existing-generic-values).

An owned environment also cannot retain borrowed factory parameter data as an
owner. Supply owned captures or explicitly clone when an independent copy is
needed. Factory or callback failure follows the normal cleanup rules; adaptation
does not duplicate the owners.

Editor help preserves factory navigation and shows the concrete callback result,
including results inferred from a generic body and optional-call absence.
