# Program-scoped test registry

Status: compiler integration and focused acceptance completed on 2026-10-14.
The owner-approved invocation-time design is implemented in the checker,
standard library, native runner, code generation, and editor projections.
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

### Invocation-time borrowing (approved revision)

The registry owns callback environments, but those environments do not retain
references to program storage. Each invocation receives fresh access to the
program owner. The borrow ends before the next hook or test starts. This permits
a setup hook to write a global and a test to read that same global, without
cloning program storage, adding shared wrappers, or using `unsafe`.

The earlier proposal, `Box<dyn FnMut() + 'p>`, only solves destruction order.
Keeping a mutable program borrow inside one callback prevents another callback
from reading that global until the first callback is dropped (Rust E0502).
Callbacks that call registration functions would also need to reach the registry
that owns them. Merely replacing `'static` with an owner lifetime is therefore
insufficient.

The generated callback interface must separate its owned environment from its
invocation inputs. A minimal native shape is:

```rust
trait ProgramFamily {
    type State<'p>;
}
type TestCallback<P> = Box<
    dyn for<'p> FnMut(
        &mut <P as ProgramFamily>::State<'p>,
        &mut TestRegistry<P>,
    ) -> Result<(), String>
>;
```

`P::State<'p>` describes program storage available to the current invocation. It is not an
`Rc`, `RefCell`, cloned snapshot, or retained borrow in the callback environment.
The lifetime must be chosen separately for each invocation. A registry generic
only over one fixed borrowed `S` can still extend the input lifetime across runs.
The witness reads and changes the original global between two runs with the same
registry to check this distinction. The callback object can own ordinary eligible
capture values. The compiler must
preserve the actual program-binding identities and access modes, rather than
removing a borrowed flag to make an ordinary owned callback pass.

Required implementation sections:

1. **Program ownership and provenance.** Add the synthetic registry after user
   program bindings. Preserve their initialization order and existing module
   capture identities. Separate program dependencies from ordinary captures in
   checked registration callbacks, including dependencies reached through named
   calls and nested callbacks. Do not classify locals by spelling or merely by
   their location inside the lowered `main`.
2. **Registration callback contract.** Introduce a distinct internal contract for
   callbacks owned by this registry. It permits invocation-time program access,
   not arbitrary borrowed environments. It cannot flow into an ordinary owned
   callback, a return value, or unrelated escaping storage. Preserve existing
   owned capture transfers and borrowed-local/input rejection. Check callback
   creation and callback invocation effects separately.
3. **Invocation effects.** Calls that run tests acquire the program access needed
   by pending callbacks. Existing overlapping loans must still reject; deferred
   access is not permission to bypass borrow checking. Preserve read versus
   write distinctions and transitive named-call effects. Pass program inputs
   through helpers that invoke, register, or forward these callbacks.
4. **Runtime and emission.** Remove callbacks from registry storage before
   invoking them, then pass program access and the registry as separate inputs.
   This avoids borrowing a callback inside a registry while also lending that
   registry to the callback. Guard registration and nested execution before any
   mutation. Restore running and suite-scope state on success, typed failure,
   and panic. Preserve reporting, hook order, counts, and callback destruction.
5. **Source and editor integration.** Update the standard-library wrappers,
   native contracts, generated callback interfaces, and editor documentation
   together. Keep source callbacks zero-argument; program inputs are compiler
   implementation details. Do not publish the new ownership guarantee in hover
   until source checking and native execution support it.

The standalone witness in
`crates/tsrust-compiler/tests/program_scoped_test_registry_design.rs` validates
only the native ownership mechanism: shared writable state, transient program
references, owned callback captures, runtime guards, failure recovery, and
registry-before-program destruction. It does not validate checker provenance,
source diagnostics, module initialization, editor behavior, or production runner
integration. Those remain acceptance requirements, not completed work.

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

## Native design validation (2026-10-14)

All eight standalone Rust witnesses pass with rustc 1.96.1. They include two
compile-fail controls: borrowed local capture rejects with E0597, and storing an
invocation borrow in a callback environment rejects with E0521. No timing was
performed. Evidence record: `tsrust-program-registry-invocation-design-2026-10-14.json`
in `benchmarks/records.sqlite`.

## Compiler integration acceptance (2026-10-14)

The program owner declares the registry after user module storage. Registered
callback environments own eligible local captures; module dependencies retain
declaration identity and become fresh invocation inputs. Access inference keeps
read/write modes and checks runner calls against existing loans. Immediate suite
callbacks retain ordinary local borrows while receiving program inputs through
the same private interface. Named registration callbacks remain reusable.

The production registry no longer stores callbacks in thread-local storage.
Execution takes pending entries out of the active registry before invoking them.
Registration and execution guards remain recoverable, failure exit status stays
1, and suite scope is restored on typed failure and native unwinding. Unused
trusted registration wrappers are omitted for assertion-only imports, preserving
ordinary generic layouts and public import hover.

Focused acceptance: 146 compiler integration tests across testing, exceptions,
module state, closure arguments, owned arguments, captures, and the seven new
registry tests. Required editor agreement suites pass (10 tests); project editor
intelligence passes all 173 tests. Nine registry unit checks, three module-state
checks, and the native failure-contract audit pass. The formatter corpus checks
240 lexable fixtures and 84 standalone contracts; project formatting is
idempotent and changes no benchmark fingerprints. Production runtime disposal
is checked with a never-run registry, in addition to the earlier design witness.
Catalog examples and the imported program-state example execute successfully.

Both existing testing workloads pass five independent-oracle cases in native
and Node-after-tsc (20 cells total), using the new untimed `testing-check.mjs`.
No performance samples were collected or relabeled. Exact evidence and earlier
failed attempts are retained in `benchmarks/records.sqlite` under
`tsrust-program-registry-acceptance-2026-10-14.json` and its referenced records.
Consumer refresh is coordinated with the following concat repair.
