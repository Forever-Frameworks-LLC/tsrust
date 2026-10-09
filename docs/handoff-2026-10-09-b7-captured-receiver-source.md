# B7 captured receiver source checkpoint

B7/gate 183 remains open. This is a source-interface checkpoint, not native feature acceptance. One new executable witness remains red. D2 discriminated unions remain a dedicated future feature. No owner question is pending. Read AGENTS.md, docs/development-workflow.md and docs/caller-borrowed-record-stores-plan.md first.

## Current interface

`Borrow` and `CapturedBorrow` now carry `source_projection`: direct binding, checked capture path or opaque environment. `retained_capture_projection.rs` isolates the representation, traversal and convergence guard. Capture paths use checked inner capture binding IDs, not parser indices or arbitrary foreign outer IDs. Unknown/cyclic paths become opaque, retain the receiver conservatively and never become direct inputs.

Call summaries map a foreign callee's hidden source through its checked receiver plus capture path. The receiver comes from `CallTarget::Closure(binding)`. A call's name expression can have no binding or type metadata; an initial attempt using that expression dropped the edge. Debug output is removed. Locally constructed calls preserve the previous direct construction mapping.

Invocation selects the corresponding nested capture loan path. Readonly data projections can retain their original data loan independently of a mutable effect-bearing installer receiver. Missing/opaque paths use conservative receiver retention. Mutable retained sources keep the existing receiver constraints. Source summaries copy projection metadata through ordinary arguments and captured inputs.

`closure_record_signature_emit.rs` explicitly leaves projected environments without a new native signature. Existing foreign receiver owner guards stay in place. `borrowed_owner_provenance.rs` only copies projection metadata into retained-summary views; it does not yet resolve projected concrete owners. Do not remove those guards or pretend a receiver marker identifies all of its captured data fields.

## Evidence

`captured_installer_receiver_retains_hidden_source_in_analysis` now reports the required capture conflict after both installer values leave scope. Its editor counterpart accepts the source-only safe shape and reports the mutation conflict. The initial source and native reds, failed mapping investigation and final source checks are retained in SQLite.

Eight affected integration suites pass 111 tests with only the new native witness excluded. Eight owner-provenance units, 27 capture-loan units, one projection convergence unit, the editor witness and eight depth canaries pass. The excluded native witness is run separately and remains red with missing replacement bounds and concrete-record versus record-view mismatches. No complete regression or timing run occurs.

The existing matched workload remains at 68 observations. Current emitted bytes exactly match SQLite correctness record `runtime/results/2026-10-09T22-44-14-386Z-verify.json`; no new runtime capability or verification run is claimed. Project formatting changes zero files/fingerprints on both passes. Five installed consumers remain B5. Fifteen unrelated fixtures remain untouched and uncommitted.

## Required next implementation

Finish `captured_installer_receivers_preserve_native_hidden_sources` end to end before accepting this form:

1. Select the nested captured data lifetime from the checked native environment layout. A captured receiver reference and its payload's capture lifetimes occupy different slots. Do not parse the first lifetime from a rendered field and assume it is the data lifetime.
2. Represent projected owner inputs distinctly. Two retained fields of one installer receiver can have different owners; a single `O<receiver binding>` marker is insufficient. Preserve named and closure-local owner scopes and complete source vectors.
3. Carry projected lifetimes/owner keys through closure method signatures, generic adapters, replacement conversion obligations, concrete invocation arguments and post-call reification. Resolve concrete captured owners only in a proved construction context.
4. Preserve the source projection diagnostics and conservative opaque behavior. Verify mutable receiver constraints, independent replacement, nullable/plain fields, multiple retained projections, exact effects and valid disposal.
5. Extend the matched workload and independent oracle/contracts only with the completed native form. Finish editor/guide/evidence and a source checkpoint. Keep B7 family acceptance and timing separate.

Keep the full compiler goal active. Other B7 plan rows and D2 remain pending. Reuse the shared Cargo target with incremental off, Node 24.11.1 and Homebrew Python. No live process remains at the source checkpoint.

## Continuation prompt

Continue Tsrust from this source checkpoint. Captured installer calls now transport explicit checked source projections and preserve source/editor borrow diagnostics. One native witness remains red. Implement projected native lifetimes and distinct projected owner keys through signature, invocation and reification, then verify the complete captured-receiver form with matched workloads. Preserve opaque guards, invocation contracts, unrelated fixtures and B5 consumers. Do not treat source acceptance as native acceptance or B7 completion.
