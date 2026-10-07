# Scoped record layout implementation in progress

Read AGENTS.md, docs/development-workflow.md and docs/scoped-callable-aggregate-plan.md
first. The preceding accepted aggregate escape safety implementation is f1dd284,
with consumer refresh recorded at 9f56d4b. This checkpoint implements concrete local
scoped record layouts but does not accept the complete record gate or Gate173.
Preserve the same 15 owner fixtures and use explicit Git paths. No owner question
is pending. Keep the existing public callback variants and owning storage contracts.

## Current implementation

scoped_record_layout::Registry caches native record shapes by nominal declaration
and concrete field types. It clones declaration metadata into distinct instances;
it never stores runtime capture values or mutates the shared declaration. Different
concrete callable families coexist under one source record name. Source compatibility
recognizes the instance's original declaration without adding structural subtyping.

record_layout admits scoped field protocol declarations through scoped_record_fields.
record_literal checks concrete field values and materializes instance metadata. A
nullable supplied handle is analyzed before signature compatibility checking so its
concrete environment payload is preserved. Direct arrows retain contextual parameter
inference. Projection invokes concrete closure or borrowed-family protocols.

The emitter makes callable record fields generic over their concrete native payloads.
The environment types carry actual capture lifetimes. Callable size validation excludes
declaration-only signature placeholders from schema budgets and includes concrete
borrowed families in instantiated budgets. An older size test assumed a non-retaining
Once parameter creates owning storage; its witness now explicitly retains the value.
The original failed evidence is preserved.

## Focused proof and remaining acceptance

Eleven scoped_callable_records tests now have passing evidence: readonly repeated invocation, mutable
original capture updates and scope release, separate instances with record/field
moves, nullable present invocation, contextual arrows, absent/undefined/omitted
optional fields, nested record projection, record/nullable-family captures, empty
owned factories and escape/conflict controls. Nine
positive source cases now explicitly pass after TypeScript compilation with Node
assertions. The reference extractor accepts multiline Rust calls and trailing commas;
its expected case count prevents silently skipping formatted fixtures.

Correction: the previous checkpoint's claim of five passing Node source counterparts
was incorrect. Its extractor missed formatted Rust calls and those later reference
runs failed. Use the current nine-case proof; original failed logs remain in SQLite.
An unrelated helper formatting change from recursive rustfmt traversal is removed.
Use skip_children when formatting Rust test files so support modules stay untouched.

Record layout unit checks and the repaired callable size budget
check pass. The final canary/traversal batch passes 70 tests after field-call dispatch
changes. The existing editor field hover/signature check passes. Whole-project
formatting changes zero files or fingerprints. These checks are implementation evidence, not full record acceptance.

Complete the following before accepting the record gate:

- Extend nullable and nested coverage to failure, mutable captures, conditional
  absence and nested transfers, including conflicting owner access while live.
- Extend record-capture coverage to mutable environments, data union/view captures,
  recursive layouts and arrays. Native environments must use named lifetimes and
  preserve generic callback bounds, not unresolved signature placeholders.
- Check record parameter, result and retaining-transfer boundaries. Do not allow an unresolved
  native signature parameter to surface as a generated Rust error.
- Track field loan provenance through inner-scope transfers, partial field moves,
  assignment, disposal, exceptions and consuming handles. Current direct conflict
  checks and scope release are insufficient to claim all aggregate lifetimes.
- Extend editor source tests and add executable guide examples, matched workload observations,
  oracle and source-contract guards, registrations and source fingerprints. Complete
  whole-editor verification and milestone/coverage/roadmap records before acceptance.

The broader array, aggregate composition, polymorphic replacement and point-sensitive
mutable environment work remain open. Anonymous record inference is a separate
dependency; explicitly owning Function/Once fields must not silently admit borrowed
environments. No benchmark timing runs or measurement-status changes are authorized
by this implementation checkpoint.

Use the shared target /private/tmp/tsrust-gate100-editor-target with CARGO_INCREMENTAL=0,
and pinned Node /Users/seancotter/.nvm/versions/node/v24.11.1/bin/node. The warm compiler
is the work-in-progress build. Installed and editor consumers remain on accepted
f1dd284; do not refresh them from this incomplete checkpoint. Store closed verified
logs in SQLite under runtime/diagnostics/scoped-record-layout/.

## Nullable absence and nested projection implementation

scoped_record_construction preserves an absent field's declared signature using the
existing uninhabited callback payload. It first validates the written null/undefined
alternative. Optional calls retain parameter help and skip argument evaluation. The
marker owns no environment and is not classified as a scoped capture. Callable size
validation accounts for its zero-byte payload and nullable tag.

Nested concrete record fields now propagate instance metadata while retaining their
nominal declarations. scoped_record_emit detects nested borrowed environments and
uses generic native slots so real lifetimes survive projection. The new editor test
checks both nested and absent calls against the declared amount parameter name and
rejects leaked BorrowedCallable text. It passes without changing the editor implementation.

The final canary/traversal batch passes 70 tests. Four aggregate fact tests and the
callable storage size control pass. Whole-project formatting remains required before
checkpoint. These are work-in-progress results; full editor verification, workload
support and the remaining lifetime/boundary checks are still owed before acceptance.
Exact current evidence is stored under runtime/diagnostics/scoped-record-continuation/.

## Captured record environment lifetime implementation

environment_lifetimes counts concrete by-value lifetime slots through record fields
and nullable alternatives with an iterative traversal. Repeated fields keep distinct
slots. Cycles, unresolved layouts and counts above 128 conservatively produce the
existing over-limit sentinel. Source closure checking includes nested closure and
borrowed-family lifetimes when deciding whether an environment borrows storage.
An owned result can no longer hide borrowed storage by capturing its enclosing record.

callable_layout separates generic capture layout from named environment rendering.
scoped_record_emit renders concrete records, callable nullable payloads and borrowed
families using the exact named slots. Existing callback bounds remain on concrete
closure arguments. Ordinary data capture behavior is retained. Full nullable data
record/view and array composition still needs its own proof; callable payload support
does not establish general aggregate-union support.

The original nested-record capture failed with a generated Rust missing-lifetime
error. Its repaired source/native witness passes. The owned escape control now
reports the existing closure-escape diagnostic; an earlier assertion incorrectly
expected callback-escape and is repaired. Additional proof covers nullable borrowed
families, direct nullable callable capture and an owned empty factory, which must
not acquire a fabricated lifetime.

The compiler batch passes 80 tests, including all eight canaries, existing scoped
traversal and ten record tests. The additional family witness passes separately,
including its later direct nullable-capture extension. Nine extracted positive
cases pass Node after TypeScript compilation. Two lifetime-count tests, four capture
layout controls, the scoped editor signature test and five existing matched adapter
cases pass. No timing or measurement-status changes occur. Current exact evidence
lives under runtime/diagnostics/scoped-record-captures/ in SQLite.

Next implement record parameter/result and retaining-transfer boundaries, then field
loan provenance across inner scopes, partial moves, mutation and disposal. Do not
accept the complete record gate before the remaining guide, workload and whole-editor
checks. The warm compiler is work in progress; installed/editor consumers remain
on the preceding accepted commit. Preserve all 15 owner fixtures.

## Record parameter boundary red witness

After a62d8e7, scoped_record_parameter_invokes_without_retaining_owner adds a
ReadOnly<Holder> parameter that invokes its scoped field twice and releases the
owner loan before mutation. The first draft used the retired read keyword; that
syntax was corrected, and the original failed log remains preserved.

The current witness reaches native generation. callback_bounds now walks nominal
record fields with a cycle guard and supplies the missing C0: ReadCallback bound.
This removes the original E0425. The witness still fails E0308: its concrete
Record1<Closure0> instance cannot pass to the declared Record0<C0> parameter.
Do not classify this partial repair as accepted. Next unify native record family
identity while keeping each instance's concrete payload slots, then exercise
nested parameters and retention rejection. The Node extractor case count will
need updating from nine to ten when this positive witness is accepted. Source
changes remain uncommitted, and executable consumers remain on accepted f1dd284.

## Native record parameter identity repair

The parameter witness now passes. CheckedProgram retains the scoped record registry.
Native record type rendering, named capture environments and record literals use
the nominal declaration's identity while preserving concrete field payload slots.
Concrete instances no longer emit redundant separate Rust record declarations.
Record callback bounds traverse fields with a cycle guard. This repairs both the
missing callback bound and mismatched nominal native record type.

All twelve scoped record tests pass, including source rejection controls and native
lifetime witnesses. All eight mandatory canaries pass (64 tests). Ten extracted
positive source cases pass Node after TypeScript compilation. Whole-project
formatting changes zero files and zero fingerprints. Exact logs are stored in
SQLite under runtime/diagnostics/scoped-record-parameters/. This is a partial source
checkpoint, not complete record gate acceptance. Next cover nested record parameters,
retaining transfers, callable result boundaries and field loan provenance. Complete
editor, guide and matched workload support remains owed. Installed consumers stay
on f1dd284. Preserve all fifteen owner fixtures.

## Nested parameter boundary witnesses

The nested ReadOnly<Envelope> parameter witness passes native execution without
further implementation changes. Its Holder callback borrows an array, receives
two arguments on successive calls and releases the loan before the owner mutates.
A separate source control rejects returning a consumed Envelope parameter retaining
scoped callback storage with callback-escape. These two focused checks pass; the
previous twelve-test batch is preserved rather than repeated. Eleven extracted
positive cases pass Node after TypeScript compilation. Whole-project formatting
changes zero files and fingerprints. Exact evidence is stored under
runtime/diagnostics/scoped-record-boundaries/ in SQLite.

Remaining work includes positive owned record results, scoped result transfers when
lifetimes permit, field provenance through scope exits and assignment, disposal and
failure paths, editor/guide and matched workload support. These witnesses do not
establish complete record gate acceptance. Source consumers remain on f1dd284.

## Owned concrete record results

A direct owned factory witness first failed native generation: its declared record
result contained an unresolved C0 generic instead of the returned concrete closure.
Named function checking now preserves the checked concrete record result from its
return expressions after validating the nominal annotation. Multiple returns still
must resolve compatibly; mixed environment result families remain pending.

A nested factory plus forwarding witness exposed a second defect: generic native
field positions disappeared when an instance's nested closure owned its captures.
scoped_record_native_parameter now selects record field positions from the nominal
declaration while concrete payload rendering retains each instance's environments.
Both factories preserve separate offsets across repeated invocation. No implicit
clone or borrowed escape relaxation is introduced.

All sixteen record tests pass after repair. Seven canaries passed in the original
batch; type_syntax was not reached after the nested record failure and passes in
the repair batch (64 canary tests total). The focused editor signature check passes
on final source. Thirteen positive cases pass Node after TypeScript compilation.
Final whole-project formatting changes zero files and fingerprints. Original failed
and repaired evidence is preserved in SQLite under
runtime/diagnostics/scoped-record-owned-results/. This is not gate acceptance.
Remaining work includes mixed record result environments, permitted scoped transfers,
field loans, assignment, disposal/failure paths, general composition, guide and
matched workload support, and complete editor acceptance. Consumers remain f1dd284.

## Independent field loan release: current red checkpoint

scoped_record_partial_field_moves_release_independent_capture_loans is a new failing
source/native acceptance witness. Pair contains two independently borrowed callbacks.
The left callback moves into an inner binding, is invoked, then leaves scope; the
right callback remains in the partially moved record. Mutating the left owner must
then succeed while invoking the right callback remains valid. Current checking
reports capture-conflict at leftValues[0] = 5 because capture_loans still stores the
original outer-scope creation entry. The witness reaches analysis, not native code.
Fourteen extracted cases pass the independent Node reference. Project formatting
changes zero files and fingerprints. Exact red/reference/format logs are stored in
SQLite under runtime/diagnostics/scoped-record-field-loans/. This checkpoint is
intentionally red and not accepted; installed consumers remain f1dd284.

Implement loan provenance in a dedicated module rather than adding unrelated state
to recursive dispatch. Give each retained capture loan an identity and track its
current owning binding and record field path. Record construction attaches field
payload loans; whole-record moves transfer every live field; partial field moves
transfer only that field's loans to the destination binding. Scope exit releases
loans owned by dying bindings while retaining loans transferred to outer storage.
Preserve existing temporary call and traversal loan handling. Reassignment must
release only overwritten environments after validating evaluation order. Branch
joins and exception snapshots must retain a loan whenever any reaching live owner
still holds it. Recursive/nested records require bounded traversal.

Do not release based only on a field's moved flag: the moved callback may remain
live in another binding. Do not truncate all record loans: the right field still
borrows its owner. Add rejection controls for mutation while the extracted left
callback lives, mutation of the right owner while pair.right lives, and escape of
an inner owner's callback into outer record storage. Add nested record transfers,
whole-record moves, overwrite, failure/finally and disposal controls before accepting
field provenance. Resume the existing red witness first, then bounded affected
checks and mandatory canaries after dispatch changes.

## Loan ownership module implementation

capture_loan_owners now provides an isolated Owners state keyed by binding and field
path. A loan identity may have multiple live owners. transfer extracts a subtree,
rebases every nested path, replaces the destination subtree and reports only loans
with no remaining owner. Self-transfer is inert. Overlapping ancestor/descendant
transfers require a checked temporary. leave_scope removes dying bindings and
reports only newly dead loans. join unions reaching ownership paths conservatively.
Four focused tests were red before implementation and now pass: independent field
disposal, nested transfer to outer storage, replacement retaining shared loans and
branch joins. The traversal uses explicit maps and vectors rather than recursion.

This module is not wired into Context or ExitState yet. The source/native field-loan
witness remains red. Do not claim repaired compiler behavior or refresh consumers.
Next use stable loan identities shared across branch snapshots; attach loans to
concrete callback payload paths during binding/record construction, transfer them
with whole/partial moves, and release only reported dead identities at scope exit.
Existing untracked temporary loans must remain conservative until explicitly owned.
Loan identities cannot be ordinary vector positions if removals shift that vector,
and separately forked branches cannot allocate colliding identities. Carry Owners
through exception ExitState capture/restore/join and branch state restoration before
allowing releases. Avoid releasing loans for moved values that remain live elsewhere.
Exact module red/green/format evidence is stored in SQLite under
runtime/diagnostics/capture-loan-owners/. Preserve all fifteen owner fixtures.

## Exception snapshot provenance integration

Context now carries capture_loan_owners. ExitState captures and restores this state,
and its joins union every reaching loan owner. A focused snapshot test first failed
because restoration discarded owners; it now verifies two independently captured
branches, their joined owner set and subsequent independent scope disposal. All 85
exception_state tests pass. No capture loan release behavior is enabled yet.

Project_outer and unwind_capture_loans intentionally retain conservative provenance
for now: payload ownership must be identified before pruning paths or loan identities.
Ordinary branch/conditional/loop snapshots still need corresponding state handling.
Do not attach and release production loans before those paths are covered. Stable
loan identity allocation, record construction paths, declaration/assignment transfers
and final scope-release wiring remain pending. The native field-loan witness remains
red. Exact red/green/format logs are stored under
runtime/diagnostics/capture-owner-snapshots/ in SQLite. Consumers remain f1dd284;
all fifteen owner fixtures remain protected.

## Ordinary branch provenance integration

Owners::reaching combines only normal-reaching arms; its new test verifies both
normal arms, either single normal arm and two abrupt arms. All five ownership module
tests pass. if_statement snapshots entry owners, records the then owners, restores
entry ownership before checking else and selects only non-abrupt owners afterward.
ConditionalPlan retains entry and left ownership states, resets before the alternate
and joins both normal expression alternatives. Value-returning logical and coalescing
expressions conservatively retain owners from skipped/present paths.

No production ownership release is enabled. LoopFrame still needs ownership state
for zero-body exits, break destinations, continue destinations and back-edge/tail
joins. Exception projection and payload unwind remain conservative. Stable loan
identities and payload attachments/transfers remain pending before the field-loan
witness can turn green. Keep the full safety controls in the field-loan section.
Current branch unit/canary/format evidence is stored in SQLite under
runtime/diagnostics/capture-owner-branches/. Consumers remain on f1dd284.

## Loop and labeled destination provenance

LoopFrame now stores independent break_owners and continue_owners. The common
control destination commit joins only the relevant destination; cleaned exception
control exits inherit this through ExitState restore. A red destination test now
passes and proves break and continue owner sets do not contaminate each other.
Labeled blocks select normal fallthrough and break owners only.

Legacy loop checking conservatively retains entry owners plus reaching fallthrough,
continue and break paths. Tail loops track zero-body exits separately, join normal
body and continue owners before the update, add condition exits after the update,
and join break owners at the final exit. An unreachable update checks an isolated
Context clone. A focused post-test loop witness verifies break, continue and return:
break/continue preserve normal-reaching owners, while return yields no normal owner
state. Production loan attachments and releases remain disabled.

The original 86 exception-state tests pass, and the added post-test loop witness
passes separately. Seventeen loop/labeled source and native tests pass. All eight
mandatory canaries pass (64 tests). Whole-project formatting changes zero files and
fingerprints. Exact failed and successful logs are preserved under
runtime/diagnostics/capture-owner-loops/ in SQLite. Next implement stable loan
identities, attach concrete field environments, transfer on moves and wire scope
release, with the existing safety controls. The record field-loan witness remains
red; gate acceptance and consumer refresh remain pending. Preserve fifteen owner
fixtures and installed consumers at f1dd284.

## Stable loan identities and declaration payload attachment

capture_loan_identity interns (captured binding, access mode, creation span) into
stable IDs. Context clones share the interner, so independently checked branches
cannot allocate colliding IDs. Removing or reordering the legacy loan vector does
not change an identity. Two focused tests were red before implementation and pass.
The registry does not remove keys while its checker context remains alive.

capture_loan_binding now attaches declaration payloads using an iterative worklist.
Record fields retain their individual paths. Checked direct-place moves transfer
source paths; readonly declarations share them. Closure payloads attach their own
creation loans and retain inherited capture environments, consuming those paths
only for consuming capture storage. Callable wrappers and conditional alternatives
preserve payload paths. Owners::extend_from merges alternative payload owners
without overwriting previously attached alternatives. Empty payloads create no owner
entries. A focused attachment test was red before implementation and now verifies
that a readonly alias does not dispose the original field, while a later field move
can dispose its independent loan. Existing capture-conflict checking is unchanged.

The final focused loan batch passes ten unit tests. Seventeen record tests pass,
including new rejection controls for both the moved left callback and remaining
right callback while their captures are live. The known scope-release witness is
explicitly excluded from that positive batch and rerun separately: it still fails
capture-conflict at the left owner mutation. All eight depth canaries pass (64 tests).
Project formatting changes zero files and fingerprints. Original red and final
proof are preserved under runtime/diagnostics/capture-loan-identities/ and
runtime/diagnostics/capture-loan-bindings/ in SQLite. This is a partial checkpoint,
not complete field-loan behavior or record gate acceptance.

Next wire assignment, destructuring and other retained payload moves before enabling
release. Unknown or opaque consumption must remain conservatively retained; it
cannot lose a loan merely because its original declaration binding leaves scope.
Calls, indexed aggregates and captured callee effects need explicit ownership or a
conservative pin. Replace lexical/exception loan-vector prefix truncation with stable
entry identities plus checked surviving payload owners before removing old entries;
otherwise earlier removals shift the saved vector positions. Preserve pending return,
throw, break and continue payloads through unwind and projection. Only then remove
loans whose tracked owners all died and whose retention is fully known. Resume the
current source/native scope-release witness with all live-owner rejection controls.
Installed consumers remain f1dd284; preserve fifteen owner fixtures.

## Assignment payload owner integration

A checker-state witness first failed because assigning a temporary back into an
outer Holder left its payload owners on the dying temporary binding. Successful
named and direct record field assignment now stage RHS payload owners before
replacing destination ownership. The staging path is internal and removed before
the store completes. This preserves whole-record self-assignment and field moves
back into the same record without discarding the source environment. Assignment
validation and evaluation still complete before ownership commit. Unknown indexed
storage is not mapped to a falsely precise record path.

The checker-state test covers whole-record restoration, field restoration and
self-assignment. It proves disposal of the inner temporary retains the outer
payload's loan identities. A new native std:testing witness exercises all three
operations and releases the original array only after the outer record leaves scope.
The focused loan batch passes eleven unit tests. The positive record batch passes
eighteen tests, explicitly excluding the unchanged red scope-release witness.
All eight mandatory canaries pass (64 tests). Fifteen extracted positive cases pass
Node after TypeScript compilation. Project formatting changes zero files and
fingerprints. Exact red and successful proof is stored under
runtime/diagnostics/capture-loan-assignment/ in SQLite.

No capture-conflict check is weakened, and actual loan release remains disabled.
Before release, cover destructuring and opaque retained consumption with explicit
ownership or conservative pins. Replace lexical and exception vector-position
unwind with stable entry identities plus surviving payload owners. Then turn the
independent field-disposal source/native witness green with the live-field controls,
branch/loop/finally coverage and pending payload transfers. Complete record gate
acceptance still needs its full guide, editor and matched workload support.
Consumers remain f1dd284; all fifteen owner fixtures remain protected.

## Destructuring ownership proof

Named record destructuring already lowers selected fields into checked declaration
moves. A new checker-state test verifies that disposing an extracted left callback
releases only its ownership paths, while the unselected right callback remains
with the original record. No new production implementation was needed for this
case. Three declaration/assignment ownership tests now format their source first
and require idempotence before parsing and checking their state.

A new std:testing native witness covers nested record patterns, remaining source
fields and a destructured ReadOnly<Holder> parameter invoking its callback. It
passes. A focused editor test verifies signature parameter names for both the
readonly parameter callback and the extracted nested callback, plus readable hover
without BorrowedCallable leakage. It passes. Sixteen extracted positive cases pass
Node after TypeScript compilation. Whole-project formatting changes zero files
and fingerprints. Exact proof is stored under
runtime/diagnostics/capture-loan-destructuring/ in SQLite.

This proves the named/nested record cases, not every pattern or retained copy.
Tuple/indexed storage, callback default joins, rest/copy behavior and opaque calls
still require explicit provenance or conservative retention before production loan
release. Current cloneability rejects ordinary scoped callback environments; keep
that contract and audit shared callable handles separately. Do not assume all
retaining operations consume their source: supported copies must also preserve
loan owners. Next implement unresolved-retention tracking, then stable lexical and
exception unwind. The independent field-scope release witness remains red. No new
parser/analyzer dispatch changed in this proof-only checkpoint, so the preceding
64-canary evidence is preserved. Consumers remain f1dd284. Preserve fifteen owner
fixtures; complete record gate acceptance remains open.

## Unresolved consumption ledger foundation

Three failing unit witnesses exposed unresolved retention disappearing on scope
disposal, interference between distinct consumption sites, and loss across a branch
join. The owner ledger now retains pending loan IDs by source place and consumption
span. Resolving a known move removes only its own IDs at that site. Scope disposal
preserves unresolved retention, and branch joins merge pending events. All eight
owner-ledger unit tests pass. Exact red and green logs are imported and verified
under runtime/diagnostics/capture-loan-retention/ in SQLite.

This foundation is not wired into checker consumption yet. Next connect successful
consuming access and acknowledge known declaration, assignment and capture moves
using the exact originating span. Then replace positional lexical/exception unwind
and prove branch restoration before enabling actual loan release. The independent
field-disposal native witness remains red. No public wrapper changes are needed:
ReadOnly, Mutable and Once describe invocation; retention is compiler analysis.
This work remains uncommitted. Consumers remain f1dd284 and owner fixtures remain
protected.

## Checker consumption integration

Successful consuming access now records unresolved retention for the exact binding
or selected record path and access span. Checked declaration and assignment moves
resolve only the IDs moved at their originating site. Consumed closure captures
resolve their corresponding creation-site event after environment transfer.
Transparent checked wrappers use the underlying source span. The three binder
state tests pass, including destructuring and assignment restoration. All eight
mandatory depth canaries pass (64 tests). Project formatting changes zero files
and fingerprints. Exact proof is verified in SQLite under
runtime/diagnostics/capture-loan-retention/.

Actual scope release remains disabled. Next add checker-state opaque-consumer and
wrapped-source controls, then replace positional unwind and prove branch loan
restoration before enabling release. Independent field disposal remains red; the
record gate and gate 173 remain open. Consumers remain f1dd284.

## Opaque consumption control and reverse identities

A checker-state control now proves that successful opaque consuming access retains
its loans after the original binding is disposed. Its paired known move through a
checked wrapper proves that the underlying access span resolves that event and
destination disposal releases the ledger IDs. This is a checker-state proof, not
a native scope-release claim. The test source formats and round-trips before analysis.

A separate red witness required exact loan recovery from a stable ID shared across
cloned branches. Registry now keeps both the tuple-to-ID map and ID-to-tuple vector.
Repeated interning preserves identity; IDs from either branch recover the original
mode and span without relying on raw vector positions. All seventeen focused loan
tests pass. Project formatting changes zero files and fingerprints. Exact logs,
including the corrected test-construction compile error, are verified in SQLite
under runtime/diagnostics/capture-loan-restoration/.

Next use reverse identities to restore live raw loans after owner-state restoration,
and replace lexical/exception positional unwind with stable entry membership. Add
branch, exceptional and pending-payload controls before enabling field release.
The independent native field-disposal witness remains red. No parser/analyzer
dispatch changed in this proof and registry checkpoint; preceding 64 depth-canary
evidence remains applicable. Consumers remain f1dd284. Preserve fifteen owner
fixtures. Record gate and gate 173 remain open.

Continuation prompt: Continue from this handoff and the scoped callable aggregate
plan. HEAD before this checkpoint is fcb7bc7. First inspect Git status. Keep three
public wrappers and ownership rules. Implement stable lexical/exception loan
restoration and release with red witnesses, bounded checks, formatting, SQLite
evidence and explicit Git paths. Never stage the fifteen owner fixtures.

## Stable lexical entry membership

A new red witness demonstrates why a saved loan-vector length is unsafe: removing
an earlier entry and adding a local loan leaves the local within the old prefix.
Boundary now captures exact entry loan tuples and unwinds by membership. Lexical
block cleanup uses this boundary instead of truncating by the saved vector length.
The boundary does not resurrect a removed entry or release tracked environment
owners; actual owner-based release remains disabled. Eighteen focused loan tests
and all eight depth canaries (64 tests) pass. Project formatting changes zero files
and fingerprints. Exact red, green and canary logs are verified in SQLite under
runtime/diagnostics/capture-loan-boundaries/.

Exception region boundaries still use counts and must convert next. Conditional
arm loan slicing also still uses positions. Convert both before enabling removal,
then restore live loan tuples from stable IDs whenever owner snapshots restore.
Preserve pending payloads and unknown retention. The independent native field
disposal remains red. This is a partial source checkpoint, not record acceptance.
Consumers remain f1dd284. Preserve fifteen owner fixtures.

Next-agent prompt: Continue from this handoff. Read AGENTS.md, development workflow
and scoped callable aggregate plan. Convert exception and conditional loan
boundaries to stable membership with red tests; verify branch restoration before
enabling scoped field release. Keep three public wrappers. Use the warm target,
bounded checks, whole-project formatting, SQLite evidence and explicit Git paths.

## Stable exception region membership

A red exception-state witness shows an old entry-count boundary retaining a new
local loan after an earlier entry disappears, even with an explicit pending payload.
ExitState unwind now accepts stable entry membership and separately preserves
unique payload loans. Context stores region boundaries in shared immutable snapshots.
Nested region entry and diagnostic restoration tests compare the exact membership
instead of a count. Break, continue, error-point and capture-free completion unwind
use these snapshots. All 88 exception-state tests, three native-region tests and
eight depth canaries (64 tests) pass. Project formatting changes zero files and
fingerprints. Exact evidence is verified in SQLite under
runtime/diagnostics/capture-loan-exception-boundaries/.

Audit found more positional temporary loan cleanup than the conditional plan alone:
analysis.rs, generic_expression_adapters.rs, deferred_generic_values.rs,
optional_chain.rs and spread_calls.rs still truncate by saved capture-loan lengths.
Convert or prove each boundary safe before enabling removal. Conditional branches
also slice new loans by the old index and require exact entry restoration plus
branch delta membership. Then use reverse identities to reconcile owner snapshots
with raw loans, preserve surviving outer payloads through unwind, and turn the
independent field-disposal source/native witness green with live-owner controls.
Actual owner-based release stays disabled. Record gate and gate 173 remain open.
Consumers remain f1dd284; preserve fifteen owner fixtures.

Next-agent prompt: Continue from this handoff and inspect Git status first. Convert
remaining positional capture-loan boundaries with red tests, then integrate stable
raw-loan restoration and scoped disposal. Keep existing ownership rules and three
public wrappers. Run bounded checks, project formatting and SQLite evidence; use
explicit Git paths and never stage owner fixtures.

## Stable conditional loan snapshots

A checker-state red witness removes an entry loan during the consequent and adds a
new loan at the old vector position. The previous conditional code missed the new
loan and began alternate analysis with a corrupted entry set. ConditionalPlan now
stores the exact entry vector. Between arms, a hash membership boundary identifies
new consequent loans and restores the original entry vector in its original order.
The existing join merges the consequent's new loans after alternate analysis.
Nineteen focused loan tests and all eight depth canaries (64 tests) pass. Project
formatting changes zero files and fingerprints. Exact red and successful evidence
is verified in SQLite under runtime/diagnostics/capture-loan-conditional-boundaries/.

Remaining production count snapshots in analysis.rs use first_capture or
first_capture_loan only for temporary capture-loan truncation. The four helper
files generic_expression_adapters.rs, deferred_generic_values.rs, spread_calls.rs
and optional_chain.rs have the same pattern. Convert these as a coherent batch
to stable Boundary unwind, preserving each existing result-retention condition.
Do not alter iteration-loan indices or test-only vector resets. Audit pending owner
metadata separately: replacing positional boundaries does not itself permit
owner-based loan release. Next reconcile raw loans with restored live IDs and
preserve surviving outer payloads through unwind before enabling field disposal.
Record gate and gate 173 remain open; consumers remain f1dd284. Preserve fifteen
owner fixtures.

Continuation prompt: Read this handoff, inspect status, and convert the remaining
temporary capture-loan boundaries. Use focused red controls and bounded affected
checks, then continue stable raw-loan reconciliation and scoped field disposal.
Keep the three public wrappers, ownership rules, warm target, project formatting,
SQLite evidence and explicit Git paths. Never stage protected owner fixtures.

## Stable temporary call boundaries

The seventeen remaining production temporary loan-count snapshots now use stable
Boundary membership. This covers thirteen analysis.rs sites plus generic expression
adapters, deferred generic values, spread calls and optional chain bodies. Each
existing result-retention condition remains intact. The retained red membership
witness demonstrates the shared positional defect; this batch applies its tested
replacement to all audited callers. Production capture-loan slicing and truncation
are absent after the audit. The two remaining test-only resets are unchanged.

Nineteen focused loan tests, four source/native witnesses and all eight depth
canaries (64 tests) pass. Source witnesses cover borrowed/owned generic factory
state, callable spread/defaults, optional owned views and nullable scoped callback
transport. Project formatting changes zero files and fingerprints. Exact evidence
is verified in SQLite under runtime/diagnostics/capture-loan-temporary-boundaries/.

Next reconcile raw loan tuples with live owner IDs whenever owner snapshots restore
or join. Track which IDs are known so untracked temporary borrows retain their
existing conservative semantics. Then integrate owner disposal into lexical and
exception unwind while preserving outer storage and pending payloads. Unknown
consumption remains pinned. Only enable release after branch, loop and exception
controls demonstrate that live callbacks still block incompatible access. The
independent field-disposal native witness remains red. Record gate and gate 173
remain open. Consumers remain f1dd284; preserve fifteen owner fixtures.

Continuation prompt: Read this handoff and inspect status. Implement raw-loan
reconciliation from stable IDs with red restoration controls, then scoped field
disposal and native live-owner controls. Keep three public wrappers and existing
ownership rules. Use bounded checks, project formatting, SQLite evidence and
explicit Git paths. Never stage protected owner fixtures.
