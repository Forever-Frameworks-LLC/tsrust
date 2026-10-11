# B7 exact callback replacement

B7/gate 183 remains in progress. This checkpoint follows `62bf6192`. Read AGENTS.md,
docs/development-workflow.md, docs/plans/caller-borrowed-record-stores-plan.md and this
handoff. The compiler objective remains active. No owner decision is pending.
Discriminated unions remain the separate required D2 family.

## Implemented

Nullable fields can replace a borrowed readonly or mutable callback with a different
callback environment. The overwritten input becomes independently callable while
the holder stays live. The replacement input remains borrowed. Nested record fields,
stable callback aliases and named relays preserve those rules. An installer that
can skip replacement retains the old input. Branches installing either a borrowed
callback or a capture-free callback keep both actual native alternatives.

Non-nullable fields use the same native reconstruction helper. Their focused old-input
reuse witness initially failed with native E0499 and now passes.

## Representation and ownership

Release replay now admits new borrowed captures only when the existing caller-owner
transport can describe them. It still replays original incoming payload tokens and
requires their disappearance on every normal exit. Defaults and exception syntax
remain conservative. Source release occurs before new retained loans are attached;
this does not let the caller access the new borrowed input independently.

A single checked forwarding-closure origin can receive different concrete callback
environments at different calls. Nullable wrappers with abstract borrowed callback
captures therefore use a checked singleton source family. Its finite native payload
alternatives retain whole correlated capture vectors. No source callable variant,
dynamic dispatch, implicit clone or lifetime cast is added. Existing origin
registration still validates known monomorphic invocation bodies.

Nullable replacement metadata unwraps the injected actual closure carrier before
joining final record storage. This also records capture-free alternatives and adds
their From conversion bounds. Source annotations and final native carriers remain
separate.

The isolated record_callback_reification_emit module selects payload alternatives
using this call's actual callback types and the checked retained capture positions.
Unknown argument types remain conservative. Reconstruction preserves null/undefined
variants and all genuine replacement alternatives, while excluding the overwritten
environments. Original-payload release and native lifetime reconstruction are separate
proof obligations.

## Verification and evidence

SQLite records use the prefix `tsrust-b7-nullable-replacement-`. The initial source
red is red.log; source-release.log exposes the native generic mismatch; carrier.log
exposes retained native lifetime; reification.log repairs direct replacement.
branch-native-red.log exposes the missing capture-free conversion. normalized.log
repairs its actual nullable carrier metadata. The earlier branches.log also contains
an invalid repeated mutable borrow in the first draft witness; the repaired witness
runs independent scenarios for the two branches instead of weakening borrow rules.
nonnullable-red.log exposes the corresponding plain-field native failure.

Focused integration coverage totals 72 cases: the initial 70-case affected batch,
the added readonly replacement case, and the non-nullable case. Shared-reification.log
reruns the four affected replacement/record suites, totaling 51 cases. Final error
controls rerun the other 21 direct-store/failure cases. Borrowed-storage unit checks
pass 34 cases. Editor nullable checks pass 23 cases, including old-input release,
new-input rejection and nullable field hover. All eight depth canaries pass. No full
compiler/LSP regressions or timing ran.

The workload now has 24 observations. The three additions observe old-input reuse,
replacement invocation and new-input reuse after clearing, with the holder's absence
checked again afterward. The independent arithmetic oracle, source contracts,
mutation guards, registration and exact hashes are current. Both
`runtime/results/2026-10-09T19-18-13-740Z-verify.json` and the final formatted-source
`runtime/results/2026-10-09T19-21-50-194Z-verify.json` verify five cases on native and
Node-after-tsc without timing samples. They are imported, byte-exact read back and
removed from loose storage. Source/compiler inputs and generated text artifacts
are in content-addressed `runtime/source-evidence/nullable-callback-replacement/`
records. Whole-project formatting changes one workload, then zero files. Formatted
registration and native behavior pass.

## Next probes and remaining work

Next test repeated replacement with two independent factory instances that share
the same source closure origin. Also test direct array/record input replacement,
where capture types match but caller owners differ. Current concrete callback-type
selection must not be mistaken for a proof that all equal-type owner instances have
independent native lifetimes. Preserve old-input reuse while the holder stays live;
do not remove that assertion to make the witness pass.

Then finish retained stores across branch joins, loops, early exits and disposal,
recursive retained-store summaries and nested generic callback environments. Keep
source diagnostics for invalid escape/conflicting access. Do not remove default or
exception guards without completion/provenance transport. B7 family acceptance,
complete regressions and matched timing still remain; this checkpoint does not close
B7 or the compiler plan.

Fifteen unrelated fixture edits remain unstaged and semantically untouched. Use
explicit Git paths. The five installed consumers remain at B5 until family
acceptance. Reuse the warm Cargo target, disable incremental compilation, put Node
24.11.1 first on PATH and use Homebrew Python for SQLite. Five closed workload files
totaling 793,646 bytes were removed after evidence preservation. No cache refill.

## Continuation prompt

Continue Tsrust from this handoff. Read AGENTS.md, docs/development-workflow.md and
docs/plans/caller-borrowed-record-stores-plan.md first. B7/gate 183 remains open. Probe
same-origin independent callback factories and direct data-input replacement with
old-input reuse while a holder remains live. Use focused red/native assertions,
editor checks, matched workload support and SQLite evidence. Preserve the fifteen
unrelated fixtures. D2 discriminated unions stay separate. Continue toward the full
compiler plan; do not treat this callback checkpoint as completion.
