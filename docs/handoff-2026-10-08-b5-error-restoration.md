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
restoration. All 77 composition tests, eight normal-stack depth canaries and 15
exception-contract tests pass. The focused editor test passes.

The matched workload now retains 30 observations. Source guards and rehashed
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
