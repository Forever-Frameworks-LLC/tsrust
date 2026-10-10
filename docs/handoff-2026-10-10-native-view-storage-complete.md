# Handoff: native view source storage matrix

Date: 2026-10-10. Previous handoff:
`docs/handoff-2026-10-10-native-view-storage-matrix.md`. Starting checkpoint was
`b223620f`; this handoff follows `ff481c9c`. The tree is clean.

Read `AGENTS.md`, `docs/development-workflow.md`,
`docs/native-view-callable-storage-plan.md` (the last four sections describe this
work), `docs/native-byte-views-implementation.md`, `docs/resources-bytes-plan.md` and
`docs/b11-resources-bytes-acceptance-map.md`.

## State

Gate 189 and S07 are open. Public `view`/`viewMut` are still guarded by
`native_binding_types::prepare`. Consumers remain at `f569190a`.

Commits of this session, oldest first:

| Commit | Content |
| --- | --- |
| `a27a2ef9` | Data arguments, optional/default/rest parameters, ordinary and `void` results, throwing callbacks; replaced origins kept as dormant provenance |
| `dc9fc468` | Arrays with mixed alternatives; captures of stored families; reusable invocation; untracked returned view diagnostic |
| `cbdbd701` | Field storage joined by the whole-function carrier planner; absent/present nullable transitions; recursive view storage diagnosed |
| `43107664` | Unrelated repair: a `return` inside a `try` or `catch` body breaks straight to the enclosing region |
| `ff481c9c` | Three editor tests follow the gate 188 `clone` catalog entry |

The acceptance map has the row-by-row status.

## How the pieces fit

- **Provenance.** Rust releases the earlier owner of a reassigned local, so conflict
  checks stay precise. A lifetime in a function signature is unified over every value
  a native variable ever holds, so replaced origins stay as dormant loans
  (`retained: false`, `retain_unified_native_view_loans`, `extend_dormant` in
  `native_view_places.rs`). They do not block owners and they reach returned values.
- **Types.** A direct field of a local record, and a local callback array, have one
  final storage type: the join of every stored alternative
  (`join_native_view_storage`, `widen_native_view_array_storage` in
  `native_view_storage_contracts.rs`). `record_sequence_carriers` and
  `native_view_array_carriers` move earlier values into that type.
- **Only whole-record replacement in the declaring block redeclares storage**
  (`native_view_storage_epochs.rs`). Analysis decides this with
  `native_view_declaration_depths`; the pass decides it by finding the `let` in the
  same statement list. Keep the two rules equal.
- **Families.** `native_view_family_emit.rs` emits `self`, `&self` or `&mut self`
  receivers and the planned error layout. Lifetime slots of a captured scoped family
  are enumerated in five places that must agree: the `lifetime_count` leaf in
  `analysis.rs`, `callable_layout.rs`, `native_capture_storage.rs`,
  `scoped_record_emit.rs::record_environment_type`,
  `native_view_environment_emit.rs` and
  `native_view_lifetime_emit.rs::native_view_capture_slots`. Order: alternatives, then
  slots.
- **Written families.** The lifetime-free family of a view callback contract has no
  environment. It is emitted as an uninhabited enum with a generic `call`.

## Remaining work for gate 189

1. **Result that borrows an invocation argument.** Closure parameters are not seeded
   with view provenance (named functions are, in `seed_native_parameters`). Seed
   them, give `closure.function.native_returns` argument roots, and name argument
   lifetimes in the family `call` signature. `validate_abi` already has the guard
   text for this case.
2. **Recursive view storage.** Needs one lifetime shared by every level of a type
   that contains itself. Today `reject_recursive_native_view_storage` diagnoses it.
   Ask the owner whether the plan requires it before building it.
3. **Publication.** Lift the guard in `native_binding_types::prepare`; add typed
   argument alternatives so that `length`, `at`, `copy` and `toUtf8` accept `Bytes`,
   `ByteView` and `ByteViewMut`, and `set` accepts `Bytes` and `ByteViewMut`; write
   complete `builtins.json` entries with executable examples; add public fixtures run
   through the CLI and module path; add matched `subarray` workloads with oracles,
   contracts, registrations and manifest entries.
4. **S07 acceptance.** Gate 190 (resources) is also pending. Then the complete
   compiler and editor regressions, measurements and performance review.

## Known defect outside this gate

A `return` of an owned value that must pass through `finally`, with a later use of
that value, still fails native compilation. The ignored test
`returned_owned_value_through_finally_remains_usable_on_other_paths` in
`crates/tsrust-compiler/tests/exception_return_moves.rs` records it. The return has to
run the cleanup first, so it travels as a completion value and Rust cannot see that
the later use is on another path. A fix needs the cleanup inlined at the return site.

## Evidence

SQLite gate logs: `tsrust-gate189-invocation-*.log`, `tsrust-gate189-matrix-*.log`,
`tsrust-gate189-storage-whole-compiler.log`, `tsrust-gate189-storage-whole-editor.log`
(the editor log holds the three `clone` failures that `ff481c9c` repairs),
`tsrust-catch-return-whole-compiler.log`, `tsrust-catch-return-whole-editor.log`.
Final runs: compiler 2,924 passed, 0 failed, 4 ignored; editor 484 passed, 0 failed,
1 ignored; no aborted target. Project formatting changes no file. No timing ran.

## Continuation prompt

Continue Tsrust at /Users/seancotter/Desktop/dev/learning/tsrust from `ff481c9c`
(tree clean). Read AGENTS.md, docs/development-workflow.md,
docs/handoff-2026-10-10-native-view-storage-complete.md,
docs/native-view-callable-storage-plan.md, docs/native-byte-views-implementation.md,
docs/resources-bytes-plan.md and docs/b11-resources-bytes-acceptance-map.md. The
private source storage matrix for byte views is complete except two diagnosed forms.
Do not redo it. First seed closure parameters with view provenance so that a callback
can return a view it received as an argument. Ask the owner whether recursive
view-containing storage is required before building it. Then publish std:bytes
view/viewMut: lift the guard in native_binding_types::prepare, add typed argument
alternatives for the shared readable and mutable observers, complete IntelliSense
with executable examples, verify public fixtures through the CLI and module path, and
add matched subarray workloads with oracles, contracts, registrations and manifest
entries. Keep real slice lifetimes and exact owner provenance: no implicit clone,
hidden owner, static lifetime, unsafe erasure or integer handle. Finish gate 190 and
the S07 family run only after that. After every cargo test run check for failed tests
and for aborted targets. Environment: CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target,
CARGO_INCREMENTAL=0, cargo --offline --locked, Node 24.11.1 first on PATH,
/opt/homebrew/bin/python3 for benchmarks/storage/store.py. Consumers remain at
f569190a.
