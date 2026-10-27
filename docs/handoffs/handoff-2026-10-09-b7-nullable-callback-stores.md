# B7 nullable callback stores and definite clearing

B7/gate 183 remains in progress. This checkpoint follows `666c05d`. Read AGENTS.md,
docs/development-workflow.md, docs/plans/caller-borrowed-record-stores-plan.md and this
handoff. No owner question is pending. Discriminated unions remain the dedicated
D2 feature, including validation-library composition; they are not part of B7.

## Implemented

Nullable readonly and mutable callback fields can start absent, receive a direct
callback input through a mutable record parameter, invoke the installed callback,
and be cleared. A mutable input can be reused while the cleared holder remains
live and is observed again. Named clear relays preserve release on both early and
ordinary normal exits. A conditional clear that leaves one populated path does not
release the input. A callee-local capture still reports callback-escape.

The local field retains its concrete nullable callback carrier. Present callback
payloads use checked conversion bounds; absent variants carry no environment.
Local record annotations infer nested closure generic arguments rather than naming
signature-only parameters. Release replay now traverses nullable members.

The isolated nullable_callback_clear_emit module proves absence on every normal
exit. Native reification then reconstructs only absent variants, ending the old
borrow without cloning, boxing or lifetime casts. Original-payload release alone
is insufficient: a replacement might contain another callback. The absence proof
handles direct stores, blocks, branches, bare early returns and named relays.
Unknown effects, loops and recursive proof cycles remain conservative.

The combined workload exposed a separate error-contract omission: record field
origin collection ignored nullable payloads. It now resolves the declared and
actual payload before collecting origins. Nullable parameter bounds therefore
include the installed wrapper's fallible result. Typed string callback failures
reach a bound catch variable, with matching editor hover.

## Evidence

SQLite keys start with `tsrust-b7-nullable-`. Original failed attempts are retained.
`escape-control.log` ran zero tests because its exact filter was incomplete; it is
not acceptance evidence. The subsequent escape check and the complete nullable
suite execute the actual rejection witness. The first error witness accidentally
consumed an owned string and correctly reported callback-once; error-contract-red
uses a reusable scalar capture and exposes the actual native bound defect.

`controls.log` passes 47 affected integration tests plus one release-summary unit
test. After the error-origin repair, `error-repair.log` passes ten store-failure,
four existing fallible-record and six nullable-store tests. `editor-controls.log`
passes 22 nullable editor checks, including clear/reuse diagnostics and typed catch
hover. `depth-final.log` passes all eight required depth canaries. No complete
compiler/LSP regressions or timing ran.

The matched workload now has 21 observations per scenario. Three new observations
exercise nullable installation, invocation, a clear relay with alternating early
exit, input reuse and a subsequent absence observation. Its independent arithmetic
oracle, source guards, mutation controls, registration and hashes are current.
Both `runtime/results/2026-10-09T19-05-49-632Z-verify.json` and the formatted-source
`runtime/results/2026-10-09T19-06-42-716Z-verify.json` verify five cases on native and
Node-after-tsc, with no timing samples. Both are stored byte-exact in SQLite and
removed from loose storage. Compiler/workload inputs and generated text artifacts
are preserved under content-addressed `runtime/source-evidence/nullable-caller-stores/`
keys. Whole-project formatting changed one workload, then changed zero files on the
second pass. Formatted registration and native execution pass.

## Next work

Start with nullable replacement by a different borrowed origin. Keep a live holder
and prove the previous input becomes independently usable while the replacement
input remains borrowed. Cover readonly/mutable alternatives, nested fields and
aliases. Add a focused red before changing carriers or release summaries.

Then cover retained stores across branch joins, loops, early exits and disposal,
followed by recursive summaries and nested generic callback environments. Existing
release replay still deliberately declines new borrowed captures, defaults and
exception syntax; do not remove those guards without explicit completion/provenance
transport. The nullable absence proof is conservative, not a completed loop model.
B7 requires the remaining implementation and its family milestone checks before
acceptance. Do not mark the compiler objective complete.

Fifteen unrelated fixture edits remain uncommitted and semantically untouched.
Use explicit Git paths. Installed five consumers remain at B5 until family
acceptance. Reuse `/private/tmp/tsrust-gate100-editor-target`, with incremental off;
Node 24.11.1 first on PATH; Homebrew Python for SQLite. Target inspected at 2.1 GiB.
Five closed workload artifacts totaling 772,607 bytes were removed after evidence
preservation. No cache rebuild was started merely to refill removed artifacts.
