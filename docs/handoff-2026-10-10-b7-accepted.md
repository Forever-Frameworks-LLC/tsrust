# Handoff: B7 accepted at gate 183

Date: 2026-10-10. Record: `docs/milestones/183-caller-borrowed-record-stores.md`.
Status of every form: `docs/b7-caller-borrowed-acceptance-map.md`.
Working method: `docs/handoff-2026-10-09-b5-continuation.md` section 4.

B7 is accepted for the scope in the acceptance map. The compiler is not complete: the
ledger rows after B7 are untouched.

## What closed B7

1. Installer closures called through closures that captured them (previous checkpoint,
   `6fb2a331`).
2. A full acceptance probe: 67 programs over the plan's acceptance sequence, each
   compared with Node. Every compiler defect found was repaired or turned into a
   source diagnostic.
3. Family regression, timing and records.

## Defects found by the acceptance probe

- **Exception regions were invisible to summaries.** `fixed_storage::visit_statements`
  and `visit_expressions` did not enter `try`, `catch` or `finally` bodies or `throw`
  values. An installer that stored inside a region had no retained-borrow summary, so
  the source check accepted mutation of a borrowed input and Rust rejected the
  generated code. Both visitors now traverse regions. Every analysis built on
  `visit_function_expressions` gains this coverage.
- **Abstract installer uses reached Rust.** Passing an installer closure or a named
  installer as a callback, converting or aliasing an installer closure, a second alias
  of a saved named installer, storing into a captured record, and a held installer with
  a mutable captured source all failed in generated code, some after the source check
  had released a capture that Rust still held. `installer_value_uses.rs` rejects them
  with `unsupported`. Owning callback fields and `const perform = install` are
  unaffected.
- **A direct store of a borrowing callback into a captured record** failed in generated
  code, also in the accepted B5 build. It is rejected with `unsupported`.

## Decisions I made without asking

- Abstract installer invocation, stores into captured records and held installers with
  mutable sources are *diagnosed*, not implemented. The plan's acceptance sequence does
  not list them, and the plan text already named them as remaining interfaces. If the
  owner wants them inside B7, reopen the row; the acceptance map lists each one.
- Conservative retention (a store that may not run, two stores in one relay, recursive
  relays) is accepted as is. It rejects some valid programs and accepts no invalid one.

## Evidence

- Whole compiler crate 2,702 passed, 0 failed, 3 ignored; whole editor crate 452
  passed, 0 failed. Logs in SQLite: `tsrust-b7-acceptance-whole-compiler-final.log`,
  `tsrust-b7-acceptance-whole-editor.log`. An earlier whole run with two failing
  unit tests (their sources used forms that are now rejected) is kept as
  `tsrust-b7-acceptance-whole-compiler.log`.
- Timing: `runtime/results/2026-10-10T00-39-35-029Z.json`, 186 samples, native 0.276 ms
  and Node-after-tsc 5.003 ms per 2,048-scenario batch. No earlier baseline.
- Workload manifest row is `measured: true`, `implementationAccepted: true`.
- Coverage row `storage.record` lists the workload; `docs/feature-coverage.md` is
  regenerated.
- The five installed consumers are refreshed from the acceptance commit `2c232b4e`
  and smoke-tested: the guide example prints 3 and 10, and mutation of an input
  borrowed through a `finally` store reports `capture-conflict`.

## Not done

- Startup, compilation and allocation profiles for the workload.
- 49 failing benchmark harness tests that also fail before this gate (see the record).
- Everything in "Possible later work" in the acceptance map.
- Ledger rows B8 to B13 and D2 to D5; structural record keys; B14 decision.

## Next

Ledger order after B7 is B8 (generic adapter compile time). Read
`docs/plans/compiler-completion-plan.md` for the dependency order.
