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

## Done after this handoff was first written

Later commits on the same day: `4d030008` publishes `view`/`viewMut`, the two view
types and the shared observers with catalog entries and editor help; `1587ca82`
switches the formatter to two-space indentation (owner request) and reformats the
project; `5b237a17` adds the matched `native_byte_views` workload; the commit that
carries this note lets a callback return a view it received as an argument.

## Remaining work for gate 189

1. **Recursive view storage.** Plan: `docs/recursive-view-storage-plan.md`. Part B
   (one shared lifetime for a recursive record) needs no language change. Part A
   (a nullable field that refers to its own record, stored behind one heap allocation)
   changes the language and waits for the owner's decision. The owner's target type is
   `Chunk { data: ByteView; next?: Chunk }`, which needs both parts.
2. **A callback that captures a view and also returns a view argument.** The scoped
   family `call` needs a named argument lifetime joined with its capture lifetimes.
   Today it is diagnosed (`native-view-family-emission`).
3. **S07 acceptance.** Gate 190 (resources) is pending. Then the complete family
   regressions, measurements of every byte workload including `native_byte_views`,
   the performance review and the consumer refresh.

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

Continue Tsrust at /Users/seancotter/Desktop/dev/learning/tsrust from the commit that
carries this handoff (tree clean). Read AGENTS.md, docs/development-workflow.md,
docs/handoff-2026-10-10-native-view-storage-complete.md,
docs/recursive-view-storage-plan.md, docs/b11-resources-bytes-acceptance-map.md and
docs/resources-bytes-plan.md. Public byte views are published and documented; do not
redo that work. Implement part B of the recursive view storage plan (one shared
lifetime for a record on a recursive cycle, proved first with array recursion). Ask
the owner to confirm part A (a nullable self-referential field stored behind one heap
allocation) before implementing it; the target is
`type Chunk = { data: ByteView; next?: Chunk; }`. Then let a view-capturing callback
return a view argument, finish gate 190, and run the S07 family acceptance: complete
compiler and editor regressions, measurements, performance review, consumer refresh.
Keep real slice lifetimes and exact owner provenance: no implicit clone, hidden shared
owner, static lifetime, unsafe erasure or integer handle. After every cargo test run
check for failed tests and for aborted targets. The formatter uses two-space
indentation. Environment: CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target,
CARGO_INCREMENTAL=0, cargo --offline --locked, Node 24.11.1 first on PATH,
/opt/homebrew/bin/python3 for benchmarks/storage/store.py. Consumers remain at
f569190a.

## Owner-approved continuation and consumer refresh

The owner approved both parts of the recursive storage plan and the proposed
continuation order. Refresh completed from source checkpoint `4367f4cc`: five
workspace executable consumers, the global CLI, and the local VS Code extension.
Debug and release builds succeeded. Three public byte-view fixtures ran through
the refreshed editor CLI, release CLI, and global CLI; the refreshed LSP passed
stdio diagnostics and hover checks. Restart/reload existing editor processes to
load the new binaries and extension contributions. No timing ran. Exact refresh
evidence is stored in SQLite as `target/editor/consumer-refresh-2026-10-10.json`.

Next: recursive view lifetimes using array recursion, nullable recursive links,
view-capturing callbacks returning argument views, the owned-value return through
finally defect, gate 190, and S07 family acceptance. Existing resource close
semantics remain approved; they do not require closing byte buffers or views.
