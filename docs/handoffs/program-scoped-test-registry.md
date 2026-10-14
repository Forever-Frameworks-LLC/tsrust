# Program-scoped test registry

Status: owner-approved direction (2026-10-14, "option 1"); implementation pending.
This closes the known foundation defect listed under Phase B of the
[compiler continuation](compiler-to-roadmap-continuation.md).

## Defect

A registered test callback cannot call a function that reads module state:

```ts
// lib.tsr
const greeting = 'hello';
export function size(): int { return greeting.length; }
// tests/lib.test.tsr
import { test, strictEqual } from 'std:testing';
import { size } from '../lib.tsr';
test('size', () => { strictEqual(size(), 5); });
```

`tsrust test` rejects it with `callback-escape`: "An owned callback cannot retain
borrowed state, including state borrowed through a called function." Reading a
module global from tested library code is ordinary, so this blocks normal unit
testing. It reproduces on the installed compiler built from `39aaec7d` and on
`515a24e7`.

## Cause

`test`, `it`, `beforeAll` and the other hooks take `Owned<Function<void>>`. The
bundled runtime (`stdlib/testing/runtime/registration.rs`, `runtime/mod.rs`)
stores callbacks in the thread-local `TSR_TESTS` registry as
`Box<dyn FnMut() -> Result<(), String>>`, which requires `'static`. Module
storage is not static: `callables/module_state.rs::lower` places every module
binding in the program owner function (the lowered `main`), and named functions
reach it through the capture solver. A callback that calls `size()` therefore
borrows the owner's `greeting`. The checker correctly refuses to move that
borrow into a `'static` registry.

## Approved direction

Scope the registry to the program owner and dispose it before module storage.
Preserve real provenance. Do not erase lifetimes, leak storage, add hidden
clones or introduce shared wrappers. `clone(message)` alone does not resolve the
boundary, because the dependency is the called function's capture of program
storage, not the returned value.

Design to validate first:

1. Lower the registry as program storage. When a program uses registration
   (`test`, `it`, `describe`, `suite`, hooks, `runTests`), add one synthetic
   module binding after all user module bindings, holding an empty registry.
   Rust drops later locals first, so the registry drops before the globals its
   callbacks borrow. Registration and `runTests` access that binding through
   the existing named-capture machinery: registration mutates it, `runTests`
   consumes or drains it.
2. Add a callback ownership category for registration parameters: an owned
   callback that may borrow program storage, and nothing shorter-lived. Locals
   of the registering function, borrowed callback inputs and non-program
   borrows remain `callback-escape` errors with today's diagnostics and
   related spans.
3. Generic registry. Change the runtime registry to carry the owner lifetime,
   for example `TsrTestRegistry<'p>` holding `Box<dyn FnMut() -> Result<(), String> + 'p>`,
   passed as the synthetic binding instead of a thread local. Keep the
   running-state guard, suite scope guard, hook order, failure reporting and
   counts unchanged.
4. Emission: the owner declares the registry after module bindings. Functions
   that register or run tests receive it like any other captured module
   binding. Generated code must compile without `unsafe`.

## Acceptance

- The example above passes under `tsrust test`. Also cover mutable module
  state read and written by test bodies, nested `describe` registration,
  `beforeAll`/`afterAll` hooks touching globals, and project imports across
  files.
- Rejections stay precise: a test callback capturing a local of `main` or of a
  registering function, a borrowed callback input, or a callback escaping
  elsewhere still reports `callback-escape` at the same spans.
- Registration during a running test still errors. Tests run in registration
  order. Registries that are never run still dispose before globals.
- `consuming_module_storage_points_to_return_use` in
  `callables/module_state.rs` keeps its meaning: returning module storage by
  value still requires `clone()`.
- Update `std:testing` catalog entries in `crates/tsrust-lsp/src/builtins.json`.
  Their ownership notes currently say "Escaping borrowed captures are
  rejected"; state the new program-storage rule precisely.
- Run the testing, module-state, closure-argument and capture suites, plus the
  editor agreement suites.
