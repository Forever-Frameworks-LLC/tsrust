# Handoff: scoped callable records and gate 173 accepted

Date: 2026-10-07. Read `AGENTS.md` and `docs/development-workflow.md` first, then
the three acceptance records:

- `docs/milestones/173-scoped-callable-records.md` (record gate)
- `docs/milestones/172-nested-function-owned-escape.md`, section "Reconciliation"
- `docs/milestones/173-nested-function-signatures.md`, section "Acceptance"

## State

Three commits close this pass: the record gate (5c5cc68), the gate 172
reconciliation (e682897) and gate 173 (the commit that adds this file). The working
tree holds only the fifteen owner fixture edits under
`crates/tsrust-compiler/tests/fixtures/`. They stay uncommitted, unstaged and
untouched. Use explicit Git paths; never `git add -A` or `git commit -a`.

Installed consumers (`target/debug`, `target/editor/debug`, `target/release`) are
built from the gate 173 sources. Project formatting changes no file.

## Verified

- Whole compiler crate: 2178 pass, 2 fail. Both failures are kept witnesses:
  `generic_call_capture_effects::literal_guard_native_callable_replacement_has_common_carrier`
  and the unit test `owned_factory_updates_are_point_sensitive_and_instances_stay_separate`.
- Whole editor crate: 339 of 339.
- Twelve matched workloads verified untimed on native and Node-after-tsc: the three
  nested function gates, the eight gate 173 workloads and `scoped_callable_records`.
- Evidence is in `benchmarks/records.sqlite`; `target/editor/gate173.log` and
  `target/editor/record-gate.log` are the concise indexes.

## What changed outside the two gates

The whole compiler crate had not been run since gate 171. Its first run showed 52
failures. Seven were regressions in accepted features and are repaired (loop exit
capture loans, read-only callback moves, unreachable catch bodies, recursive `Once`
factories, two diagnostics, carrier planning on unchecked generic bodies, callback
contract grounding for unreferenced functions). About 30 were expectations that
accepted features had made stale. The record gate milestone lists each one.

Run the whole compiler crate at each gate acceptance. It takes about 20 minutes and
is the only check that finds these.

## Open work

1. **Benchmark harness.** `npm test` in `benchmarks/runtime` passes 368 of 419. The
   51 failures are source contracts that match exact one-line text; the project
   formatting milestone reformatted the workloads. Rehashed fingerprints were
   propagated to every pinned copy in this pass (the count was 153 failures before).
   Make the remaining contracts layout-tolerant with `source-contract-tokens.mjs`
   without weakening their mutation checks.
2. **Callable replacement.** The two kept witnesses: replacing a generic callable
   local by a different origin, and point-sensitive mutable factory state.
3. **Scoped callable arrays and composition** (`docs/scoped-callable-aggregate-plan.md`):
   arrays, Maps, Sets and unions of records with borrowing callbacks, replacement of
   nullable borrowing fields, and callbacks stored through a record parameter that
   borrow from the caller.
4. **Fallible callback fields behind record parameters.** A function that calls a
   callback field of a record parameter has no resolved error contract for it.
5. **Unreachable catch bodies** are dropped without type checking.
6. **Named-declaration family milestone** of the nested function plan: requirement
   audit, complete regressions and measurement of the accumulated workloads.

## Environment notes

- Cargo: `CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target`,
  `CARGO_INCREMENTAL=0`, `--offline --locked`.
- Node 24.11.1 must come first on `PATH`
  (`/Users/seancotter/.nvm/versions/node/v24.11.1/bin`); Homebrew's Node 26 breaks the
  benchmark runner. Compiler tests that compare with Node need it on `PATH` too.
- `benchmarks/storage/store.py` needs Python 3.11 or later: use
  `/opt/homebrew/bin/python3`.
- New manifest rows go at the end of the workload list.

## Checks

```
export PATH=/Users/seancotter/.nvm/versions/node/v24.11.1/bin:/opt/homebrew/bin:$PATH
export CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target CARGO_INCREMENTAL=0
cargo test --offline --locked --no-fail-fast -p tsrust-compiler
cargo test --offline --locked --no-fail-fast -p tsrust-lsp
(cd benchmarks/runtime && node run.mjs --verify --only=scoped_callable_records --engines=native,node-tsc)
node tools/format-project.mjs
```
