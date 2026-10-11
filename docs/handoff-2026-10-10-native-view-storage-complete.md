# Handoff: recursive views and finally cleanup

Current at the commit carrying this note. The original October 10 handoff is
superseded; prior checkpoint evidence remains in Git, SQLite and
`docs/b11-resources-bytes-acceptance-map.md`. S07 is not accepted or measured.

Read `AGENTS.md`, `docs/development-workflow.md`, the acceptance map,
`docs/resources-bytes-plan.md`, `docs/recursive-view-storage-plan.md` and
`docs/standard-library-roadmap.md`. All decisions below are already approved.

## Approved scope and completed work

- Nullable recursive links, including mutual cycles, use unique heap ownership.
  Nonrecursive layouts stay unchanged. No shared ownership or implicit byte copies.
- Recursive records containing views share a real borrow lifetime. Array recursion,
  nullable links, provenance, iterative link destruction/clone and public fixtures
  are implemented. Views remain non-clonable. Recursive records storing callback
  fields still have the explicit callable-layout diagnostic and remain pending.
- View-capturing callbacks can return argument views or views of borrowed Bytes
  arguments. Terminal-branch provenance and temporary record calls are repaired.
- The installed CLI/editor consumers were refreshed from `4367f4cc` with executable
  fixture and LSP smoke checks. Later source work is not installed yet.
- Resource close consumes ownership even on Err. Later use and double close are
  static errors. Drop is a nonthrowing fallback. Bytes and views need no close.
- Whole-owner finally returns were repaired in `ed469b32` using initialized stack
  slots. Partial-field direct cleanup routes landed in `eba57bd2`, preserving
  returned allocations and sibling liveness without cloning or placeholders.

## Current checkpoint: Map callback storage through finally

The retained `Closure(1) to Closure(0)` emitter panic is repaired. Return projections
are limited to their body phase. Cleanup does not acquire a source return carrier
merely because it preserves a pending return. Narrowed Map projections exclude
impossible callback origins instead of converting their environments.

`map_return_storage.rs` consumes a precise typed Map return payload and runs cleanup
before direct exit. `map_catch_finally_storage.rs` applies the same route to body and
catch returns. `exception_contract_emit.rs` preserves direct return/error dispatch
through enclosing plain cleanup. Cleanup with no source control exit does not emit
hypothetical break/continue paths. Ordinary, caught, replacement and nested Map
returns pass, including Map declarations inside an enclosing try.

The Map checkpoint passed 44 exception integration tests, 17 scoped-record
finally tests and 35 finally unit tests. The latest lexical-boundary repair adds
focused disposal coverage; see below. A partial-return disposal witness remains
ignored and still fails when explicitly run. The separate Map native
probe passes allocation identity and exact environment disposal at O0/O3. Formatter
corpus, public assertion fixture, editor capture diagnostics and three benchmark
registration checks pass. Whole-project formatting is unchanged. Exact red and
repaired logs use the SQLite prefix `tsrust-map-finally-`.

The matched `exception_completion` workload now invokes the retained Map callback
on each return path. Its independent expected trace ends in 4 or 8. Source hashes,
mutation guards and Node-after-tsc support are current. See the acceptance map for
the final untimed verification record. No timing or complete family regression ran.

## Latest repair and next step: partial-field completion transport

Regions without partial-field return transfers now keep their lexical completion
boundary. Existing initialized root slots preserve whole-owner availability without
inline cleanup. The original try-local disposal test is enabled; ordinary returns,
conditionally moved local fields and renewed Map callbacks pass at O0/O3. The
classification uses actual checked transfers within each return expression and
ignores earlier field moves. The Map return hooks use the same restriction. Final
focused checks pass: 38 finally unit tests, 44 exception tests, 17 scoped-record
finally tests and the formatter corpus. Whole-project formatting changes no file;
no editor behavior or benchmark source changed. No timing ran.

`try_locals_drop_before_partial_field_return_cleanup` in `src/finally_move_slots.rs`
is the remaining ignored native witness. A local inside try survives into inline
cleanup on a partial-field return. Resource acceptance remains open. Do not replace
this with manual drops that fail on conditional/partial local moves.

A proposed replacement has executable native proof in SQLite:
`tsrust-finally-normal-field-transport.rs` and `.log`. Carry affected outer fields in
`TsrCompletion::Normal(fields)` as well as return payloads. The original body scope
then exits normally and Rust disposes its locals. Match each completion before
cleanup: restore normal fields inside the Normal arm, leave return-transferred
fields unavailable in the Return arm, and retain untouched siblings in their
original storage. Emit cleanup for each checked completion state. Transport the
surviving fields on normal cleanup and restore them inside the final Normal dispatch
arm. Do not first convert that completion back to Normal(()): doing so loses Rust's
initialization proof again. The prototype verifies eight early/normal and cleanup
outcomes at O0/O3, including a borrowed sibling, pointer identity, replacement/error
and restored-field continuation. This is design evidence only; compiler integration
must cover catches, nested cleanup, control exits and precise ownership snapshots.

The previous release at `eba57bd2` reproduced the initial disposal defect. Its
source/native code/log remain under `tsrust-finally-local-drop-`; current root-repair
and residual partial-return evidence uses `tsrust-finally-lexical-`. The release
scratch compiler still contains `4d887f9d` source; the debug compiler contains the
latest repair. Installed consumers remain `4367f4cc`.

Then finish recursive stored callback fields, gate 190's resource backend and
synchronous file proof, and S07 family acceptance: complete compiler/editor
regressions, matched measurements, regression review, justified optimization and
consumer refresh. Continue through the existing S08–S21 roadmap after S07. Preserve
completed evidence; rerun affected checks and resume unrun measurement groups.

## Working environment

One agent in this tree. `CARGO_INCREMENTAL=0`,
`CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target`, cargo `--offline --locked`.
Node 24.11.1 is first on PATH; use `/opt/homebrew/bin/python3` for SQLite blob imports.
Format the whole project before every source checkpoint with
`TSRUST=/private/tmp/tsrust-gate100-editor-target/debug/tsrust node tools/format-project.mjs`.
Keep focused TDD and milestone-only heavy checks. Import/read back exact closed
records before deleting loose copies. Prune obsolete native/test outputs and retain
current executable consumers. Do not create archive directories or snapshots.
