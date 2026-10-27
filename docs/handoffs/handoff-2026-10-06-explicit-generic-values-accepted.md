# Explicit generic value applications checkpoint

Read AGENTS.md, docs/development-workflow.md and both generic value plans. This
checkpoint follows red commit 1f81feb. Explicit applications on named retained
generic values now preserve supplied arguments and captured environment identity.
Full Gate173 and the thread objective remain open.

## Implementation and evidence

Generic Cache retains lexical value-call annotations by enclosing concrete function
name and original call span. Source drain substitutes enclosing bindings rather
than rejecting lexical aliases prematurely. Checked preparation rejects nongeneric
values, requests the original explicit specialization with projected capture types,
and publishes it through the same dedicated helper used by inferred calls. Normal
argument/result/ownership checking remains responsible for compatibility. Explicit
rest calls pack arguments before specialization. Deferred method keys now include
checked generic application arguments as well as argument and result types, keeping
unused explicit type arguments distinct without separating equivalent instances.

The original callable payload red witness passes native execution. Strict
TypeScript/Node reference passes counter output 1,2 and returned results 8,9.
Controls cover scalar/string signatures sharing one captured counter, enclosing
generic substitutions, result-only arguments, defaults/rest, wrong counts/types,
same-T conflicts, nongeneric aliases, transferred argument reuse and consuming
capture reuse. Fifty adapter tests pass as a batch; the additional consuming-capture
control and modified transfer control pass separately, covering 51 unique tests.
Seventeen generic value controls pass. Four cache/layout controls pass, including
unused explicit type arguments and equivalent instance reuse. All 63 depth
canaries pass on final compiler source.

The whole editor crate covers 309 unique passing tests. Preserve the original run:
171 intelligence tests passed and one existing assertion failed because an overly
broad test edit changed its expected navigation token. That edit was restored;
the affected test passes. Unrun suites were resumed and pass. The new explicit-call
test verifies concrete help/hover and navigation to the local alias. No production
editor workaround was introduced.

Matched support now has 101 observations per scenario. Explicit scalar/string and
subsequent inferred boolean calls observe the same factory state. Independent
oracle, fingerprints, semantic guards and registration pass. All five native/Node-
after-tsc cases pass untimed. Exact record
benchmarks/runtime/results/2026-10-07T01-56-31-516Z-verify.json is stored byte-for-byte
in benchmarks/records.sqlite. Original failures and repaired/final logs use
/private/tmp/tsrust-explicit-callable-*.log record paths. Closed loose records are
removed and SQLite quick_check passes. No timing or full compiler regression.

Five consumers are refreshed atomically with hash verification. Continue using
CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target and CARGO_INCREMENTAL=0.
Obsolete test executables and closed generated matched outputs are pruned; retained
libraries and five consumers remain. No processes are live at checkpoint.

## Next audit

Continue full Gate173 requirements. Field/optional generic applications, parameter
patterns, nested default/rest callable payloads, borrowed-result escapes and
remaining alias/editor contexts need focused positive/negative witnesses. Validate
unused generic declaration annotations under the original source-validation policy;
do not let late lexical resolution silently weaken that policy. Keep ownership,
source identity, specialization bounds and one environment per factory invariant.
The broad workload implementationAccepted flag remains false.

Four owner fixture edits remain uncommitted and untouched: comma-contexts/guards.tsr,
interfaces/shapes.tsr, map-for-each/values.tsr and map-group-by/documented.tsr. Never
stage them; use explicit Git paths.

## Continuation prompt

Continue Tsrust from docs/handoffs/handoff-2026-10-06-explicit-generic-values-accepted.md. Read
AGENTS.md, workflow and both generic value plans. Explicit named generic value
applications are accepted; full Gate173 remains open. Audit field/optional contexts,
parameter patterns, callable defaults/rest, borrowed escapes and unused source
validation with red witnesses. Preserve four owner fixtures. Complete native/Node,
editor, matched support, bounded depth checks, SQLite evidence and checkpoint commits.
