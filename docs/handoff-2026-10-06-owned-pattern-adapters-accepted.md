# Owned parameter-pattern adapters accepted

Read AGENTS.md, docs/development-workflow.md, docs/generic-value-adapters-plan.md,
docs/generic-function-values-plan.md and the nested function plan first. This accepts
focused pattern/callable-payload behavior after implementation checkpoint bcf5114.
Full Gate173 and the broader thread objective remain open.

Four owner fixtures remain uncommitted and untouched: comma-contexts/guards.tsr,
interfaces/shapes.tsr, map-for-each/values.tsr and map-group-by/documented.tsr.
Never stage them. Use explicit Git paths. One agent works in this tree at a time.

## Behavior and architecture

Forwarding wrappers use private forwarded-pattern names instead of source pattern#
names. Actual source functions retain their destructuring bodies and initialization
boundaries. This prevents a wrapper return from being checked as void setup without
weakening generic constraints. Pattern adapter concrete results and parameter help
remain consistent with the original declaration.

Explicit consuming parameter patterns transfer array elements via remove, preserving
skipped positions and adjusting subsequent indices. Default positions check length
before removal and yield undefined when missing, preserving lazy default evaluation.
Nested holders track separate offsets. The helper follows record/array mixtures and
nested rest holders through their actual checked field/element types. Explicit rest
retains independent-copy semantics. Readonly patterns and ordinary indexing borrow;
required missing positions remain range errors. No implicit clones are introduced.
Sequential remove can shift remaining elements; performance has not been measured.

Call expression statements may discard returned callable values. The existing
restriction on other unbound closure statement forms remains; do not claim universal
expression-statement parity. Discarding does not duplicate environments or restore
consumed captures. Callable payload optional/default/rest contracts preserve default
effects and spread forwarding.

## Verification and evidence

Final bounded compiler batch passes 159 tests, including all 68 generic adapter tests,
destructuring/default suites and all eight depth canaries (63 tests). An additional
expanded owned-pattern fixture check passes after adding explicit undefined selection.
Strict TypeScript/Node matches the successful ordinary pattern fixture. Native range
failures and ownership constraints are tested separately; JavaScript missing required
array positions are not claimed to have native range semantics.

O0/O3 pattern disposal observes three original records, exact count 3 and identity
mask 14, across success and short-array failure. Discarded factory results drop their
original captured record once at both optimization levels. Dedicated editor result
hover and parameter help pass; final whole editor passes 311 tests.

The workload pair retains the previous 106 observations and adds pattern/counter and
callable payload values for 116 observations per scenario. Oracle, fingerprints,
mutation guards, registrations and five native/Node-after-tsc cases pass untimed.
The original 112-observation run and failed expanded run are preserved. Exact final
record benchmarks/runtime/results/2026-10-07T02-33-38-582Z-verify.json, initial record
2026-10-07T02-30-53-632Z-verify.json and all tsrust-mixed-pattern logs are imported into
benchmarks/records.sqlite, read back byte-for-byte and removed as loose files.
SQLite quick_check passes. No whole compiler regression or timing was performed.

Initial disposal source lacked a nominal record type in a deferred generic call.
Typed array disposal passes; separate concrete adapter inline-record-array context
passes. Do not misreport this as an unfixed concrete adapter inference defect.

Five consumers are refreshed atomically with SHA256 verification. Reuse
CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target and CARGO_INCREMENTAL=0.
No process remains live. Prune obsolete generated executables and matched/Node scratch
outputs before handoff; retained libraries and five consumers remain.

## Remaining work and continuation prompt

Full Gate173 remains open. Continue alias/editor, borrowed-result escape, generic
field contracts and unused-template source-validation audits. Keep existing ownership,
finite specialization bounds and environment identity. Do not turn monomorphic stored
callbacks into generic functions or weaken constraints to make witnesses pass.

Continue Tsrust from this handoff. Read AGENTS.md, workflow and both generic value
plans. Focused owned parameter patterns and callable payload/default/rest behavior
are accepted. Audit remaining Gate173 requirements with red witnesses, complete
native/Node/editor/matched support and bounded checks, store evidence in SQLite,
then commit accepted chunks. Preserve four owner fixtures. No routine timing or full
compiler regressions before the meaningful feature-family milestone.
