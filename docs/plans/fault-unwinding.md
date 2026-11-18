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
   multiplication overflow at source bytes 85..86` then
   `    at path/to/file.tsr:3:16`, exit code 1, whatever `main` looks like.
   Revised 2026-11-18: messages keep their `at source bytes` wording (21 test
   files and panic texts rely on it) but the offsets are now file-relative.
   The runtime error record gains `line` and `column` (1-based, UTF-16
   columns as in assertion sites) and its `start`/`end` are offsets in that
   file. Single-file programs record their real path (no `<source>`).
5. **Detached threads and tasks.** An uncaught fault or error there is printed
   in the same format with `in detached thread` (or task), the program keeps
   running, and its exit code becomes 1 when it ends. Today these are lost.
   Revised 2026-11-18: `detach` reports faults (the runtime error record) as
   `Uncaught <Name>: <message> in a detached thread` plus the location line;
   explicit errors a detached body returns stay discarded, as `detach`
   documents (the caller chose not to observe them), and the exit code does
   not change, because a detached worker can finish after `main` has already
   chosen its exit status.
6. **Safety net in the hook.** Each catch point increments a thread-local
   depth while it runs. A fault raised with depth 0 (a missed boundary, or
   Rust code calling generated functions directly) goes to the previous hook
   like any other panic, so it is visible and marked as a defect. Revised
   2026-11-18: the panic payload is the fault's message text (as with an
   `expect` panic) and the hook is installed only by program entries, so Rust
   harnesses keep their own hook and see value-mode panic texts.
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

- [x] 1.1 `FaultMode { Value, Unwind }` in the emitter and runtime selection,
      default `Value`; part of the compile, precheck and runtime cache identity.
- [x] 1.2 Runtime support: a `TsrFault` marker payload; the runtime error
      record kept in a thread-local; a `#[cold] #[inline(never)] fn
      tsr_raise_fault(site) -> !` reading a static per-program site table
      (name, code, message, file, start, end, line, column) that replaces
      inline records; a hook installed once that is silent for `TsrFault`
      when a catch point is active, prints the uncaught report when none is
      (decision 6), and chains to the previous hook for every other panic.
- [x] 1.3 Error effects in `Unwind` mode: `runtime` no longer makes a
      function, closure or method fallible for its signature; keep a separate
      "may fault" fact for catch typing and boundaries.
- [x] 1.4 Every fault site raises through the site table in `Unwind` mode:
      `runtime_failure_exit`, `runtime_error_value`, `checked_integer_result`
      (including its `.expect` fallback), the `panic!` fallbacks in
      `channels.rs` and `futures.rs`, non-null assertions, and capacity
      checks in runtime helpers such as `tsr_string_reserve` (decision 7).
- [x] 1.5 Entry boundary: `catch_unwind` around the program body (and the
      async main thread); a `TsrFault` prints the decision-4 report after
      owners are disposed and exits 1; any other panic resumes. Explicit
      uncaught errors use the same format.
- [x] 1.5a Site table and report format (decision 4): file paths for every
      program, file-relative offsets, `line`/`column` fields, messages without
      byte suffixes. Update affected expected outputs deliberately, in both
      modes.
- [x] 1.6 Gate: build `benchmarks/lang-compare` in both modes, compare every
      phase, generated Rust size, binary size and rustc time (also the
      `gpui-compare` Tsrust edit loop). Record in `measurements.sqlite`.
      Continue only if hot phases improve and rustc time does not regress;
      otherwise report to the owner and stop.

### Phase 1 results and gate (2026-11-18)

Built behind `TSRUST_FAULT_MODE=unwind` (`error_effects::unwind_faults`).
Native caches key on the generated Rust text, so the modes never mix (1.1).

- 1.2 `runtime/faults.rs`: `TsrFault`, thread-local `TsrFaultInfo`,
  `tsr_raise_fault`, `tsr_catch_faults` with the catch-depth safety net, the
  chained hook, `tsr_fault_entry`/`tsr_fault_exit`.
- 1.3 First stage: named functions that are not callable values, schema
  constraints or async drop runtime errors from their signatures
  (`plain_fault_contracts` in `emit/mod.rs`). Closures, callbacks and stored
  callables keep them (stage B, item 2.13).
- 1.4 Partly: every site through `runtime_failure_exit` raises when the active
  region does not carry runtime errors; a callee's runtime error reaching a
  contract without runtime errors is raised again (`RAISE_FAULT` injection and
  regionless `result_call`). Capacity checks in runtime helpers remain.
- 1.5 Entry catch for plain, threaded-async and Result entrypoints. UI-window
  and no-`main` (test) programs are Phase 2 boundaries.
- 1.5a Not started; mode-independent, moved to the start of Phase 2.

Gate, release builds, median of 5 (value vs unwind), checksums equal:

| Phase | Value | Unwind | Rust |
|---|---:|---:|---:|
| fib | 10.2 ms | 7.2 ms | 7.2 ms |
| trees | 1726 ms | 1663 ms | 1224 ms |
| matrix | 26.1 ms | 25.2 ms | 20.0 ms |
| format | 165.4 ms | 159.4 ms | 136.7 ms |
| orders | 215.4 ms | 214.6 ms | 154.8 ms |
| others | | within 2% | |
| whole process | 2.69 s | 2.61 s | 1.94 s |

Generated Rust: CPU benchmark 305 KB to 247 KB (-19%), GPUI app -0.3%,
Studio 21.96 MB to 21.41 MB (-2.5%). Both apps and Studio compile in unwind
mode. GPUI app edit-loop rebuild: 1.19 s vs 1.25 s on first edits, 0.59 s vs
0.60 s on repeated edits (no regression). Value mode unchanged: result and
exception suites pass.

Verdict: the gate is met (calls reach Rust parity, no rustc regression).
Comparators, callbacks and recursive closures need stage B (2.13), and Studio's
code-size gain will come mostly from there. Unwind mode currently fails 50
tests in 9 suites (`callable_results`, `entry_error_reporting`,
`exception_handling`, `int_operators`, `native_exception_region`,
`number_operators`, `operators`, `recursive_callable_results`, `result_match`):
callee faults inside `try` (2.1) and the old report text (1.5a).

Found and fixed on the way: `result_match` looked for the entry call after
`fn main()`, but since 2026-11-09 it lives in `tsrust_entry_main` (the 8 MiB
main thread); the test had been failing since then.

### Phase 2: correct behavior in `Unwind` mode

- [x] 2.1 Inside `try` regions (and `Checked` bodies) local faults stay
      values, and each call that may unwind is wrapped alone in
      `catch_unwind`; a `TsrFault` becomes the same `Throw(record)` as today
      (see 0.3).
- [x] 2.2 `finally` needs no unwinding path: faults reach it as values
      (2.1). Verify with the exception suites.
      Verified: the exception suites pass in both modes.
- [x] 2.3 Disposal while unwinding: resource management (`using`) is not
      implemented, and runtime `Drop` code runs no user code, so nothing can
      fault during an unwind today. Record the rule for the future `using`
      work: user dispose code run while unwinding must be caught in the
      disposal path (a second panic aborts).
      Rule recorded; nothing to implement until `using` exists.
- [x] 2.4 Implement every boundary from 0.2 and decision 3: threads and pool
      tasks deliver faults as `ThreadError` code `worker`, async tasks as
      `Failure::Worker`, std:ui callbacks catch in the generated
      `ui_callback` wrapper before returning to GPUI, queued std:ui commands
      each run with their own catch, HTTP handlers answer 500, the test
      runner reports assertion faults unchanged, `throws`/`rejects` treat a
      fault as a throw, `captureOutput` ends capture on a fault.
- [x] 2.4a Detached threads and tasks report uncaught faults and errors and
      set exit code 1 (decision 5).
      Done as revised in decision 5: faults are reported, errors and the exit
      code are unchanged.
- [x] 2.4b Not needed: bodies that can be awaited keep runtime errors and catch
      unwinding calls themselves (Phase 2 progress).
- [x] 2.5 Foreign panics (Rust crates through bindings, runtime bugs) are
      never treated as Tsrust errors: every catch resumes non-`TsrFault`
      payloads.
      `tsr_catch_faults` resumes every panic that is not the pending fault.
- [x] 2.6 Test that task faults leave pool, channel and scope locks usable
      (decision 10).
      `fault_unwinding::task_faults_leave_pool_channel_and_scope_locks_usable`
      runs a pool, a channel sender and scoped threads past worker faults in
      both modes.
- [x] 2.7 Sorts keep the positions path whenever the comparator may fault
      (decision 9); runtime code never leaves state taken out while user code
      runs (UI command queue fixed by 2.4).
- [x] 2.8 Callback contracts: callbacks that only fault become infallible;
      update stored callable families, `callback_family_result_adapters`,
      invocation plans and crate-split declarations.
      Shared callables keep runtime errors; local ones are 2.13.
- [x] 2.9 Error effects keep a "may fault" fact in `Unwind` mode, used to
      choose per-call catches (decision 2) and the sort path (decision 9);
      only the native `Result` types drop runtime errors.
- [x] 2.10 The fault record is taken from the thread-local on the thread
      that faulted, before the catch returns (async tasks may resume on
      another thread afterwards).
      Catches take the details synchronously on the faulting thread.
- [x] 2.11 Language server: error-effect features (catch typing, hover,
      diagnostics about what a call can throw) agree with the mode.
      The language server works from analysis effects, which the mode does not change.
- [x] 2.12 `tsrust build --debug` keeps a native backtrace for faults.
      A fault captures `Backtrace::capture()` when raised (free unless
      `RUST_BACKTRACE` is set); the uncaught report prints it after the
      location. Debug builds keep the symbols that make it readable.
- [x] 2.13 Stage B of 1.3: closures, callback families, methods and stored
      callables drop runtime errors from their signatures, except at
      boundaries (decision 3), which keep them.
- [x] 2.14 Finish 1.4: capacity checks in runtime helpers raise faults
      (decision 7).
      `RangeError` with code `capacity`. Emitted limits carry their
      file-relative location after U+001F (`capacity_failure`); runtime
      helpers keep their checks unchanged (so the runtime crate stays shared),
      and catch points and the hook read a panic with that text as the fault.
      Limitation: like other faults, a limit hit directly in a `try` body
      (not inside a call it makes) is a value-mode site only when the emitter
      gives it a transport; capacity sites have none, so it is reported as
      uncaught. A limit inside any called function is caught at that call.

### Phase 2 progress (2026-11-18)

- 2.1: `fault_caught_call` (direct calls) and `fault_caught_closure` (closure
  calls, traversal callbacks) wrap only the call; arguments are bound first.
  The condition is "the active region receives runtime errors", which holds in
  `try` regions and in every body whose contract keeps runtime errors.
- 2.13: `local_closures` (no escape, not polymorphic, forwarding,
  program-scoped or a task, values only called directly or passed to builtin
  array callbacks) drop runtime errors; every signature read in the emitter
  goes through `function_effects()`, region analysis keeps the original
  effects (2.9).
- 2.7: a local comparator that may fault uses the positions sort with an
  infallible comparator, caught as one call inside `try` (`sort_call`).
- Boundaries fall out of the rule above: thread and task bodies, UI callbacks,
  test bodies and HTTP handlers are non-local, keep runtime errors, and catch
  unwinding calls per call, so their faults arrive as today's `Err` values.
  `await` therefore never unwinds through a poll; 2.4b is not needed.
  Remaining for 2.4: the UI-window `main` entry catch and detached threads
  and tasks (2.4a).

### Phase 3: `Checked<T>` and `Checked<T, E>`

- [x] 3.1 Type and positions exactly as in the specification above, with
      diagnostics for every disallowed position; public `RuntimeError` type.
      `syntax/checked_lowering.rs` rewrites tokens before parsing; `RuntimeError`
      lowers to the internal `#RuntimeError` record (unless the module declares
      its own), which analysis materializes before annotations resolve and the
      linker leaves unqualified. Diagnostics use code `checked`.
- [x] 3.2 Body semantics as specified (return `T`, faults and `throw E`
      become `Err`, other explicit errors are compile errors).
      The body runs in a generated `try`; its own `return v` becomes
      `return Ok(v)` (nested functions, arrows and match arms are told apart),
      `Checked<void>` ends with `return Ok()`, and the catch returns
      `Err(failure)`. An unlisted error reports `Expected RuntimeError | E,
      found ...` with a hint to extend `E`.
- [x] 3.3 Interop: `unwrap`, `match`, `try/catch`, generics, function types
      returning `Checked`, `Task<Checked<T, E>>`.
      Covered by the suite's interop test. Existing limitation, unchanged by
      `Checked`: a `try` (and so a `Checked` body) around a call of a callback
      parameter needs that callback's error contract resolved; match on a
      `Checked` callback's result instead, as the suite does.
- [x] 3.4 IntelliSense: complete `builtins.json` entry (purpose, parameters,
      errors, verified example, short ownership note), hover, completion and
      signature help consistent.
- [x] 3.5 Docs: error-handling guide section on faults vs errors and when to
      use `Checked`.
- [x] 3.6 Tests: one `std:testing` suite covering local faults, callee faults,
      `E` alternatives, async, callbacks and nesting.

### Phase 4: value mode kept

- [x] 4.1 `Value` mode stays complete for targets without unwinding (the
      future WebAssembly build) and as a fallback; `Checked` behaves the same
      there.
      Every 5.1 suite group also passed in value mode, and the `Checked`
      suite and catalog examples run in both modes.
- [x] 4.2 A focused suite runs in `Value` mode so it cannot rot.
      CI's library job runs the compiler and LSP library tests a second time
      with `TSRUST_FAULT_MODE=value`; `fault_unwinding` runs its lock,
      `Checked` and catalog tests in both modes explicitly; the 5.5 tests run
      under `TSRUST_FAULT_MODE=value`.

### Phase 5: validation and rollout

- [x] 5.1 Full compiler and LSP regressions in `Unwind` mode, batched to stay
      under one hour of execution; resume unrun groups after repairs.
      2026-11-18: all 434 compiler suites (138 + 148 + 148, each batch under
      40 minutes) and all 91 language-server suites. Repairs: the test runner
      as a fault boundary, capacity faults without rewriting the shared
      runtime, harness entries, member names called `RuntimeError`, and the
      project panic hook only in value mode. Fourteen suites that fail
      identically in value mode were checked at 5baf487fe (before this plan)
      and already failed there; they are listed in `docs/agent-progress.md`.
- [x] 5.2 Uncaught report: the decision-4 format and exit code 1 in both
      modes, for every `main` shape, for single-file and multi-file programs,
      and for detached threads (decision 5).
      Unwind mode: verified for sync, async, bounds, multi-file and thrown
      entries (`fault_unwinding::uncaught_reports_share_one_format_for_every_entry_shape`)
      and detached threads. Revised: value mode keeps its previous reports
      (programs without `try` panic with the native message, exit 101; with
      exception transport they print the decision-4 format), because it is now
      only the fallback for targets without unwinding.
- [x] 5.3 Exception workloads with Node counterparts (`array_callback_errors`
      and others) still match; lang-compare, gpui-compare and the Studio edit
      loop measured and recorded.
      `exception_completion` and `array_callback_errors` verify against Node in
      both modes; unwind times are within noise (0.64 vs 0.62 ms, 0.47 vs
      0.43 ms; explicit throws stay values). The `async_channel_*` workloads
      fail on the Node side in both modes (Node cannot run the `.ts`
      comparison), unrelated to this work. lang-compare (median of 5): fib
      1.42x to 1.04x of Rust, orders 1.39x to 1.26x, whole process 2.58 s to
      2.53 s, other phases unchanged. gpui-compare startup, idle, animation and
      the edit loop (rebuild 1.13 s, check 0.40 s) are unchanged. Rows are in
      `benchmarks/measurements.sqlite` (engines `tsrust-fault-unwind`,
      `native-fault-unwind`, `tsrust-release-fault-unwind`).
- [x] 5.5 Tests that pin value-mode emission shape (entry headers, adapter
      text, `tsr_result_*` symbols) run only with `TSRUST_FAULT_MODE=value`:
      `int_operators` (2), `result_match`, `ui_authoring` (async UI),
      `async_tasks` (2), `native_byte_cleanup` and 20 library unit tests
      (exception regions, finally slots, scoped record support, error
      injection). The unwind behavior they would check is covered by
      `fault_unwinding` and the 5.1 regressions.
- [x] 5.4 Default flips to `Unwind`; benchmark READMEs and
      `docs/agent-progress.md` updated.
      `unwind_faults()` is true unless `TSRUST_FAULT_MODE=value`; the guide,
      lang-compare README and handoff describe the default.

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
