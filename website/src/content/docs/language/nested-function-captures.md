---
title: Capture state in nested functions
description: Borrow enclosing bindings, store local callbacks and return independent environments.
---

A [nested declaration](/tsrust/language/nested-functions/) can read or change
bindings from its enclosing functions. Direct calls use those bindings, so a change
is visible to later calls and to the enclosing code.

After [native setup](/tsrust/tools/rust-setup/), save each complete example separately
as `main.tsr` and run `tsrust run main.tsr`. Comments show expected output.

## Share state across direct calls

```tsrust
function main() {
  let total = 0;
  const history: int[] = [];
  function add(amount: int): void {
    total += amount;
    history.push(amount);
  }

  function addTwice(amount: int): void {
    add(amount);
    add(amount);
  }

  addTwice(2);
  add(1);
  console.log(total, history.join(",")); // 5 2,2,1
}
```

Reassigning a captured scalar requires `let`; assigning to a `const` scalar is an
error. A `const` array binding can still permit element mutation and methods such
as `push`, subject to ownership checks.

A direct call borrows each capture with the access its body needs, including
captures used by other nested functions it calls. Loans end after the call unless
its result retains them. Transferring a captured owner makes the original binding
unavailable; ending the call does not restore an owner that was transferred.

## Initialize captures before use

The function may be declared before a captured binding. That binding must have a
value on every path reaching a call or a use of the function as a value.

```tsrust
function main() {
  function read(): int {
    return value;
  }

  const value = 3;
  console.log(read()); // 3
}
```

Moving `read()` before `const value = 3` produces an uninitialized-binding error.
Initializing a shadowing variable in an inner block does not initialize this binding.
A failing statement can also leave a capture uninitialized on a `catch` path.

An unused nested declaration is still type checked, but its presence alone does
not borrow its inputs or require them to be initialized for an outer call. Calling
or retaining it makes those capture requirements live.

## Keep stored callback loans in scope

Using a captured declaration as a value follows the capture rules of an
[arrow callback](/tsrust/language/arrow-functions/). A stored callback can keep a
capture loan alive beyond an individual invocation.

```tsrust
function main() {
  const base = [10];
  function shifted(value: int): int {
    return value + base[0];
  }

  console.log([1, 2].map(shifted).join(",")); // 11,12

  const counts = [0];
  function tick(): int {
    counts[0] += 1;
    return counts[0];
  }

  {
    const run = tick;
    console.log(run(), run()); // 1 2
  }

  console.log(counts[0]); // 2
}
```

Here `run` holds mutable access to `counts` until its block ends. Reading `counts`
while that callback remains in scope conflicts with its capture.

Passing a callback does not always transfer its captures: the receiving contract and whether the
callback is retained determine the checked access. See
[borrowed local callable contracts](/tsrust/language/callable-locals/).

Arguments and live views also matter:

- Passing `counts` to a function that also mutates its captured `counts` creates
  conflicting access. Passing a copied scalar such as `counts.length` can be valid.
- Keeping an array-element view alive across a call that changes the array creates
  a view conflict.
- Transferring an owner cannot overlap an argument that borrows that same owner.

## Return an owner from a factory callback

An escaping owned callback retains the owners its body needs. A callback that
consumes a captured owner can be invoked only once.

```tsrust
type User = {
  name: string;
};

function make(user: User): () => User {
  function take(): User {
    return user;
  }

  return take;
}

function main() {
  const take = make({
    name: "Pat"
  });
  const user = take();
  console.log(user.name); // Pat
}
```

Calling `take()` again is a consumed-callback error. A borrowed `ReadOnly<User>`
parameter cannot be retained as an owned capture by this factory. Pass an owned
value, or explicitly `clone()` when you need two independent owners; callbacks do
not implicitly clone captures.

## Retain independent reusable state

Retaining an owner does not make every callback single-use. A callback that updates
its environment without consuming it can be called repeatedly. Each factory call
creates a separate environment.

```tsrust
function make(start: int): () => int {
  let value = start;
  function next(): int {
    value += 1;
    return value;
  }

  return next;
}

function main() {
  const actions = [make(10), make(20)];
  const first = actions[0]();
  const second = actions[1]();
  const again = actions[0]();
  console.log(first, second, again); // 11 21 12
}
```

Nonempty callable arrays infer a contract from the first element and check the
remaining elements against it. A once-callable must be removed from its array
before invocation; indexing borrows the element instead of transferring its
environment. Empty arrays need a type or suitable context. See
[callable array storage](/tsrust/language/callable-arrays/) for extraction and
ownership examples.

Declarations inside loop bodies or blocks use the bindings of that iteration or
block entry. Recursive peers share their enclosing environment; each invocation
has its own locals for functions nested inside it.

Editor hover, definitions, references and completion follow the captured binding,
including shadowed names.
