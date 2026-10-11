# B7 consuming stores and installer failures

B7/gate 183 remains in progress. This checkpoint follows `1ad9589`. Read AGENTS.md,
docs/development-workflow.md, docs/plans/caller-borrowed-record-stores-plan.md and
docs/b7-direct-callback-stores.md first. D2 discriminated unions remain a dedicated
feature. No owner question is pending.

The direct-store forwarding helper now handles owned callback contracts as well as
borrowed contracts. Consuming parameters resolve to an owned single-use contract.
The previous owned-contract skip left an abstract parameter in stored construction;
removing it enables ordinary ownership capture and native conversion. Single use
passes; a second record-field call reports partial-transfer. No implicit clone,
shared cell, boxing or lifetime cast was introduced.

Installer failure before the assignment preserves the previous callback. Failure
after assignment preserves the installed callback and mutable state. Catching that
failure does not release its retained mutable borrow. Holder disposal releases it.

## Evidence and outstanding red

SQLite keys use `tsrust-b7-store-`. `store-failures-red.log` records the consuming
baseline failure and two passing installer cases. `store-once-forwarding.log` repairs
the baseline. `store-failure-controls-final.log` has six passing consuming/completion
checks and seven existing direct-store checks. An earlier control expected the wrong
diagnostic; it correctly reports partial-transfer rather than closure-consumed.
`store-once-editor.log` checks result hover and consumed-field diagnostics.
`store-failure-depth.log` contains all eight passing canaries. Registration and whole
project formatting logs preserve source contracts, mutation guards and idempotence.

`directly_stored_callback_preserves_typed_failure_and_success` is intentionally red
in caller_callback_store_failures.rs. The callback throws a string, the caller catches
it with a bound error variable, then invokes the success path. It reports error-effect:
the catch contract remains unresolved. The independent Node reference passes. This
is the next defect to repair. Keep the bound catch and typed comparison; an untyped
catch would conceal the gap. Inspect error_effects.rs inferred callback origins,
record_field_origins and executable_callers, plus provisional handler inference.

The matched workload now has seventeen observations: the preceding fourteen plus
consuming storage, failure after installation and failure before installation. The
independent arithmetic oracle, source contract, mutation guards and manifest hashes
are updated. Untimed native/Node-after-tsc verification retains five cases per engine.
Measurement and family acceptance remain false. Exact sources and closed records
are read back from SQLite before deleting loose files. No full regressions or timing ran.
The final formatted-source verification is
`runtime/results/2026-10-09T18-37-15-544Z-verify.json`; the preceding pre-format
verification at `18-36-18-191Z` is also retained. Source evidence uses the
content-addressed `runtime/source-evidence/caller-store-failures/` prefix.

Continue the callback-thrown-error defect, then nullable stores, mutable data aliases,
exact replacement/clearing, branches, loops, early exits and disposal. Nested generic
callback environments and complex recursive retained-store summaries remain pending.
Preserve fifteen unrelated fixture edits; use explicit Git paths. Installed consumers
remain at B5. Reuse the shared target with incremental disabled, Node 24.11.1 first
on PATH, and Homebrew Python for SQLite.
