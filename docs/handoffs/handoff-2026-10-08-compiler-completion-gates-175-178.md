# Handoff: compiler completion run, gates 175 to 178

Date: 2026-10-08. Read `AGENTS.md`, `docs/development-workflow.md` and
`docs/plans/compiler-completion-plan.md` first. The plan file holds the owner's execution
plan and, below it, the audited requirement ledger with the state of every row.

## State

HEAD is the commit that adds this file. The working tree holds only the fifteen
protected owner fixture edits. Five consumers are rebuilt from e1fb13c and
smoke-tested (named function expression, recursive inference, `std:path.relative`,
unreachable handler). Evidence is in `benchmarks/records.sqlite`.

## Done in this run

| Gate | Requirement | Result |
| --- | --- | --- |
| audit | Stage 1 ledger | published in `docs/plans/compiler-completion-plan.md` |
| 175 | B1 named function expressions | accepted; workload `named_function_expressions` |
| 176 | B2 recursive result inference | accepted; workload `recursive_result_inference` |
| 177 | B3 unreachable catch bodies; B4 witnesses | accepted |
| 178 | Section A inventory records | nineteen rows reconciled |

Each gate has a milestone record under `docs/milestones/`, bounded compiler checks
with all eight depth canaries, the whole editor crate (364 pass at gate 177) and,
where runtime behavior changed, a matched workload verified untimed.

No complete compiler crate run was made after gate 177; the last complete run is
the integration baseline (2,350 pass). Run it at the next family milestone.

## Not done: the compiler is not complete

The ledger lists what remains. In dependency order:

1. **B15** assignment as a value for strings, arrays and records. Small. Intended
   rule to confirm: the value of the expression is a read-only view of the target.
2. **B5 to B7** scoped aggregate composition, fallible callback fields behind record
   parameters, callbacks stored through a record parameter that borrow from the
   caller. Large; B6 first.
3. **B10 → B11 → B12/B13** S06 Result and exhaustiveness, S07 resources and bytes,
   thread prerequisites, async compiler. Each is a family.
4. **B9** S04 project editor. Independent.
5. **B8** generic adapter compile time. `milestones/generic-state-join-efficiency.md`
   is still marked in progress; reconcile it against the accepted scheduling family
   before new work.
6. **Section C** needs owner decisions before any of it is in scope: classes and
   `this`, the type-level feature set, generators and iterator protocol, BigInt,
   decorators, JavaScript coercions and legacy syntax, and several globals. 179
   language inventory rows are still pending; most are in section C.

## Decisions needed from the owner

1. Which section C families belong to the compiler's approved scope, and which are
   policy exclusions?
2. B14: should a non-empty `switch` case fall through to the next one, or stay an
   error with guidance?
3. B15: confirm that an owned assignment value is a read-only view of the target.
4. Gate 176 accepts recursive functions without a result annotation that strict
   TypeScript rejects (TS7023). Keep this superset, or require the annotation where
   TypeScript does?

## Notes for the next agent

- New gate numbers start at 179.
- `scratch` helper used for matched workloads is not in the repository; copy the
  shape of `benchmarks/runtime/recursive-result-inference-*.mjs` for a new family and
  append the manifest row at the end.
- Strict `tsc` checks the TypeScript counterpart. `tools/node-reference` strips types
  without checking, so a fixture that passes there can still need annotations in a
  counterpart.
- Node 24.11.1 first on `PATH`; `/opt/homebrew/bin/python3` for `store.py`;
  `CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target`, `CARGO_INCREMENTAL=0`.
