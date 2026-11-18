# Fault unwinding and `Checked<T, E>`

Owner decision, 2026-11-17: runtime faults (integer overflow, out-of-bounds
writes, division by zero and similar checks) stop travelling as hidden
`Result` values and unwind instead. Explicit `throw`, `Result<T, E>` and
`unwrap()` keep today's value model in this plan. `try/catch` still catches
faults. A new opt-in return type, `Checked<T>` or `Checked<T, E>`, delivers a
function's faults (plus an optional error type `E`) to its caller as a value,
for code where faults are expected and must stay cheap.

Why: almost every function is fallible only because of fault checks, so each
call returns and checks a `Result`. Reproductions in
`benchmarks/lang-compare/README.md` ("Per-call Result cost") put that at
1.2 to 1.4x on calls, recursion and comparators; the completion lowering and
the error record's size are not the cause. Unwinding costs nothing until a
fault happens and removes most inline error records, which should also shrink
generated Rust and rustc time.

## What the survey found

- `FailureSet` (`error_effects/mod.rs`) already separates `runtime` from
  `explicit` and `opaque` failures, so "faults unwind, throws stay values" has
  a natural seam.
- Fault sites funnel through `runtime_failure_exit` and `runtime_error_value`
  (`emit/mod.rs`, `emit/exception_emit.rs`; also used by `channels.rs`,
  `futures.rs`, `spread_emit.rs`). Where no transport carries runtime errors,
  some sites already panic today.
- `runtime` is broader than arithmetic: test assertions
  (`std:testing-runtime` `checkAssertion`), channel/interval/cache creation,
  `std:path` `relative`, and every native binding without a rule (the
  `_ => FailureSet::runtime()` default in `native_binding_effect`).
- `catch_unwind` already exists in the thread pool (`stdlib/threads/runtime_core.rs`)
  and the test runner (`stdlib/testing/runtime/mod.rs`).
- Async runs on `tsrust_native_futures::block_on`; HTTP and some streams use
  tokio.
- The uncaught-error report lives in `tsr_entry_result` (`runtime/completion.rs`).
- No generated Cargo profile sets `panic = "abort"`
  (`cli/runtime_build.rs`). The CLI has no WebAssembly build yet; `tsrust-ui`
  only has a `gpui-web` cfg.

## Planning passes

1. Draft from the survey: switch, fault helper, signatures, try/catch,
   boundaries, `Checked`, validation.
2. Failure-mode review added: classifying every `runtime` producer (assertions
   and native failures are not all bugs), guarantees that rely on returning
   errors (sort leaves storage intact, callback-store restoration, Map entry
   epochs, finally move slots), cleanup that faults during unwinding (double
   panic aborts), foreign Rust panics that must not be caught as Tsrust errors,
   mutex poisoning, and the callback-contract ripple (callbacks that only fault
   become infallible, which changes stored callable signatures and crate-split
   declarations).
3. Delivery review added: a mode switch that is part of the compile and cache
   identity, a measured go/no-go gate before the large semantic work, keeping
   value mode for targets without unwinding, the one-hour test-execution
   budget, mandatory IntelliSense documentation for `Checked`, and identical
   stderr text and exit codes for uncaught faults.
4. Review of the written plan added: language-server agreement, taking the
   fault record on the faulting thread, and debug backtraces.

## Prioritized checklist

Order: cheap audits that change the design first, then the shared core, which
everything else builds on, then a measured gate, then correctness at every
boundary, then `Checked`, then rollout.

### Phase 0: audits (no behavior change)

- [x] 0.1 Classify every producer of `FailureSet::runtime()` as a fault
      (unwinds) or an expected failure (becomes explicit, stays a value).
      Record the table here. Assertions: faults (the runner catches them).
      Unknown native bindings: decide per binding instead of by default.
- [x] 0.2 List every place generated code is entered from non-Tsrust frames:
      entry `main`, spawned and scoped threads, pool tasks, async tasks and
      awaits, std:ui callbacks (GPUI event, render, ticker, Kit callbacks),
      HTTP handlers, SQLite callbacks, cache loaders, compression streams,
      schema pipelines, the test runner, `using`/dispose and other `Drop`
      paths that run user code.
- [x] 0.3 List guarantees that rely on an error returning normally: sorts
      leave storage intact, callback-store failures restore inputs
      (`caller_callback_store_failures`, `caller_borrowed_callback_replacement`),
      Map entry epochs (`map_exception_storage_plan`), `finally_move_slots`,
      `field_completion_transport`. Decide each one's behavior under unwinding.
- [x] 0.4 List runtime `lock().unwrap()` sites (threads, caches, shared state)
      and pick a poisoning policy (recover with `into_inner`).
- [x] 0.5 Confirm no profile, `tsrust package` path or portable export can
      select `panic = "abort"`; add a guard test.

### Phase 0 findings (2026-11-17)

**0.1 Runtime failure producers** (`error_effects/mod.rs`). Unknown native
bindings default to `opaque`, not `runtime`.

| Producer | Examples | Class |
|---|---|---|
| `own_expression_effect` | int `+ - * / %`, negation, `++`/`+=`, index writes and checked reads, `x!`, spread length checks | fault |
| Statement compound assignment (`traversal.rs`) | `total += x` | fault |
| `native_effect` default (`_ => runtime`) | builtin argument and range errors: `repeat(-1)`, `toFixed(200)`, capacity overflow in `splice`/`padStart` | fault |
| Array callback methods (`runtime` part) | capacity and index checks around `map`, `filter`, `sort`, ... | fault (callback effects stay as they are) |
| `std:futures` `__channel`, `__interval`; `std:cache` `__create`; `std:threads` `__channelCreate` | invalid capacity or period, no active runtime | fault |
| `std:path` `relative` | needs the current directory | fault for now; candidate for an explicit `Result` later |
| `std:testing-runtime` `checkAssertion`, `runTests`, `registerTest`, hooks, `invokeSuite` | failed assertions, runner misuse | fault (the runner already catches panics) |

No producer is an expected, frequently-hit failure, so nothing has to move to
explicit errors before Phase 1. Item 2.9 shrinks to `std:path` `relative`.

**0.2 Entry boundaries** (runtime code calling generated code):

| Boundary | Today | Needed |
|---|---|---|
| Program entry (`tsr_entry_result`, `runtime/completion.rs`) | reports `Err` and exits 1 | catch, same report and exit code |
| Async main (`tsrust_main_thread`, `emit/async_runtime.rs`) | `resume_unwind` on panic | same catch after the thread joins |
| Thread pool (`stdlib/threads/runtime_core.rs` `execute`) | `catch_unwind` maps any panic to `JoinError::Panic` | map a `TsrFault` to `JoinError::Worker(record)` |
| Async tasks (`stdlib/futures/runtime.rs`, `tokio::spawn`) | a panic becomes `Failure::Panic` | map a `TsrFault` to `Failure::Worker(record)` |
| HTTP handlers (`stdlib/http/runtime.rs`, connection tasks) | handler `Err` becomes a 500; a panic drops the connection | catch around the handler future, answer 500 |
| std:ui callbacks (`ui_callback`, GPUI `emit`/`render`/ticker) | `Result` turned into the UI failure report | catch before returning to GPUI (Objective-C frames abort on unwinding) |
| Test runner (`stdlib/testing/runtime/mod.rs`) | `catch_unwind` already | read the fault record for the report |

No other runtime module invokes user callbacks (`Callback::invoke` is only
emitted into generated code). SQLite, I/O, process and compression runtimes
take data, not callbacks.

**0.3 Guarantees that rely on errors returning normally.** All of them
(`field_completion_transport`, `finally_move_slots`, the Map storage plans,
callback-store restoration in `caller_callback_store_failures`) concern the
function that catches: owners moved into a `try` region return through its
completion values. Wrapping a whole `try` body in one `catch_unwind` closure
would lose them, so 2.1 changes: inside `try` regions and `Checked` bodies,
local faults stay values, and each call that may unwind is converted back to
a value at the call site (`catch_unwind` around that call only). The existing
transport, catch typing and `finally` then work unchanged. Two gaps remain
for 2.7: in-place sorts whose comparator can only fault (the `tsr_sort` path
would leave a partly sorted array; keep the positions path), and runtime code
that takes state out (`mem::take`/`mem::replace`, 53 sites) while user code
runs: none must leave state missing when that code unwinds.

**0.4 Locks.** About 99 `lock().unwrap()`-style sites (threads 33, compression
streams 18, futures 17, terminal 11, I/O 10, others). In the code read, no lock
is held while user code runs (the pool releases its statistics lock before the
task), so poisoning needs a runtime bug. Policy for 2.6: a test that a task
fault leaves every pool, channel and future lock usable, and `into_inner` on
poisoned guards where user code could ever run under a lock.

**0.5 Panic strategy.** Only `profiles()` in `cli/runtime_build.rs` writes
Cargo profiles; neither it nor `application_optimization` sets a panic
strategy. Guard test: `generated_profiles_keep_panics_unwinding`.

### Phase 1: shared core behind a switch, then measure

- [ ] 1.1 `FaultMode { Value, Unwind }` in the emitter and runtime selection,
      default `Value`; part of the compile, precheck and runtime cache identity.
- [ ] 1.2 Runtime support: a `TsrFault` marker payload; the `#RuntimeError`
      record kept in a thread-local (works for non-`Send` data); a
      `#[cold] #[inline(never)] fn tsr_raise_fault(site) -> !` reading a static
      per-program site table (name, code, message, source, start, end), which
      replaces inline records; a silent hook installed once that stays quiet
      only for `TsrFault` and chains to the previous hook otherwise.
- [ ] 1.3 Error effects in `Unwind` mode: `runtime` no longer makes a
      function, closure or method fallible for its signature; keep a separate
      "may fault" fact for catch typing and boundaries.
- [ ] 1.4 `runtime_failure_exit`, `runtime_error_value` and the other fault
      sites emit `tsr_raise_fault(site)` in `Unwind` mode.
- [ ] 1.5 Entry boundary: `catch_unwind` around the program body; a
      `TsrFault` prints exactly what `tsr_entry_result` prints and returns the
      same exit code, after owners are disposed; any other panic resumes.
- [ ] 1.6 Gate: build `benchmarks/lang-compare` in both modes, compare every
      phase, generated Rust size, binary size and rustc time (also the
      `gpui-compare` Tsrust edit loop). Record in `measurements.sqlite`.
      Continue only if hot phases improve and rustc time does not regress;
      otherwise report to the owner and stop.

### Phase 2: correct behavior in `Unwind` mode

- [ ] 2.1 Inside `try` regions (and `Checked` bodies) local faults stay
      values, and each call that may unwind is wrapped alone in
      `catch_unwind`; a `TsrFault` becomes the same `Throw(record)` as today
      (see 0.3).
- [ ] 2.2 `finally` needs no unwinding path: faults reach it as values
      (2.1). Verify with the exception suites.
- [ ] 2.3 Disposal while unwinding: resource management (`using`) is not
      implemented, and runtime `Drop` code runs no user code, so nothing can
      fault during an unwind today. Record the rule for the future `using`
      work: user dispose code run while unwinding must be caught in the
      disposal path (a second panic aborts).
- [ ] 2.4 Implement every boundary from 0.2: threads and pool tasks deliver
      faults as their join error, async tasks catch per poll
      (`catch_unwind` around the future), std:ui callbacks catch before
      returning to GPUI (and report through the existing failure path), HTTP
      handlers still answer 500, SQLite, cache loaders and streams convert as
      today, the test runner reports assertion faults unchanged.
- [ ] 2.5 Foreign panics (Rust crates through bindings, runtime bugs) are
      never treated as Tsrust errors: every catch resumes non-`TsrFault`
      payloads.
- [ ] 2.6 Poisoning policy from 0.4 applied.
- [ ] 2.7 Guarantees from 0.3 kept (for example sorts keep the positions
      approach whenever the comparator may fault).
- [ ] 2.8 Callback contracts: callbacks that only fault become infallible;
      update stored callable families, `callback_family_result_adapters`,
      invocation plans and crate-split declarations.
- [ ] 2.9 `std:path` `relative`: decide fault or explicit `Result` (0.1).
- [ ] 2.10 The fault record is taken from the thread-local on the thread
      that faulted, before the catch returns (async tasks may resume on
      another thread afterwards).
- [ ] 2.11 Language server: error-effect features (catch typing, hover,
      diagnostics about what a call can throw) agree with the mode.
- [ ] 2.12 `tsrust build --debug` keeps a native backtrace for faults.

### Phase 3: `Checked<T>` and `Checked<T, E>`

- [ ] 3.1 Type: a global like `Once`, `Mutable`, `ReadOnly`, `Owned`; allowed
      as the return type of functions, arrow functions and async functions;
      `E` must be an error type; diagnostics for other positions.
- [ ] 3.2 Semantics: callers see `Result<T, RuntimeError | E>`; inside, the
      function's own faults stay values (cheap when frequent); faults from
      callees are caught once at the boundary; `throw` of `E` becomes `Err`.
- [ ] 3.3 Interop: `unwrap`, matching, `try/catch`, generics, callbacks typed
      `Checked`, `Task<Checked<T>>`.
- [ ] 3.4 IntelliSense: complete `builtins.json` entry (purpose, parameters,
      errors, verified example, short ownership note), hover, completion and
      signature help consistent.
- [ ] 3.5 Docs: error-handling guide section on faults vs errors and when to
      use `Checked`.
- [ ] 3.6 Tests: one `std:testing` suite covering local faults, callee faults,
      `E` alternatives, async, callbacks and nesting.

### Phase 4: value mode kept

- [ ] 4.1 `Value` mode stays complete for targets without unwinding (the
      future WebAssembly build) and as a fallback; `Checked` behaves the same
      there.
- [ ] 4.2 A focused suite runs in `Value` mode so it cannot rot.

### Phase 5: validation and rollout

- [ ] 5.1 Full compiler and LSP regressions in `Unwind` mode, batched to stay
      under one hour of execution; resume unrun groups after repairs.
- [ ] 5.2 Uncaught-fault parity: identical stderr text and exit codes in both
      modes for the error-handling fixtures and examples.
- [ ] 5.3 Exception workloads with Node counterparts (`array_callback_errors`
      and others) still match; lang-compare, gpui-compare and the Studio edit
      loop measured and recorded.
- [ ] 5.4 Default flips to `Unwind`; benchmark READMEs and
      `docs/agent-progress.md` updated.

## Risks

| Risk | Mitigation |
|---|---|
| A missed boundary aborts the app or loses an error silently | 0.2 inventory, 2.4 tests per boundary, hook records faults it silences |
| Frequent faults get slower (microseconds per caught fault) | `Checked<T>` keeps them values; 0.1 moves expected failures to explicit errors |
| Cleanup that faults during unwinding aborts | 2.3 |
| Partially updated state after a caught fault | 0.3 and 2.7 decide each guarantee |
| Foreign panics swallowed | 2.5 |
| Two models drift | 4.2 |
| Another crate replaces the panic hook | Chain on install; document; fault reporting does not depend on the hook |

## Not in this plan

- Unwinding for `unwrap()` and explicit `throw` (a later owner decision; it
  would speed up code that unwraps in hot paths but makes throws expensive).
- A WebAssembly build target.
