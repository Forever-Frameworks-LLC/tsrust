# Handoff: B5 carrier joins

Date: 2026-10-09. Supersedes `docs/handoffs/handoff-2026-10-09-b5-nullable-fields-accepted.md`.
Status table: `docs/milestones/b5-scoped-aggregate-acceptance-map.md`.
User guide: `docs/guides/language/scoped-callable-transport.md`.

**B5 is not complete.** Most storage forms for capturing records are now supported.
The forms under "Still diagnosed" below report source diagnostics and need work.

## Owner decisions taken on 2026-10-09

1. Control-flow stores use scope-end release (Rust-style, coarser than precise
   release). Details in the acceptance map.
2. Structural record keys reject `number` fields.
3. Structural record keys reject callable fields. `Set<Record>` for a capturing record
   is therefore rejected by policy.
4. A function may return `Record | undefined` when the callback owns its captures.

## Accepted since the previous handoff

Commits, oldest first:

| Commit | Content |
| --- | --- |
| `f4e7a2ac` | Nullable callback records stored in branches, loops, `try` blocks; whole-record replacement; scope-end release rule |
| `996d0e89` | Conditional expressions; nullable results; nullable fields of array elements; formatter keeps `a?.b` together |
| `eda956de` | Record parameters read nullable callback fields; one native enum per nullable record type |
| `2a17bbb5` | Callbacks that own every capture join in record fields, results and elements |
| `2612264c` | Arrays whose element type is a nullable callback record |
| `9e3b87fd` | `pop`/`shift` and record fields for such arrays; clearing through a mutable parameter |
| this commit | Arrays, nullable-element arrays and Maps of capturing records as parameters; empty arrays; array results; owned families in results |

## Model

- A local is *sticky* when a store to it, or to one of its nullable fields, sits in a
  branch, loop, exception region or labelled block, or replaces the whole record
  (`nullable_record_fields::control_stores`, `Analyzer::sticky_nullable`).
- A sticky local has one carrier type for the whole function: the join of every stored
  environment (`record_branch_carriers::join_nullable_record_unions`,
  `join_sticky_record_types`). Stored values are retyped in place
  (`retarget_nullable_store`); reads are retyped by `retype_carrier_read`.
- Capture loans of a sticky local are never released by a store
  (`capture_loan_binding::replace_target_capture_loans`), so the generated Rust and the
  source rule agree.
- Array elements use the same join per array (`nullable_array_slot_carriers`).
- Locals with only direct stores keep the earlier point-specific planner
  (`array_storage_epochs`), which releases captures precisely.
- A callback field of a record joins owned and borrowing callbacks through one family
  layout (`record_field_callback_family`), because a field is invoked through a shared
  borrow of its record.

## Defects repaired on the way

- A call through a retyped callback field kept its checked "absent callback" target, so
  a fallible callback was invoked as infallible. Calls are retargeted to the planned
  environment.
- `x?.method() ?? fallback` failed under an expected type.
- The formatter printed `a ?. b`.
- A record or record array whose nullable field owns its captures had no generic
  argument in its native type.
- Different callbacks that own their captures reached a compiler panic when assigned to
  one record variable.
- `push` did not widen an array that started empty, and a never-populated array of
  capturing records had no native element type.
- Every concrete environment of one Map type now shares its entry layout, so a caller's
  Map matches a callee that iterates entries.
- A returned record or array whose callbacks own every capture was reported as
  borrowing local storage once those callbacks formed a family.

## Evidence

- Whole compiler crate and whole editor crate ran clean at every commit above. The last
  run for this commit is recorded in SQLite as
  `tsrust-b5-containers-whole-compiler.log` and
  `tsrust-b5-containers-whole-editor.log` (2,575 and 424 tests, no failures).
- `scoped_record_composition` workload: registration test passes; native and
  Node-after-tsc verification pass, untimed. New helpers: `nullableControlStores`,
  `nullableValueJoins`, `nullableParameterRead`, `nullableOwnedJoin`,
  `nullableElementSlots`, `containerBoundaries`.
- Earlier whole-crate logs for each step are in SQLite under
  `tsrust-b5-*-whole-compiler*.log`.

## Not measured

No timing run and no performance review. The workload keeps
`measurementStatus: pending-scoped-aggregate-family` and `measured: false` until B5 is
complete.

## Still diagnosed

| Form | Result |
| --- | --- |
| New callback record stored into a nullable field through a `Mutable<Record>` parameter | `unsupported` |
| Two parameters that carry a nullable callback record of one type | `unsupported` |
| Parameter typed `ReadOnly<Record> \| undefined` | `unsupported` |
| Union of a capturing record and another record type | `unsupported` or `type` |
| Moving a nullable callback record out of an array element field | `unsupported` |

Two of these share one root: a nullable capturing record has no borrowed view yet.
The argument of a `ReadOnly<Record> | undefined` parameter is checked as an owned
union, so it would move the caller's value, and a nullable field of an array element
cannot be read into a local. Data records already have such views
(`Type::BorrowedUnion`); the scoped injection path in `analysis.rs` produces
`Type::Union` instead.

Suggested order: borrowed views of nullable capturing records, then parameter stores
together with ledger row B7, then the two-parameter case.

## Other open items

- Structural record keys for data records: policy decided, not started. It does not
  involve capturing records; give it its own ledger row.
- Ledger rows B7 to B13 and D2 to D5 are untouched.
- Owner decisions still open: B14 `switch` fallthrough; whether the formatter may
  rewrite the protected fixtures.

## Working rules

Follow `AGENTS.md` and `docs/development-workflow.md`. Stage explicit paths. Fifteen
fixtures stay modified and unstaged; copy them aside before
`node tools/format-project.mjs` and restore any that change. Use
`CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target`, `CARGO_INCREMENTAL=0`,
`--offline --locked`, Node 24.11.1 first on `PATH`, and `/opt/homebrew/bin/python3` for
`benchmarks/storage/store.py`.
