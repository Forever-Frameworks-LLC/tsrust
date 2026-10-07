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

## Lexical live-loan restoration and completed Once invocation

A red checker-state witness showed owner IDs surviving while raw loan tuples were
missing. A dedicated non-inlined helper restores live tuples through Registry,
keeps unknown raw borrows and avoids duplicates. Lexical block cleanup now disposes
local owner paths before restoring loans carried by surviving owners or pending
consumption. This restores surviving payloads without reviving dead local owners.
The helper is not yet wired into every branch and exception snapshot restoration.
Old tracked loan removal remains disabled.

The first canary batch exposed a real regression: successful Once invocation was
still pinned as opaque consumption, reviving a completed callback's exclusive
capture after its lexical scope. A completed checked callable invocation now
resolves its own receiver event when its result cannot retain captures. Direct
named callees resolve their binding through Context because their syntax node is
not necessarily annotated as an evaluated value. Opaque argument transfers and
invocations with retaining results remain conservative. The original mixed-capture
Once native witness passes after the repair.

Final checks pass twenty loan unit tests, nineteen record source/native tests
(excluding the deliberately red independent-field release case), and all eight
depth canaries (64 tests). Independent field release was rerun separately and
still fails capture-conflict. Project formatting changes zero files and fingerprints.
All original failed and successful logs are verified in SQLite under
runtime/diagnostics/capture-loan-reconciliation/.

Next add tracked-ID history distinct from live ownership so removal never touches
unknown borrows. Integrate stable raw-loan reconciliation at branch, loop, logical
and exception owner-state restoration/join, with restoration and live-owner
controls. Exception projection/unwind must dispose dead local paths while retaining
outer assignments and pending payloads. Then enable tracked loan release and turn
the independent native field witness green. Complete editor, guide, matched
workload and gate records before acceptance. Record gate and gate 173 remain open;
consumers remain f1dd284. Preserve fifteen owner fixtures.

Continuation prompt: Inspect status and read this handoff, workflow and scoped
callable aggregate plan. Continue tracked-ID history and branch/exception raw-loan
restoration, then scoped field disposal. Keep three public wrappers and ownership
rules. Preserve red evidence and passed checks. Use bounded tests, project formatting,
SQLite evidence and explicit Git paths; never stage protected owner fixtures.

## Tracked history and snapshot restoration

A red ledger witness required disposed and branch-only IDs to remain known even
after their last owner disappears. Owners now keeps tracked history separately
from live paths and pending consumption, and joins merge that history. Attach
updates both sets without constructing a temporary set. This supplies provenance
for later removal; it does not yet remove dead raw loans.

A separate red exception witness required an owner snapshot to recover a missing
raw loan. Live restoration now runs after exception state restore, conditional
entry reset/join, if entry reset/join, loop continuation/exit joins, labeled block
join and logical/coalescing joins. Original unknown raw borrows remain conservative.
Tests exposed local owner paths surviving exceptional unwind and reviving discarded
callbacks during cleanup. Unwind now disposes local paths using the exit state's
outer binding prefix while preserving explicit payload loan tuples. A synthetic
loop owner test now binds its intended outer binding before constructing metadata;
it no longer pretends binding zero exists in an empty Context.

Twenty-one loan tests pass. The initial exception batch had two failures; the
repaired batch passes all 89 exception-state tests. Nineteen positive record tests
and all eight depth canaries (64 tests) pass on snapshot integration. Five source/
native finally checks supplement the unwind repair. Project formatting changes
zero files and fingerprints. All original red/failing and successful logs are
verified in SQLite under runtime/diagnostics/capture-loan-history/. No timing.

Next enable removal of tracked IDs with no live owner or unresolved retention,
leaving untracked borrows untouched. Before removing explicit pending payload
loans, give those payloads surviving ownership or an explicit conservative pin.
Test branch restoration before in-arm access, loop continuation/break, exceptional
outer assignments and finally disposal. Resume the independent field-disposal
native witness and its live-field rejection controls. Complete editor/guide,
matched workload and gate records before acceptance. Record gate and gate 173
remain open. Consumers remain f1dd284; preserve fifteen owner fixtures.

Continuation prompt: Inspect status and read this handoff plus the scoped callable
aggregate plan. Add dead tracked-loan reconciliation with unknown/pending-payload
controls, then turn independent field disposal green and check branch/loop/finally
live-owner safety. Keep three public wrappers and existing ownership rules. Use
bounded checks, project formatting, SQLite evidence and explicit Git paths. Never
stage protected owner fixtures.

## Dead tracked loan removal and explicit payload retention

A new red reconciliation control requires dead tracked loans to disappear while
live loans, unknown borrows (including interned but untracked tuples) and opaque
retention survive. Live-loan reconciliation now filters only IDs with tracked
provenance and no live owner or pending retention, then restores missing live
tuples. The independent field-disposal witness now passes analyzer checking.

A separate red exceptional payload control showed that local owner disposal must
not erase a pending payload loan. ExitState now carries the shared registry and
records explicit payload retention before disposing local paths. Payload retention
is replaced by each unwind's identified payload set, so a subsequent explicit
discard releases it. Pinning does not turn previously unknown raw borrows into
tracked ownership; an existing boundary/payload unit test exposed and verified
that distinction. Payload retention joins with reaching states and survives
ordinary owner disposal.

Final proof passes 23 loan unit tests, 90 exception-state tests, all eight depth
canaries (64 tests), and five source/native finally checks. The full record batch
passes nineteen tests and fails one. The independent field-disposal failure is now
Rust E0502, not Tsrust capture-conflict: emitted code retains a borrow of leftValues
while later reading pair.right after the left field's inner scope. This remains a
required backend repair, not accepted field-disposal behavior. The native diagnostic
and every original failed/green log are verified in SQLite under
runtime/diagnostics/capture-loan-disposal/. Project formatting changes zero files
and fingerprints. No timing; consumers remain f1dd284.

Next inspect the checked left-field carrier and emitted initializer/storage access
for the field-disposal witness. Ensure consuming extraction moves its environment
without retaining a new reference to the source field, while ReadOnly parameter
borrows remain borrows. Do not add implicit clone, change wrapper semantics or
weaken the witness. Keep both live-field rejection controls. Then add native
branch/loop/finally disposal cases and complete editor, guide, matched workload and
gate records. Record gate and gate 173 remain open. Preserve fifteen owner fixtures.

Continuation prompt: Inspect Git status and read this handoff plus scoped callable
aggregate plan. Repair native independent field disposal: analysis now passes,
emitted Rust fails E0502. Diagnose concrete field carriers and storage emission
before changing ownership. Keep live-owner controls and three wrappers. Use the
warm target, bounded checks, project formatting, SQLite evidence and explicit Git
paths. Never stage protected owner fixtures.

## Native field storage diagnosis and implementation plan

The native failure is not an accidental borrow for left extraction. A diagnostic
probe through the current compiler library confirms emission already uses
`let value3: Closure0 = (value2).field0`. The remaining aggregate's later field1
use still keeps the left capture lifetime relevant to Rust checking. A generated
Rust experiment destructures Record0 into separate field bindings and routes field
selection to them. It compiles and runs with exact output `3 11` then `11`. The
probe executable was removed; original emission and native proof are imported and
verified under runtime/diagnostics/scoped-record-field-storage/ in SQLite.

Read docs/scoped-record-field-storage-plan.md for the concrete milestones and
invariants. Next implement an isolated native field-place layout and declaration/
selected access path, then complete whole-record transport, borrowing bridges,
assignment, captures and completion. Do not repair this through implicit clone,
unsafe lifetime erasure or weakening ownership checks. Keep nominal record ABI
and independent field lifetimes. Existing parameter and factory cases must remain
valid. No compiler source changed during this diagnostic checkpoint; preceding
23 loan, 90 exception and 64 canary evidence remains current. The independent
project/native witness still fails. Record gate and gate 173 remain open.

Continuation prompt: Inspect status; read this handoff, workflow and scoped record
field storage plan. Implement independent native field places with the original
red project witness, preserving whole-record and borrowed parameter behavior.
Use bounded compiler/native/editor checks, formatting and SQLite evidence. Keep
three public wrappers. Never stage the fifteen owner fixtures.

## Independent field-place layout foundation

New scoped_record_places.rs computes independent leaf paths from checked static
records in nominal declaration order, with stable native names based only on
binding IDs and numeric paths. It excludes borrowed parameters, captured references,
borrowed places/elements and non-scoped records. Nested records flatten iteratively;
unions remain opaque leaves. Missing metadata, cycles and paths beyond 128 reject
layout construction rather than guessing storage. Three focused tests pass, with
formatted/idempotent source checking. Exact red and successful evidence is verified
in SQLite under runtime/diagnostics/scoped-record-field-layout/.

The module is not yet wired into emission. Whole-record read/mutate bridges must
also support non-copy data fields: assembling a record of field references cannot
pretend an &String is an owned String. Existing ReadCallback forwarding for &F
handles callable fields, but data-field view layout and selection need explicit
support. Plan this alongside declaration and selected access integration so existing
borrowed parameter cases remain valid. Preserve nominal native ABI and ownership.
No recursive checker/emitter dispatch changed; prior bounded evidence remains
current. Project formatting changes zero files and fingerprints. Native independent
field disposal remains red; record gate and gate 173 remain open. Consumers remain
f1dd284. Preserve fifteen owner fixtures.

Continuation prompt: Inspect status and read scoped-record-field-storage-plan.md.
Continue emitter integration of the field-place module with whole-record transport
and a borrowing bridge for callback and non-copy data fields. Use original native
red and live-owner controls. Keep nominal types, three wrappers and no implicit
clone. Follow bounded checks, formatting, SQLite evidence and explicit Git paths;
never stage protected owner fixtures.

## Field declaration and owning reconstruction rendering

The layout module now renders nested destructuring declarations and whole owning
record reconstruction. A constructor-symbol callback preserves the caller's
nominal declaration mapping; native names remain numeric binding/field paths.
A red native component witness required initializer evaluation exactly once,
selected payload disposal, field restoration, whole reconstruction and final
disposal. The payload has no Clone implementation. Generated native code now
compiles and passes all runtime assertions, with one drop before restoration and
one after reconstructed ownership ends. Four focused layout/render tests pass.
Source used to derive the layout formats and round-trips before checking. The
native driver reads source on stdin and removes its executable on success/failure.

Exact red and green evidence is verified in SQLite under
runtime/diagnostics/scoped-record-field-rendering/. Project formatting changes
zero files and fingerprints. This is component verification, not full emitter
integration: the original project/native independent-field witness remains red.
No shared compiler dispatch changed. Record gate and gate 173 remain open.
Consumers remain f1dd284; preserve fifteen owner fixtures.

Next implement whole-record borrowing views for non-copy data and callable fields,
then connect declaration/selected access/owning reconstruction to emitter storage
paths without regressing existing readonly parameters and factory results. Borrowed
data fields need explicit native selection support rather than pretending references
are owned values. Preserve signature ABI and ownership. Run original project red,
live-owner controls, parameter/assignment/destructuring/factory positives, and the
required depth canaries after dispatch integration.

Continuation prompt: Inspect status and read the scoped record field storage plan.
Continue whole-record view bridges and main emitter wiring using the tested layout
renderers. Preserve non-copy data borrowing, nominal record ABI, three wrappers
and no implicit clone. Use bounded checks, project formatting, SQLite evidence and
explicit Git paths; never stage protected owner fixtures.

## Shared and mutable view rendering foundation

The layout module renders shared and mutable views by borrowing independent leaf
places. A native component witness uses a non-copy String, checks pointer identity
for the shared view, mutates the original string and numeric fields through the
mutable view, and reconstructs an owning record afterward. The generic inspection
helper accepts the owned record and shared view through a test-local trait. All
five layout/rendering tests pass. Native test executables have unique names for
parallel test workers and are removed on success or failure.

Exact red and green evidence is verified in SQLite under
runtime/diagnostics/scoped-record-views/. Whole-project formatting changes zero
files and fingerprints. No shared compiler dispatch changed. This verifies view
construction only: production nominal view declarations, borrowing parameter ABI,
field selection and main emitter wiring remain pending. The original native
independent-field disposal witness remains red. Record acceptance and gate 173
remain open; consumers remain f1dd284. Preserve fifteen owner fixtures.

Keep ReadOnly, Mutable and Once as the public callable model. Capture storage,
lifetime and disposal are implementation obligations beneath those wrappers; no
fourth public variant has been justified by the current witnesses.

Continuation prompt: Continue Tsrust from this handoff. Read AGENTS.md,
docs/development-workflow.md and docs/scoped-record-field-storage-plan.md. Connect
the tested field layout and view renderers to nominal borrowing bridges and emitter
storage paths. Preserve non-copy data borrowing, owning moves, field restoration,
existing parameter/factory cases and both live-owner rejection controls. Use focused
red tests, bounded native/editor checks, required dispatch depth canaries, automatic
whole-project formatting, SQLite evidence and explicit Git paths. Never stage the
fifteen owner fixtures. Keep three public wrappers and no implicit clone.

## Generated view declarations and generic field propagation

The layout module now generates nested native view declarations instead of relying
on test-local handwritten declarations. It carries caller-provided native generic
parameter names from leaf types through parent views, preserves field order and
emits each nested declaration once. Shared views contain references; mutable views
contain mutable references. Constructor symbols remain supplied by the caller.
The traversal uses the already validated acyclic layout with its depth bound.

Six focused component tests pass. The original string borrowing test now compiles
generated declarations with a generic nested string field and verifies pointer
identity, mutation and owning reconstruction. A new native test borrows a mutable
callback whose environment owns a String, invokes it twice through a generic view
consumer, updates the record label, and invokes the callback again after owning
reconstruction. No cloning or lifetime erasure is used. Formatted Tsrust source
provides the checked record layout; native assertions verify the rendering.

Exact red and successive successful evidence is verified in SQLite under
runtime/diagnostics/scoped-record-view-declarations/. Project formatting changes
zero files and fingerprints. These are component results, not project acceptance.
Production nominal read/mutate trait bridges, generic bounds and parameter/field
emission remain pending. The original independent-field native witness remains
red. No shared dispatch changed, so previous depth evidence remains current.
Consumers remain f1dd284. Preserve all fifteen owner fixtures.

Continuation prompt: Continue from this handoff and the scoped record field storage
plan. Connect generated nominal views and their generic field slots to production
read/mutate parameter bridges, then declaration and selected field storage emission.
Use the unchanged project witness and live-owner rejection controls. Preserve
non-copy fields, callback bounds, nominal identity, owning moves, three public
wrappers and no implicit clone. Keep bounded checks, formatting, SQLite evidence
and explicit Git paths; never stage the fifteen owner fixtures.

## Selected independent field rendering

The layout module now renders a selected numeric field path with Read, Mutate,
Consume or Copy access. It selects the original independent leaf or reconstructs a
static subtree using the existing view/owning renderer. Invalid indices and attempts
to descend into an opaque leaf return no selection rather than guessing a place.
Existing native string disposal and shared/mutable view tests now use this boundary.

A new native component witness derives a checked Pair layout from formatted Tsrust
source, moves the left callback into an inner scope, mutates its original captured
array after that scope ends, and calls the remaining right callback successfully.
Generated selected places reference independent bindings. All seven component tests
pass. Exact red and green evidence is verified in SQLite under
runtime/diagnostics/scoped-record-selection/. Whole-project formatting changes zero
files and fingerprints. No shared analyzer/emitter dispatch changed.

This does not close production integration. The unchanged project/native witness
remains red. Main declarations, field expressions, storage-selection writes and
whole-record read/mutate parameter bridges still need routing. Whole borrowing
must use a nominal bridge with appropriate data/callback bounds; reconstructing an
owned record merely to borrow it would move non-copy fields and is not a solution.
Preserve property-order metadata when choosing eligible independent storage. Keep
parameter, capture, nullable and completion boundaries in the full implementation
scope. Consumers remain f1dd284; preserve fifteen owner fixtures.

Continuation prompt: Continue Tsrust from this handoff and the scoped record field
storage plan. Implement the nominal read/mutate borrowing bridge and production
emitter routing using the tested declaration, selected-value, owning and view
renderers. Retain the unchanged project red witness and live-owner rejection
controls. Preserve nominal identity, non-copy fields, callback bounds, property
ordering, ownership and three public wrappers. Use bounded native/editor checks,
dispatch depth canaries, project formatting, SQLite evidence and explicit Git paths.
Never stage the fifteen owner fixtures. Record gate and gate 173 remain open.

## Nominal borrowing bridge generator

New scoped_record_bridge.rs generates nominal read and mutate access traits for
validated leaf paths. Associated leaf types carry data and callback environments.
Owned record carriers implement read and mutate access. Shared views implement
read access only. Mutable views implement both; getters reborrow original leaf
storage without moving references or payloads. Names use numeric field paths.
Caller-provided native carrier names, generic parameter names and leaf types keep
nominal mapping and callback bounds under production emitter control.

Seven component tests pass. The nested string witness now uses the generated
bridge for generic inspection and mutation, with pointer identity and owning
reconstruction assertions. A native rejection control tries passing a shared view
to the mutable consumer and requires rejection naming MutateOuter. The callback
witness uses one generic mutable consumer with an associated FnMut bound for both
the mutable view and reconstructed owned record. Its environment owns a String.
No clone, unsafe erasure or fourth public wrapper is introduced.

Exact red, successful and rejection-control evidence is verified in SQLite under
runtime/diagnostics/scoped-record-bridge/. Whole-project formatting changes zero
files and fingerprints. No shared compiler dispatch changed. Native executables
are cleaned after both accepted and rejected compilation checks. Production
parameter signatures, callback-bound collection, field access and storage routing
still need integration. This generator does not make the original project/native
independent-field witness green; that witness remains unchanged and red. Keep
property-order metadata, capture/completion transport and opaque composition
boundaries in scope. Consumers remain f1dd284; fifteen owner fixtures stay untouched.

Continuation prompt: Continue Tsrust from this handoff and the scoped record field
storage plan. Integrate the generated nominal borrowing bridge with production
parameter signatures and associated callback/data constraints. Route independent
declarations, selected field expressions, stores and whole moves using the tested
layout renderers. Keep the original project red and live-owner controls unchanged.
Run bounded native/editor checks and dispatch depth canaries after routing changes.
Use automatic project formatting, SQLite evidence and explicit Git paths; never
stage the fifteen owner fixtures. Record acceptance and gate 173 remain open.

## Independent generic slots for equal callback contracts

Re-running the unchanged original project witness confirms native E0502 remains
open. Parameter emission currently uses concrete native record carriers and existing
callback generic bounds in emit.rs. Storage selection still starts from value{id}
and walks native fields. These boundaries need production routing.

Before routing, view and bridge metadata now receives field identity rather than
only Type. View declarations receive record ID, direct field index and checked type.
Bridge associated types receive the full Leaf, including numeric path. Equal callback
contracts can therefore use different native environment slots. This is necessary
for nominal declarations independently of whether a checked instance has distinct
closure types.

Eight component tests pass. The new witness starts from checked nominal Pair
metadata, deliberately reuses an equivalent callback type identity for both fields,
and creates native closures owning a String and a Vec respectively. Generated
views carry separate F0/F1 slots. A generic nominal read consumer invokes both
associated callback types and verifies result 14. The initial metadata identity
assertion failed because this checker assigns distinct callback IDs at the two
declaration sites; that diagnostic is retained. The corrected backend witness
then failed native compilation before field-aware metadata was implemented.

Exact diagnostics, native red and successful results are verified in SQLite under
runtime/diagnostics/scoped-record-view-slots/. The original production E0502 proof
is retained there too. Whole-project formatting changes zero files and fingerprints.
No shared compiler dispatch changed. This repairs the renderer metadata boundary;
production parameter, field and storage routing remains pending. Consumers stay
f1dd284. Preserve fifteen owner fixtures.

Continuation prompt: Continue Tsrust from this handoff and the scoped record field
storage plan. Integrate field-aware nominal views and borrowing bridges into
production parameter and storage emission. Preserve separate callback field slots,
including equal contracts and distinct environments. Nested owned record carriers
may use generic child records; their accessors need child trait constraints rather
than assuming a generic child has concrete native fields. Keep the original project
red and live-owner controls unchanged. Use bounded checks, dispatch depth canaries,
formatting, SQLite evidence and explicit Git paths; never stage owner fixtures.

## Generic child borrowing contracts

The bridge generator now accepts separate owned-carrier generic parameters, read
bounds, mutate bounds and leaf access expressions. Owned associated leaf types may
project from a child read contract. Read and mutate implementations use the
corresponding child constraints rather than assuming an arbitrary child generic
parameter has native fields. Existing direct-field callers retain their simple
wrapper. Shared and mutable view implementations retain their original references.

Nine component tests pass. The new witness generates inner and outer bridges,
defines an owning outer native record with a generic child carrier, inspects owned
and shared forms, mutates through a mutable view, reconstructs ownership and mutates
again through the same generic consumer. A child shared view also works inside the
outer owned carrier for inspection. A native rejection control requires MutateInner
when that readonly child is passed to the mutable outer consumer. No implicit copy,
clone or lifetime erasure is used.

Exact red, successful and rejection-control evidence is verified in SQLite under
runtime/diagnostics/scoped-record-child-bridge/. Whole-project formatting changes
zero files and fingerprints. No shared dispatch changed. Production emitter routing
remains open, and the original project/native witness remains unchanged and red.
Consumers remain f1dd284; preserve fifteen owner fixtures.

Before routing nested views, add a witness with two sibling occurrences of the same
nominal generic child record but distinct callback environments. Parent view slots
must preserve each occurrence independently; the current child-parameter propagation
needs verification at that boundary. Keep the full transport, capture, property-order
and completion scope. These foundations do not constitute record acceptance.

Continuation prompt: Continue Tsrust from this handoff and the scoped record field
storage plan. Verify repeated nominal child occurrences with distinct environments,
then connect view/bridge metadata to production parameters and storage emission.
Keep nested child trait bounds, associated callback/data constraints, original red
witness and live-owner controls. Use bounded native/editor checks, dispatch depth
canaries, formatting, SQLite evidence and explicit Git paths. Never stage owner
fixtures. Record gate and gate 173 remain open.

## Repeated nominal child environment arguments

The new native witness confirms parent view declarations previously collapsed
generic arguments when two sibling fields referenced the same nominal child. The
fix renames each child's generic arguments by its numeric field occurrence in the
parent. Nominal child declarations remain shared and emitted once. Renaming also
handles explicit lifetime parameter names; the view's own borrow lifetime remains
shared across its fields. Field order and parameter order remain deterministic.

Ten component tests pass. The witness derives checked nested Holder/Pair metadata,
uses the same nominal child in both field slots, and creates callbacks with distinct
String and Vec environments. The generated shared view accepts both environments
and a generic consumer invokes them with result 14. Existing nested data, mutation,
child contract, disposal and readonly rejection controls remain green.

Exact native red and successful evidence is verified in SQLite under
runtime/diagnostics/scoped-record-repeated-child/. Project formatting changes zero
files and fingerprints. No shared dispatch changed. Production emitter routing
remains pending; the original project/native independent-field witness remains red.
Consumers remain f1dd284. Preserve fifteen owner fixtures.

Continuation prompt: Continue Tsrust from this handoff and the scoped record field
storage plan. Build production metadata for nominal views and borrowing bridges
using field-aware generic slots, child argument renaming, generic child constraints
and associated leaf types. Connect parameter, declaration, selected access/store and
whole-move routing. Keep opaque boundaries, property ordering, captures/completion,
original red and live-owner controls in scope. Use bounded checks, required depth
canaries after dispatch changes, automatic formatting, SQLite evidence and explicit
Git paths. Never stage owner fixtures. Record gate and gate 173 remain open.

## Checked production metadata adapter

Layout::for_record now validates a static nominal record directly. Binding storage
eligibility remains in for_binding, which delegates to the same validator. Ten
layout/rendering tests pass, including direct nominal layout native verification.
Test-only native compilation helpers are shared with the new emitter adapter tests.

New scoped_record_support_emit.rs derives borrowing support from checked program
metadata and Emitter::native_type/scoped_record_native_parameter. It produces view
slots, child occurrence arguments, owned ABI generic parameters, child read/mutate
constraints and associated leaf access expressions. It traverses the checked record
family, generates views and nominal bridges, and preserves concrete data leaf types.
The helper is not yet invoked by ordinary emission.

A focused native test appends this adapter's output to the ordinary compiler-emitted
program and executes an explicit verification function. It creates two nested Holder
instances with distinct String and Vec callback environments using the actual
compiler-owned Record ABI, then invokes both through the generated generic nominal
read bridge on owned storage and a shared view. Assertions require result 14.
The initial source used retired read syntax; its diagnostics remain preserved.
The corrected ReadOnly<Pair> witness establishes the adapter API red, then passes
generated native compilation and execution after connection to checked metadata.

Exact type-layout and adapter diagnostics/results are verified in SQLite under
runtime/diagnostics/scoped-record-production-metadata/. Project formatting changes
zero files and fingerprints. Shared parser/analyzer/emitter dispatch is unchanged.
The original project/native independent-field witness remains red. Ordinary support
emission needs shared declaration caching across roots so nested families do not
redeclare their views or bridges. Parameter and storage routing remain open. Preserve
property-order metadata and opaque/capture/completion boundaries. Consumers remain
f1dd284; preserve fifteen owner fixtures.

Continuation prompt: Continue Tsrust from this handoff and the scoped record field
storage plan. Connect checked borrowing support to ordinary type emission with
shared view/bridge declaration caches across nominal roots. Then integrate generic
parameter constraints, field selection, declarations/stores and whole moves. Keep
original red and live-owner controls unchanged. Use bounded native/editor checks,
dispatch depth canaries after routing changes, automatic formatting, SQLite evidence
and explicit Git paths. Never stage owner fixtures. Record gate and gate 173 remain
open; no performance measurements have been taken for this feature.

## Ordinary support emission and declaration caching

Ordinary native type emission now registers borrowing support for eligible nominal
record families. SupportCache owns separate shared-view and mutable-view parameter
maps plus a bridge declaration set. Layout::view_declarations_cached preserves
child generic arguments across roots while emitting declarations once. Completed
roots return empty support immediately; empty or invalid layouts retain their
existing no-support result. Caches are local to one emission, never global.

The native adapter witness now relies on ordinary emission rather than appending
support manually. It also requests parent, child and repeated parent support with
one cache, requiring one shared child declaration and one nominal parent bridge.
The original cache witness failed native compilation on duplicate declarations.
The ordinary-emission witness failed because support was missing, then passed
generated native execution after registration. Distinct nested callback environments
remain independently typed and return 14 through owned and shared forms.

Ten component tests and all 64 controls across the eight depth canary suites pass.
The scoped_callable_records project batch remains 19 passed and one known native
E0502 failure: scoped_record_partial_field_moves_release_independent_capture_loans.
Both live-owner rejection controls remain green. The completed-cache fast path
passes the focused native adapter check. No parameter or independent storage
routing is implemented yet, so the original witness has not been weakened or
described as accepted.

Exact cache red/green, emission red/green, project, depth, component and final cache
evidence is verified in SQLite under
runtime/diagnostics/scoped-record-support-emission/. Project formatting changes
zero files and fingerprints. Consumers remain f1dd284; preserve fifteen owner
fixtures. Record gate and gate 173 remain open. Full editor acceptance and matched
record benchmark support remain owed; no timing was performed.

Continuation prompt: Continue Tsrust from this handoff and the scoped record field
storage plan. Borrowing support now emits automatically with shared declaration
caches. Integrate parameter constraints and selected field access using nominal
bridge traits, then independent local/parameter declarations, stores and whole
moves. Keep opaque composition, property ordering and capture/completion transport
in scope. Preserve original native red and live-owner controls. Use bounded checks,
required depth canaries after dispatch changes, automatic formatting, SQLite
evidence and explicit Git paths. Never stage the fifteen owner fixtures.

## Disjoint mutable borrowing red checkpoint

Two new focused tests remain intentionally red in scoped_record_support_emit.rs.
The checker witness nested_mutable_arguments_borrow_disjoint_sibling_records
requires touch(pair.left, pair.right) with Mutable<Holder> parameters to accept
statically distinct record fields. It currently fails conflicting-access at the
second argument. analysis.rs Loans is a binding-root map of (Mode, Span) entries;
ordinary fully available field access does not establish a projection for that
temporary loan check. Preserve source overlap rejection while adding field identity.

The native witness nominal_mutable_consumer_preserves_disjoint_sibling_fields
first compiles the ordinary emitted program successfully. Its additional generic
mutable consumer holds two sibling string references from separate bridge getters.
Native compilation then fails E0499 because both getters borrow the entire generic
carrier. This confirms that mutable parameter bodies need one concrete view from
a whole-record transport operation; direct repeated mutable getters are insufficient.
The initial combined witness stopped at the checker gap, so the tests were separated
to retain independent evidence for both boundaries.

Exact checker and native red diagnostics are verified in SQLite under
runtime/diagnostics/scoped-record-disjoint-borrowing/. Project formatting changes
zero files and fingerprints. No production source changed this checkpoint; earlier
64 depth, ten component and 19 accepted project cases remain the previous evidence.
The original E0502 project witness is still red. Consumers remain f1dd284. Fifteen
owner fixtures remain untouched. No gate or feature acceptance is claimed.

Continuation prompt: Continue Tsrust from this handoff and the scoped record field
storage plan. First implement place-aware temporary argument loans for static
record fields, retaining same-field, ancestor, whole-root, opaque-index and hidden
capture rejection controls. Run required depth canaries after analyzer dispatch
changes. Then implement whole-record view transport and one concrete parameter
entry view to preserve native disjoint mutation. Keep both new red witnesses and
the original E0502 witness; never weaken them. Continue independent storage and
full capture/completion/editor/benchmark scope. Use bounded checks, automatic
formatting, SQLite evidence and explicit Git paths; never stage owner fixtures.

## Static argument field loans

Temporary argument loans now retain proven static record field paths in the isolated
argument_access_loans module. Legacy root loan insertion remains conservative and
invalidates earlier field proofs for that binding. Two paths overlap when either is
a prefix of the other. Unknown, indexed and hidden capture accesses remain root-wide.
Ordinary field access establishes a projection while temporary loans are active;
fully available reads without loans retain the previous allocation-free path.

The committed disjoint sibling argument witness now passes analysis. Five controls
pass for same-field and whole-root conflicts, ancestor paths, opaque indices, mixed
mutable/shared access, hidden captures and broad-insertion invalidation. Mutable
scalar contracts remain rejected with callback-access because scalars copy; the
initial proposed scalar-loan witness was invalid and was corrected to preserve the
existing language rule. All 64 tests across the eight required depth suites pass.
Exact compilation diagnostics and red/green results are verified in SQLite under
runtime/diagnostics/scoped-record-argument-loans/. Project formatting changes zero
files and fingerprints. Fifteen owner fixtures remain unstaged and unchanged.

This is an analyzer checkpoint, not feature acceptance. The native E0499 bridge
witness and original E0502 independent-field witness remain unresolved. Implement
whole-record view transport and one concrete mutable parameter entry view next;
then complete declaration/access/store/whole-move routing and capture/completion,
editor and matched benchmark support. Existing ReadOnly, Mutable and Once contracts
remain the public capability model. No additional wrapper is introduced. Consumers
remain f1dd284; no timing or full editor acceptance is claimed.

Continuation prompt: Continue Tsrust from this handoff and the scoped record field
storage plan. Static sibling argument loans now pass with conservative overlap
controls. Implement whole-record native view transport and one parameter entry view,
then independent storage and remaining capture/completion/editor/benchmark scope.
Keep native red witnesses unchanged. Preserve fifteen owner fixtures and use explicit
Git paths, bounded checks, required depth canaries, project formatting and SQLite.

## Whole-record native view transport

Production nominal borrowing bridges now provide tsr_fields and tsr_fields_mut.
Each operation borrows the carrier once and returns a concrete flat view containing
separate references to its static leaves. These internal view structs parameterize
all leaves with associated types, including unsized leaves, so callback environments
remain distinct and concrete scalar constraints remain expressible. Existing nested
view declarations and single-field accessors remain available.

Owned parent records request one child transport per generic child field, then move
the child's independent references into the parent transport view. They do not call
multiple mutable getters on the same generic child. Shared views implement read
transport only; mutable views reborrow their concrete fields for both operations.
No cloning, allocation, unsafe erasure or new public ownership wrapper is introduced.

The new native transport witness was red on missing tsr_fields_mut, then passed.
It holds sibling string references simultaneously, checks mutations, exercises owned,
shared and mutable carriers with distinct String and Vec callback environments,
checks callback result 14, and verifies shared string pointer identity. A generated
native rejection control confirms shared views cannot invoke a mutable consumer.
Ten layout/rendering controls and the existing production metadata adapter pass.
The original separate-getter E0499 witness and independent-storage E0502 witness
remain retained; transport support alone does not route ordinary parameter bodies.
No shared parser/analyzer dispatch changed in this checkpoint.

Next, establish one concrete transport view at each eligible borrowed parameter
entry. Route selected field access through its independent references and rebuild
nested views only when needed for borrowed child arguments. Then integrate independent
local and owned-parameter storage, stores, whole moves, property ordering and all
capture/completion boundaries. Complete editor and matched benchmark acceptance
before claiming the record feature accepted. Consumers remain f1dd284 and fifteen
owner fixtures remain untouched. Project formatting changes zero files and hashes.
No performance timing or full editor acceptance is claimed.

Continuation prompt: Continue Tsrust from this handoff and the scoped record field
storage plan. Static argument loans and whole-record native transport now exist.
Implement parameter entry transport and selected access, then independent storage,
assignment and full capture/completion/editor/benchmark scope. Preserve original
red witnesses, fifteen owner fixtures, bounded checks, automatic formatting and
SQLite evidence. Do not refresh consumers from incomplete feature source.

Exact native red/green, rejection, component and retained storage diagnostics are
verified in SQLite under runtime/diagnostics/scoped-record-whole-transport/.

## Parameter entry and leaf access components

The isolated scoped_record_parameter_views emitter module now derives one transport
operation for an eligible borrowed scoped-record parameter. Shared entry uses
let record_fieldsN=valueN.tsr_fields(); mutable entry uses the corresponding mutable
transport with a mutable view binding. Eligibility follows the checked binding's
mode and input type, nominal declaration and validated static layout. Scalars,
ordinary non-scoped records and explicitly owned parameters keep their existing ABI.
A bare unused record parameter can be inferred as borrowed; explicit Owned is needed
for the ownership boundary control. This existing inference was not changed.

The leaf-place helper validates a complete numeric leaf path before returning a
place rooted at its entry view reference. It rejects unknown paths and record
subtrees, which still need an explicit nested-view assembly operation. A native
component witness uses both read and mutable entry output, reads callbacks before
and after updates with result 6, and holds sibling mutable string references at once
through generated leaf-place expressions. Mode and path controls pass. Source is
formatted and a stable second pass is required before analysis.

Neither helper is connected to ordinary emission yet. The new routing witness
ordinary_borrowed_record_parameters_use_one_entry_view remains red on missing mutable
entry output. Signature generic constraints, associated callback/scalar bounds,
entry initialization for ordinary and completion bodies, selected leaf access and
nested borrowed arguments must be integrated together. Do not merely add an unused
entry borrow: current aggregate access would conflict with that borrow, and ordinary
parameter types cannot yet accept reconstructed borrowed child views.

No shared dispatch changed; previous depth evidence remains the latest evidence.
Original getter and independent-storage native failures remain open. Full editor,
guide, property-order/capture/completion coverage and matched benchmarks remain owed.
Consumers remain f1dd284. Fifteen owner fixtures remain untouched. This is a component
checkpoint with a retained integration red, not feature acceptance.

Continuation prompt: Continue Tsrust from this handoff and the scoped record field
storage plan. Native transport and parameter entry/leaf-place helpers now pass
component checks. Ordinary entry routing remains red. Integrate borrowed record
signature constraints, one entry view, selected leaf access and nested child argument
views coherently, including completion bodies. Then finish independent storage and
remaining acceptance. Preserve fifteen owner fixtures and original red witnesses.
Use bounded tests, dispatch depth canaries, project formatting and SQLite evidence.

Exact entry API red, native component success, corrected ownership control and
ordinary routing red are verified in SQLite under
runtime/diagnostics/scoped-record-parameter-entry/. Project formatting changes zero
files and fingerprints.

## Ordinary scoped-record parameter routing

Ordinary source function signatures now use nominal read/mutate bridge parameters
for eligible borrowed scoped records. Each callback leaf has its own associated
trait bound rather than sharing a callback environment type across sibling fields;
concrete leaves retain equality constraints. Callback view slots also require Sized,
matching the existing nested view declarations. Redundant callback generic parameters
from these record inputs are excluded from the ordinary signature. The existing
result and other parameter bounds remain in place.

Parameter initialization establishes one flat transport view. Static leaf storage
selection starts from the corresponding view reference and retains subsequent index
and union selection behavior. Borrowed whole records and static record subtrees are
rebuilt as nested shared/mutable views from those independent references. Empty
records remain opaque leaves. Source parameter routing deliberately excludes inherited
capture slots: these are not source argument bindings and may not have a local valueN.
Their existing capture transport remains unchanged and still needs its own integration.
Layout eligibility is factored to avoid formatting an unused entry statement during
bound, leaf and subtree lookup.

Four parameter-view controls pass, including the previously red ordinary entry
witness. A native source program calls mutable child consumers through a parent
parameter, invokes distinct callback environments through readonly parameters, and
checks exact output 14 and left!/right! mutations. A complementary pure try/finally
program checks finally mutation and exact output done/right!. The checker sibling
argument witness is strengthened with native compilation and passes. All 64 required
depth controls pass. The scoped_callable_records project batch is again 19 passed,
with only its original E0502 independent-storage witness failing. Initial inherited
capture and unsized-slot regressions were repaired; their diagnostics are preserved.

Remaining boundaries are explicit. A new native red witness
scoped_parameter_can_borrow_a_plain_nested_record fails E0308: a scoped parent produces
a mutable child view, while a consumer of an ordinary callback-free record still
expects its owned-record ABI. Extend nominal borrowing compatibility to that child
consumer without reconstructing owned values or cloning. Parameter ABI coherence
must cover these ordinary child records as well as scoped children. The previous
plain-record ABI control documents the current boundary and must be revised only
with a corresponding native compatibility test when that boundary is implemented.

A try/finally variant retaining runtime-fallible array-index callback captures also
exposes unresolved error contracts for abstract record callbacks. An intermediate
variant with unused fallible callback fields reaches native Result-versus-scalar
callback trait errors. Pure completion routing is verified; fallible callback error
contract and carrier convergence remain open. Do not claim full completion parity
from the pure control or discard these failures. Original separate-getter E0499,
independent local storage, property ordering, captures, opaque composition, full
editor/guide and matched benchmark acceptance remain pending. Consumers stay f1dd284.
Fifteen owner fixtures remain untouched. No timing or feature acceptance is claimed.

Continuation prompt: Continue Tsrust from this handoff and scoped record storage plan.
Ordinary borrowed scoped-record signatures, entry views and selected access now pass
focused native checks. Fix the retained plain-child borrowing E0308 witness next,
then independent local/owned-parameter storage and fallible callback contracts.
Preserve original native failures and owner fixtures. Finish capture/completion,
property-order/opaque boundaries, editor, guide and matched benchmark acceptance.
Use explicit Git paths, bounded checks, dispatch depth canaries, project formatting
and SQLite evidence. Do not refresh consumers from incomplete feature source.

Exact routing red/green, corrected regressions, completion boundaries, plain-child
red and depth evidence are verified in SQLite under
runtime/diagnostics/scoped-record-parameter-routing/. Project formatting changes
zero files and fingerprints.

## Plain child borrowing compatibility and nominal join registration

Native emission now computes the nominal borrowing family once. It includes each
validated scoped record root and its static record descendants, including ordinary
callback-free children. Parameter eligibility uses that family set, so a function
accepting a plain child can also accept its reconstructed shared or mutable view.
Unrelated ordinary record parameters retain their previous ABI. Empty descendants
remain opaque leaves and retain their owned-record parameter representation. The
family set and support declaration caches belong to one emission.

The previous E0308 plain-child witness now passes. It checks mutable child updates,
readonly length access, direct and deeper nested children, and exact native output
7/changed. Six parameter controls pass, including empty children, ownership/mode
boundaries, distinct callbacks and pure completion routing. The production metadata
adapter passes. All 64 required depth controls pass. The project scoped-record batch
remains 19 passed with its original E0502 independent-storage failure.

A property-order witness exposed two independent boundaries. The isolated witness
borrowed_plain_child_preserves_constructor_property_order still fails E0609 because
the borrowed view lacks key_order. It requires constructor order second,first to
survive the child view, not declaration order. Implement metadata transport for each
ordered nested node before claiming this borrowing family complete; Object.keys,
values/entries, console inspection and spread/rest must retain observable ordering.

The other failure was stale array join registration for generic/scoped record
metadata. The new record_join_emit module emits only nominal record registrations,
with the same generic field slots as the owned struct declarations. Tuple generic
slots receive TsrJoinText bounds, and tuple fields delegate nullable formatting to
the existing union join implementations. This removes references to nonexistent
specialized record names and missing generic arguments. Its new native witness
passes exact output a,b and 1; all four array_join integration tests pass, including
nullable leaves, nested commas, captures, allocation/disposal and argument loans.

The property-order witness was separated from join so each boundary has independent
evidence. No new public ownership wrapper, cloning or unsafe erasure is introduced.
Fallible callback contracts, captured-record entry transport, independent local and
owned-parameter storage, opaque composition, editor/guide and matched benchmark
acceptance remain open. Consumers remain f1dd284. Fifteen owner fixtures remain
untouched. This is an implementation checkpoint, not record feature acceptance.

Continuation prompt: Continue Tsrust from this handoff and the scoped record field
storage plan. Plain child borrowing and nominal array join registration now pass.
Fix the retained property-order view witness next by transporting ordered metadata
for every nested node. Then finish independent storage, captures and fallible
callback contracts. Keep all original native failures and fifteen owner fixtures.
Complete editor/guide and matched benchmarks before acceptance. Use explicit Git
paths, bounded checks, required depth canaries, project formatting and SQLite.

Exact native child success, controls, metadata, project/depth evidence, property-order
red and join red/green are verified in SQLite under
runtime/diagnostics/scoped-record-plain-child/.


## Ordered borrowing views and explicit snapshots

Ordered records now carry property-order metadata through readonly and mutable
parameter transports at every nested record node. Mutable transports borrow the
original metadata exclusively; readonly transports borrow it without allocation.
Repeated child records retain separate order arrays. Nominal specializations agree
on metadata presence before support declarations are emitted.

Console inspection accepts borrowing views through the existing record traits.
Explicit record copies, spread and rest reconstruct owned records from borrowed
leaves and copy their order metadata. Object.values and Object.entries also copy
nested record owners rather than attempting to clone borrowing views. These paths
remain subject to the existing cloneability rules; callback-bearing snapshots are
not newly supported. No implicit clone or public ownership capability is added.

Fifteen parameter-view controls and four Object.values/entries integration tests
pass. The retained rest batch passes 23 tests and the record console batch passes
two tests. The project batch still passes 19 tests with the original independent
owned-field storage E0502 failure. Original failed runs remain evidence, including
metadata mutability, console view adaptation and nested snapshot failures.
All 64 depth canaries pass after the snapshot changes. Project formatting changes
zero files and zero benchmark fingerprints.

This is a foundation checkpoint, not scoped-record feature acceptance. Independent
local and owned-parameter field storage, captured-record transport, opaque
composition, fallible callback contracts, editor/guide and matched benchmark
acceptance remain open. Executable consumers remain at f1dd284. Fifteen protected
owner fixtures remain unstaged and untouched. Diagnostic evidence is stored under
runtime/diagnostics/scoped-record-order/ in benchmarks/records.sqlite.

Continuation prompt: Continue Tsrust from this handoff and the scoped record field
storage plan. Ordered metadata, borrowing-view console inspection, spread/rest and
nested Object.values snapshots now pass. Finish independent owned-field storage
next, retaining the original E0502 witness. Then complete captures, opaque
composition and fallible contracts. Preserve fifteen owner fixtures and existing
three callable capabilities. Use focused red tests, bounded checks, editor and
benchmark acceptance, project formatting, SQLite evidence and explicit Git paths.


## Independent local storage integration in progress

The next implementation chunk starts from committed HEAD 460e998. The original
project E0502 witness was reproduced unchanged, then turned green by routing owned
local record declarations and static selections through independent native leaves.
The new scoped_record_owned_places module also renders nested views, owning
reconstruction and property-order metadata. Whole replacement uses destructuring
assignment so its RHS is evaluated before fields are committed. The existing
field/whole restoration witness passes after this routing change.

This work is uncommitted and is not accepted. The first project batch passed 16
of 20 tests, including the original independent capture-release witness and both
live-capture rejection controls. Two failures require closure record captures to
carry borrowing views instead of concrete owned-record references. The replacement
failure is repaired. Optional absent callback inference has a typed initializer
repair whose focused verification must be inspected before claiming success.
Owned parameters, deferred declarations, capture entry routing and broader
composition still need implementation. Do not commit a narrowed eligibility
workaround to hide these boundaries. Preserve the red and intermediate logs in
/private/tmp/tsrust-record-independent-*.log until they are imported and verified
in SQLite. No consumers were refreshed and no protected owner fixtures changed.

Continuation prompt: Continue the uncommitted independent record storage work from
460e998. Inspect status and current scoped_record_owned_places module. Verify the
optional-field focused test, then implement borrowing-view capture types and entry
routing to fix the two retained capture E0308 failures. Complete owned-parameter
and deferred storage, control-flow and opaque composition support. Keep existing
public callable capabilities and no implicit clones. Run bounded project controls,
depth canaries and formatting, store exact evidence in SQLite, and commit only
explicit implementation paths after the chunk is verified.


## Independent local storage and record capture transport

Owned scoped-record locals now destructure once into independent native leaves.
Static selections move or borrow those leaves directly. Whole owning moves rebuild
nominal records without cloning. Whole replacement uses destructuring assignment
and evaluates the replacement before committing fields. A typed initializer keeps
absent callable union parameters determined before destructuring. Ordered nested
nodes retain independent metadata storage and borrow it through views.

Readonly and mutable record captures now store borrowing views of these leaves,
with the existing capture and environment lifetimes. Capture bodies establish a
single field transport rather than assuming a concrete owned record reference.
The original independent-field E0502 witness is green. Both live-capture rejection
controls remain unchanged. All 20 scoped-record project tests and 15 parameter-view
controls pass. The original four integration failures are preserved as evidence;
replacement, optional callable inference and two capture view failures are repaired.

This is an implementation checkpoint, not full scoped-record acceptance. Owned
parameters and deferred declarations still use their existing aggregate storage.
Their independent representation needs dedicated witnesses and entry/assignment
integration next. Opaque composition, mutable capture/disposal controls, fallible
contracts, editor/guide and matched benchmark acceptance remain open. No consumer
refresh or timing is claimed. Fifteen owner fixtures remain untouched.

Continuation prompt: Continue Tsrust independent record storage. Local storage,
record borrowing captures and all 20 project controls now pass. Add red witnesses
for partial capture release from owned parameters and deferred record bindings;
implement independent parameter entry and deferred declarations/restoration next.
Then finish mutable capture/disposal, opaque composition and fallible contracts.
Preserve all owner fixtures and existing three callable capabilities. Use bounded
checks, depth canaries, project formatting, SQLite and explicit Git paths. Complete
editor/guide and matched benchmark support before feature acceptance.

All 64 depth canaries pass. Project formatting changes zero files and zero
fingerprints. Exact red/intermediate/green records are verified in SQLite under
runtime/diagnostics/scoped-record-independent/.


## Owned parameter entry and deferred record storage

Scoped owned record parameters now destructure into independent leaves after
parameter default initialization, preserving the existing nominal argument ABI.
Inherited capture slots remain excluded from owned parameter entry. Deferred local
records declare independent leaf variables and metadata arrays before assignment.
Callback leaves use inferred native types so nominal callback slots do not name
undeclared generic parameters. Existing definite-assignment checks still govern
all reads and replacements; assigning the record commits leaves only after its
replacement expression is evaluated.

A new deferred declaration witness exposed missing independent locals, then
undeclared callback generic types, and finally an incorrect local callback receiver.
The repaired native witness proves the left capture is released independently of
the right capture after a deferred record assignment. Local callbacks now borrow
owned native receiver storage when invoked instead of assuming an existing native
reference. The existing 20 project controls and this new deferred witness pass.
An explicit Owned<Pair> parameter with readonly callback fields also passes native
execution and an entry-storage source assertion. Its first proposed witness used
ordinary owned callback fields, which are outside the scoped borrowing family;
that witness was corrected rather than broadening claims from its failed assertion.

This is another representation checkpoint, not full record acceptance. Defaulted
and destructured owned parameter combinations, branch/loop/exception deferred
initialization, mutable record captures, disposal, opaque composition and fallible
callback contracts still need focused boundary coverage. Editor/guide and matched
benchmark acceptance remain open. No consumers or protected owner fixtures change.

Continuation prompt: Continue independent scoped-record storage from this checkpoint.
Owned parameter entry and deferred local storage now have focused native coverage.
Next add boundary witnesses for branch/loop/exception initialization, defaulted and
destructured owned parameters, mutable captures and payload disposal. Repair every
failure without weakening ownership. Finish opaque composition and fallible callback
contracts, then editor/guide and matched benchmark acceptance. Preserve fifteen
owner fixtures, use bounded checks and SQLite evidence, and format the whole project
before explicit-path commits. Installed consumers remain f1dd284.

All 15 view controls and 64 depth canaries pass. Formatting changes zero files
and fingerprints. The parameter entry assertion is isolated from main local
record storage. Exact evidence is verified in SQLite under
runtime/diagnostics/scoped-record-entry/.


## Branch-assignment carrier witness: implementation pending

The mutable record capture boundary passes native execution without another
implementation change. Its new witness mutates an independent string field,
invokes the captured mutable callback repeatedly, releases the record capture,
then invokes the original field and finally mutates its original scalar owner.
Expected assertions are 1, 2, changed, 3 and 7.

A separate deferred record branch/loop witness fails native E0308. Both branch
assignments satisfy the same nominal readonly callback contract but construct
Closure0 and Closure1. Independent leaf type inference chooses Closure0 after
the first assignment; the other assignment requires Closure1. The source is valid
and remains unchanged. This is a common-carrier gap, not a definite-assignment
failure or permission to merge capture ownership.

Record construction currently specializes each callback field independently in
analysis.rs scoped record construction (around lines 9220-9270). Conditional
callable expressions already select common borrowed families through
common_borrowed_conditional_family in conditional completion (around line 9045).
Branch statements assigning a record do not apply that conversion to the common
storage leaf. The next implementation must collect compatible origins for each
static record field storage slot, register their common borrowed carrier, and
normalize every assignment and later selected invocation to that carrier. Reuse
the existing callable contract checks and dispatch family; do not erase types,
clone captures or make source branches share a JavaScript capture cell. Keep
point-sensitive capture loans and independent disposal even though the native
carrier covers multiple possible origins. Nested fields, replacement and moved
record inputs require the same slot identity and conversion discipline.

Both new witnesses use formatting round trips and std:testing project execution.
This red-test checkpoint is not feature acceptance. No compiler source changed.
Continue implementation from the unchanged branch witness before adding unrelated
features. Earlier completed local, parameter and capture checkpoints stay closed.

Continuation prompt: Continue from the branch-assignment red-test checkpoint. Run
scoped_callable_records::deferred_record_branch_and_loop_restoration_preserve_capture_release
and implement common native callable carriers for independent record leaf storage
across branch assignments. Reuse existing borrowed conditional family checking and
wrapping, preserve point-sensitive ownership and both origin environments, and keep
the witness unchanged. The mutable record capture witness already passes. After
the branch fix, verify the project batch and required depth controls, then proceed
to default/exception/disposal and opaque composition boundaries. Preserve fifteen
owner fixtures, format the project, import exact evidence into SQLite, and commit
explicit paths. Record family acceptance still requires editor/guide and benchmarks.

Exact mutable-capture success, branch failure and project formatting evidence
are verified in SQLite under runtime/diagnostics/scoped-record-control/.


## Branch carrier implementation in progress

Uncommitted record_branch_carriers.rs joins compatible callback origins in direct
record assignments across two if arms. It reuses common_borrowed_conditional_family,
wraps both original environments, materializes a common nominal record instance and
updates the storage binding and both assignment target types. Capture-loan state
remains the result of the existing point-sensitive branch join. The original
branch/loop red witness passes, and the existing 24 project tests pass.

A new nested branch witness with three callback origins remains red. The inner
branch creates a specialized common record instance before the outer else assignment
is checked; that assignment still expects its concrete family field type and rejects
the third closure. Existing borrowed families represent exactly two origins, and the
current direct-arm collector does not traverse nested assignment bodies. This chunk
must not be accepted as a complete carrier solution. Keep the new witness unchanged
and extend registration/conversion and storage-slot collection to handle more origins
and nested control flow. Do not widen source borrowing or erase concrete environments.
Current loose records are /private/tmp/tsrust-record-branch-*.log; import only closed
records into SQLite after verification. HEAD remains 5196559. No consumers refreshed.

Continuation prompt: Continue uncommitted branch-carrier implementation. The original
witness and 24 project controls pass. Fix nested_record_branch_assignments_preserve_three_actual_origins,
which currently fails while checking the outer third assignment. Generalize borrowed
families and record field conversion together; collect nested/static storage slots
rather than only direct branch literals. Preserve point-sensitive capture loans and
all original environments. Reuse bounded checks and required depth canaries, format
before explicit-path commits, and import exact evidence into SQLite. No acceptance
claim until nested and nonliteral assignment boundaries have native coverage.

All 64 depth canaries pass on this uncommitted implementation. The four closed
branch records are now imported and verified under
runtime/diagnostics/scoped-record-branch/; loose copies were removed.


## Nested branch carriers and variable-cardinality borrowed families

Borrowed family identity now stores the complete sorted, deduplicated origin set.
Native declarations and dispatch support more than two concrete origins while
preserving the existing Left/Right ABI for two-origin families. Registration checks
every origin against the same argument, access, cardinality and result contract.
Compaction remaps the full origin set. This compiler metadata change does not clone
runtime environments or introduce a new public ownership capability.

Record branch normalization now collects assignments through nested blocks, loops,
if arms and exception regions. Compatible record literals materialize a common
nominal instance whose callback fields use the joined borrowed family. Existing
construction and replacement checks compare nominal identity and the written
callback contract before branch normalization, allowing another compatible concrete
origin without structural subtyping. Already wrapped literals retarget their storage
wrapper rather than retaining a redundant intermediate family owner.

All 25 scoped-record project tests pass. The nested three-origin test executes each
branch separately, observing 3, 7 and 11 and releasing the original borrowed array
before mutation. All 64 depth canaries pass. This is a literal branch representation
checkpoint, not complete record acceptance. Record-valued RHS expressions, independent
field replacement, sequential assignments outside branches and common carriers for
owning environments still need direct witnesses and conversion support. The normalizer
currently joins record literal expressions; do not claim arbitrary record assignment
coverage. Preserve point-sensitive capture loans and independent field disposal.
No consumers or owner fixtures change; no timing or full editor acceptance is claimed.

Continuation prompt: Continue after nested literal branch carriers. Twenty-five project
tests and all three nested origin selections pass. Add red witnesses for record-valued
RHS moves, field replacement and sequential assignments; implement one-evaluation
carrier conversion without cloning or lifetime erasure. Do not reopen solved local,
parameter or literal branch chunks. Then complete default/exception/disposal controls,
opaque composition and fallible callback contracts, followed by editor/guide and matched
benchmarks. Preserve fifteen owner fixtures, format before explicit-path commits and
store closed evidence in SQLite. Installed consumers remain f1dd284.

Thirty-one borrowed-related unit controls and ten callable storage controls pass.
Project formatting changes zero files and zero fingerprints. Exact intermediate
and final records are verified in SQLite under runtime/diagnostics/scoped-record-many/.


## Record-valued conversion and sequential replacement

A checked RecordCarrier node now converts compatible nominal records into joined
storage carriers. Its isolated emitter evaluates the source once, moves original
leaves, reconstructs nested records and forwards property-order metadata. Existing
borrowed-family payloads dispatch into the destination family by their actual origin.
No cloning, unsafe conversion or public syntax is introduced. Capture-loan payload
traversal treats the conversion as ownership-preserving, and the internal node keeps
the original source-depth accounting.

Branch assignments can now join record-valued RHS expressions rather than requiring
both RHS nodes to be literals. The new native witness executes both selections and
uses a side-effecting comma expression to prove one RHS evaluation. Different source
constructor orders survive the conversion and Object.keys observes the selected order.
Expected callback results are 3 and 11; the original arrays are mutable after all
owners are disposed.

Sequential local record writes also normalize their compatible native field carriers.
Construction checks the declared callback contract rather than requiring the previous
concrete origin. Checking records the actual replacement instance at each program
point. Local write normalization runs after statement checking, retaining the existing
point-sensitive loan decisions. The replacement helper now reconciles live capture
loans after staging and transferring the RHS, removing discarded old owners while
retaining independently moved or borrowed owners.

The sequential native witness observes 3 before replacement and 11 after replacement,
then mutates the released first array while the second callback remains live. The
27-test project batch, seven capture-loan unit controls and all 64 depth canaries pass.
This is an implementation checkpoint, not whole record acceptance. Normalization of
sequential writes currently covers owning local binding writes in one block. Cross-block
writes, independent callback field replacement, borrowed parameter replacement,
record-valued capture conversion and owning callback carrier composition still need
boundary witnesses and implementation. Complete default/exception/disposal, opaque
composition and fallible contracts before editor/guide and matched benchmark acceptance.

Continuation prompt: Continue after record-valued branch conversion and same-block
sequential local replacement. Add red witnesses for independent callback field
replacement, cross-block writes, borrowed parameter replacement and captured carrier
conversion. Preserve one evaluation, constructor order, point-sensitive loans and
independent disposal; no implicit clones. Existing successful chunks stay closed.
Then complete default/exception/disposal, opaque composition and fallible contracts,
editor/guide and matched benchmark support. Preserve fifteen owner fixtures and use
bounded checks, depth canaries, project formatting, SQLite and explicit Git paths.
Installed consumers remain f1dd284.

A separate rejection control verifies that replacement cannot release a capture
owned by a moved-out callback. Formatting changes zero files and fingerprints.
Exact red/intermediate/green evidence is verified in SQLite under
runtime/diagnostics/scoped-record-conversion/.


## Callback field replacement and cross-block local writes

Local callback field replacement now checks the declared nominal callback contract,
rather than demanding the preceding concrete environment origin. After access checks,
analysis materializes the replacement field instance at that program point while
preserving the types of sibling fields. Existing capture ownership replacement keeps
its field path, so releasing the old field does not release the sibling or another
owner of a moved-out callback.

The storage normalizer now collects relevant whole-record and callback-field writes
through nested blocks, branches, loops and exception regions. It ignores scalar writes.
Whole and selected-field writes join compatible carriers by binding identity and static
field names. A whole record initializer can convert its selected callback to the common
carrier while forwarding the remaining leaves and order metadata. Subsequent field
writes wrap their original environments in that same carrier. Ordinary source callback
ownership capabilities and nominal record identity remain unchanged.

The new field native witness observes 3 before replacement, 7 after replacement and
11 from the unchanged sibling. The released first array can mutate while both remaining
callbacks stay usable. The cross-block whole-record replacement witness also passes.
Thirty project tests and seven capture-loan unit controls pass. Three separate rejection
cases retain conflicts for the moved-out original callback, the new callback and the
unchanged sibling. This is a representation checkpoint, not whole record acceptance.

Borrowed parameter replacement, captured carrier replacement, expression-level stores,
loop-dependent origin changes, defaults/exception disposal and owning callback carrier
composition still require focused coverage. Native conversion currently preserves the
existing borrowed family representation; do not claim all owning or opaque callbacks
are supported. Finish those boundaries and fallible contracts, then editor/guide and
matched benchmark acceptance. No consumer refresh or timing is claimed.

Continuation prompt: Continue after callback field replacement and cross-block local
writes. Add boundary witnesses for expression-level stores, loop-dependent origins,
borrowed parameter and captured carrier replacement, defaults and exception disposal.
Keep one evaluation, original sibling fields, constructor order, point-sensitive loans
and moved-out owners. Finish owning/opaque composition and fallible callback contracts
before editor/guide and matched benchmarks. Do not reopen verified chunks. Preserve
fifteen owner fixtures, use bounded checks and depth canaries, format before explicit
Git commits, and import closed evidence into SQLite. Consumers remain f1dd284.

All 64 depth canaries pass. Formatting changes zero files and fingerprints.
Exact red/green and retained-conflict evidence is verified in SQLite under
runtime/diagnostics/scoped-record-writes/.


## Expression stores and loop-origin controls

The loop record replacement boundary already passes with zero, one and three
iterations. Its native witness observes the original callback at zero iterations
and the replacement callback after executed iterations, then mutates both arrays
after storage disposal. No loop implementation change was needed for this witness.

The first parenthesized store witness parsed as an assignment statement and passed.
It was strengthened to an assignment inside a comma expression. That native source
failed because expression emission still named the removed aggregate local and
carrier planning missed writes inside an expression initializer. Expression-level
void stores now use the same independent owning-record assignment renderer as
statements. Relevant record and callback-field stores are collected through comma
expressions, calls, wrappers, array/record children and other expression operands.
Checking retains its existing replacement ownership and evaluation-order rules.

Both whole-record and callback-field comma-store native witnesses pass. They observe
the replacement callback result, mutate the released old captured array while the
new callback stays live, and release the replacement capture at block exit. This
checkpoint does not add owned assignment-result values or an implicit clone: these
stores are discarded prefixes and the subsequent expression observes retained storage.

Nested stores inside an owning record RHS, condition operands, expression-result
alias transport, borrowed parameter replacement, captured carriers, default/exception
disposal, owning/opaque composition and fallible callbacks remain boundary work.
Complete those and editor/guide plus matched benchmark support before record feature
acceptance. No consumers or owner fixtures change; no timing is claimed.

Continuation prompt: Continue after discarded expression stores and loop-origin controls.
Whole-record and callback-field comma-store witnesses pass; zero/one/three loop
iterations preserve actual origins. Next cover nested stores/conditions and borrowed
parameter or captured-carrier replacement, then default/exception disposal and
owning/opaque/fallible composition. Preserve one evaluation, live-capture rejection
and independent disposal. Do not treat owned assignment-result aliasing as implemented.
Finish editor/guide and matched benchmarks before acceptance. Keep fifteen owner
fixtures untouched, use bounded checks, depth canaries, project formatting, SQLite
and explicit Git paths. Installed consumers remain f1dd284.

All 34 project tests and 64 depth canaries pass. Formatting changes zero files
and fingerprints. Exact evidence, including the passing initial statement witness
and the failing strengthened expression witness, is verified in SQLite under
runtime/diagnostics/scoped-record-expressions/.


## Condition-store checkpoint

Record stores in condition operands now participate in native carrier planning.
The focused witness failed with a concrete callback type mismatch before the fix.
It now executes the condition once, retains the replacement callback origin and
releases the replaced capture while the replacement remains live. The collector
also visits loop conditions, iterable operands and updates in their execution order.

All 35 scoped-record project tests and 64 depth canaries pass. Project formatting
changes zero files and fingerprints. Four exact red/green and regression records
are imported and verified in SQLite under runtime/diagnostics/scoped-record-conditions/.
The fifteen owner fixtures remain untouched. Installed consumers remain f1dd284;
this implementation checkpoint does not claim whole-feature acceptance or timing.

Next boundary: stores nested inside a record-valued RHS. The current mutable
write collector cannot retain both parent and child expression references. Start
with a focused native witness, then separate immutable write planning from the
mutation traversal if the witness exposes that limitation. Preserve original
capture origins and distinguish checked point types from finalized native storage
types when normalized records move between bindings. Previously verified chunks
must not be reopened because a user comment is repeated.


## Nested RHS stores and native carrier transport

An immutable, iterative write collector now records both an outer record store and
stores nested inside its RHS. It records binding identities, static field paths,
source spans, checked types and direct source-storage dependencies. Carrier planning
no longer retains overlapping mutable AST references or clones source subtrees.
A separate rewrite traversal applies the finalized destinations.

Local carrier sets widen to a fixed point across record moves. Native conversion
uses the actual independent source storage type rather than its narrower checked
point type. Point-sensitive capture origins and ownership checking remain intact;
this change adds neither shared capture cells nor implicit clones. A native witness
observes both replacement callbacks, labels and exactly one nested side effect.
Another witness retains the replacement capture conflict. Single-hop and multi-hop
record moves retain the current callback and release replaced captures.

All 39 scoped-record project tests and 64 depth canaries pass. The independent Node
reference agrees on the nested-store and multi-binding move results. Project
formatting changes zero files and fingerprints. Nine exact records are verified in
SQLite under runtime/diagnostics/scoped-record-nested/. The first attempted red
command selected zero tests because its filter was combined with --exact; the
corrected focused command demonstrates the failing native callback type mismatch.
Both records are retained with their actual outcomes.

This is an implementation checkpoint, not whole scoped-record feature acceptance.
Borrowed parameter and captured-carrier replacement, defaults and exception disposal,
owning/opaque composition, fallible callback contracts, editor/guide and matched
benchmark support remain open. Next test moving a callback field from normalized
source storage into a record literal, including nested aggregate transport. Direct
source dependencies are supported; physical types inside record-valued expression
construction require their own boundary evidence. Fifteen owner fixtures stay
untouched, and installed consumers remain f1dd284. No timing is claimed.


## Record construction and field-to-field transport

Native carrier planning now resolves record-valued expressions, including callback
fields moved into record literals and records placed inside nested aggregates.
The isolated record_value_carriers module resolves finalized owning storage types
without changing point-sensitive capture loan facts. The immutable collector holds
borrowed expression references, allowing overlapping parent and child facts without
cloning AST subtrees. Literal emission types describe their actual native fields;
conversion wrappers widen existing borrowed families instead of treating a family
value as one concrete callback origin.

Three native red witnesses are now green: a normalized callback field moved into
a record literal, a normalized record moved into a nested aggregate literal, and a
callback field moved between independent storage slots. They preserve callback
results, replacement behavior and capture release. The independent Node reference
agrees on their observable results. Focused editor coverage preserves declared
callback argument names, hover types and clean diagnostics after formatted literal
transport; internal BorrowedCallable names remain hidden.

All 42 scoped-record project tests, 64 depth canaries and the focused editor witness
pass. Formatting changes zero files and fingerprints. Thirteen exact records,
including the initial Rust compile repair, are imported and verified in SQLite
under runtime/diagnostics/scoped-record-aggregate/. No owner fixture or installed
consumer changes. No timing or whole-feature acceptance is claimed.

Continuation prompt: Continue after native record construction and field-to-field
transport. Start with a focused borrowed-parameter replacement witness; distinguish
capture-free replacement from an invalid escaping borrow. Then cover captured-carrier
replacement, defaults, exception/payload disposal and owning/opaque/fallible composition.
Complete the whole editor crate, guide examples and matched benchmark support before
record feature acceptance. Keep point-sensitive loan rejection, one evaluation,
independent disposal and public ownership rules. Do not reopen verified chunks.
Preserve fifteen owner fixtures, bounded checks, the shared nonincremental target,
project formatting, SQLite evidence and explicit Git paths. Installed consumers
remain f1dd284.
