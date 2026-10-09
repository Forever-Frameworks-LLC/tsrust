# B5 error restoration checkpoint

## Discarded array removal checkpoint, 2026-10-09

Standalone pop/shift now release only the removed payload's captures for known
dense local or record-field arrays, including moved arrays and repeated removal
to empty. Source shape state follows moves and rejects uncertain positional
inference. Native survivor conversion and empty environments preserve exact
payload disposal. The affected batch passed 22 tests; two expectation repairs pass
separately. Thirteen owner-module tests, the focused editor check and all eight
depth canaries pass. Pop/shift hover examples are verified. All five matched cases
pass untimed with 115 observations; indexed-capture-release-plan.md records the
SQLite evidence and conversion costs.

B5 remains incomplete. Next implement precise provenance and release for retained
removal results, then same-expression release points and aggregate/control-flow
compositions. Structural record keys, family regressions, performance review and
installed-consumer refresh remain pending. Earlier sections retain prior evidence.


## Indexed replacement checkpoint, 2026-10-09

Known constant-index replacement in straight-line dense local record arrays and
nested record-field arrays now
releases the replaced callback capture through source ownership and a moving
native carrier epoch. Twenty-two affected array tests, exact disposal at optimization
levels 0 and 3, repeated replacement, later element borrowing, array/record moves and two focused editor
checks and all eight depth canaries pass. The matched composition workload has
112 observations and passes all five cases untimed on native and Node-after-tsc.
Evidence and conversion costs are documented in indexed-capture-release-plan.md.

B5 remains incomplete. Next implement precise removal and retained result ownership, indexed arrays
inside aggregates, and reaching slot states across control flow. Nullable and
same-expression compositions, structural record keys, family regressions,
performance review and installed-consumer refresh remain pending.


<!-- current-b5-state -->
## Current state, 2026-10-09

The latest checkpoint implements typed controls created inside finally, following
8c8fb60. Direct and record-field Map completion transport now has focused coverage
for normal completion, errors, returns, labelled break/continue, nested cleanup
and cleanup overrides. The affected batch passes 101 tests, 27 layout proofs
pass, and six related editor checks pass. This is focused acceptance, not B5
family acceptance.

Next reconcile and implement the remaining aggregate and same-expression capture
release requirements in scoped-callable-aggregate-plan.md, starting with precise
array-element storage. Structural owned record keys follow the owner's Rust-style
equality/hash decision. Full compiler/LSP regressions, matched measurements,
performance review and installed-consumer refresh remain required at the completed
family milestone. The older dated sections below retain historical evidence and
superseded next-step notes; use this section for the current direction.
<!-- /current-b5-state -->


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
record value keys. B5 remains unfinished; five user consumers remain at gate 181.

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
changed no files. Five user consumers still represent gate 181.

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
pending. Five user consumers still represent gate 181.

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
timing remain pending. Five user executable consumers still represent gate 181.

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
timing. Five user executable consumers remain at gate 181.

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
timing are not run. Five user executable consumers remain at gate 181.

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
Five user executable consumers remain at gate 181.

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
timing remain pending. Five user executable consumers remain at gate 181.

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
the paired-owner repair; user executable consumers remain at gate 181.

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
this repair; five user executable consumers remain at gate 181.

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
and timing remain deferred. User-facing consumers are still at gate 181.

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

## Branch refill after narrowing checkpoint

A new witness compiled but panicked in native execution: restoration from a
branch-local callback family to the previously narrowed incoming family excluded
the newly inserted origin. Conditional pre-entry widening now includes checked
incoming payloads, just as loop widening does. Arm restoration therefore targets
a family that represents both the untaken arm and newly inserted callbacks.

Native witnesses cover both branch outcomes, a moved Map, multiple incoming
origins and exact O0/O3 disposal. All 71 Map-filtered composition tests pass. One
editor check accepts mutation after clear and rejects mutation while the new
entry may retain its capture. No recursive checker dispatch changed.

Matched support now has 56 observations. Paired sources, independent oracle,
whole-output token contracts, exact registration values and rehashed mutation
guards cover two incoming origins after prior narrowing. Whole-project formatting
changes one workload and one token-preserving fingerprint. Registration passes.
Native and Node-after-tsc verification passes all five cases, untimed. SQLite
preserves runtime/results/2026-10-09T01-04-58-652Z-verify.json and original failure
and repair logs; byte-exact readback precedes loose removal.

B5 remains open. Next requirements include in-iteration release, same-expression
release, precise array-element release, record Set equality/hash key constraints
and broader alias/error composition. Family regression and timing remain deferred.
User-facing consumers are still at gate181; scratch compiler builds are current.

## In-iteration entry release: active red witness

HEAD f04c9f9 remains the last accepted source checkpoint. The new
map_entry_release_inside_iteration_preserves_refill_and_survivor witness checks
an outer Map with a persistent second entry. Each iteration inserts and deletes
a first entry, changes its released capture, and continues. Zero iterations must
preserve the second entry; repeated iterations must observe changed first values.
The source checker accepts this program, but native Rust reports E0502 because
its persistent Map representation still carries the first callback environment.
A negative companion asserts that mutation of the surviving second capture is
still rejected. The witness is uncommitted; no implementation is claimed.

The existing completion conversion occurs after the loop and cannot end this
iteration's native borrow before capture mutation. Simply planning the body with
inherited paths is insufficient: widening back to a family containing the removed
environment would carry its reference lifetime through the backedge. Likewise,
map_empty_loop_epoch cannot be reused by discarding storage: the second entry is
nonempty and must survive, including its exact disposal and key order.

Implement an isolated iteration transport plan with a stable carried family
containing the actual entry and all reaching backedge payloads. Collect entry,
normal body, continue and break ownership payloads through the existing joins;
never derive reachability from a deletion snapshot alone. Widen inside an iteration
before incoming writes, narrow before conflicting capture access, and move actual
surviving storage into the carried representation at backedges. Preserve break,
continue, post-test/update, zero-iteration, return, throw and finally behavior.
For-of must retain iterable ownership and its existing control targets. Independent
Map fields require independent transport without touching record siblings. No
implicit clone or survivor drop is allowed.

Acceptance must include while, counted, post-test and for-of variants, negative
retained/early-exit paths, O0/O3 disposal, affected Map composition, editor behavior,
matched TypeScript/oracle/contracts and whole-project formatting. Run all eight
normal-stack canaries after recursive checking/dispatch changes. Family regression
and timing remain deferred until B5 is complete. Keep current consumers unchanged
until rebuilding from a source checkpoint. Original native failure evidence is
retained in SQLite under tsrust-b5-map-iteration-release-red.log.

### Iteration entry versus backedge proof

Inspection of finish_loop_body confirms that pre-test completion payloads join
entry, normal body, continue and break owners. In contrast, tail_loop explicitly
omits a zero-body exit for do-while. Its final completion proof can therefore omit
an environment present on the first body entry. Do not use final completion
payloads alone as the uniform first-entry representation. Record actual entry
and backedge payloads separately before the ownership states are consumed.

A second native witness covers post-test zero-limit and repeated execution. Its
zero-limit case must run once; this prevents lowering through a pre-test shortcut.
A stable second callback survives all iterations in both witnesses. The initial
while witness and this post-test witness remain red. No compiler source changed
in this investigation.

When entry and backedge families differ, consider a distinct first-iteration path
followed by a loop carrying the proven backedge family. This can end initial
borrow environments before mutation while retaining exact condition evaluation,
zero-body behavior and survivors. Any such lowering must preserve continue/update,
break/finally, condition failures and iterable advancement. Do not hide these
requirements behind a larger common carrier that retains the eliminated borrow.

The strengthened post-test witness seeds a first-entry callback before the loop,
then replaces and deletes it inside the first body. Source checking succeeds and
native borrow checking fails. Both original and strengthened failure logs are
in SQLite as tsrust-b5-map-iteration-post-test-red.log and
tsrust-b5-map-iteration-post-test-entry-red.log. No live checks remain.

## Loop edge metadata checkpoint

The checker now records Map payloads separately at Entry, Back and Break edges.
Payload snapshots group decoded entry paths under their owning Map paths while
keeping record fields independent. Ordinary loop completion records normal and
continue owners before joining entry/exit state. Tail loops record first entry,
reachable checked update/condition backedges and break owners before exit joins.
Existing ownership joins are unchanged. Replay unions alternatives conservatively;
empty reachable recorded edges are distinguishable from absent edges.

The focused unit verifies that removing an entry at the backedge does not remove
its first-entry or break payload, and that replayed incoming origins do not leak
into entry facts. Its strengthened version verifies an empty reachable backedge.
All eight normal-stack depth canaries pass after the checker hooks. Five editor
entry-release checks pass. The Map composition batch passes 71 tests and retains
exactly the two expected native failures: in-iteration while release and the
post-test first-entry witness. Whole-project formatting changes no files/hashes.
Original logs are imported into SQLite and read back byte for byte before removal.

This is a prerequisite source checkpoint, not iteration or B5 acceptance. The
new edge APIs are not consumed by native lowering yet; their temporary dead-code
annotations mark that outstanding consumer explicitly. No new matched workload
or timing is claimed for metadata alone. Implement carried storage using these
separate facts next, preserving first-iteration evaluation and surviving entries,
then repair both native witnesses and expand control/error/loop-form coverage.
The five user-facing consumers remain at gate181; scratch debug compiler is current.

## Native iteration transport in progress

The ordinary while witness is now green, including exact O0/O3 disposal. The
isolated map_iteration_plan and map_iteration_emit modules use separate checked
Entry and Back payloads. For equal nonempty families on a normally completing
body, native storage narrows before loop entry. Each body can widen before an
insertion and narrow after deletion, then moves its surviving Map back through a
tuple assignment. This preserves the existing second callback and its key order;
it does not clear, clone or discard surviving entries.

Planning requires a consistent checked release candidate and invalidates local
paths passed to opaque callees before selecting iteration transport. The initial
path supports ordinary while bodies; abrupt controls, nested loops/try regions and
differing first-entry/backedge families are deliberately not claimed yet. The
new metadata is consumed by lowering, while the separate edge-presence query is
still needed by the forthcoming empty/different-entry path.

The current Map-filtered batch passes 72 tests and retains only the expected
post_test_map_iteration_release_keeps_first_entry_state_distinct failure. The
ordinary while mutation witness and its retained-sibling rejection pass. All eight
normal-stack canaries pass. Original green, batch and canary logs are preserved
in SQLite under tsrust-b5-map-iteration-transport-*.log with exact readback.
These compiler edits are uncommitted. No implementation acceptance, benchmark
expansion, timing or user-facing executable refresh is claimed.

Next implement distinct first-entry transport for post-test/populated entry cases,
then targeted continue/break and cleanup transport, counted/for-of forms, independent
record Map fields and failures. Complete editor and matched workload support and
whole-project formatting before accepting the implementation chunk. HEAD fd2aa00
remains the prerequisite source checkpoint; preserve the fifteen fixture edits.

## Normal iteration transport verification

Native transport now covers normally completing while, counted for, do-while
and array/string for-of bodies, including a local record Map field and empty
backedge storage. Do-while emits its first body using the original entry family;
subsequent bodies use the separately proven backedge family. Conditions execute
after each body, and body-local storage leaves scope before counted updates.
Surviving entries retain their key order without implicit cloning. O0/O3 disposal
witnesses cover the while and post-test paths.

The final Map batch passed 72 checks and exposed one additional empty post-test
defect: iteration callback calls retained a concrete closure target after their
payload became absent. Retargeting to the absent callback contract repairs the
focused post-test witness. All eight normal-stack depth canaries passed. One
focused editor check covers accepted while/count bodies and surviving-capture
rejection. Matched support now has 58 observations, including counted and
post-test transport; verification results remain pending until recorded.

These changes remain uncommitted until final formatting and matched verification
finish. B5 is not accepted. Abrupt break/continue, nested control/cleanup, differing
pre-test entry and backedge families, multiple independent Map fields, same-
expression release, precise array element release and structural record Set keys
remain open. The owner selected Rust-style equality and hashing for record Set
keys. No timing or user-facing consumer refresh is claimed.

Final bounded verification: benchmark registration passes; all five independent
result cases pass for native and Node-after-tsc with 58 observations. The untimed
record is runtime/results/2026-10-09T01-51-04-431Z-verify.json in SQLite. Final
project formatting changes zero files and zero fingerprints. Closed logs, including
the original failed batch and repaired focused check, are imported into SQLite
and read back byte for byte before their loose copies are removed.

## Abrupt iteration transport

A focused native witness first failed with E0502 when break/continue followed
known-entry deletion and capture mutation. Each abrupt statement now records its
point-sensitive storage inputs. Selected loop controls convert to the proven
carried survivor family and yield a stop flag plus owned Maps from a labelled
body scope. The enclosing loop restores those Maps before acting on the flag.
Break skips counted updates and condition evaluation; continue runs counted
updates. Do-while retains its first-entry distinction, and for-of retains its
shared iterator evaluation and binding layouts. No surviving entry is cloned.

The focused witness covers while, counted, post-test and array for-of, observable
counted updates, exact O0/O3 disposal and retained-capture errors. Break before
deletion rejects a later mutation because the entry remains live. The affected
Map batch passed 71 checks, the editor witness passes, and all eight normal-stack
canaries pass before the final update guard. Separate body/update representations
preserve existing normal-loop lowering. Abrupt loops whose counted update touches
Map storage require further transport and are not claimed by this chunk.

Matched support appends a 59th observation for abrupt iteration, with independent
arithmetic oracle and rehashed guards for deletion, break, continue and tail
execution. Registration and native/Node-after-tsc verification pass on the initial
implementation. Final guarded-source verification and evidence import remain
pending. B5 remains open: cleanup/nested control transport, different pre-test
entry families, independent Map fields, same-expression/array release and
structural record Set keys still need work. User-facing consumers are unchanged.

Final guarded-source checks pass: abrupt four-form native/disposal witness, normal
iteration regression witness and all eight depth canaries. Final formatting
changes no files/fingerprints. All five independent result cases pass for native
and Node-after-tsc with 59 observations. The final untimed record is
runtime/results/2026-10-09T02-00-12-888Z-verify.json in SQLite. Original red,
intermediate compiler errors, repaired checks and both verification records are
retained independently with byte-exact readback; loose copies are removed.
This is a source checkpoint for abrupt iteration transport, not B5 acceptance.

## Counted Map update transport

The new counted_map_update_keeps_body_payload_until_update_deletes_it witness
first failed in generated Rust with E0502 for the body capture and the temporary
update capture. The body creates a temporary third entry after deleting first;
continue must preserve third until the counted update observes size two and
deletes it. Narrowing to the backedge family at body completion would discard
that still-live entry.

Planning now joins normally completing body storage with each break/continue
input into a separate body-completion family. Update expressions are retargeted
to that family. The update converts to the backedge family only after its effects;
the stopped path skips the update and converts its separately proven survivors.
The emitter shadows the body storage when the native type changes instead of
assigning incompatible families. Normal loops retain their previous lowering.
The blanket prohibition on Map-touching counted updates is removed. Distinct
pre-test entry/backedge and break/backedge payloads still need later transport.

Scalar observations such as console.log(handlers.size) no longer invalidate a
local Map proof. They pass a scalar, not storage. Passing scoped Map storage or
a record containing it to an opaque callee still invalidates the proof.

Focused checks cover observable update order, surviving second entries, third
release, break skipping updates, capture-conflict before deletion, formatting
and exact O0/O3 disposal. The affected Map batch passes 75 tests. One focused
editor witness and all eight normal-stack depth canaries pass. Matched support
appends a 60th observation with independent arithmetic and rehashed source guards.
Final registration, native/Node verification and SQLite import remain pending.
No timing or user-facing consumer refresh is claimed. B5 remains open.

Final counted-update registration and all five independent native/Node result
cases pass with 60 observations. The untimed record is
runtime/results/2026-10-09T02-07-31-272Z-verify.json in SQLite. Formatting
preserves tokens, formats one changed workload and refreshes its fingerprint.
Original failing and repaired checks are retained independently in SQLite with
byte-exact readback; closed loose copies are removed. The implementation chunk
is ready for its source checkpoint; B5 and family measurements remain pending.
Next cover cleanup/nested-control transport and distinct pre-test entry families.

## Exception-phase payload prerequisite

The new map_entry_iteration_control_runs_finally_before_leaving_body native
witness fails before implementation. It deletes a known first entry, mutates its
released capture, then continues or breaks through finally. Cleanup must inspect
and call the surviving second entry before the loop control takes effect. Its
O0/O3 disposal expectations remain asserted; native lowering is not accepted yet.

The isolated map_completion_payloads module records checked payload alternatives
at Body, Handled and Cleaned phase boundaries, separately for normal, return,
throw, and each targeted break/continue. Repeated alternatives are conservatively
joined within their own phase/completion identity. A recorded empty normal state
means reachable empty storage; a missing state means that completion was not
recorded. No phase may substitute its normal payloads for error or control
payloads. Source availability remains governed by storage_exit_availability;
payload metadata alone cannot revive a transferred source binding.

Two focused proof tests cover different control targets, repeated alternatives,
empty-versus-missing states and real checker states. The checker witness has
explicit failures before deletion in both body and handler. Normal/return paths
retain only second; pre-deletion errors retain first and second. Cleanup clearing
records reachable empty normal and return storage. Earlier failed proof assertions
assumed the wrong handler error shape; strengthened explicit-error witnesses
replace those assumptions. Original logs remain separate evidence.

The phase hooks run after checked body/error collection, after catch, and after
finally, without changing native dispatch. Native accessors remain test-only
until lowering consumes this metadata; no unused production API is introduced.
This prerequisite has no new observable method, catalog entry, workload or timing
claim. Existing 60-observation matched support remains current. Final bounded
checks and source checkpoint are pending. B5 and cleanup implementation remain
open. Next plan phase-specific native storage and completion payload layouts,
then route control through cleanup before yielding loop survivor storage.

Final prerequisite evidence: both phase proof tests pass, all eight normal-stack
canaries pass, six existing editor entry-release checks pass, and the affected
Map batch passes 75 tests with exactly the newly retained native cleanup witness
red. The independent transpiled Node runtime reference passes its complete
observable-output expectation. Whole-project formatting changes no files or
fingerprints. Existing matched workload registration passes. Original failed
proof assumptions and the repaired explicit-error tests, plus all closed check
logs, are imported into SQLite and read back byte for byte before loose removal.
No cleanup implementation acceptance, new matched observation, timing or installed
consumer refresh is claimed. The next source chunk must consume phase payload
proofs, preserve per-operation storage types and source availability, and carry
normal/error/control storage through finally before resuming loop control.


### Native storage projections and post-test conditions

Native completion values now consult point-specific Map storage representations.
When live storage differs from the destination family, emission moves entries and
projects callback payloads into that family. An unavailable transferred binding
produces typed empty storage; projection cannot revive its source owner. The
availability regression covers transfer boundaries and storage renewal.

Focused proof tests cover narrowing after deletion, widening before refill and
post-test conditions. The post-test witness initially failed because lexical
lookup retained the last body statement's working family. The loop planner now
records the condition after backedge conversion with the carried family. All four
phase/native-point proofs pass. This is a prerequisite for cleanup transport,
not cleanup acceptance. The retained finally/control native witness remains red;
phase-specific cleanup storage, nested control transport and structural record
Set keys still require implementation. No timing or consumer refresh is claimed.


Final native-point prerequisite checks: all four phase/point proofs pass, the
projection availability proof passes, all eight depth canaries pass, and the
existing post-test native witness passes. Six editor checks and the matched
workload registration pass. The final cleanup/control witness still fails native
compilation; earlier affected batches retain their 75 passes plus that red.
Whole-project formatting changes zero files and zero fingerprints. Closed check
logs, including the post-condition red and repaired proof, are imported into
SQLite and verified byte for byte before deleting loose copies. B5 remains open.


### Materialized exception phase storage families

The isolated map_completion_layouts analyzer now materializes a concrete Map
payload family for each recorded phase and completion identity. It joins only
payloads reaching that identity, preserving normal, return, error and targeted
control boundaries. Places present in another phase but empty in this phase get
an empty callback environment. Missing completion identities get no layout.
Record field paths remain distinct; layout types describe storage, not renewed
source availability. The planner runs outside recursive exception dispatch.

A new red proof initially found no normal layout. Its repaired witness proves
that normal and return storage retain only the surviving callback, pre-deletion
errors retain the wider family, and normal/return cleanup clearing gets a third,
empty family. The normal layout's payload is checked against the exact single
record in the ownership proof. An absent break completion remains absent.

Five phase/native-point proofs, all eight normal-stack depth canaries and six
editor entry-release checks pass. The selected map_ native batch has 72 passes
and the retained finally/control compilation failure; this is not full compiler
acceptance. Phase layouts are checked metadata with test-only accessors pending
native consumers. They do not yet repair that cleanup witness. No new catalog
entry, workload observation, timing or installed consumer refresh is claimed.
Next consume phase layouts in carried completion emission, specialize cleanup
storage for its incoming family, and resume control only after cleanup. Keep
error and cleanup override behavior, ownership and disposal assertions intact.


Whole-project formatting changes zero files and zero fingerprints. Matched
scoped-record-composition registration passes. Ten closed logs, including both
red witnesses and strengthened proof checks, are imported into SQLite, read back
byte for byte, then removed from scratch storage. B5 remains incomplete.


### Deletion through straight-line exception cleanup

Checked local entry epochs now run within try bodies, handlers and cleanup.
Native carried completion discovers these entry conversions as well as clears.
Normal completion moves a narrowed live Map into the existing common transport
family; unavailable source owners remain typed empty. Plain branch tuple emission
keeps its existing survivor family instead of converting to the function-wide
family. The first experimental shared helper caused two branch regressions;
separating completion conversion repaired both without weakening their witnesses.

Borrowed Map iteration under carried storage now uses an inner typed completion
region. It finishes the iterator scope before transporting the owning Map on
error or escaping control. Local loop controls still act on the iterator loop.
This fixes moving a Map while its iterator temporary still borrows it. The new
native witness checks deletion, mutation of the released capture, surviving
callback invocation in finally, error during borrowed iteration, catch visibility,
cleanup order and exact disposal at both optimization levels. The editor witness
accepts mutation of the deleted capture and rejects mutation of the survivor.

Matched verification exposed a tail-expression boundary defect in an existing
replacement scenario. Short statement spans did not include the final native
conversion. The planner now records exact body-exit storage types, and normal
completion uses that type before projecting into the common transport family.
A focused proof checks the exit against the actual tail replacement epoch.
Phase-specific normal/error/cleanup layouts remain available checked metadata;
separate phase-specialized emission is still required. This chunk does not claim
that the phase layout consumer or the broader B5 family is complete.

The original loop-cleanup witness remains red. Planning its inner try exposed a
moved outer Map on repeated iterations: resuming break/continue must first
reassign the carried loop tuple. The exploratory while recursion is removed;
the failing evidence is retained. Next combine loop entry/back/body transport
with resumed cleanup control, without installing direct source control hooks
inside try that could bypass finally. Preserve cleanup overrides and disposal.


Additional handler and cleanup witnesses now pass native execution and exact
callback disposal at both optimization levels. A named pending completion alone
was insufficient: its inactive storage variants kept capture lifetimes alive.
For phases with nonempty entry narrowing, control and storage leave that owner
scope separately before entering the phase. Clear-only renewal retains its
existing independently inferred normal storage lifetime; unconditionally
combining normal/error storage under one explicit tuple type regressed six
accepted clear/restoration checks. Those failed runs are preserved. Selecting
transport by the checked entry conversion restores all accepted cases; the final
selected batch is 73 passing tests and only the retained loop-cleanup red.

Six phase/native-boundary proofs, eight normal-stack depth canaries and seven
editor entry-release checks pass. Matched support adds finallyEntryMutation,
with 61 scalar observations per iteration and an independent 3*value+10 oracle.
Token-based source guards reject removal of deletion, mutation and survivor
invocation. Registration literal expectations are extended; the initially stale
60-value assertion and repaired run remain separate evidence. Native/Node untimed
verification and final formatting are recorded below when complete. No timing,
full-family acceptance or installed consumer refresh is claimed.


Final matched native and Node-after-tsc correctness verification passes for all
61 observations and all retained input cases. Canonical final record:
runtime/results/2026-10-09T03-05-12-987Z-verify.json. Whole-project final
formatting changes zero files and zero fingerprints. Original failed checks,
repaired checks and both successful untimed records are imported into SQLite
and read back byte for byte before their loose copies are removed. B5 remains
open; this is a focused native transport checkpoint, not a family milestone.


### Cleanup controls and carried iteration storage

The retained finally/control witness now passes. Iteration planning accepts
checked exception regions and records cleanup controls separately from direct
source controls. A source break or continue inside try receives no direct
map_body hook. Resuming cleanup yields the enclosing body tuple only when that
loop belongs to the current completion region; an enclosing finally receives
its pending control first. The isolated map_cleanup_control_emit module owns
this dispatch. Its fallback excludes already intercepted loop targets so native
control flow cannot syntactically skip tuple reassignment on impossible arms.
The first native attempt exposed that moved-owner path; its failure is retained.

Cleanup can replace a pre-deletion error with continue or break. Those edges may
retain a wider payload family than normal execution or initial loop entry.
For cleanup loops, the carrier joins entry, back and break payloads. Proven
emptiness is recorded for the carrier separately from the actual entry and
backedge. Initial populated storage is not discarded merely because the
backedge is empty. A focused native/disposal witness exposed an absent-callback
conversion panic before this repair and now observes size one on first entry,
size zero on the next, and correct callback disposal.

Native execution and disposal pass for while, counted, post-test and for-of
loops, pending continue/break, cleanup overriding both controls, a wider break
family, and nested finally ordering. The nested witness proves inner cleanup
precedes outer cleanup before loop control resumes. Seven phase/native proofs
and all eight normal-stack canaries pass. The affected native map_ batch has
74 passes and no retained red. The separate populated-entry/empty-backedge
witness passes. Eight editor checks preserve acceptance of the deleted capture
and rejection of mutation of a surviving capture. These are bounded checks;
whole compiler/LSP regressions and timing are not claimed.

Matched support adds cleanupIterationMutation as observation 62, with an
independent arithmetic oracle for one and three iterations. Source guards are
scoped to individual helpers where identical control snippets occur in both;
an initially weakened presence guard failed its mutation test and is repaired.
Registration passes. Native and Node-after-tsc untimed verification passes for
all retained cases. Final formatting, immutable record identities and SQLite
retention are recorded below after checkpoint verification.

B5 is still incomplete. This acceptance covers the tested cleanup iteration
paths, not all Return/Throw, labelled/nested-loop compositions or phase-specific
normal/error/cleanup emission. Ordinary loops with different entry/back/break
families remain separate pending work. Stable owned record Set keys should use
the owner's chosen structural equality and hashing, with borrowing keys and
mutable stored keys excluded, explicit cloning for retained copies, and a
consistent numeric-field policy before float-containing records are accepted.
That key feature is not implemented in this chunk.


Final untimed correctness record: runtime/results/2026-10-09T03-29-23-041Z-verify.json. Whole-project checkpoint formatting changes zero files and zero fingerprints. All 27 closed logs, including red proofs, native failures and repaired checks, plus the matched record are imported into SQLite and read back byte for byte before loose removal. No installed consumer refresh, whole-family acceptance or timing is claimed.


### Pending callable exits through Map cleanup loops

Return and throw no longer exclude a checked loop from entry storage planning.
The focused red witness checked successfully but generated Rust retained a
borrow of the deleted capture. Conditional callable exits now use the existing
completion transport, preserving finally execution and disposal before return
or error propagation. Cleanup return, throw and continue retain their ordinary
replacement precedence. Source callable exits receive no direct loop hook.

A separate native red exposed loops with no backedge. Cleanup still needs
working storage for deletion and survivor iteration even when every body path
exits the callable. Planning now retains that entry family. Counted and
post-test loops also need this plan when their tail is unreachable; unchecked
updates and post-test conditions must not be analyzed or emitted. The initial
counted witness failed in generated Rust; the repaired witness executes while,
counted, post-test and for-of forms. A post-test witness removes its unreachable
fallback return rather than changing the language's reachability diagnostic.

Native witnesses verify pending integer returns and errors, cleanup return/error
replacement, cleanup continue cancelling a return, no-backedge exits and a
return expression evaluated before cleanup mutation. Exact callback disposal
is checked at both optimization levels. Cleanup mutation must still account for
failure before deletion: an unconditional mutation there is rejected, while an
explicit cleanup deletion releases the capture before mutation. The failed
unsafe-cleanup attempt is retained as evidence, not accepted behavior.

Final bounded checks pass: 76 selected map_ native/ownership tests, eight
phase/native-boundary proofs, eight normal-stack depth canaries and nine editor
entry-release checks. No whole compiler/LSP regression or timing is claimed.
Matched support adds five return/error cleanup observations with independent
arithmetic oracles, bringing the family to 67 observations per iteration. Its
wrapper narrows a caught integer and rethrows runtime errors; an initially
unnarrowed wrapper failed checking, and both runs remain evidence. Token guards,
mutation rejection and registration pass. Native and Node-after-tsc untimed
verification passes for all five retained correctness cases. Canonical record:
runtime/results/2026-10-09T03-47-01-259Z-verify.json.

Whole-project final formatting changes zero files and zero fingerprints. All
28 closed logs and the matched record are imported into SQLite and read back
byte for byte before their loose copies are removed. The five installed
consumers remain at gate 181; scratch compiler binaries are current. B5 is still
open: labelled/nested-loop compositions, ordinary differing loop edge families,
phase-specialized native storage, broader aggregate/within-expression precision
and structural owned record Set/Map keys remain pending. This checkpoint does
not claim those features or feature-family acceptance.


### Labelled and nested Map cleanup loops

The labelled-loop red checked successfully but failed native capture mutation.
Eligibility and exception storage discovery had both excluded labelled controls,
even though their checked destinations are the same loop targets used by
unlabelled controls. The new map_iteration_shape module inspects resolved targets
and completion boundaries with an iterative walk. The shared
map_exception_storage_plan module discovers conversions for body, handler and
cleanup, including planned nested loops. It replaces duplicated discovery code.

The nested witness exposed two further defects. Completion storage had no local
control destinations, so an inner break incorrectly escaped its owning phase and
ultimately reached the callable's impossible-control arm. Phase emission now
records its local destinations. Once that path ran, an absent inner backedge was
mistaken for an empty break: the survivor disappeared before outer cleanup.
MapIterationEpoch now preserves break emptiness independently, and all loop
forms select the appropriate back or stopped conversion after cleanup.

Counted variants exposed physical exit and lexical scope mismatches. A checked
exception transport returns the common native family, although its body may
narrow storage before leaving. The isolated map_exception_storage_types module
records that normal exit representation without cloning the source tree.
Counted-loop initializer blocks now return storage converted by their labelled
loop, rather than dropping the new owner in the block and using the moved outer
owner afterwards. The original native failures and repaired checks are retained.

Native witnesses pass labelled while, counted, post-test and for-of controls;
nested versions of those loop forms; inner-finally before outer-finally order;
catching an inner error; inner cleanup continue replacing a pending outer exit;
and rejection of mutation of a surviving capture. The base labelled and nested
witnesses verify exact callback disposal at both optimization levels. A raw
storage proof distinguishes absent backedge, live break payload, common phase
exit and the narrowed body representation.

Final bounded checks pass: 78 affected map_ tests, nine phase/native-boundary
proofs, eight normal-stack depth canaries and ten editor entry-release checks.
Matched support adds nestedCleanupEntryMutation with and without inner cleanup
override, bringing the family to 69 scalar observations per iteration. Independent
arithmetic oracles, token guards and mutation rejection pass. Native and
Node-after-tsc untimed verification passes all five retained correctness cases.
Canonical record: runtime/results/2026-10-09T04-05-37-523Z-verify.json.

Whole-project final formatting changes zero files and zero fingerprints. All
23 closed logs and the matched record are imported into SQLite and read back
byte for byte before loose removal. Installed consumers remain at gate 181;
scratch compilers are current. No whole-family regression, timing or B5
completion is claimed. Pending work includes arbitrary labelled blocks, direct
nonlocal controls outside completion regions, ordinary differing loop edge
families, independent native phase layouts, broader aggregate/within-expression
precision and structural owned record Set/Map keys.

Next continuation should add a focused direct nonlocal-control witness in nested
loops before changing eligibility. Source control outside try must not skip a
carried tuple reassignment. Labelled block exits need their own owner transport;
do not treat a labelled block as an ordinary lexical block. Keep source ownership
proofs authoritative, run the eight normal-stack canaries after analyzer changes,
and preserve the 15 unrelated fixtures with explicit Git paths.


### Direct nested Map loop controls accepted

The direct ancestor-control witness now passes. Iteration shape validation accepts
resolved loop destinations, while controls to labelled blocks remain excluded.
Loop planning joins reaching edge families for nested loops and for a loop with
a direct ancestor control. The body-family walk collects nested control input
representations. Direct conversion installation descends through nested loops
but handles only controls whose checked destination names the current loop.
This prevents an inner plan from replacing an ancestor destination.

The red investigation initially proposed forwarding through each intervening
loop. Native execution proves a simpler ownership-preserving transport: convert
the innermost live owner directly into the destination loop's carried body tuple.
All intervening loop scopes are exited, so their moved storage is never used
again. The destination loop performs the normal break/backedge conversion.
Exception boundaries retain their existing completion/finally transport; this
change does not bypass cleanup or introduce cloning.

The focused witness covers inner while, counted, post-test and for-of loops,
ancestor break and continue, local inner break, surviving callback invocation,
format round trips, capture-conflict rejection and exact disposal at both native
optimization levels. The affected batch passes 79 Map tests, nine phase/storage
proofs, eight normal-stack depth canaries and 11 editor entry-release checks.

Matched sources add directNestedEntryMutation, an independent arithmetic oracle,
source guards and mutation rejection. A shared helper guard was scoped to its
function so the new helper cannot mask a deleted snippet from the earlier one.
All five native and Node-after-tsc cases agree on 70 scalar observations per row.
Canonical record: runtime/results/2026-10-09T04-15-23-075Z-verify.json. Sixteen
closed logs and that record are imported into SQLite, read back byte for byte
and removed from loose storage. Final project formatting changes zero files and
zero fingerprints. No timing or whole-family acceptance is claimed.

B5 remains open. Next implement owner transport for arbitrary labelled block
exits with a focused failing witness; keep source ownership and disposal proofs
authoritative. Ordinary differing loop edge families, independent native phase
layouts, broader aggregate/within-expression precision and structural owned
record keys remain pending. Installed consumers remain at gate 181; scratch debug
compiler and benchmark-built native compiler are current. Preserve the 15
unrelated fixture edits and stage only explicit paths.


### Labelled block exits: active failing witnesses

After checkpoint 9e3cdb3, two uncommitted witnesses cover a labelled block whose
normal path and break path retain a second callback entry after deleting the
first. `labelled_map_block_direct_exit_preserves_survivors` tests direct break.
`labelled_map_block_exit_preserves_survivors_after_cleanup` tests break through
finally, including cleanup observation followed by continued use outside the
block. Both check and format successfully, but generated Rust rejects mutation
of the deleted capture with E0502. Both independent Node references produce the
expected output. These tests are active red witnesses, not accepted features.

The current local epoch planner explicitly skips labelled blocks. The lexical
block emitter likewise declines storage results for labelled blocks; plain
control dispatch emits a valueless break. Existing labelled-block analysis joins
normal and break capture owners, but does not retain distinct native storage
inputs for a storage-bearing exit. This is separate from the now-fixed loop
transport, whose targets return carried tuples.

Implementation requirements:

1. Preserve normal and break owner proofs for the labelled block after cleaned
   completions join. Discover eligible local Map places without admitting maps
   whose source ownership remains unavailable on any reaching path.
2. Plan the block's working and output families in an isolated module. Recurse
   through its body with local Map epochs, retaining each direct break input.
   Convert each reaching path to one checked block result family.
3. Emit a labelled block expression returning its Map owner tuple. Direct breaks
   return that tuple; cleaned breaks return it only after every exited finally
   runs. Valueless dispatch cannot exit a storage-bearing block expression.
   Preserve destination identity for nested blocks and loops.
4. Verify skipped bodies, conditional retained entries, nested block destinations,
   cleanup overriding the pending break, exact disposal at both optimization
   levels and surviving-capture rejection. Add editor checks and matched workload,
   independent oracle, scoped token guards and registration mutation controls.
5. Run affected tests and all eight depth canaries, format the entire project,
   retain SQLite evidence and commit explicit paths. B5 remains open until its
   remaining family requirements are fulfilled.

Three closed logs are imported into SQLite, read back byte for byte and removed:
tsrust-b5-labelled-block-red.log, tsrust-b5-labelled-block-direct-red.log and
tsrust-b5-labelled-block-node.log. No compiler source or installed consumer is
changed in this investigation. The two new tests remain uncommitted together with
this plan update; the 15 unrelated fixture edits remain untouched.


### Labelled block source join proof implemented; native transport pending

The labelled-block analyzer now records completion survivor metadata after its
normal/break capture-owner and availability joins. It reuses the existing
record_map_completion_release helper; it does not authorize elimination from an
individual deletion path. The new raw proof checks both deletion on every exit
(one retained payload) and an early break before deletion (both payloads remain).
The valid red showed missing block completion metadata. The repaired proof and
all ten phase/native-boundary tests pass. All eight normal-stack depth canaries
pass on the modified analyzer.

The first attempted proof lacked main and failed entry validation. It was repaired
and rerun red with implementation removed before adding the analyzer change.
Those failed attempts remain distinct immutable records, not acceptance evidence.
The two native labelled-block witnesses still fail E0502 after the metadata fix;
there is no claim of usable labelled-block storage transport yet.

Next add a dedicated block storage plan using this joined proof. Carry the checked
working family into the block, collect point-sensitive direct-break inputs, and
return a common result owner on normal and break paths. Wire cleaned block breaks
through completion dispatch without bypassing finally. The existing epoch walk's
early labelled-block skip and valueless block break must be replaced together.
Keep the block destination distinct from loop stop flags. Verify the retained
native reds plus conditional retained entries and disposal before acceptance.

Seven new closed logs are retained in SQLite, read back byte for byte and removed:
proof-red, proof-first, proof-valid-red, proof-fixed, proofs, canaries and
native-still-red under the tsrust-b5-labelled-block prefix. Compiler changes,
raw proof, native witnesses and plan updates remain uncommitted in-progress work.
HEAD remains 9e3cdb3. Preserve the 15 unrelated fixture edits. B5 remains open.


### Labelled block Map transport accepted

Labelled blocks now retain checked normal/break owner joins and plan their storage
in map_block_storage_plan. The planner recurses with eligible local Map places,
joins all reaching callback payloads, records point-sensitive direct break
conversions and restores the output family at the block boundary. Early exits
retain entries they bypass; newly inserted callback origins remain present.
Source ownership proofs remain authoritative and opaque callee effects do not
create a local elimination proof.

map_block_storage_emit emits a labelled result expression carrying the Map owner
tuple. Direct breaks return that tuple. Cleaned breaks dispatch to the block only
after required finally bodies have completed. Nested block and loop destinations
remain distinct. Exception discovery and lexical outer scopes now recognize
planned labelled-block results. No captured value is cloned implicitly.

Expanded witnesses exposed two additional native defects. A post-test body that
cannot reach its tail emitted a non-reaching empty projection after its completion.
It now returns without that projection. A body ending in a checked break emitted
its conversion before the break and again in an unreachable tail; the second
conversion used an obsolete carrier family. Definitely abrupt iteration bodies
now emit a typed unreachable tail instead of repeating conversions or reading
moved owners. Failed logs are retained separately from repaired evidence.

Native witnesses cover direct and cleaned block breaks, nested block destinations,
loop-to-block breaks for while, counted, post-test and for-of loops, an early exit
retaining the first entry, a refill with a third origin, cleanup overriding the
pending completion, mutation after deletion and after the block, survivor conflict
rejection and exact disposal at both optimization levels. The raw join proof
preserves one payload after unconditional deletion and two after an early break.

Final bounded evidence passes 83 affected Map tests, 12 editor entry-release
checks, ten phase/storage proofs and all eight normal-stack depth canaries.
Whole-project formatting changes zero files and zero fingerprints at the final
checkpoint. Matched sources, independent arithmetic oracles, token guards and
mutation controls cover plain labelled blocks and nested-loop block exits.
All five native and Node-after-tsc cases agree on 72 observations per row.
Current record: runtime/results/2026-10-09T04-36-26-109Z-verify.json. The earlier
successful 71-observation record, runtime/results/2026-10-09T04-28-28-494Z-verify.json,
is preserved independently. Twenty-nine closed logs and both records are imported
into SQLite, read back byte for byte and removed from loose storage.

B5 remains open: ordinary differing loop edge families, independent native phase
layouts, broader aggregate/within-expression precision and structural owned
record Set/Map keys remain pending. No whole-family regression or timing is
claimed. Installed consumers remain at gate 181; scratch compilers are current.
Next add a focused witness for an ordinary loop whose entry, backedge and break
retain different callback origins before changing its carrier eligibility.
Keep benchmark support current as it lands and preserve the 15 unrelated fixtures
using explicit Git paths.


### Ordinary loops with distinct Map edge families accepted

The ordinary-loop red passed source checking but retained the deleted capture in
native Rust. The planner previously excluded different entry/backedge and
break/backedge payload sets. It now joins every reaching edge while preserving
entry, backedge and break emptiness independently. Empty environments contribute
no live callable origin when joining control inputs; treating their phantom
callable as an origin caused an internal join panic and is repaired.

A narrowed pre-loop owner exposed a runtime excluded-environment panic. The first
owner held only the old callback, while a later backedge held the new callback.
MapIterationEpoch now records a separate working family. That family joins
physical incoming environments with reaching carrier environments; first and
repeated bodies convert into it before their local deletion/refill transitions.
It must retain physical incoming types because exception cleanup can observe them
before deletion finishes. An intermediate narrower working-family implementation
broke five cleanup witnesses; the repaired join restores them. Source capture
availability remains authoritative and no implicit clone is added.

Native witnesses cover zero/repeated iterations, empty break, an early break
retaining the old entry, replacement origins, while/counted/post-test/for-of forms,
a pre-loop deletion and refill that narrows the initial owner, and exact disposal
at both optimization levels. Raw proofs distinguish entry/backedge origins and
empty break, and prove a narrowed incoming type differs from its working family.
The editor rejects mutation of a surviving capture. The first editor rejection
attempt mutated before capture and was legal; the repaired witness mutates after
the loop while the Map still owns the callback.

Bounded evidence is retained without replacing failed runs. The original affected
batch passed 84 tests. After adding the working-family repair, a later batch
passed 79 and failed five cleanup cases. All five failures and the ordinary-loop
witness pass the six-case focused repair run. Current source also passes 13 editor
entry-release checks, 11 phase/storage proofs and all eight normal-stack canaries.
There is no claim of a new full compiler/LSP or whole-family run.

Matched support adds ordinaryEntryEdges and narrowedOrdinaryEntryEdges with three
observations, independent arithmetic oracles, current token guards and mutation
rejection. Native and Node-after-tsc untimed verification passes all five cases
with 75 observations per row. Canonical record:
runtime/results/2026-10-09T04-55-27-316Z-verify.json. Twenty-eight closed logs and
the matched record are imported into SQLite, read back byte for byte and removed
from loose storage. Final project formatting changes zero files and fingerprints.
Installed consumers remain at gate 181; scratch compilers are current.

B5 remains open: ordinary loops with no backedge and no completion boundary,
independent native phase layouts, broader aggregate/within-expression precision
and structural owned record keys remain pending. No timing is claimed. Next add a
focused ordinary no-backedge break/return witness before removing its eligibility
guard. Preserve the original failed evidence and the 15 unrelated fixture edits,
and commit only explicit paths.

## Ordinary loops without a backedge (2026-10-09)

The ordinary-loop eligibility guard incorrectly excluded loops whose bodies only
break, return or throw. The failing break witness now preserves the live second
entry while releasing the deleted first callback before capture mutation.
While, counted, post-test and for-of variants pass native execution. Return and
throw witnesses also verify exact callback disposal; those already passed before
the eligibility repair and are retained as regression coverage.

The variant batch exposed a second defect in post-test loops. A body without a
reachable tail can still break to the normal exit. Emission previously skipped
the exit projection, leaving a physical Map carrier incompatible with the next
borrowed iterator's checked record binding. The emitter now projects storage when
a break edge reaches that exit, while leaving return/throw-only tails untouched.
The original diagnosis of a for-of defect was incorrect; the failing generated
program was the post-test variant.

Focused acceptance passes 86 affected Map native tests, 14 editor release checks,
12 raw storage/completion proofs and all eight depth canaries. Matched support
adds noBackedgeEntryBreak and noBackedgeEntryReturn, independent arithmetic
oracles, mutation-rejecting source guards and two literal reference rows. All five
native/Node-after-tsc cases pass untimed with 77 observations per row. The canonical
record is runtime/results/2026-10-09T05-12-18-733Z-verify.json. Closed logs and this
record are imported into SQLite, read back byte for byte and removed from loose
storage. Whole-project formatting changes one workload and refreshes its verified
formatting fingerprint. Installed consumers remain at gate 181.

B5 remains incomplete. Next implement independent native phase layouts, then
broader aggregate and within-expression release precision. Structural owned
record Set/Map keys remain pending with the owner's Rust-style equality and
hashing decision. No full regressions or timing are claimed. Preserve the 15
unrelated fixture edits; use explicit Git paths. This is focused acceptance,
not the B5 feature-family milestone.

## Independent normal results after a catch (2026-10-09)

A new native witness showed that a try body and catch handler could both delete
the first entry, yet the joined native Map still retained its capture. Source
analysis correctly permitted a following mutation, but generated Rust rejected
it with E0502. Both paths now return the checked Handled/normal payload layout,
independent of the common storage carried by errors and pending controls.
The exit planner records that same layout for subsequent statements. Native
storage projection is factored through a leaf conversion helper so named Maps
and Maps in record fields use the same conversion.

This checkpoint consumes normal phase layouts for catch regions without finally.
It does not claim independent error/control layouts or cleanup-result emission.
Those remain the next phase-consumer work. Existing finally transport retains
its accepted implementation and lifetime behavior.

The focused witness passes successful and caught paths, mutation inside the
handler, nested record-field storage, and a throwing handler that retains both
entries until unwinding. Disposal is checked exactly at both optimization levels.
Editor checks accept released captures and reject both retained survivors and a
handler that omits deletion. Three failed field-test setup attempts are preserved:
the record needed a top-level named declaration and a field terminator. They were
witness setup defects, not accepted compiler behavior. Benchmark guard failures
from reused throw literals are also retained; the new helper uses a distinct
literal so the older exact-count guards keep their existing meaning.

Evidence: 87 affected Map native tests pass, supplemented by the repaired and
expanded focused witness. Fifteen editor release tests, one strengthened editor
negative check, 13 phase/storage proofs and eight depth canaries pass. Matched
support adds phaseJoinedEntry with successful and caught observations, independent
arithmetic oracles, literal reference rows and mutation rejection. All five
native/Node-after-tsc cases pass untimed with 79 observations per row. Canonical
record: runtime/results/2026-10-09T05-17-56-492Z-verify.json. Closed evidence is
imported into SQLite, read back byte for byte and removed from loose storage.
Project formatting changes the workload and refreshes its fingerprint after token
preservation; the final formatter pass changes zero files. Installed consumers
remain at gate 181. No full regression or timing claim is made.

Next continue independent phase layouts through finally and abrupt completions.
Then finish broader aggregate/within-expression precision and structural owned
record Set/Map keys with Rust-style equality and hashing. B5 remains open. Preserve
the 15 unrelated fixture edits and use explicit Git paths.

## Cleaned normal results after finally (2026-10-09)

The initial native witness already passed on the previous checkpoint. Its log is
named tsrust-b5-finally-result-red.log, but it is passing exploratory evidence,
not a runtime red. The focused storage-layout witness fails against the previous
implementation: following statements receive Map(6), while the checked cleaned
normal result is Map(7). The baseline sources were held only in process memory
while running that red witness and the working changes were restored afterward;
no archive or source snapshot was created.

The exit planner now selects Cleaned/normal for regions with finally and
Handled/normal otherwise. Both catch-plus-finally and finally-without-catch emit
normal cleanup results with the checked cleaned layout. Error and pending-control
cleanup continuations retain their existing common transport. This completes the
normal cleanup-result consumer, not independent abrupt layouts.

Native witnesses cover successful and caught bodies, no catch, propagated errors,
return from cleanup overriding pending completion, and Map storage in a nested
record field. Exact disposal is checked at both optimization levels. The affected
Map batch passes 88 tests, supplemented by expanded and record-field runs. All
16 editor entry-release checks, 14 phase/storage proofs and eight depth canaries
pass. The editor continues rejecting mutation of a surviving callback capture.

Matched support adds finallyNormalJoinedEntry with an independent arithmetic
oracle, a literal reference row and mutation-rejecting source guards. All five
native/Node-after-tsc cases pass untimed with 80 observations per row. Canonical
record: runtime/results/2026-10-09T05-24-29-449Z-verify.json. Closed logs and the
record are imported into SQLite, read back byte for byte and removed from loose
storage. Project formatting changes one workload and refreshes its fingerprint
after token preservation; the final formatter pass changes zero files. Installed
consumers remain at gate 181. No full regression or timing claim is made.

Next consume checked abrupt phase layouts. Start with error storage separated
from normal results; preserve per-operation availability and handler entry types.
Then handle return, break and continue storage by resolved destination, including
cleanup override and disposal. Do not combine independently checked payloads into
one storage tuple merely to satisfy the backend. Broader aggregate/within-expression
precision and Rust-style structural owned record Set/Map keys remain pending.
B5 is still open. Preserve the 15 unrelated fixture edits; use explicit Git paths.

## Body error storage and handler entry

Try/catch bodies without finally now transport checked Body/Throw Map layouts
independently from normal and control results. Explicit throw, fallible calls and
nested completion dispatch use per-operation error projections. Handler analysis
starts with the corresponding error layout. A separate emitter helper owns this
projection logic; no implicit clone is introduced.

The initial native witness already passed; the failing witness was the emitted
error envelope contract, which still used the common owner type. Focused checks
exposed clear/refill regressions. Clear-only regions now participate in storage
planning, exit conversion is based on actual Map operations, and eligible places
follow explicit moves into record fields. Treating every layout key as an emitted
storage conversion was incorrect for nested regions with no Map operations and
was removed. Original failures and repair runs are retained in SQLite.

All 89 affected Map native tests, 16 phase/storage proofs, 17 editor entry-release
checks and eight depth canaries pass. New native coverage checks direct throws,
fallible calls, nested finally, record fields and exact disposal at both
optimization levels. Existing moved-field clear/refill and borrowed get coverage
passes. The standalone borrowed-get type proof was positive coverage, not a red
witness. No complete regression or timing run is claimed.

Next separate errors through finally, outgoing handler errors and cleanup errors,
then return/break/continue storage by resolved destination. Rust-style structural
owned record Set/Map keys remain subsequent work. B5 remains open. Installed
consumers remain at gate 181; preserve the 15 unrelated fixture edits.

Matched verification exposed three borrowed-get mismatches in loop-local Maps.
A focused while-loop native witness reproduced the failure. Clear-only layouts
were bypassed by an early local-storage guard, and loops without carried Map
epochs only recursively planned for-of bodies. Those paths now plan while,
counted and post-test bodies as well. The new witness executes all three forms.
The original 89-test pass is preserved; the repaired batch includes this witness.

Final repair acceptance: 90 affected native tests pass, with the 16 storage
proofs, 17 editor checks and eight depth canaries rerun on the repaired source.
Matched native/Node-after-tsc verification passes all five cases, with 81
observations per row. Canonical record:
runtime/results/2026-10-09T05-44-53-954Z-verify.json. The errorPhaseEntry helper
has an independent arithmetic oracle, literal reference rows and deletion/mutation
source guards. Existing ABI, driver and report support applies unchanged.
Project formatting is token-preserving and idempotent; its final pass changes
zero files. Closed evidence is imported, read back byte for byte and removed
from loose storage. Focused acceptance is complete; family measurements and
complete regressions remain deferred until B5 is ready.

## Handler entry with finally

A finally clause no longer disables precise Body/Throw transport into a catch
handler. The handler's initial native Map type now matches the checked error
payload, and the body error envelope uses that layout. Normal cleanup, outgoing
handler errors and pending controls retain their existing independent paths.

The new failing phase proof exposed the old common Map type at handler entry.
Native coverage adds finally observation to the existing successful/caught
scenario and verifies exact callback disposal at both optimization levels.
Editor coverage checks successful analysis, rejection of surviving-capture
mutation and rejection when failure can precede deletion, with and without
finally. Matched support adds finallyErrorEntry, an independent arithmetic
oracle and mutation-rejecting source guards. The first registration attempt
found the old full observation row contract; it was updated for the added row
element. This is a support-contract repair, not a native semantic defect.

This chunk covers body errors entering a catch when finally follows it. Errors
propagating through finally without a catch, outgoing handler/cleanup errors and
return/break/continue layouts remain pending. B5 remains open; no family timing
or complete regression run is claimed.

The affected native batch initially passed 89 cases and failed the existing
outer_map_catch_loop_releases_iteration_captures witness. The no-carried-epoch
loop path dropped inherited outer Map eligibility, so a handler refill could
retain the narrowed input family rather than add its new callback origin.
Preserving eligible outer places repairs the witness, including record-field
storage and exact disposal. An exploratory contextual-carrier change did not
repair the defect and was removed; no general carrier conversion was changed.
The initial matched run exposed the same failure during compiler emission.
Both original failures and focused repair evidence are retained.

The next affected batch passed 89 cases and exposed one post-test direct-break
regression: an outer Map was narrowed both inside the body and at the checked
loop completion, moving the same owner twice. The fallback now retains inherited
places except when a consistent releasing loop completion already owns their
conversion. The post-test witness passes again. Matched verification on the
intermediate repair passed and is retained as an intermediate record; it does
not replace the affected native failure or final-source verification.

Final acceptance: 90 affected Map native tests, 17 phase/storage proofs, 17
editor entry-release checks and eight depth canaries pass on the final source.
The full native batch includes the newly expanded finally/disposal witness.
Matched native/Node-after-tsc verification passes all five cases with 82
observations per row. Canonical final record:
runtime/results/2026-10-09T05-57-32-074Z-verify.json. The earlier passing
intermediate record is runtime/results/2026-10-09T05-54-22-483Z-verify.json.
Closed logs and both records are imported into SQLite, read back byte for byte
and removed from loose storage. Project formatting is token-preserving and
idempotent; the final formatter pass changes zero files. Installed consumers
remain at gate 181. No timing or whole-regression claim is made.

Next consume Body/Throw storage through finally without a catch, then outgoing
handler and cleanup errors. Cleanup entry must respect the pending completion's
Map layout; normal results, error results and resolved control destinations must
not be forced into one common tuple. Preserve disposal order, replacement and
cleanup override. Structural owned record keys and the wider B5 family are
still pending.

## Independent cleanup entry for pending errors

Finally without a catch now carries Body/Throw Map storage separately into
cleanup. A dedicated map_cleanup_phase_plan module plans the cleanup AST and
native snapshots from the checked Handled/Throw input, which equals the body
error input when no handler runs. Only the error-entry plan is materialized and
consumed in this chunk. The plan interface retains the completion identity for
later normal/control consumers. Native planning snapshots are reset before
replanning; source ownership facts remain intact. Plans share their registry
through Arc when contexts are cloned, and store binding-type differences rather
than copies of every binding's metadata. Nested plans remain separate.

The failing witness initially found no error-entry cleanup plan. The proof now
checks its input type and emitted FinallyIncoming error envelope. Native coverage
checks normal and thrown body outcomes, cleanup iteration, cleanup-time mutation
of a released capture, record fields and exact callback disposal at both
optimization levels. Editor coverage rejects mutation of a surviving capture
and rejects a body error that can precede deletion.

The first native run exposed conversion into an absent callable environment.
An empty Map environment has no payload to convert: changing its representation
now drops the old empty owner and constructs the checked empty destination.
Empty environments are retained in native storage snapshots, preventing fallback
to a stale nonempty type. Matched verification exposed this snapshot omission
in the new nested outer-catch helper.

The first affected batch passed 90 cases and failed the existing cleanup-deletion
witness because the pending completion enum kept a capture loan alive. Finally
now separates normal, error and control owners into independently typed options
inside a completed enum scope. Exactly one option is live; every option is
fully consumed or dropped before cleanup runs. The owner tuple is destructured
directly, avoiding a residual tuple temporary that can retain a loan. This
preserves phase types instead of merging them into one common storage tuple.
Original failures and repair runs are preserved in SQLite. A mutation variant's
initial source substitution missed the formatter's `} finally {` spelling; the
assertion caught the no-op, and the witness now executes the intended source.

Cleanup outgoing errors, resumed pending-error storage after cleanup and distinct
return/break/continue result layouts still use their existing transport and
remain pending. This acceptance concerns precise body-error transport into
cleanup; it does not claim all abrupt phases complete. B5 remains open.

Final acceptance: 91 affected Map native tests pass, supplemented by the expanded
cleanup-mutation and record-field witness. All 18 phase/storage proofs, 18 editor
entry-release checks and eight depth canaries pass. Matched native/Node-after-tsc
verification passes all five cases, with 83 observations per row. The new
finallyPendingError helper includes a nested outer catch, cleanup iteration and
clear, normal-only work after cleanup, and a distinct caught-error observation.
Its oracle is independent arithmetic; literal reference rows and mutation guards
are registered. Existing ABI/driver/oracle/run/profile/report hooks apply.
Canonical record: runtime/results/2026-10-09T06-12-33-163Z-verify.json. Closed
logs and the record are imported into SQLite, restored byte for byte and removed
from loose storage. Project formatting preserves tokens and is idempotent; the
final pass changes zero files. Installed consumers remain at gate 181. No full
regression or timing claim is made.

Next consume checked outgoing handler/cleanup error layouts and the cleaned
pending-error result. The new error-entry Plan supplies phase-local native
metadata; do not fall back to the joined cleanup AST for that path. Extend the
independent owner transport to resolved return/break/continue destinations, with
cleanup override and disposal witnesses. Structural owned record keys remain
subsequent work. B5 is not complete.


### Handler rethrow storage

A catch body that removes a known Map entry and then throws carries its checked
survivor layout to the enclosing handler. Error forwarding projects that layout
into the enclosing error destination instead of reusing the ordinary statement
input layout. Native transport retains an initialized owner independently of
source-name availability; forwarding does not make a transferred source binding
usable again. The error dispatch helper accepts only an already selected Throw
completion, so unrelated return and loop-control storage is not converted.

Catch entry consumes the pending completion into independent normal, error and
control owners before selecting a branch. This matters for owners with disposal:
an inactive completion alternative must not retain the deleted entry's capture
loan while the outer catch mutates that capture. Native witnesses cover direct
Map bindings and record fields, survivor invocation, released-capture mutation
and exactly-once disposal at both optimization levels. Editor checks accept the
released capture and reject mutation of the surviving capture.

Focused evidence: 92 affected Map tests and 19 storage proofs pass. The expanded
record-field/disposal witness and isolated error dispatch pass separately. The
new editor diagnostic witness passes. No analyzer/parser dispatch changed, so
this chunk does not repeat the depth canaries or run a family regression.
The matched workload adds handlerRethrowEntry with independent arithmetic
2 * value + 3, literal reference rows and mutation guards; each row now has 84
observations. Native and Node-after-tsc verification is untimed. Canonical final record:
runtime/results/2026-10-09T06-23-10-157Z-verify.json. All five cases pass on both
engines. Closed logs and both matched records are imported into SQLite, restored
byte for byte and removed from loose storage. Project formatting changes zero
files on its final pass; diff checks pass.

Still pending: handler errors with finally, cleanup errors, resumed pending
errors after cleanup and distinct return/break/continue destination layouts.
Structural owned record keys follow those capture-release repairs. B5 remains
open; installed consumers remain at gate 181.


### Cleanup outgoing error transport without catch

The no-catch finally emitter now consumes the checked Cleaned/Throw Map layout
for cleanup errors and for the normal completion that resumes a pending error.
It attaches that layout to each cleanup error envelope and projects the selected
outgoing owner into the enclosing error destination. Return and loop-control
owners retain their separate existing representation.

The original execution witness already passed on the previous checkpoint. A
stronger native-layout witness first missed the specialized path; adding a
released-capture write activates it and proves the actual defect: checked Map7
was widened to Map6 in the cleanup error envelope. That focused witness fails on
the previous emitter and passes with this change. Runtime witnesses cover resumed
and replacement errors, released and surviving captures, record-field storage
and exactly-once disposal. Editor diagnostics retain surviving-capture conflicts.

The matched cleanupOutgoingEntry helper observes both error identity and the
surviving callback. Its independent result is 2 * value + 30, plus one when
cleanup replaces the pending error. The workload now observes 85 values per row;
literal reference rows and mutation guards cover the added behavior. Timing
remains deferred. Final checks and the canonical record are recorded below.

This slice concerns finally without its own catch. Catch-plus-finally still uses
its joined pending owner path and needs separate outgoing handler/cleanup plans.
Distinct return/break/continue layouts remain pending. B5 remains open, followed
by Rust-style structural owned record keys. Installed consumers remain at gate 181.


The affected Map batch completed with 92 passes and one loop-cleanup failure.
The failure exposed unrelated return/control arms being generated for an already
selected error when the enclosing function had no carried Map owner. Selected
errors now use the error-only dispatch even without an outer storage tuple. The
failed loop witness passes after repair; passed batch evidence is retained rather
than rerunning the entire group. All 20 phase/storage proofs, the new editor
witness and expanded record-field/disposal checks pass. Parser/analyzer dispatch
is unchanged; no depth, full-family or timing run is claimed.


Final matched verification passes all five cases on native and Node-after-tsc,
with 85 observations per row. Canonical record:
runtime/results/2026-10-09T06-30-32-665Z-verify.json. Original failed checks,
repair checks, final checks and both matched records are preserved in SQLite,
restored byte for byte and removed from loose storage. Final project formatting
changes zero files and fingerprints. Source diff checks pass.

Next: consume independent handler/cleanup error plans in catch-plus-finally,
then destination-specific return/break/continue owners and cleanup overrides.
Keep those phases separate rather than widening all pending owners together.


### Independent catch-plus-finally owners

Catch-plus-finally now has a dedicated emitter module,
map_catch_finally_storage.rs. Body completion is consumed before entering catch;
handler completion is then consumed into independent normal, error and control
owners before entering cleanup. Errors use the checked Handled/Throw layout and
the existing phase-local cleanup plan. Cleanup failures and resumed errors use
the checked Cleaned/Throw layout. Normal and control owners remain separate;
inactive alternatives cannot retain a deleted entry's capture borrow.

Two baseline witnesses fail before this implementation: the handler error
transport widens checked Map7 back to Map6, and generated Rust rejects a capture
write in cleanup after catch removes the only entry borrowing that capture.
Both pass with independent owners. The former joined-owner/clear-all special
case and unused dispatch wrappers are removed, reducing shared exception code.

Focused execution covers normal body completion, handled failure, handler
rethrow, cleanup replacement, surviving callback invocation, direct Map bindings
and record fields. Disposal is checked exactly once at both optimization levels.
Editor diagnostics accept released-capture writes and still reject writes to a
surviving capture. The affected Map batch has 94 passes; an additional expanded
completion-mode witness passes separately. All 21 phase/storage proofs pass.
No parser or analyzer dispatch changes; no depth/full-family/timing run is claimed.

The matched catchFinallyEntry helper covers all four completion modes. Independent
results are 2 * value + 3, +4, +43 and +44 respectively. Literal reference rows and
mutation guards include each mode, handler deletion, rethrow and cleanup override.
Each workload row now contains 89 observations. Final verification and SQLite
record details follow below when the run closes.

Still pending: destination-specific return/break/continue storage, cleanup
control overrides and remaining aggregate/expression precision. Rust-style
structural owned record keys follow the capture-release work. B5 remains open;
installed executable consumers remain at gate 181.


Final matched verification passes all five cases on native and Node-after-tsc,
with 89 observations per row. Canonical record:
runtime/results/2026-10-09T06-37-22-450Z-verify.json. Closed logs and the matched
record are stored in SQLite, restored byte for byte and removed from loose
storage. Final project formatting changes zero files and fingerprints; source
diff checks pass. The current compiler build has no warnings.

Next: resolve return/break/continue owners by their actual destination, preserve
cleanup overrides and disposal, then complete the remaining B5 aggregate and
expression witnesses before the family regression/performance review. Structural
owned record keys retain the accepted Rust-style value semantics.


### Control cleanup planning foundation

The checker already distinguishes Return, Break(target) and Continue(target) in
its exception snapshots. Cleanup planning previously materialized only Throw.
It now materializes a separate native cleanup plan for every abrupt Handled
completion, preserving the complete resolved target in the existing plan key.
Two focused baseline witnesses fail because those plans are absent. They pass
after the change: a narrowed return owner seeds cleanup correctly, and two
separate break destinations plus one continue destination retain their respective
Map survivors. The destination witnesses deliberately use different surviving
callback origins, so a merged plan cannot satisfy the assertions.

This is planning infrastructure, not acceptance of precise native control
transport. Existing emitted return/break/continue carriers still use common
storage. The next implementation must consume the plans without re-widening
owners and must preserve pending return values and cleanup overrides.

The remaining transport design is explicit:

1. Generate a control carrier with a separate typed storage payload per resolved
   completion destination. Use generic payload parameters so local enum
   declarations do not capture anonymous closure lifetimes.
2. Attach each checked phase layout to return and control emission, as error
   emission already does. Preserve source-name availability independently of
   initialized native owners; never revive a transferred source binding.
3. Fully consume the carrier before entering cleanup. Select the plan by the
   exact Return/Break/Continue kind and target, then emit its phase-local body and
   native metadata. Do not choose a plan only by the general control category.
4. Project cleaned storage into the actual enclosing destination. Local loop and
   labeled-block destinations must receive their checked carried layout; an
   enclosing finally receives control before a local destination is resumed.
5. Verify returned owned values, both loop control kinds, multiple labels, nested
   cleanup, overriding return/error/control and exactly-once disposal. Add matched
   observations and editor diagnostics with each observable behavior change.

Control transport remains open. No new runtime capability or performance result
is claimed by this planning checkpoint. Its checks are recorded below when closed.


Bounded verification passes: 95 affected Map native tests, 23 phase/storage
proofs, 21 editor entry-release checks and all eight depth canaries. Project
formatting changes zero files and fingerprints; diff checks pass. Existing
matched workload support is unchanged at 89 observations per row. A focused
untimed native/Node-after-tsc verification checks the current compiler; the final
record is recorded below after the run closes. No new timing is performed.


Final matched verification passes all five cases on both engines. Canonical record:
runtime/results/2026-10-09T06-43-43-721Z-verify.json. Original red witnesses,
closed checks and the matched record are preserved in SQLite, restored byte for
byte and removed from loose storage. Source acceptance concerns control cleanup
planning only; the destination-specific native carrier remains the immediate
next implementation. B5 and the broader compiler completion plan remain open.


### Precise return transport through finally without catch

map_return_storage.rs introduces a consuming generic control carrier. Its Return
variant owns both the evaluated result and the checked return-phase Map storage;
its Common variant preserves the existing loop-control transport. Generic payload
parameters keep local enum declarations independent of closure lifetimes.
No-catch finally emission attaches the checked Body/Return layout to return
production, consumes that owner before cleanup and uses the independent Return
cleanup plan. Normal cleanup results and overriding returns project the checked
Cleaned/Return storage into the enclosing return destination. Cleanup errors
continue to use independent error storage.

The baseline execution example already passed. The focused failing layout
witness proves the actual precision defect: checked Map16 was absent from the
return envelope, which used the common Map18 storage. The new carrier includes
the checked return layout and native execution preserves the result evaluated
before cleanup. Cases cover scalar and owned array results, cleanup return and
throw overrides, record fields, nested catch-plus-finally forwarding, surviving
callbacks and exactly-once disposal at both optimization levels. Editor checks
retain surviving-capture diagnostics.

The affected batch initially has 95 passes and two failures. Both failures concern
cleanup that clears all entries: absence of a Map layout does not imply absence
of reachable return completion. Return resumption now uses cleanup reachability,
and both failed witnesses pass after repair. A nested-return witness also checks
that forwarding selects the outer Return variant rather than a loop-control
fallback. Passed original checks are preserved. All 24 phase/storage proofs and
the new editor witness pass. No parser/analyzer dispatch changes or full-family
regression/timing run is claimed.

The matched returnCarrierEntry helpers observe a pending return, an overriding
cleanup return and an overriding cleanup error. Independent results are
2 * value + 3, +13 and +13. Literal reference rows and mutation guards cover all
three modes and cleanup survivor observations. Each workload row now contains
92 observations; verification details follow below after the run closes.

Precise control work remains open: break/continue destinations still use common
storage, and catch-plus-finally return transport still needs this carrier. Extend
the carrier by resolved target, consume the existing target-specific cleanup
plans and verify cleanup control overrides. B5 remains open, followed by the
accepted Rust-style structural owned record key work. Installed consumers remain
at gate 181.


Final matched verification passes all five cases on native and Node-after-tsc,
with 92 observations per row. Canonical record:
runtime/results/2026-10-09T06-59-05-488Z-verify.json. The current return-envelope
proof passes after the repairs. The original 95 native passes, both repaired
cases, nested forwarding and expanded owned-result/disposal evidence are retained;
no complete batch restart is claimed. The current compiler build has no warnings.
Closed baseline, red, repair and final logs plus the matched record are stored
in SQLite, restored byte for byte and removed from loose storage. Final project
formatting changes zero files and fingerprints; source diff checks pass.

Next consume precise return carriers in catch-plus-finally, then replace Common
loop-control storage with destination-specific break/continue payloads. Preserve
returned values across cleanup and drop them exactly once on override. Complete
remaining B5 aggregate/expression witnesses before family regressions and timing.


## Catch-plus-finally return storage, 2026-10-09

The handler return envelope now uses the checked Handled/Return Map layout.
Body returns and handler returns join through one consuming return carrier;
the selected owner is consumed before cleanup enters its independent return
plan. This retains the saved return value while allowing cleanup to mutate
captures released by deleted entries. Surviving entries retain their loans.
Cleanup return and throw overrides dispose of the original saved result.

The focused red layout witness found Map17 absent from the legacy control tuple,
which carried Map18. The repaired witness passes with all 25 completion-layout
proofs. Native checks cover handler returns, body returns bypassing catch, owned
array results, return and error overrides, record-field storage, and disposal at
both optimization levels. Existing return cleanup and normal/error catch-finally
checks also pass. Editor checks accept the released capture and still diagnose
mutation of the surviving capture. These are bounded checks, not full regressions.

The matched workload adds three catch-return modes and independent expected
results, totaling 95 observations per row. Source mutation checks initially found
a duplicate trace statement masking removal of the original observation; the new
helper now uses a distinct trace parameter, and registration checks pass. The
original failed checks are retained with the repaired checks in SQLite. Timing
and installed-consumer refresh remain deferred until the B5 family milestone.

Next implement destination-specific break/continue carriers. Common loop-control
storage is still a separate unfinished path. B5 remains incomplete, including
remaining aggregate/expression precision and structural owned record keys.

Final untimed verification passes all five cases on native and Node-after-tsc.
Canonical record: runtime/results/2026-10-09T07-09-15-676Z-verify.json.
The intermediate verification record and all closed diagnostic logs are imported
into SQLite, restored byte for byte, and removed from loose storage. Final project
formatting changes zero files and fingerprints. Executable consumers remain at
the previously accepted family checkpoint; the scratch compiler is current.


## Destination-specific pending loop controls, 2026-10-09

Pending break and continue completions now have distinct native payload variants
for each resolved label. Body and handler phases attach only the source points
that can reach that destination. Each variant enters the checked cleanup plan
for its exact CompletionKind and resumes through a selected destination dispatch.
Local loop and block restoration project from the actual surviving Map payload,
including record fields. Enclosing finally regions receive the selected owner
before an outer loop or block consumes the control.

The red layout witness found destination Map7 missing from the common Map6
control tuple. Source planning already distinguished two break labels and one
continue label; native transport now retains those separate layouts. The native
witness covers all three destinations, catch handlers, nested finally forwarding,
record-field storage and disposal. A mixed return/empty-break case also passes.

Implementation checks exposed and repaired eager projections at unrelated branch
points, a record-root versus Map-leaf projection error, and nested generic dispatch
that tried to use a precise owner for hypothetical inactive control alternatives.
The 100-case affected Map batch preserved 94 passes and six failures. Follow-up
repairs consume the completion enum completely before cleanup so inactive variants
cannot retain capture loans through a match temporary. Passed batch evidence is
retained; only affected cases are rerun.

The new map_control_storage module owns control layout selection, attachment,
cleanup entry and selected dispatch. Generic unknown completion forwarding keeps
its common fallback; fresh controls overriding cleanup still need dedicated
precision witnesses. Do not describe every cleanup override as independently
transported yet. Remaining B5 work includes aggregate/expression precision and
structural owned record keys. Full family regressions, timing and installed
consumer refresh remain deferred.


Focused acceptance retains the initial 94 Map passes and targeted green repairs
for all six failed cases. Final expanded disposal checks also pass for direct
Maps, record fields and catch handlers. All 26 layout proofs and five related
editor checks pass. The matched workload adds body and handler variants for two
break labels and one continue label, with six independent result formulas and
101 observations per row. Source registration and mutation guards pass. The
intermediate five-case native and Node-after-tsc verification record is
runtime/results/2026-10-09T07-22-54-565Z-verify.json; this is untimed evidence.

Next add precision witnesses for controls created by cleanup overrides, preserving
common fallback behavior until each override has a checked destination-specific
projection. Then finish remaining B5 aggregate/expression precision and owned
structural record keys before family regressions and measurements.

Final formatted-source verification passes both engines and all five cases.
Canonical record: runtime/results/2026-10-09T07-24-52-777Z-verify.json.
The intermediate record and all closed diagnostic logs are stored in SQLite,
restored byte for byte and removed from loose storage. Final project formatting
changes zero files and fingerprints; source diff checks pass. No timings or
complete family regressions were run. The scratch compiler is current, while
installed executable consumers remain deferred to the family checkpoint.


## Controls created by finally, 2026-10-09

Cleanup now emits its own checked return, break and continue payload layouts.
The shared map_cleanup_storage module handles normal cleanup, pending errors,
pending returns and pending loop controls. It consumes the cleanup completion
enum before restoring the selected owner and forwarding it. Error storage stays
independent, and replaced saved returns or errors are dropped before dispatch.
Control overrides use the exact cleaned destination layout, including local loop
or block restoration and forwarding through an enclosing finally.

The native baseline for the override example already passed; the red proof
identified native type widening rather than a wrong baseline result. It found
checked cleanup Map7 absent from the legacy Map6 control tuple. The new proof
passes, along with all 27 completion-layout proofs. New native coverage includes
cleanup continue followed by break, replacing pending return, throw or break,
record fields, catch handlers, owned array results and survivor disposal.
Editor coverage accepts mutation of a deleted entry's capture and rejects
mutation of a surviving entry's capture.

The matched workload adds eight cases: pending return, throw, continue and break,
from both body and catch paths. Each case has an independent result formula,
literal reference rows and source mutation guards. The workload now has 109
observations per row. Full B5 regressions and timing remain deferred. Generic
unknown forwarding retains a common fallback; do not treat that fallback as
proof of precision for aggregate or expression paths that still need witnesses.


The affected Map batch passes all 101 cases without failures. All 27 layout
proofs, the new editor check and five related editor checks pass. The new owned
result, field and handler variants pass formatted native execution, with direct
survivor disposal checked at both optimization levels. Source registration and
mutation rejection controls pass. These checks cover the changed storage family;
no whole compiler or LSP regression run is claimed.

Final formatted-source matched verification passes native and Node-after-tsc,
all five cases, with 109 observations per row. Canonical record:
runtime/results/2026-10-09T07-36-51-906Z-verify.json. The intermediate record,
original red layout proof, passing native baseline and closed acceptance logs
are stored in SQLite, restored byte for byte and removed from loose storage.
Final project formatting changes zero files and fingerprints; source diff checks
pass. The scratch compiler is current. Family timing and installed-consumer
refresh remain deferred. Current-state sections now supersede the older dated
next-step notes; precise indexed-element release is the next implementation.

## Current continuation: retained array removal results

The source ownership foundation is checkpointed in `d27d463`. Native follow-up
now specializes immutable pop/shift results, narrows surviving arrays, and carries
survivors out of normal lexical blocks. The original red witness passes, including
shift, result moves, nested blocks, record fields, mixed Map storage and exact
disposal at optimization levels 0 and 3. The affected array batch passes 27 tests.
A fallible result regression also passes after retargeting invocation error effects
to the specialized callback origin. Keep the indexed capture plan authoritative
for remaining requirements and final evidence. Reassignable result storage and
same-expression release are not yet accepted. Structural record keys follow the
owner's Rust-style equality decision. B5 remains incomplete.

### Retained-result checkpoint evidence

All five matched cases pass untimed on native and Node-after-tsc. The workload
has 117 observations per iteration; the added independent formulas are
`3 * value + 4` for result-scope disposal and `3 * value + 5` for result
independence. Source contracts, mutation guards and literal oracle expectations
pass. SQLite preserves `runtime/results/2026-10-09T08-38-50-598Z-verify.json`,
with empty startup, compilation and execution timing arrays. Twenty-three
diagnostic logs, including original failures and repaired checks, were imported
and restored byte for byte before deleting their loose copies. A supplementary
explicit thrown-callback test passes after narrowing the caught runtime-error
union; its original rejection and repaired result are also retained. All four
existing pop/shift regression tests pass. Final editor hover and diagnostics,
eight depth canaries and zero-change project formatting pass. Pop/shift catalog
examples include retained-result scope release, with native outputs checked by
the focused tests. No full regression, timing or installed-consumer refresh ran.

## Direct mutable-result continuation

Direct reassignable pop/shift results now use separate native storage epochs,
with RHS-before-disposal order and concrete nullable environments. New callback
origins, repeated removals through empty and normal lexical-block replacements
pass. Empty source snapshots clear the previous result owner. The isolated
`array_result_storage.rs` module holds result-specific planning and retagging.
Branch/loop replacements with new captured environments receive source diagnostics
until their joins are implemented. Next continue same-expression release and
aggregate/control-flow result storage; do not treat B5 as accepted. The matched row
has 119 observations and five untimed native/Node-after-tsc cases pass. Record:
`runtime/results/2026-10-09T08-58-51-249Z-verify.json`. The indexed capture plan
contains focused coverage and limits. Installed consumers remain unrefreshed.

Mutable-result evidence also covers self-assignment and preservation of a moved
result owner. Pop/shift catalog examples verify direct sentinel replacement with
outputs `3 9` and `5 9`; exact disposal is checked independently. Nineteen closed
diagnostic logs, including original failures and repaired checks, and the matched
verify record were imported into SQLite, restored byte for byte, and removed from
loose storage. Project formatting is token-preserving and idempotent; the final
checkpoint formatter changes zero files and fingerprints.

After the small eligibility-scan optimization, the current debug compiler's
emitted Rust matches the verified workload prefix byte for byte under the
harness's stdout trimming and entry-point rename. The initial comparison
incorrectly required an extra newline; its corrected comparison and final
zero-change formatter log are also stored and restored exactly in SQLite.
No new native verification run or timing was needed for unchanged generated code.
