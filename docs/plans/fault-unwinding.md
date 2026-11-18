# Fault unwinding and `Checked<T, E>`

Owner decision, 2026-11-17: runtime faults (integer overflow, out-of-bounds
writes, division by zero and similar checks) stop travelling as hidden
`Result` values and unwind instead. Explicit `throw`, `Result<T, E>` and
`unwrap()` keep today's value model (decision 1). `try/catch` still catches
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
5. Owner asked to resolve every deferred decision and to verify flows end to
   end instead of assuming today's behavior is right. Running small programs
   showed four existing defects (raw panics for uncaught faults depending on
   `main`'s shape, wrong offsets, missing file paths, silent detached-thread
   failures); the resolved decisions and the `Checked` specification below
   replace every open question.

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

### End-to-end trace of today's behavior (2026-11-17)

Small programs run with the current compiler, before any change:

| Case | Today | Correct? |
|---|---|---|
| Fault in a callee, caught by `try` in the caller | caught, `error.code` is `overflow` | yes |
| Fault in a `map` callback inside `try` | caught | yes |
| Uncaught fault, `main` has no `try` or `unwrap` | raw Rust panic message, exit 101 | **no**: leaks a Rust panic (owner direction 2026-10-05) |
| Uncaught fault, `main` has an `unwrap` somewhere | `Uncaught RangeError: ...`, exit 1 | report yes, but the format depends on unrelated code |
| Location in the report, multi-file program | message says bytes `84..85`, location says `83..84` | **no**: the message uses whole-program offsets |
| Location in the report, single file without imports | `(<source>:46..47)` | **no**: file path missing |
| Fault in a thread, then `join` | `ThreadError` with code `worker` | yes |
| Fault in a detached thread | nothing printed, exit 0 | **no**: silently lost |
| Capacity overflow in string/array helpers | `expect` panic | **no**: uncatchable RangeError |
| Faults where no error transport exists (channel, interval and other fallbacks) | `panic!`/`expect` | **no**: uncatchable |

Why: a function gets an error transport only when something above it can
receive errors, so whether a fault is catchable or reported cleanly depends on
the shape of `main`. The unwinding design removes that dependency: every fault
site raises the same fault, and every boundary catches it.

### Resolved decisions (2026-11-17)

1. **What unwinds.** Only faults (the 0.1 table, plus capacity overflow in
   runtime helpers). `throw`, `unwrap()` and `Result` stay values. Reason:
   they are the explicit, expected-error channel; their cost is visible in the
   code that uses them, and making them unwind would make expected errors
   about 1,000 times slower to throw. Not revisited later.
2. **Where faults turn back into values.** Inside `try` regions and
   `Checked` bodies: local faults are emitted as values (as today), and each
   call or `await` that may unwind is caught individually (`catch_unwind`
   around the call; a small future adapter that polls inside `catch_unwind`
   for `await`). Everywhere else faults unwind. This keeps ownership
   transport, catch typing and `finally` unchanged (0.3).
3. **Boundaries are implicit `Checked`.** Thread and task entry functions,
   HTTP handlers, std:ui callbacks, test bodies and the testing helpers that
   call user code (`throws`, `rejects`, `captureOutput`) catch faults and
   deliver them exactly as today (`ThreadError` code `worker`, task
   `Failure::Worker`, HTTP 500, the UI failure report, test failure).
   `captureOutput` stops capturing when its callback faults (a guard), and
   each queued std:ui command runs with its own catch so one failing command
   does not drop the rest.
4. **Uncaught report, one format everywhere.** `Uncaught RangeError: integer
   multiplication overflow` then `    at path/to/file.tsr:12:9`, exit code 1,
   whatever `main` looks like. Messages lose the `at source bytes N..M` suffix
   (the location has its own fields). The runtime error record gains `line`
   and `column` (1-based) and its `start`/`end` become offsets in that file.
   Every program records real file paths (no `<source>`).
5. **Detached threads and tasks.** An uncaught fault or error there is printed
   in the same format with `in detached thread` (or task), the program keeps
   running, and its exit code becomes 1 when it ends. Today these are lost.
6. **Safety net in the hook.** Each catch point increments a thread-local
   depth while it runs. If a fault is raised with depth 0 (no catch point on
   this thread, meaning a missed boundary), the hook prints the uncaught
   report itself, so a missed boundary is never silent. Other panics go to
   the previous hook unchanged: they are defects and must stay visible
   (owner direction).
7. **Capacity overflow** in string and array helpers raises a `RangeError`
   fault (catchable, like JavaScript's invalid length). Real allocation
   failure still aborts (outside language control).
8. **`std:path` `relative`** stays a fault: its failures (no current
   directory, paths on different Windows drives) are rare and environmental,
   and its documented contract throws.
9. **Sorting.** A comparator that may fault keeps the positions path, so a
   fault leaves the array untouched, as the language specification requires
   (it sorts a copy and writes back). Fault-free comparators keep the
   in-place sort.
10. **Locks.** Verified in threads and futures: no user code runs under a
    lock (the cancellation `shutdown` closure under the futures lock is
    runtime code). Poisoning can only come from a runtime defect, and defects
    must stay visible, so no blanket `into_inner`; Phase 2 adds a test that
    task faults leave pool, channel and scope locks usable.
11. **Targets without unwinding.** None exist today. `Value` mode stays for
    them and as the fallback switch; there, fault sites without a transport
    print the uncaught report through the hook and abort.
12. **Mode switch.** `FaultMode` is a compiler option, default `Value` until
    5.4, included in the compile, precheck and runtime cache identity.

### `Checked<T>` and `Checked<T, E>` specification

- **Where:** the declared return type of a function declaration, function
  expression or arrow function, and inside `Task<...>` for async functions
  (`Task<Checked<T, E>>`). Also allowed in function-type return positions.
  Not allowed on `main`, in parameters, fields, variables or nested
  (`Checked<Checked<T>>`); each gets a diagnostic.
- **Body:** written like an ordinary function: `return value` returns `T`.
  A local fault, a fault from any call it makes, or `throw` of an `E` ends it
  with an error. Throwing or propagating (through `unwrap`) any explicit
  error that is not in `E` is a compile error naming the type.
- **Callers:** the call's value is `Result<T, RuntimeError | E>`, used with
  `unwrap`, `match`, `Ok`/`Err` like any other `Result`. `unwrap` of it
  rethrows as an explicit error (a value, not a fault).
- **`RuntimeError`** becomes a public global type name for the record that
  `catch` already receives (`name`, `code`, `message`, `source`, `start`,
  `end`, `line`, `column`).
- **`Checked<T, RuntimeError>`** is accepted and equals `Checked<T>`.
- **Value mode:** identical behavior (the function body is already in value
  mode).

### Phase 1: shared core behind a switch, then measure

- [ ] 1.1 `FaultMode { Value, Unwind }` in the emitter and runtime selection,
      default `Value`; part of the compile, precheck and runtime cache identity.
- [ ] 1.2 Runtime support: a `TsrFault` marker payload; the runtime error
      record kept in a thread-local; a `#[cold] #[inline(never)] fn
      tsr_raise_fault(site) -> !` reading a static per-program site table
      (name, code, message, file, start, end, line, column) that replaces
      inline records; a hook installed once that is silent for `TsrFault`
      when a catch point is active, prints the uncaught report when none is
      (decision 6), and chains to the previous hook for every other panic.
- [ ] 1.3 Error effects in `Unwind` mode: `runtime` no longer makes a
      function, closure or method fallible for its signature; keep a separate
      "may fault" fact for catch typing and boundaries.
- [ ] 1.4 Every fault site raises through the site table in `Unwind` mode:
      `runtime_failure_exit`, `runtime_error_value`, `checked_integer_result`
      (including its `.expect` fallback), the `panic!` fallbacks in
      `channels.rs` and `futures.rs`, non-null assertions, and capacity
      checks in runtime helpers such as `tsr_string_reserve` (decision 7).
- [ ] 1.5 Entry boundary: `catch_unwind` around the program body (and the
      async main thread); a `TsrFault` prints the decision-4 report after
      owners are disposed and exits 1; any other panic resumes. Explicit
      uncaught errors use the same format.
- [ ] 1.5a Site table and report format (decision 4): file paths for every
      program, file-relative offsets, `line`/`column` fields, messages without
      byte suffixes. Update affected expected outputs deliberately, in both
      modes.
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
- [ ] 2.4 Implement every boundary from 0.2 and decision 3: threads and pool
      tasks deliver faults as `ThreadError` code `worker`, async tasks as
      `Failure::Worker`, std:ui callbacks catch in the generated
      `ui_callback` wrapper before returning to GPUI, queued std:ui commands
      each run with their own catch, HTTP handlers answer 500, the test
      runner reports assertion faults unchanged, `throws`/`rejects` treat a
      fault as a throw, `captureOutput` ends capture on a fault.
- [ ] 2.4a Detached threads and tasks report uncaught faults and errors and
      set exit code 1 (decision 5).
- [ ] 2.4b The `await` adapter that polls inside `catch_unwind`, used inside
      `try` regions and `Checked` bodies (decision 2).
- [ ] 2.5 Foreign panics (Rust crates through bindings, runtime bugs) are
      never treated as Tsrust errors: every catch resumes non-`TsrFault`
      payloads.
- [ ] 2.6 Test that task faults leave pool, channel and scope locks usable
      (decision 10).
- [ ] 2.7 Sorts keep the positions path whenever the comparator may fault
      (decision 9); runtime code never leaves state taken out while user code
      runs (UI command queue fixed by 2.4).
- [ ] 2.8 Callback contracts: callbacks that only fault become infallible;
      update stored callable families, `callback_family_result_adapters`,
      invocation plans and crate-split declarations.
- [ ] 2.9 Error effects keep a "may fault" fact in `Unwind` mode, used to
      choose per-call catches (decision 2) and the sort path (decision 9);
      only the native `Result` types drop runtime errors.
- [ ] 2.10 The fault record is taken from the thread-local on the thread
      that faulted, before the catch returns (async tasks may resume on
      another thread afterwards).
- [ ] 2.11 Language server: error-effect features (catch typing, hover,
      diagnostics about what a call can throw) agree with the mode.
- [ ] 2.12 `tsrust build --debug` keeps a native backtrace for faults.

### Phase 3: `Checked<T>` and `Checked<T, E>`

- [ ] 3.1 Type and positions exactly as in the specification above, with
      diagnostics for every disallowed position; public `RuntimeError` type.
- [ ] 3.2 Body semantics as specified (return `T`, faults and `throw E`
      become `Err`, other explicit errors are compile errors).
- [ ] 3.3 Interop: `unwrap`, `match`, `try/catch`, generics, function types
      returning `Checked`, `Task<Checked<T, E>>`.
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
- [ ] 5.2 Uncaught report: the decision-4 format and exit code 1 in both
      modes, for every `main` shape, for single-file and multi-file programs,
      and for detached threads (decision 5).
- [ ] 5.3 Exception workloads with Node counterparts (`array_callback_errors`
      and others) still match; lang-compare, gpui-compare and the Studio edit
      loop measured and recorded.
- [ ] 5.4 Default flips to `Unwind`; benchmark READMEs and
      `docs/agent-progress.md` updated.

## Risks

| Risk | Mitigation |
|---|---|
| A missed boundary aborts the app or loses an error silently | 0.2 inventory, 2.4 tests per boundary, the hook prints any fault raised with no catch point active (decision 6) |
| Frequent faults get slower (microseconds per caught fault) | `Checked<T>` keeps them values; 0.1 moves expected failures to explicit errors |
| Cleanup that faults during unwinding aborts | no cleanup runs user code today (2.3); rule recorded for `using` |
| Partially updated state after a caught fault | catches inside `try` stay per call, so ownership transport is unchanged (decision 2); 2.7 |
| Foreign panics swallowed | 2.5 |
| Two models drift | 4.2 |
| Another crate replaces the panic hook | Chain on install; document; fault reporting does not depend on the hook |

## Not in this plan

- A WebAssembly build target (value mode is ready for it, decision 11).
