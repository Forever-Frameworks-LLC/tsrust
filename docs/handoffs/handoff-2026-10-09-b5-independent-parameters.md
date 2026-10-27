# Handoff: independent callback environments at parameter boundaries

This checkpoint extends `fc094cd0`. Read AGENTS.md, docs/development-workflow.md,
docs/milestones/b5-scoped-aggregate-acceptance-map.md,
docs/handoffs/handoff-2026-10-09-b5-parameter-stores.md and this file. B5 remains incomplete
pending the mixed-record union scope decision. The fifteen unrelated owner fixture
changes remain uncommitted and untouched.

## Supported behavior

Parameters of one nominal callback-record type can carry independent environments.
Supported boundaries include nullable record fields, direct nullable parameters,
nested record leaves, arrays of records or nullable records, Map values, generic
functions and relay calls. Arrays and Maps of non-nullable callback records also
keep independent environments. Mutable parameters can install independent owned
callback records. Passing the same value to two readonly parameters remains legal;
overlapping mutable arguments still report `conflicting-access`. Captures follow
the existing ownership rules; no runtime clone occurs.

`map.get(key)?.run()` borrows an entry from stable Map storage. The key is evaluated
once, the callback runs only for a present entry, and the Map remains available.
Map iteration and destructuring retain each parameter's concrete callback type.
Callback failures remain catchable through the existing declaration error contract.

## Implementation and lessons

- `nullable_parameter_environments.rs` binds a fresh callback contract identity to
  each environment leaf, reusing its structural callable signature. Nullable,
  array and Map boundaries share this traversal. Scoped record instances keep
  their nominal declaration; parameter-contract markers distinguish abstract
  environments from constructed runtime values.
- `analysis.rs` applies binding after generic source constraints are solved.
  Applying it earlier causes false `Box` versus `Box` mismatches. Compatibility
  accepts marked parameter contracts by nominal identity, including nullable
  array elements. Do not broaden arbitrary concrete-environment compatibility.
- `scoped_record_parameter_views.rs` keeps the nominal transport bridge while
  using this parameter occurrence's nullable leaf types for bounds and accesses.
  The duplicate-contract rejection in `scoped_record_containers.rs` is removed.
  Native names remain `C{id}`, but each leaf has its own checked contract ID.
- `error_effects.rs::record_field_origins` links fresh parameter contracts to their
  declaration error contract. Parameter contracts are not constructed callback
  origins. Reversing this edge leaves fresh contracts ungrounded and breaks catches.
- `scoped_map_carriers.rs` accepts marked nominal parameter instances without
  weakening compatibility between unrelated records.
- `optional_chain.rs` recognizes `get()` on stable Map storage as a borrowed
  source. Consuming its optional view wrongly produces `borrowed-view`.
- `maps.rs::map_iteration_entry` materializes exact checked key/value types into
  the nominal tuple. Reconstructing only their annotations loses callback
  environment identity and leaves an undeclared native `C0` in iteration bodies.

Keep nominal identity, structural callable signatures, concrete environments and
error contracts separate. The fresh IDs are compile-time variables, not runtime
copies or new source record types.

## Verification scope

The initial composition run produced 198 passes and one nullable-array compatibility
regression out of 199 tests. After adding non-nullable container witnesses, the
closing run produced 200 passes and one Map iteration regression out of 201 tests.
Both defects were repaired. The affected array/Map boundary test, independent
container test and optional Map callback test pass after the final repair. All eight
Map entry regression tests pass. Completed unrelated evidence was retained rather
than restarting full regressions.

The expanded independent-parameter witness covers direct nullable arguments,
nested relays, mutable replacement, record arrays, nullable arrays, Maps, generic
functions, callback failures and overlapping mutable argument rejection. A separate
readonly alias witness passes. The optional-chaining suite passes all eight tests.
Both focused editor witnesses and all eight depth canaries pass after the optional
chain repair; the later tuple repair changes checked Map entry metadata only.

The guide example prints `14` on native and Node. The Map optional-call witness
prints `3 -1 3 1 2` natively and on Node, checking presence, Map reuse and key
counts. No full compiler/LSP milestone or timing run is claimed here.

## Benchmark and evidence

The matched workload has 136 observations. Its new helper checks independent
nullable parameter reads and stores, arrays, Map lookups and Map iteration. The
independent oracle formula is `12 * value + 27`, with hand values 39 and 51. Source
pins, mutation guards, TypeScript counterpart and hashes are maintained together.

Intermediate untimed verification is retained in SQLite under
`runtime/results/2026-10-09T15-39-22-614Z-verify.json`. It predates the final plain
container and Map iteration additions and is not final acceptance evidence.
The final verification identifier and exact storage checks follow below.

## Remaining decision

The sole diagnosed B5 form is `Holder | Other` where Holder contains a callback.
The owner has not answered whether to defer it to a tagged-union feature or include
explicit discriminant narrowing in B5. Do not invent a narrowing policy, silently
remove the row, or start dependent work without the answer.

If deferred, record the decision and finish B5's complete compiler/LSP regressions,
matched timing protocol and performance review before marking B5 accepted or the
workload measured. If included, first agree on narrowing, then implement it with
red source/native/editor witnesses and matched workload support.

Structural data-record keys remain separate, with the decided Rust-style equality
policy and number/callable-field exclusions. B7's borrowed parameter escapes are
also separate and are not implemented here. B14 switch fallthrough remains open.
Do not reopen accepted ownership decisions.

## Continuation prompt

Continue from this handoff and the B5 acceptance map. Preserve independent callback
parameter environments, owned parameter stores, readonly views, optional Map calls
and exact Map iteration tuples. Resolve the owner's pending mixed-record union
scope decision before dependent work. Complete the resulting B5 scope, then its
family regressions and timing. Keep unrelated fixtures uncommitted, use explicit
Git paths and preserve exact evidence in SQLite. Do not substitute diagnostics for
supported behavior or describe untimed verification as measurement.

## Environment and cleanup

Use one agent, `/private/tmp/tsrust-gate100-editor-target`, `CARGO_INCREMENTAL=0`,
offline locked Cargo, Node 24.11.1 first on PATH and `/opt/homebrew/bin/python3` for
store.py. Do not compile native-heavy suites alongside the optimized workload.
Format the entire project, holding protected fixture bytes in memory and restoring
only formatter changes to them. No snapshots or archives. Refresh five consumers
atomically from the accepted source commit and smoke the three compiler binaries.
Obsolete variant inspection found none after the previous checkpoint's pruning.

## Final closed evidence

Final formatted registration and mutation guards pass. All five native and
Node-after-tsc correctness cases pass with 136 observations per scenario. The final
SQLite key is `runtime/results/2026-10-09T15-54-48-826Z-verify.json`; `verifiedOnly`
is true and startup, compilation and execution arrays are empty. The exact result
and all closed logs were restored byte for byte from SQLite before loose copies
were removed. Log keys use prefix `tsrust-b5-independent-parameters`.

The last complete compiler/LSP runs remain the earlier `2e49f92b` baseline (2,575
compiler and 424 editor tests). This checkpoint makes no whole-crate claim. Final
whole-project formatting changes zero files and zero fingerprints, with all
fifteen owner fixture edits preserved in memory. Consumer refresh evidence is
stored separately under this checkpoint's log prefix after the source commit.

## Subsequent owner decision (2026-10-09)

The owner chose a dedicated discriminated-union feature for mixed-record unions.
The decision above is resolved: retain the requirement in D2, and complete B5 family
regressions and timing. Do not reopen the choice or describe union support as done.
