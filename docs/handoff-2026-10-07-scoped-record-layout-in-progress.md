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
