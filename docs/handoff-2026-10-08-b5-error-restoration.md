# B5 error restoration checkpoint

B5 scoped aggregate composition remains incomplete. Gate 181 is the last accepted
numbered gate. This checkpoint extends the work after `0a2a5e5`; it is not family
acceptance. The active goal remains the full compiler/editor completion plan.

Read `AGENTS.md`, `docs/development-workflow.md`,
`docs/compiler-completion-plan.md` and `docs/scoped-callable-aggregate-plan.md`.
Only one agent works in this checkout. Preserve the fifteen existing unrelated
fixture modifications without committing them. Their formatting is allowed;
changing their semantics is not. Stage explicit paths.

## Completed work

Checked statement-entry and exact expression availability now select error payloads.
Expression snapshots follow argument/operand analysis and include callee/operator
spans used by native errors. Nested expressions sharing a start byte remain distinct.
Native witnesses cover failure before and after a comma-expression transfer,
nested arithmetic, bounds, and a failing array factory before a later index transfer.
Disposal instrumentation verifies optimization levels 0 and 3. A Map
transferred in a previous statement contributes empty internal storage; source
availability remains consumed. Catch must explicitly restore storage before use.
Empty Map assignment creates fresh native storage, ending the previous lifetime.
Outside completion transport, restoration skips dropping a moved native binding.

Native witnesses cover explicit throw, a later fallible call, a later bounds
failure, record-field storage and an independent sibling field. Disposal tests
at optimization levels 0 and 3 prove exactly one callback drop on both early and
late failure paths. The editor test verifies restoration and rejects reads before
restoration. All 78 composition tests, eight normal-stack depth canaries and 15
exception-contract tests pass. The focused editor test passes.

The matched workload now retains 31 observations. Source guards and rehashed
mutation controls pass. Native and Node-after-tsc verification passes untimed.
The independent oracle includes `value + 29` and `value + 30` for early and late
statement restoration, plus `value + 39` and `value + 40` for expression failures. SQLite preserves exact failures and repaired records; see the plan
for keys, including `runtime/results/2026-10-08T18-30-50-740Z-verify.json`.
Loose copies are removed. Project formatting changes no files on the second pass.
No full family regression or timing run was performed.

## Conditional survivor repair

The committed disposal witness now passes on both branches at optimization levels
0 and 3. `native_storage_availability.rs` separates initialized native owners from
source usability. Checked snapshots retain transfer spans. Completion entry and
balanced branch/block returns initialize native storage; later transfers invalidate
that state. Branch emission isolates sibling states. Error and normal payloads
retain survivors until replacement or scope exit. Source reads remain rejected.

Expanded witnesses cover nested blocks, record-field siblings, later transfer and
subsequent restoration. Direct annotated record field binding removes a named
partially moved temporary that otherwise kept the old callback borrow alive.
Another 42 scoped callable-record tests and four fallible record-callback tests
pass after that change. The 27 matched observations include conditional storage
survivors on normal and error paths. All original failed attempts and repaired
checks are preserved in SQLite; see the current plan for evidence keys.

Native initialization now has its own scoped stack and works outside completion
storage regions. Callable, branch and region boundaries save/restore state.
Outside replacement passes exact disposal checks for both branches, nested blocks,
record-field siblings and later transfer/restoration. Source reads before
replacement still receive transfer diagnostics. The matched workload adds one
independent outside-replacement observation. Evidence is preserved in SQLite as
`tsrust-b5-outside-replacement-batch.log` and the latest verify record.

Next address finally without catch, then loop and other control exits, with
focused native disposal and source witnesses. Nested exception transport and
remaining B5 composition still need implementation; do not infer their support
from the now-passing replacement cases.

## Next implementation

1. Audit rewritten and synthetic expression spans as new compositions land.
   Exact spans now cover the verified call, bounds and arithmetic failure cases.
2. Extend the verified straight-line no-catch finally transport to control exits
   and nested regions without dropping surviving owners before cleanup.
3. Extend completion storage to loops, return, break, continue, nested try and
   finally without catch, with checked control-flow evidence.
4. Finish precise element and Map-entry capture release and the remaining callable
   compositions described in the plan.
5. Implement record Set keys using the owner's Rust-style value equality/hashing
   decision. Equal eligible fields mean one key. Prevent stored-key mutation.
   Callback fields have no automatic equality; explicit stable-key design and
   floating-point policy remain pending.

Maintain matched workload support during implementation. Run complete compiler,
LSP and timing checks only at the completed family milestone, preserving prior
records and resuming unrun groups. B5 is not accepted through conservative rejection.

## Environment

Use `CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target` and
`CARGO_INCREMENTAL=0`, offline locked Cargo. Node 24.11.1 must be first on PATH;
use `/opt/homebrew/bin/python3` for `benchmarks/storage/store.py`. Scratch debug
and release compilers include this checkpoint. Five user executable consumers
still represent gate 181; refresh atomically after accepted source, not as a
claim that B5 is complete. Reuse the single target and prune obsolete artifacts
before a longer handoff; do not create archives or snapshots.

## Continuation prompt

Continue Tsrust B5 from docs/handoff-2026-10-08-b5-error-restoration.md. Read
AGENTS.md, docs/development-workflow.md and docs/scoped-callable-aggregate-plan.md
first. Error restoration after prior-statement and within-expression transfers is verified;
conditional survivor disposal, outside replacement and straight-line finally
without catch now pass; control exits and nested regions remain next. Keep Rust-style record
Set equality/hashing, explicit cloning and existing ownership rules. Use focused
red witnesses, native disposal checks, editor coverage and matched benchmarks.
Preserve fifteen unrelated fixture edits, stage explicit paths, format the whole
project before checkpoints, and preserve exact evidence in SQLite. B5 and the
full compiler/editor completion goal remain active.

## No-catch finally repair checkpoint

The dedicated `map_finally_storage_epochs.rs` helper carries live Map owners into
cleanup on both normal and throwing paths. It preserves pending errors after
successful cleanup and lets cleanup errors override them. Root and record-field
storage pass exact callback disposal checks at optimization levels 0 and 3.
The helper currently covers unlabeled blocks, conditionals, declarations,
expressions, assignments and throws. Loops, break/continue and nested
try regions remain implementation work, not accepted rejection boundaries.

Record inspection admission now excludes borrowed callable fields and descends
through Map/Set contents. Caught-error logging no longer emits invalid inspection
helpers for unrelated callback-bearing records. The affected batch passed 75
composition, 42 scoped-record, four fallible-record, eight depth-canary and 15
exception-contract tests, plus the focused editor check.

The matched workload has 28 independent observations and passes native and
Node-after-tsc verification without timing. Its latest exact SQLite record is
`runtime/results/2026-10-08T18-47-39-945Z-verify.json`. Evidence includes
`tsrust-b5-no-catch-finally-batch.log`,
`tsrust-b5-no-catch-finally-variants-repaired.log`,
`tsrust-b5-no-catch-finally-registration.log` and
`tsrust-b5-no-catch-finally-format-repaired.log`. Earlier failed drafts are retained
separately: unreachable statements after unconditional cleanup throw, and a retired
`mutate` annotation in the new workload helper. Every record was read back byte
for byte before removing its loose copy. Formatting changed no files.

B5 remains unfinished; user executable consumers still represent gate 181.

## Next red witness

`returning_through_finally_preserves_renewed_map_until_cleanup` now includes a
passing early-return case and a failing return-or-fallthrough composition. The
early branch returns 23 through cleanup; the other branch clears/refills, runs
cleanup, then mutates the released old/new captures and returns 29. Source
analysis accepts both, but the second generated native program fails with E0502.
The no-catch helper excludes returns, so this falls back to completion emission
without the required native lifetime renewal. Preserve both cases and extend
completion storage to carry owners through pending returns without premature
disposal. Exact evidence is verified in SQLite as
`tsrust-b5-return-finally-red.log` (passing initial case) and
`tsrust-b5-return-finally-fallthrough-red.log` (confirmed defect).

Continuation: start from checkpoint `18f4833` plus this committed red witness.
Repair the combined return/fallthrough case, keep exact Drop checks at O0/O3,
then extend editor and matched workload coverage. Continue B5's remaining scope
from the plan; the full compiler/editor goal remains active.

## Return-through-finally repair

The red witness from `a611cb7` is repaired. Return transport carries the native
Map owner alongside the already evaluated result; cleanup receives that owner
before resuming the result or replacing it with its own return/error. Checked
post-return snapshots preserve consumed storage rather than restoring source
availability. Native root and record-field witnesses verify early return,
fallthrough capture mutation and cleanup-return override with exact disposal
at optimization levels 0 and 3.

The affected batch passed the other 75 composition tests, 42 scoped-record tests
and four fallible-record tests. The sole failed override case was repaired and
rerun with all variants. Eight depth canaries, 15 exception-contract tests and
the focused editor test pass. Whole-family regression and timing remain deferred.
The formatter changed only the new workload layout and its verified fingerprint.
The 29-observation workload passes native and Node-after-tsc verification untimed;
its SQLite record is `runtime/results/2026-10-08T18-55-03-597Z-verify.json`.
Repair attempts, the original failed batch and mutation-control check, and their
successful supplements remain byte-exact in SQLite under
`tsrust-b5-return-finally-*.log`; loose copies were removed after verification.

Next: add focused control-exit and nested-region witnesses; implement the
required transport without shortening capture lifetimes. Keep precise element/
entry release and Rust-style record Set equality/hashing in B5's remaining scope.
B5 is incomplete. Five user consumers remain at gate 181.

Continuation prompt: Continue from this return-through-finally checkpoint. Read
AGENTS.md, docs/development-workflow.md and docs/scoped-callable-aggregate-plan.md.
Return/fallthrough and cleanup-return override pass with disposal evidence.
Finish B5 control exits/nested regions, precise capture release and record keys,
using red witnesses, native/editor coverage and matched untimed workload support.
Preserve the fifteen unrelated fixture edits; stage explicit paths and format
the entire project before checkpoints. Keep SQLite evidence exact and do not
claim family acceptance or measurements before their milestone checks.

## Loop-local control exits repaired

After red checkpoint `a95e5af`, storage-carrying exits distinguish return, break
and continue. Finally receives live Map owners before dispatching the pending
exit. Root and record-field disposal witnesses at O0/O3 cover normal fallthrough,
continue, break and cleanup break overriding pending continue. Mutation after
normal cleanup releases the old/new callback borrows before another iteration.

The affected batch passes 77 composition, 42 scoped-record and four fallible-record
tests, eight depth canaries, 15 exception-contract tests and the focused editor
check. All 30 matched observations verify untimed on native and Node-after-tsc.
The record is `runtime/results/2026-10-08T19-01-37-909Z-verify.json`; exact
`tsrust-b5-loop-finally-*.log` repair/batch/canary/registration/format evidence is
also read-back verified in SQLite. Loose copies are removed. Formatting changed
only the new workload layout and its verified fingerprint.

These witnesses declare their Map within the loop. Next add a witness whose Map
is declared before the loop and must survive break/continue. Loops inside try
and nested regions remain open, along with precise entry/element release and
record value keys. B5 remains unfinished; five user consumers remain at gate181.

Continuation prompt: Read this handoff, AGENTS.md and development-workflow.md.
Resume B5 from the loop-local completion-storage checkpoint. Add a red outer-Map
loop-boundary witness before extending the carrier; preserve exact native
disposal, editor behavior, matched workload/oracle/guards and SQLite evidence.
Continue remaining B5 scope from scoped-callable-aggregate-plan.md. Use explicit
Git paths, preserve unrelated fixtures and format the whole project. Do not
claim family acceptance or measured performance before completing milestone work.

## Outer Map loop boundary red witness

The loop-local repair is committed as `2e63d02`. Its outer-storage follow-up,
`outer_map_survives_loop_exit_cleanup_and_capture_renewal`, is source-valid but
fails native E0502. A Map declared before the loop is refilled each iteration,
cleared in finally on continue, normal and break paths, then observed after
the loop. Captures are mutated after normal cleanup and after the loop. Exact
disposal remains asserted at O0/O3. SQLite retains the original failure as
`tsrust-b5-outer-map-loop-red.log`; its loose copy was removed after read-back.

Next continuation: repair the loop storage boundary before nested regions.
Carry renewed owners across iterations and out of the loop without extending
callback loans or dropping owners before cleanup. Preserve the complete witness
and the accepted loop-local cases. Continue remaining B5 work and the full
compiler/editor plan after this repair.

## Empty outer-Map loop epochs repaired

The red witness in `a8cf844` now passes. The separate
`map_loop_storage_epochs.rs` helper admits an initially empty root Map declared
immediately before a while loop, with an unconditional direct finally clear and
no later refill. It retains the original empty owner and creates an independent
owner for each iteration, forwarding declaration context into completion storage.
Size reads after clear are admitted. Zero iterations and continue/fallthrough/
break paths preserve the empty owner observed after the loop. Exact callback
disposal is verified at O0/O3, including post-clear size observation.

The affected batch passes 78 composition, 42 scoped-record and four fallible-record
tests. Eight normal-stack depth canaries and the focused editor check pass. A
post-clear size read discovered by benchmark compilation was repaired and rerun
in the expanded focused native witness. Registration checks each loop helper
independently; duplicate scenario operations no longer mask deleted operations.
All 31 matched observations verify untimed on native and Node-after-tsc, record
`runtime/results/2026-10-08T19-14-53-091Z-verify.json`. Failed attempts and passing
supplements are verified byte-exact in SQLite under
`tsrust-b5-outer-map-loop-*.log`; loose copies are removed. Final formatting
changed no files. Five user consumers still represent gate181.

This is not general loop-family acceptance. Maps retaining entries across
iterations, populated initial owners, outer record-field storage, broader loop
forms and nested exception regions remain implementation work. Source-valid
compositions that need these boundaries must be implemented, not relabeled as
accepted rejections. Precise entry/element release and Rust-style record Set
equality/hashing also remain in B5. Full family regressions and timing are pending.

Continuation prompt: Continue B5 from this empty outer-Map iteration checkpoint.
Read AGENTS.md, development-workflow.md and scoped-callable-aggregate-plan.md.
Add red witnesses for retained/populated owners and outer record-field storage,
then extend the loop boundary transport while preserving exact disposal,
source ownership and native lifetime safety. Continue nested regions and the
remaining B5 scope. Keep compiler/editor parity, matched untimed workloads,
per-helper mutation guards and exact SQLite evidence current. Preserve unrelated
fixture edits, stage explicit paths and format the whole project before commits.
The full compiler/editor completion goal remains active.

## Populated and record-field loop witnesses

Retained entries across iterations already pass, including replacement disposal
and explicit post-loop clear. A populated initial owner requires that explicit
clear before capture mutation: zero iterations can retain its callback. The
first test draft omitted clear and correctly received `capture-conflict`; the
corrected populated witness passes exact native disposal.

The outer record-field loop witness instead exposes an emitter panic before
loop lowering: `unchecked record carrier conversion Map(8) to Map(2)`.
`nested_empty_map_initializer_adapts_record_callback_carriers` isolates that
panic without a loop. Repair contextual empty Map initialization/Map carrier
conversion, then resume outer record-field iteration support. Exact originals
are in SQLite as `tsrust-b5-populated-retained-loop-red.log`,
`tsrust-b5-outer-record-loop-red.log` and
`tsrust-b5-map-carrier-conversion-red.log`; loose copies were removed after
byte-exact read-back. Existing owner fixture changes remain untouched.

## Contextual nested Map initializer repaired

The constructor panic isolated in `8014586` is repaired. Checked record carrier
chains now contextualize empty Map initializers in the destination carrier.
Record literal emission retains source initializer order, once-only evaluation
and omitted optional fields. Values and existing owners still use ordinary
carrier conversion; the repair introduces no runtime clone. Native examples
verify marker initialization exactly once, reordered fields and absent labels.

The affected batch passes 81 composition tests, 42 scoped-record and four
fallible-record tests; the pending outer record-field loop witness is excluded
from that green batch and rerun independently. It now fails native E0502 rather
than panicking, confirming the separate lifetime boundary work. Eight normal-
stack depth canaries and focused editor diagnostics pass. All 32 matched
observations verify untimed on native and Node-after-tsc, record
`runtime/results/2026-10-08T19-28-27-873Z-verify.json`. Exact failed attempts and
passing supplements are read-back verified in SQLite under
`tsrust-b5-map-carrier-*.log`; the remaining red native witness is
`tsrust-b5-outer-record-loop-native-red.log`. Loose copies are removed.
The workload's first draft used Box rather than RefillBox for its marker field;
that draft failure is preserved separately. Project formatting changed only the
new workload layout and its token-preserving fingerprint.

Next: extend `map_loop_storage_epochs.rs` to empty owning record leaves. Resolve
the Map leaf through the record layout, prove its initializer is empty through
checked carrier wrappers, and retain sibling fields and key-order metadata.
Permit independent sibling access after cleanup while preventing refills or
whole-record transfers from bypassing the epoch proof. Preserve exact root and
record-field disposal, zero-iteration behavior and source ownership rules.
Retained/populated root Maps with explicit post-loop clear already pass; do not
reimplement them or weaken the correct zero-iteration capture-conflict rule.

Continuation prompt: Continue B5 from this constructor repair checkpoint. Read
AGENTS.md, development-workflow.md and scoped-callable-aggregate-plan.md. Repair
`outer_record_map_cleanup_releases_captures_across_iterations` (native E0502) by
extending empty loop epochs to record leaves, with sibling and zero-iteration
witnesses. Keep compiler/editor parity, matched workloads, independent oracles
and mutation controls current. Preserve exact evidence in SQLite, unrelated
fixture edits and explicit Git staging. Format the whole project before commits.
B5, full family regressions/timing and the full compiler/editor goal remain open.

## Empty record-leaf loop epochs repaired

The remaining record-field witness now passes. `map_loop_storage_epochs.rs`
resolves owning Map leaves through record layouts and proves their literal
initializers empty through checked carrier wrappers. Each iteration receives
a fresh native Map leaf; the original empty owner and sibling fields remain
intact. Post-cleanup path checks allow independent siblings, size reads and
Object.keys metadata reads, while excluding transfers/refills of the selected
leaf and whole-record operations that could retain it.

Native disposal witnesses at O0/O3 cover continue, normal fallthrough, break,
sibling mutation, record key order and zero iterations. The affected batch
passes all 82 composition tests, 42 scoped-record and four fallible-record
tests. Eight normal-stack depth canaries and focused editor diagnostics pass.
All 33 independent matched observations verify untimed on native and
Node-after-tsc, record `runtime/results/2026-10-08T19-37-33-183Z-verify.json`.
Exact `tsrust-b5-record-leaf-loop-*.log` evidence is read-back verified in
SQLite; loose copies are removed. Formatting changed only the new workload
layout and its token-preserving fingerprint.

B5 remains open. The epoch helper still targets a directly initialized empty
owner, one proven cleanup boundary and while loops. Multiple renewed leaves,
broader loop forms, nested exception regions, precise entry/element capture
release and remaining callable/nullable compositions require witnesses and
implementation. Retained/populated root Maps with explicit post-loop clear
already pass; preserve the correct zero-iteration capture-conflict rule.
Rust-style record Set equality/hash work and family regressions/timing remain
pending. Five user consumers still represent gate181.

Continuation prompt: Continue B5 from this record-leaf loop checkpoint. Read
AGENTS.md, development-workflow.md and scoped-callable-aggregate-plan.md. Root
and record-leaf empty loop epochs now pass disposal, sibling/key-order and
zero-iteration checks. Add nested exception-region and broader loop witnesses,
then extend completion storage without premature disposal or implicit cloning.
Keep compiler/editor parity, matched workloads, independent oracles, mutation
controls and exact SQLite evidence current. Preserve unrelated fixtures and
explicit Git staging; format the whole project before checkpoints. Complete
remaining B5 scope before family regressions/timing or acceptance. The full
compiler/editor completion goal remains active.

## Nested finally red witness

`nested_finally_preserves_map_owners_and_cleanup_order` is source-valid but
fails native E0502 after both cleanup blocks. It checks normal/failing body paths,
inner-before-outer cleanup order, exact callback disposal and capture mutation
after successful cleanup. SQLite preserves `tsrust-b5-nested-finally-red.log`
byte-exact; its loose copy was removed. Extend nested completion dispatch to
carry initialized storage and evaluated control payloads into enclosing regions,
without restoring consumed source names or dropping owners before cleanup.

## Nested finally storage repair checkpoint

Nested no-catch finally regions now carry Map storage through normal completion,
errors and evaluated returns. Cleanup errors replace pending errors while outer
cleanup still runs. Root Maps and owning record fields preserve exact callback
disposal and inner-before-outer cleanup order. Consumed source names are not
restored, and the transport introduces no implicit cloning.

The affected batch passes 83 composition tests, 42 scoped-record tests and four
fallible-record tests. All eight normal-stack depth canaries, 15 exception-contract
unit tests and focused editor diagnostics pass. Exact native disposal witnesses
pass at O0 and O3, including cleanup-error override and a pending return. The
matched workload verifies all 34 independent observations untimed on native and
Node-after-tsc. Registration and rehashed mutation rejection checks pass after
project formatting. SQLite preserves the failed initial repair and all successful
`tsrust-b5-nested-finally-*.log` evidence byte-exact, together with
`runtime/results/2026-10-08T19-49-50-065Z-verify.json`; loose copies are removed.

B5 is not accepted. Nested catch regions, broader loop forms, multiple renewed
owners, precise entry/element capture release and remaining callable compositions
still need implementation witnesses. Record Set keys will use Rust-style value
equality and hashing, immutable stored keys and explicit ownership transfer or
clone. Floating-point key policy remains unresolved. Family regressions and
timing remain pending. Five user executable consumers still represent gate181.

Continuation prompt: Continue B5 from this nested-finally repair checkpoint. Read
AGENTS.md, docs/development-workflow.md and docs/scoped-callable-aggregate-plan.md.
Add a source-valid nested-catch storage witness next; repair completion transport
without premature disposal, implicit clone or restoring consumed source names.
Keep compiler/editor behavior, matched workload support and SQLite evidence
current. Preserve unrelated fixture semantics and use explicit Git staging.
Format the whole project before each source checkpoint. Complete remaining B5
scope before its family regressions, timing and acceptance. The full compiler
and editor goal remains active.

## Nested catch discovery repair checkpoint

Outer finally storage discovery now traverses nested catch bodies as well as
inner try and finally bodies. Previously source-valid nested handlers left
callback capture loans alive in generated Rust after outer cleanup. The focused
witness now passes exact disposal and cleanup-order assertions at O0 and O3
on normal, caught and rethrown paths. The affected composition batch passes
84 tests; focused editor diagnostics pass. The matched workload adds an
independent caught-refill observation, with source guards and rehashed mutation
controls. All 35 observations verify untimed on native and Node-after-tsc.

SQLite preserves `tsrust-b5-nested-catch-*.log` byte-exact, including initial
witness-count and command errors, and the successful verification record
`runtime/results/2026-10-08T19-56-56-300Z-verify.json`. Loose copies are removed.
Project formatting preserves tokens and refreshes only the changed fingerprint.

This checkpoint does not accept B5. Nested catch regions with their own abrupt
cleanup or return/control exits still need witnesses. Broader loops, multiple
renewed owners, precise entry/element release, remaining callable compositions
and Rust-style record Set keys remain open, followed by family regressions and
timing. Five user executable consumers remain at gate181.

Continuation prompt: Continue B5 after nested catch storage discovery. Read
AGENTS.md, docs/development-workflow.md and docs/scoped-callable-aggregate-plan.md.
Add a nested catch-plus-finally witness whose handler throws or returns and whose
inner cleanup must preserve owners until outer cleanup. Use focused red tests
before extending carried completion transport. Preserve explicit ownership,
exact disposal, editor behavior, matched workload oracles and SQLite evidence.
Format the entire project before checkpoints; stage explicit paths and preserve
unrelated fixture semantics. B5 and the full compiler/editor goal remain active.

## Nested handler cleanup storage checkpoint

Catch-region cleanup now carries initialized owners on abrupt completion and
rebinds them before resuming a pending error. It shares the dedicated carried
cleanup dispatch helper with no-catch finally regions. This repairs native E0382
when a nested handler rethrows through its own finally into outer cleanup.

Native witnesses pass exact callback disposal and inner-before-outer cleanup
order at O0 and O3. A cleanup error replaces the handler error, and outer cleanup
still observes the explicitly cleared owner. The affected batch passes all
84 composition, 42 scoped-record and four fallible-record tests. Focused editor
diagnostics pass. Its first fixture included unconditional abrupt cleanup and
unreachable statements; the reported diagnostic was valid, and the fixture was
corrected to the conditional native witness.

All 36 matched observations verify untimed on native and Node-after-tsc. Source
guards and rehashed mutation rejection pass; the repeated throw-9 snippet now
has an occurrence-count guard so a different scenario cannot mask its removal.
Project formatting preserves tokens and updates the changed source fingerprint.
SQLite preserves every `tsrust-b5-nested-handler-cleanup-*.log` byte-exact, including
the original native failure and failed check attempts, plus verification record
`runtime/results/2026-10-08T20-00-54-839Z-verify.json`. Loose copies are removed.

B5 remains open. Return/break/continue through nested handlers, broader loops,
multiple renewed owners, precise element/entry release, remaining callable
compositions and record Set equality/hash remain pending. Family regressions and
timing are not run. Five user executable consumers remain at gate181.

Continuation prompt: Continue B5 from the nested handler cleanup checkpoint.
Read AGENTS.md, docs/development-workflow.md and the scoped aggregate plan. Add
a source-valid return through a nested catch-plus-finally region next, preserving
the evaluated return payload, callback disposal and outer cleanup. Extend handler
completion storage for control exits where the native witness requires it. Keep
editor behavior, matched workload support and exact SQLite evidence current.
Format the whole project before checkpoints and stage explicit paths. Preserve
unrelated fixture semantics. B5 and the full compiler/editor goal remain active.

## Nested handler return storage checkpoint

Catch-region completion now carries evaluated return payloads with initialized
Map owners, runs a dedicated cleanup path for pending control and rebinds storage
before resuming into the enclosing region. Normal return and handler return both
preserve exact callback disposal and inner-before-outer cleanup order.

The initial return witness already passed. Its stronger mutation witness first
reported a valid source capture conflict: fallible console observation before
inner clear can bypass that clear. The retained rejection asserts that rule.
Explicit outer clear makes the source valid and exposed native E0502; the storage
transport repair resolves that defect without changing source ownership rules.
Native witnesses at O0/O3 verify return evaluation exactly once, unchanged return
values and capture mutation after explicit cleanup. The affected batch passes
85 composition, 42 scoped-record and four fallible-record tests; focused editor
diagnostics pass. All 37 matched observations verify untimed on native and
Node-after-tsc, including return values evaluated before cleanup changes captures.
Registration, mutation controls and token-preserving project formatting pass.

SQLite preserves `tsrust-b5-handler-return-*.log` byte-exact, including isolation
and the original rejection/native failure, plus verification record
`runtime/results/2026-10-08T20-07-26-037Z-verify.json`. Loose copies are removed.
B5 remains open; loop exits through handlers, broader loops, multiple owners,
precise element/entry release, remaining callable compositions and record Set
equality/hash still need work. Family regressions and timing remain pending.
Five user executable consumers remain at gate181.

Continuation prompt: Continue B5 from the nested handler return checkpoint.
Read AGENTS.md, docs/development-workflow.md and the scoped aggregate plan. Add
loop-exit witnesses through catch-plus-finally, including continue, break and
cleanup overrides, then repair any failure. Do not reimplement passing returns
or weaken the correct capture-conflict when an inner observation can fail before
clear. Keep exact native disposal, editor behavior, matched workloads and SQLite
evidence current. Format the whole project before checkpoints; stage explicit
paths and preserve unrelated fixture semantics. B5 and the full compiler/editor
goal remain active.

## Catch loop storage checkpoint and retained cleanup-mutation witness

Nested handler continue/break and an outer cleanup overriding continue already
pass through the existing carried control transport. New exact-disposal witnesses
cover these paths at O0/O3. A direct catch-plus-finally loop with an initially empty
outer Map exposed native E0502 after cleanup. Empty while epochs now admit catch
regions under the existing unconditional-clear and no-refill proof. Root and
record-field owners, zero iterations, normal fallthrough, continue and break pass.

One distinct defect remains red: `catch_cleanup_clear_releases_captures_before_cleanup_mutation`
is source-valid but fails generated Rust E0502 when capture mutation occurs after
clear within that same finally block. Moving the mutation after the cleanup passes;
the original behavior remains a real executable witness, not an accepted source
restriction. Investigate generic owner lifetimes in typed cleanup completion and
end the emptied owner's native lifetime before mutation without weakening source
checks or moving observable cleanup operations. The first attempt merely admitting
catch loops did not repair this defect; original evidence is retained.

The affected batch passes 87 composition, 42 scoped-record and four fallible-record
tests, explicitly excluding that one red witness. Focused editor diagnostics pass.
All 38 matched observations verify untimed on native and Node-after-tsc, with source
guards, independent arithmetic oracle, registration and rehashed mutation rejection.
Project formatting preserves tokens and refreshes the changed source fingerprint.
SQLite preserves `tsrust-b5-handler-loop-*.log`, `tsrust-b5-direct-catch-loop-*.log`
and `tsrust-b5-catch-cleanup-mutation-*.log` byte-exact, including failed attempts,
plus `runtime/results/2026-10-08T20-14-14-148Z-verify.json`. Loose copies are removed.

B5 is not accepted. Fix the retained cleanup-mutation defect next, then broader
loops, multiple owners, precise entry/element release, remaining callable
compositions and Rust-style record Set equality/hash. Family regressions and
timing remain pending. Five user executable consumers remain at gate181.

Continuation prompt: Continue B5 from this catch-loop checkpoint. Read AGENTS.md,
docs/development-workflow.md and the scoped aggregate plan. Repair the retained
`catch_cleanup_clear_releases_captures_before_cleanup_mutation` native E0502.
It is source-valid; do not reject it, move its mutations outside cleanup or remove
the witness. Direct catch-loop root/record empty epochs and nested control exits
already pass. Preserve exact O0/O3 disposal, cleanup order and source ownership.
Complete editor and matched workload coverage for the repair, then proceed with
remaining B5 scope. Keep SQLite evidence, whole-project formatting and explicit
Git staging. Preserve unrelated fixtures. The full goal remains active.

## Cleanup-mutation investigation checkpoint

The source-valid cleanup-mutation witness remains red at the loop checkpoint.
Focused native experiments show that naming completion temporaries alone,
separating control from storage, splitting cleanup at the clear, and reconstructing
empty storage after an owner-free suffix do not independently resolve E0502.
Combined separation and scoped-suffix experiments also fail. All unsuccessful
compiler changes and temporary instrumentation have been removed; source remains
at the verified catch-loop implementation. The retained executable witness is
unchanged. This is an investigation checkpoint, not an implementation acceptance.

Diagnostics identify both completion shells and scoped Map bindings whose drop
glue retains borrowed callback lifetimes across cleanup. The exact cause still
needs reduction; do not treat the temporary-shell explanation as proven sufficient.
SQLite contains each `tsrust-b5-cleanup-*.log` experiment, original reproduction
and the complete formatted generated Rust as `tsrust-cleanup-native-repro.rs`.
Each record is read back byte-exact and its loose copy removed. Project formatting
changes no files. No family regression, timing or consumer refresh was performed.

Next work should reduce the generated Rust ownership/control shape before further
compiler changes. One untested direction is dispatching abrupt prefix completion
directly into its enclosing region instead of joining its borrowed owner type with
fresh empty suffix storage. Preserve nested outer cleanup: any escaping error must
carry surviving owners where required, and normal cleanup must drop them before
mutation. Keep the current source witness and exact disposal assertions. Avoid
unsafe lifetime erasure, implicit clones, moving source mutations, or rejecting
source-valid cleanup. B5 and the full goal remain active.

Continuation prompt: Continue the retained B5 cleanup-mutation repair from this
investigation checkpoint. Read AGENTS.md, development-workflow.md and the scoped
aggregate plan. Restore `tsrust-cleanup-native-repro.rs` from SQLite into the shared
temporary build area for focused reduction. Do not repeat naming-only or scoped
suffix experiments as if untested; their native failures are preserved. Repair
`catch_cleanup_clear_releases_captures_before_cleanup_mutation`, then complete
compiler/native/editor and matched workload coverage. Preserve checked ownership,
cleanup order, exact disposal, current passed evidence and unrelated fixtures.
Format the whole project and stage explicit paths before source checkpoints.

## Owner-scope reduction

The direct-prefix and Result-control bridge experiments also fail. Removing
impossible Result control exits first produced E0282; specifying the unused
return type exposed E0502 again. The inference failure had masked borrow
checking. All experimental compiler changes are removed. Do not treat this
bridge change as a verified ownership repair.

A reduced Rust witness succeeds when both the input owner and its completion
enum are declared inside a block that returns only the extracted owner. After
that block ends, dropping the extracted owner permits capture mutation. The
witness includes an unreachable control variant and an observable destructor.
Native output is `drop` followed by `9`. This proves a useful scope construction;
it does not prove that the full generated cleanup shape is repaired.

Next, join pending try and catch outcomes into one carried completion in an
inner scope. Extract pure control/error and storage after that scope ends, then
emit cleanup once outside the pending and handler owner scopes. Cleanup itself
may still need a separate prefix scope ending at its unconditional clear. Keep
checked error conversions, pending return evaluation, cleanup overrides and
exact disposal. Test the retained red witness before broadening this change.

SQLite stores `tsrust-cleanup-owner-scope.rs` and five direct-prefix/Result-bridge
logs with exact read-back verification. Loose copies are removed. The compiler
source is restored to HEAD. B5 remains incomplete, and no consumer refresh,
family regression or timing has occurred.

The owner explicitly prefers Rust-style record Set equality and hashing. Eligible
records compare by field value; insertion follows ownership transfer, with an
explicit clone when a retained copy is needed. Stored equality/hash inputs must
remain stable. Records with unsupported fields require a stable key rather than
implicit object identity. Floating-point key policy remains unresolved.

## Cleanup capture release checkpoint

The retained `catch_cleanup_clear_releases_captures_before_cleanup_mutation`
witness now passes native execution at O0 and O3, including exact disposal.
The test also checks a record-field Map owner. Pending try/catch outcomes finish
inside an owner scope, and the emitted storage tuple is destructured directly
before cleanup begins. This ends the completion shell and input-owner lifetimes
before a clear releases callbacks and capture mutation follows.

This lowering applies when cleanup directly clears every carried Map. Paths
retaining callbacks keep their existing branch-specific transport: using the
joined owner type for those paths caused six native regressions. The failed run
is retained, followed by a corrected affected batch with 88 composition, 42
scoped-record and four fallible-record tests passing. The additional record-field
witness and focused editor diagnostic check pass separately. No source restriction,
implicit clone or unsafe lifetime erasure was introduced.

The matched caught-loop scenario now mutates captures inside finally after clear.
All 38 observations verify untimed on native and Node-after-tsc. Independent
arithmetic expectations remain unchanged; source guards, mutation controls and
registration pass with refreshed hashes. SQLite retains the original failed and
successful `tsrust-b5-cleanup-joined-*` and `tsrust-b5-cleanup-selected-*` logs plus
`runtime/results/2026-10-08T21-24-07-353Z-verify.json`, verified by exact read-back.
Whole-project formatting changes only the benchmark source layout and fingerprint.

B5 remains incomplete. Continue with broader loop/nested-region discovery,
multiple owners, precise Map-entry/array-element release and remaining callable
compositions, then Rust-style record Set equality/hash. Keep full family checks,
timing and consumer refresh separate from this focused source checkpoint. Five
user executable consumers remain at gate181; the scratch CLI includes this repair.

Continuation prompt: Continue B5 after the cleanup capture release repair.
Read AGENTS.md, development-workflow.md and this handoff. The former red cleanup
witness now passes for root and record-field Maps at O0/O3. Preserve branch-specific
transport for retained callbacks, and direct destructuring for release cleanup.
Proceed with broader loops, multiple owners and precise entry/element release.
Keep editor, matched workloads and SQLite evidence current, format the whole
project before checkpoints, and stage explicit paths. Preserve unrelated fixture
semantics. The family and full goal remain active.

## Multiple empty Map loop owners checkpoint

A new paired-owner witness exposed native E0382 and E0502 when two record-field
Maps were cleared in catch/finally. Empty while epoch discovery previously
renewed only one leaf. It now discovers consecutive empty Map declarations and
all empty owning record Map leaves, proves each selected leaf is directly cleared,
checks each post-clear suffix independently for refill/transfer, and renews all
selected native owners per iteration. The original empty owners remain available
for zero iterations and outer errors. Source ownership checks remain unchanged.

Exact O0/O3 disposal passes for paired record fields and separate local Maps,
including continue, normal fallthrough, break and zero iterations. The affected
batch preserves 88 existing composition, 42 scoped-record and four fallible-record
passes. The new witness passes separately after correcting a test-source
replacement that had left its record initializer in the local-Map variant. The
original failed batch and corrected test are both preserved. Focused editor
checks pass for the paired-owner source.

The matched workload adds a 39th paired-owner observation. Its independent oracle
is `2 * value + 904`: three cleanup visits see six total entries before clear,
zero afterward, final captures sum to `2 * value + 301`, and turn is three.
All 39 observations verify untimed on native and Node-after-tsc; source guards,
rehash mutation controls and registration pass. SQLite stores
`runtime/results/2026-10-08T21-29-08-920Z-verify.json` and `tsrust-b5-paired-map-*.log`
with byte-exact read-back. Loose evidence is removed after verification.

B5 remains active. Next extend discovery across nested exception regions and
broader loop shapes, then precise entry/element release and remaining callable
compositions. Rust-style record Set equality/hash remains planned. No family
regression, timing or user consumer refresh has occurred. The scratch CLI includes
the paired-owner repair; user executable consumers remain at gate181.

Continuation prompt: Continue B5 from the multiple empty Map loop owners checkpoint.
Read AGENTS.md, development-workflow.md and the scoped aggregate plan. Root and
record-field paired-owner disposal now pass with existing cases preserved. Extend
nested-region/loop discovery without treating a refill as empty, then precise
entry/element release. Keep focused editor and matched benchmark support ready,
SQLite evidence durable, whole-project formatting automatic and explicit Git
staging. Preserve unrelated fixture semantics. The full goal remains active.

## Nested paired-owner cleanup checkpoint

Catch-region storage discovery now walks nested try, catch and finally bodies,
matching the existing no-catch discovery behavior. A new paired record Map loop
witness failed native E0502 before this change and passes at O0/O3 afterward.
Exact disposal covers normal fallthrough, handler continue/break, nested rethrow
into an outer catch and an outer cleanup break overriding pending continue.

The override exposed a separate conservative loop-proof gap. Post-clear storage
independence now walks blocks and conditionals iteratively, checking every branch
and its condition for references to the selected owner. It continues to reject
refill/transfer from this empty-epoch proof; no source-valid program is rejected
by that proof, since unmatched shapes retain ordinary lowering. No parser or
analyzer recursion/dispatch changed.

The affected batch passes 90 composition, 42 scoped-record and four fallible-record
tests. The added rethrow variant passes separately. Focused editor diagnostics
pass. A 40th matched observation exercises nested paired-owner cleanup; its oracle
is `2 * value + 910`, including six inner-cleanup turn observations in addition
to the paired-owner arithmetic. All 40 observations verify untimed on native and
Node-after-tsc. Source guards, rehash mutation controls, registration and project
formatting pass. SQLite preserves `tsrust-b5-nested-pair-*.log` and
`runtime/results/2026-10-08T21-34-27-759Z-verify.json`, read back byte-exact before
loose copies are removed.

B5 remains active. Broader loop forms, precise entry/element release, remaining
callable compositions and Rust-style record Set equality/hash remain. Family
regressions, timing and user consumer refresh remain pending. Scratch CLI includes
this repair; five user executable consumers remain at gate181.

Continuation prompt: Continue B5 from nested paired-owner cleanup. Read AGENTS.md,
development-workflow.md and the scoped aggregate plan. Nested catch/finally Map
discovery and post-clear conditional independence now pass with exact disposal.
Proceed with broader loop forms and precise Map-entry/array-element capture
release. Preserve retained-owner semantics, source ownership checks, focused
editor behavior, matched benchmark support and SQLite evidence. Format the whole
project before explicit-path source checkpoints. The full goal remains active.

## Retained refill and unrelated scalar Map checkpoint

Conditional refill after cleanup retains its callback until a later explicit
clear. A new exact O0/O3 witness verifies the retained callback result, size,
disposal and capture release. Moving capture mutation before that clear correctly
reports `capture-conflict`. The optional `get` view uses an explicit inner scope;
its original unscoped form correctly reports `view-conflict` under current rules.

An unrelated scalar Map adjacent to two borrowing-record Maps exposed E0502:
loop discovery required even that scalar Map to participate in the scoped owner
clear proof. Candidates now include only Maps with scoped callback payloads,
while discovery may continue past empty scalar Map declarations. Root and record
leaf selection use the same scoped-payload predicate. Paired-owner disposal with
an adjacent scalar Map now passes; no scalar Map is renewed or discarded.

One newly retained red witness remains:
`empty_scoped_record_map_has_a_native_layout_without_insertions`. A never-filled
`Map<string, Holder>` whose Holder field is `ReadOnly<Function<int>>` is rejected
by declaration-only scoped container validation before emission. The earlier
spare borrowing-record Map variation exposed this same limitation. Do not remove
the witness or equate an empty container with an unsupported payload operation.

Investigate empty concrete record environments next. Existing `Analyzer::absent_callback`
creates a distinct callback contract, and `TsrAbsentCallable` supplies an
uninhabited native environment. Reuse that mechanism per empty record instance
rather than changing a shared declaration callback to absent. Check native trait
emission as well: `TsrAbsentCallable` is currently emitted only when a nullable
union mentions callbacks. Preserve normal later insertion/widening and callback
parameters elsewhere. Cover empty size, clear, lookup, iteration, movement and
later insertion before acceptance. Do not simply remove validation and allow
undefined native generic parameters.

The affected batch passes 91 composition, 42 scoped-record and four fallible-record
tests, explicitly excluding the one red empty-layout witness. Focused editor and
registration checks pass. The paired workload now includes the adjacent scalar
Map and observes its unchanged empty size. All 40 observations verify untimed on
native and Node-after-tsc with unchanged independent expectations. SQLite stores
`tsrust-b5-retained-refill-*.log`, `tsrust-b5-spare-*.log`,
`tsrust-b5-empty-scoped-map-red.log` and
`runtime/results/2026-10-08T21-42-25-884Z-verify.json`, read back byte-exact before
loose copies are removed. Formatting and explicit staging preserve owner fixtures.

B5 remains incomplete. Empty scoped record container layout is the next concrete
repair, followed by broader loop forms, precise entry/element release, remaining
callable compositions and Rust-style record Set equality/hash. Family regressions,
timing and user consumer refresh remain pending.

Continuation prompt: Continue B5 from retained refill and the adjacent scalar Map
repair. Read AGENTS.md, development-workflow.md and this handoff. Keep the newly
retained empty scoped record Map witness and implement its concrete absent
callback environment using existing mechanisms where suitable. Preserve populated
Map inference, ownership and callback parameter behavior. Then continue broader
loops and precise release. Keep focused editor/native/matched checks and SQLite
evidence current, format the whole project and stage explicit paths. The full
goal remains active.


## Empty scoped record Map layout checkpoint

Empty local Maps whose record payloads contain borrowing callbacks now receive
concrete absent environments after populated storage inference has finished.
The implementation lives in empty_scoped_maps.rs. It specializes local storage
and iteration carriers without changing shared declaration contracts or parent
storage before later insertions. Native Map entry aliases carry their payload
generic, and absent callback trait implementations are emitted whenever used.
Impossible callback calls retain an explicit result contract.

Matched verification exposed one further defect: absent contracts contributed
unknown callback failures to callers with catches. A focused red native witness
reproduced it. Distinct absent contracts now ground error inference as pure;
populated callback declarations and ungrounded callback cycles remain unchanged.

Acceptance passes 96 composition, 42 scoped-record and four fallible-record
checks (142 total), 42 error-effect unit checks, all eight depth canaries and the
focused editor contract. Source fixtures execute after format round trips.
Project formatting changes zero files. Registration and all 41 matched native
and Node-after-tsc observations pass untimed. Original failed and successful logs
are stored byte-exact in SQLite under tsrust-b5-empty-scoped-map-*.log and
tsrust-b5-empty-map-effects-*.log. The verified result is
runtime/results/2026-10-08T22-05-55-611Z-verify.json. Twenty-nine closed records were
imported, read back and their loose copies removed. No timing or consumer refresh
is claimed. Owner fixture changes remain uncommitted.

The formerly retained empty-layout red witness is now green. B5 remains open:
broader loop forms, precise entry/element release, remaining callable compositions
and Rust-style record Set equality/hash still need implementation. Nested Map or
array callback fields inside empty record payloads are not covered by this slice.

Continuation prompt: Continue B5 from this empty scoped record Map checkpoint.
Read AGENTS.md, development-workflow.md and this handoff. Preserve the accepted
absent environment and failure contracts, populated insertion inference and owner
fixture semantics. Add the next focused witness for broader loop forms or precise
entry/element release, implement in isolated helpers, then native/editor/docs and
matched support. Format the whole project, preserve SQLite evidence and stage
explicit paths. Family regressions, timing and user consumer refresh are pending.


## Counted and post-test loop storage checkpoint

Proven-empty scoped Map storage epochs now support counted for and do-while loops
as well as while loops. Their body scopes release callback environments before
an update or post-test executes. Continue still executes the tail, while break
skips it. Counted loops preserve initializer scope and zero-iteration behavior.
The existing conservative proof requires finally to clear every selected owner
and rejects tail expressions that retain its payload. Independent intervening
local declarations do not prevent discovery of the empty owner. Lexical block
emission keeps visible declaration context for the synthetic counted-loop block.

A focused native witness initially failed with E0502 after cleanup changed a
captured array. O0 and O3 now verify exact disposal and output for a counted loop,
a counted initializer, zero iterations and a post-test loop. The affected batch
passes 97 composition, 42 scoped-record and four fallible-record tests (143 total).
All eight depth canaries and focused editor checks pass. Matched sources now add
two loop observations: all 43 independent observations pass native and
Node-after-tsc verification untimed. Registration includes mutation controls for
both observations and their capture updates. Project formatting preserves tokens
and fingerprints. Failed source-contract evidence remains stored separately from
the corrected pass.

SQLite contains tsrust-b5-counted-map-loops-*.log and
runtime/results/2026-10-08T22-12-20-160Z-verify.json, imported and read back byte-exact
before loose copies were removed. No timing or consumer refresh is claimed.
B5 remains incomplete. For-of storage renewal, precise individual release,
remaining callable compositions and Rust-style record Set equality/hash remain.

Continuation prompt: Continue B5 from counted and post-test scoped Map renewal.
Read AGENTS.md, development-workflow.md and this handoff. Preserve exact disposal,
finally control semantics and owner fixtures. Add a focused for-of renewal or
individual-release witness, then isolated implementation, editor, guide and
matched benchmark support. Keep SQLite evidence and project formatting current;
stage explicit paths. Family acceptance and executable refresh remain pending.


## For-of scoped Map storage checkpoint

Proven-empty scoped Map epochs now renew inside for-of bodies. Shared iteration
emission lives in iteration_emit.rs and keeps normal iterable evaluation,
conditional temporaries, entry adapters, copy/borrow bindings and string handling.
Both ordinary and renewed loops use this path. The renewal proof traverses
constructor operands and record fields rather than treating independent collection
initializers as unknown references. Actual owner references remain disqualifying.

The focused native witness initially failed with E0502 after finally cleared the
Map and changed its captured array. O0 and O3 now verify exact disposal, cleanup,
continue and break for array, Set, Map-entry, Unicode-string and literal-string
iteration, plus empty iteration. A resumed affected batch passes 98 composition
and three string-iteration tests. The preserved original batch already passed
42 scoped-record, four fallible-record and ten for-of tests; its one failing
composition witness is repaired in the resumed batch. Total affected checks:
157 passed. All eight depth canaries, focused editor and benchmark registration
checks pass. Failed originals remain separate evidence.

All 44 matched observations now verify native and Node-after-tsc untimed.
SQLite stores tsrust-b5-for-of-map-*.log and
runtime/results/2026-10-08T22-17-53-107Z-verify.json. Closed originals were imported,
read back byte-exact and removed as loose copies. B5 remains incomplete: precise individual
entry/element release, remaining callable compositions and Rust-style record Set
equality/hash still need implementation. No timing or consumer refresh is claimed.


Continuation prompt: Continue B5 from for-of scoped Map renewal. Read AGENTS.md,
development-workflow.md and this handoff. The while, counted, post-test and
for-of clear-in-finally paths have focused native/editor/matched acceptance.
Next implement precise individual entry/element capture release with independent
failing witnesses and exact disposal checks. Preserve remaining entries, removed
values and ownership semantics. Keep matched support and SQLite evidence current,
format the whole project and stage explicit paths. B5 family acceptance, timing
and user executable refresh remain pending.


## Precise entry release: red witness and implementation plan

The next retained red witness is deleting_one_map_entry_releases_only_its_capture.
Deleting the first of two differently capturing callback entries still reports
capture-conflict on mutation of the first array. Its negative partner protects
the second entry and its O0/O3 observer specifies separate deletion and clear
disposal. The independent Node observable reference passes. No compiler source
has changed in this checkpoint.

Read docs/map-entry-capture-release-plan.md before implementation. Both keyed
capture ownership and point-sensitive native environment narrowing are required;
a source-only loan deletion is insufficient. The existing carrier planner joins
all insertions for the entire function, and record carrier conversion currently
only widens borrowed callable families. Preserve unknown-key aliases, reaching
branch states, moves, lookup loans, order, failures and exact disposal. Avoid
unconditional survivor scans on ordinary deletion. B5 remains unfinished.

Continuation prompt: Continue precise Map entry release from the new red witness
and docs/map-entry-capture-release-plan.md. Read AGENTS.md and development workflow.
Implement keyed ownership plus checked native storage epochs together, then
negative aliases, O0/O3 disposal, editor, guide and matched benchmark support.
The whole composition suite now includes one intentionally red entry-release
witness; prior 157 affected checks passed before it was added. Preserve owner
fixtures and use explicit Git paths. Keep the full goal active.


## Keyed source ownership work in progress

Uncommitted map_entry_capture_owners.rs now assigns decoded scalar literal keys
separate internal owner paths. Map.set stages its RHS before replacing a known
slot, unknown keys retain conservative container ownership, and Map.delete
removes only the known slot. Existing owner transfer and reaching-state joins
carry these paths. Parameter and captured storage cannot discard external loans.
Recognized number literals preserve existing Map SameValueZero normalization;
unknown expressions and union keys retain the conservative path.

The original red witness now passes source checking and fails native compilation
with E0502. This is expected evidence of the second required layer, not feature
acceptance. Its remaining-entry negative assertion now runs before native
execution. Six additional source cases pass and protect shared captures, unknown
insertion aliases, dynamic deletion, missing deletion, conditional deletion and
replacement retaining the same capture.

Do not commit this as accepted implementation. Native point-sensitive payload
types and checked survivor conversion still need implementation, followed by
full focused editor/matched acceptance. The current source module contains no
payload-type state yet. Add that coordinated metadata next rather than deriving
payload types solely from capture IDs: distinct records may share captures, and
unknown slots or reaching branches must retain their own actual types. Existing
scoped_map_carriers still widens all insertions into a whole-function type.


The source-only affected batch passes 99 composition, 42 scoped-record and four
fallible-record tests (145 total), explicitly skipping the retained native
entry-release red witness. All eight depth canaries pass on the ordinary worker
stack. SQLite stores tsrust-b5-map-entry-release-keyed.log (native E0502),
tsrust-b5-map-entry-release-guards.log, tsrust-b5-map-entry-release-depth.log and
tsrust-b5-map-entry-release-source-batch.log; all four logs were read back
byte-exact before removal. Source, tests and these progress notes remain
uncommitted. No benchmark/editor acceptance or new consumer refresh is claimed.

Continuation prompt: Continue the uncommitted keyed Map ownership work from
map-entry-capture-release-plan.md. HEAD remains 1c48673. The source half works,
but deleting_one_map_entry_releases_only_its_capture still fails generated Rust
with E0502. Complete payload-type state and checked native storage narrowing;
do not merely weaken the source guard or hide the red witness. Keep unknown
aliases and surviving-entry loans. Then run native disposal, editor, matched
support, formatting and SQLite acceptance before committing the combined fix.


## Per-entry payload state work in progress

Uncommitted map_entry_payloads.rs now tracks payload-type alternatives separately
from capture loan IDs. Owners carry this state through consuming moves, borrowed
copies, replacement, discard, scope exits and reaching-state joins. Known slots
receive their actual record type; unknown insertions accumulate a conservative
root bucket. Queries select only this Map's immediate slots, excluding payloads
inside nested containers. A state test proves differently typed records sharing
one capture loan survive uncertain deletion and movement independently.

Known deletion records its input Map type and remaining payload alternatives in
storage_exit_availability::Exits. The metadata table is allocated lazily to avoid
adding a full HashMap to every ordinary analyzer context. Repeated analysis unions
survivors. A changed input type or owner path marks the proof inconsistent so it
cannot authorize native narrowing. A focused replay test passes. The retrieval
method is currently test-only until the native planner consumes these records.

Native phase conversion is still unwired. The original end-to-end deletion witness
must remain red and this source is not accepted or committed. Next expose the
recorded snapshots to scoped_map_carriers, preserve statement/expression order,
plan a native payload type for survivors, and use a checked moving conversion.
Current record_carrier_conversion only widens borrowed callable families; add
excluded-variant handling only on the proven narrowing path. Keep later lookups,
iteration, storage moves, error contracts and replacement/refill epochs consistent.
Do not rerun unrelated full suites or claim editor/benchmark acceptance yet.


The payload-state batch passes ten ownership-state unit tests, the replay test,
145 affected compiler tests with the retained native red witness explicitly
excluded, and all eight depth canaries. Original red and green state logs,
replay evidence, source batch and depth evidence are stored byte-exact in SQLite
under tsrust-b5-map-entry-payload-*.log. No source checkpoint or feature acceptance
is claimed. HEAD remains 1c48673; the combined keyed ownership/payload metadata
changes and notes remain uncommitted. Complete native conversion next.

## Precise entry-release source checkpoint

The current source checkpoint adds checked scalar-key Map entry ownership and
payload alternatives, straight-line native survivor conversion, empty storage,
replacement/refill, direct Map moves, record-field and nested record storage,
enclosing record moves, iteration and typed callback failure transport. Read
docs/map-entry-capture-release-plan.md for the current requirements and chronological
evidence. Parser closure IDs are no longer used to index checked capture origins.
Nested record moves use the isolated map_storage_edges module.

Focused evidence: 107 composition, 42 scoped callable-record and four fallible
record-callback tests pass; all eight normal-stack depth canaries and two focused
editor witnesses pass. The matched workload has 47 observations and passes native
and Node-after-tsc verification without timing. SQLite retains the exact record
runtime/results/2026-10-08T23-38-22-518Z-verify.json and failed/repaired evidence.
This does not accept B5. Conversion scans are still emitted more broadly than the
plan's efficiency requirement; fix that with focused witnesses before claiming
ordinary updates remain constant time. Conditional/loop transport, same-expression
release, broader composition and array-element release remain open. Keep all
owner fixture edits uncommitted. Five user-facing executable consumers still have
not been refreshed from this checkpoint.

## Map conversion demand checkpoint

Demand-driven entry narrowing now avoids rebuilding survivors when the released
capture has no later source use or opaque named call. The isolated
map_entry_demand module computes conservative demand once per checked body and
passes it through nested planning. Successful release snapshots retain released
loan roots and union them on replay. Unchanged incoming payloads retain the
existing native epoch representation. Global Map canonicalization is unchanged.

The initial composition run passed 107 of 108 and caught a nested refill carrier
regression from an overly broad canonicalization change. The repair passes that
witness and all 63 Map-filtered composition tests. The new no-scan witness checks
native O0/O3 disposal and iteration. One replay unit, eight depth canaries and
two editor entry-release tests pass. Matched native and Node-after-tsc verification
passes untimed; SQLite holds runtime/results/2026-10-08T23-49-40-057Z-verify.json
and exact failed/repaired logs. Formatting changes no files or hashes.

This remains a source checkpoint, not B5 acceptance. Next: extend conversion demand
to distinguish read access from conflicting mutation/consumption, reduce opaque
call conservatism with checked capture summaries, then complete conditional/loop
entry transport, same-expression release and array-element release. Preserve the
owner fixtures. Five user-facing consumers still need refreshing before checks
that execute those binaries. No timing or final family regression is claimed.

## Conditional entry-release checkpoint

Conditional normal-completion joins now narrow Map environments from joined
reaching payloads. Both-arm deletion/replacement, mixed clear/delete, early return,
branch-local insertion/deletion, independent Maps sharing a capture and multiple
Map fields with preserved siblings have native O0/O3 witnesses. The isolated
map_entry_branch_release module records lazy per-join survivor collections;
map_post_entry_epochs handles ordinary and custom conditional emission. Existing
ownership joins remain authoritative. One-arm retention or reinsertion continues
to reject conflicting capture mutation.

The final composition batch passes 113 tests, eight depth canaries pass and three
focused entry-release editor tests pass. Matched support has fifty observations
with independent arithmetic oracles and source-contract mutation guards. Whole
project formatting changes only that Tsrust workload and its proven token-preserving
fingerprint. Final matched native and Node-after-tsc verification passes all five configured
correctness cases, untimed. SQLite preserves runtime/results/2026-10-09T00-02-52-098Z-verify.json
and the earlier verification, plus exact original failure and repaired logs.
Readback is byte-exact and verified loose copies are removed. Do not claim timing
or B5 acceptance.

Next requirements: loop join transport, release inside a branch before its normal
completion join, same-expression release, array-element release and remaining
composition cases. Five user-facing consumers remain at gate181 until an explicit
refresh; the scratch debug compiler includes this source. Preserve the fifteen
unrelated fixture edits. Do not restart completed checks after a repaired witness.

## Loop completion source checkpoint

The synthetic-block transport defect is repaired. Eligible Map paths and changed
storage types now pass through unlabeled lexical blocks; native block returns
transport the renewed Map representation. Loop completion proofs use the actual
joined exit payloads. Zero iterations and early break/continue retain potentially
live captures. Native witnesses cover post-test and unconditional-break loops,
zero/repeated while and counted loops, and array/string for-of loops at O0/O3,
including callback disposal order. Conversion inside iterations remains pending.

The initial normal-stack depth run passed closures and overflowed in for-of.
Inspection of compiled frames showed loop_body retained 6,592 bytes. Moving its
post-recursion join into loop_body_completion reduced that frame to 3,456 bytes.
The failed for-of canary and the six unrun canaries then passed on the ordinary
worker stack. No stack-size increase was used.

A strengthened no-scan witness exposed a separate demand boundary defect: reads
inside a loop were mistaken for reads after completion because the statement span
covered its keyword. The existing expression walk now computes the full statement
end. The witness passes O0/O3 and verifies no survivor conversion is emitted.

The earlier coherent composition batch passed 116 tests. After the final demand
repair, all 67 Map-filtered tests pass, and the separate new no-scan witness passes.
Four focused editor checks pass. Matched support has 53 observations with paired
TypeScript, independent arithmetic oracles, source contracts and rehashed mutation
guards. Final native and Node-after-tsc verification passes all five correctness
cases, untimed. Whole-project formatting changes no files or fingerprints; the
current registration check passes. Final verification record:
runtime/results/2026-10-09T00-46-58-730Z-verify.json.

This is a source checkpoint, not B5 acceptance. Loop-carried refill/widening,
in-iteration release, array-element release, same-expression release and broader
composition remain open. Full family regression and timing are deferred. The
five user-facing consumers remain at gate181; scratch compiler builds are current.

## Loop refill follow-up in progress

The loop completion checkpoint is committed as 2f0dc05. A new native witness
then exposed a compiler panic when a narrowed Map was refilled in a loop by a
new callback origin. Loop storage now widens before entry to include both existing
survivors and checked incoming payloads. Several writes at the same boundary
preserve the original conversion input while accumulating the output family.

The counted-loop variant exposed duplicate emission because the synthetic Block
and its Loop share a source span. Pre-conversion now emits at the inner statement
only, and block return transport includes widened storage. The native witness
passes zero/repeated while and counted loops, including two incoming origins.
These follow-up edits are uncommitted. All 68 affected Map checks pass; editor,
additional loop forms, disposal checks, matched support and final formatting are
still owed before acceptance. Preserve the original failed and repaired logs.

Owner direction for record Set keys is Rust-style value equality and hashing with
stable owned keys, rather than JavaScript identity. Eligible fields, float policy,
mutation restrictions and ownership must receive focused witnesses and explicit
documentation before implementation acceptance. This remains independent open B5
work; no record Set support is claimed by these Map changes.

## Loop refill source checkpoint evidence

The follow-up now covers zero/repeated while, counted for, array for-of and
post-test loops. The multiple-origin variant verifies both inserted callbacks and
preserved key order. Instrumented O0/O3 native execution verifies replacement
and final clear disposal. All 68 Map-filtered composition checks pass. Two focused
editor checks pass, including rejection of mutation of a newly captured value
before clear. No parser or recursive checker dispatch changed in this follow-up.

Matched support now has 54 observations. The added counted refill observer retains
the surviving callback and two incoming origins after known-key narrowing. Paired
TypeScript, independent arithmetic oracle, full observation contract, rehashed
mutation guards and exact registration expectations are current. Whole-project
formatting changes one workload and its token-preserving fingerprint; registration
passes afterward. Native and Node-after-tsc verification passes all five cases,
untimed. SQLite preserves runtime/results/2026-10-09T00-55-57-942Z-verify.json and
original failed/repaired evidence, read back byte for byte before loose removal.

This remains focused B5 progress. Release inside an iteration or branch before
completion, same-expression release, precise array-element release, record Set
keys and broader error/alias/composition checks remain open. Full family regression
and timing remain deferred. User-facing consumers are still at gate181.

## Release inside branch source checkpoint

Two red O0/O3 disposal witnesses demonstrated native borrow errors when a branch
deleted an entry and then changed its released capture before completion. The
isolated map_entry_branch_transport planner now plans each arm with independent
storage types and the enclosing capture demand. Nonabrupt arms record restoration
to the incoming common storage family; abrupt arms keep their exit disposal.
Native tuple transport emits these restoration conversions before normal joins.
The lazy restoration table lives in storage_exit_availability and follows existing
checked-storage cloning. Existing ownership joins remain authoritative.

Witnesses cover returning arms, falling-through arms, clearing before completion
and retaining the surviving entry across the join. All 70 Map-filtered composition
tests pass. Eight normal-stack depth canaries pass. One editor witness accepts
released-capture mutation and rejects mutation of a still-captured sibling.

Matched support has 55 observations. Paired sources, independent oracle, exact
registration values, whole-output token contracts and rehashed mutation guards
cover deletion and capture mutation inside a branch. Whole-project formatting
changes one workload and one token-preserving fingerprint. Registration passes;
native and Node-after-tsc verification passes all five cases, untimed. SQLite
retains runtime/results/2026-10-09T01-01-45-311Z-verify.json and failed/repaired logs,
read back byte for byte before loose removal.

B5 is incomplete. Next verify branch refill after prior narrowing, including new
callback origins and aliases; do not assume restoring the incoming family can
represent newly introduced origins. Also complete in-iteration and same-expression
release, precise array-element release, record Set key rules and broader error
composition. Full family regressions and timing remain deferred. User-facing
consumers remain at gate181; scratch compilers contain current source.
