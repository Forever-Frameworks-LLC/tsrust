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

Eight scoped_callable_records tests now pass: readonly repeated invocation, mutable
original capture updates and scope release, separate instances with record/field
moves, nullable present invocation, contextual arrows, absent/undefined/omitted
optional fields, nested record projection and three escape/conflict controls. Seven
positive source cases now explicitly pass after TypeScript compilation with Node
assertions. The reference extractor accepts multiline Rust calls and trailing commas;
its expected case count prevents silently skipping formatted fixtures.

Correction: the previous checkpoint's claim of five passing Node source counterparts
was incorrect. Its extractor missed formatted Rust calls and those later reference
runs failed. Use the current seven-case proof; original failed logs remain in SQLite.
An unrelated helper formatting change from recursive rustfmt traversal is removed.
Use skip_children when formatting Rust test files so support modules stay untouched.

Record layout unit checks and the repaired callable size budget
check pass. The final canary/traversal batch passes 70 tests after field-call dispatch
changes. The existing editor field hover/signature check passes. Whole-project
formatting changes zero files or fingerprints. These checks are implementation evidence, not full record acceptance.

Complete the following before accepting the record gate:

- Extend nullable and nested coverage to failure, mutable captures, conditional
  absence and nested transfers, including conflicting owner access while live.
- Audit closures that capture a scoped record, recursive layouts and arrays. Native
  captured environments must use named lifetimes, not unresolved signature placeholders.
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
