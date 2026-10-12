# Module variables

Module-level `const` and `let` values belong to the program. Each imported module
initializes once, after its dependencies. Declarations initialize in source order.
An initializer can call a function, but that function cannot read a later binding
before initialization. An initializer failure prevents `main` from running.
Module initialization also runs when the entry file has no `main`.

```tsrust
// state.tsr
export const settings = { retries: 3 };
export let requests = 0;

export function recordRequest() {
  requests++;
}

export function configure(retries: int) {
  settings.retries = retries;
}
```

`const` prevents rebinding; it does not freeze fields. `let` permits rebinding.
Neither declaration adds an implicit clone. Imported bindings are live, but the
importer cannot assign a new value to the binding itself. Calls and field mutations
still follow exclusive-access checking.

```tsrust
import { settings, requests, recordRequest } from './state.tsr';

function main() {
  recordRequest();
  console.log(requests); // 1
  const local = clone(settings);
  local.retries = 9;
  console.log(settings.retries); // 3
}
```

Functions borrow module storage according to their access, including access through
other functions. An active borrow prevents conflicting mutation or consumption.
Use explicit `clone` when a caller needs an owned value. A function cannot leave
program-owned storage consumed.

Callbacks also retain the live module owner. They do not take a private snapshot
of scalar globals. Use a borrowing callback contract when invoking such callbacks:

```tsrust
let completed = 0;

function apply(action: Mutable<Function<void, []>>) {
  action();
}

function main() {
  apply(() => { completed++; });
  console.log(completed); // 1
}
```

The existing owned `Function` contract requires an environment without borrowed
captures. It therefore rejects a callback that retains module storage. This keeps
ownership explicit and prevents unnoticed copies. Thread sharing and pool policy
are separate work after async; module declarations add no locks, reference counting,
or thread-safety permission.

Reusable schemas can be module constants and retain their inferred types across
imports. See the executable example in
[`module-state`](../../crates/tsrust-compiler/tests/fixtures/module-state/main.tsr).
