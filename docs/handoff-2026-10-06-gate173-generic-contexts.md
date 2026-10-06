Current continuation: [deferred data boundary](handoff-2026-10-06-gate173-generic-data-boundary.md).

# Handoff: contextual generic function values

Read AGENTS.md, docs/development-workflow.md, docs/nested-function-declarations-plan.md,
docs/generic-function-values-plan.md and the gate173 milestone. Inspect Git for this
checkpoint hash. Gate172 is accepted at d74f34d. Generic red witnesses landed at
43f8e82 after captured rest support at 2f4ad5d. Gate173 remains incomplete.

Preserve the four owner fixtures, untouched and uncommitted: comma-contexts/guards.tsr,
interfaces/shapes.tsr, map-for-each/values.tsr and map-group-by/documented.tsr.
Explicit Git paths only. One agent in this tree. Hourly checks remain paused.

## Implemented contextual construction

generic_function_values.rs is an analysis child module. Its iterative annotation
walker identifies parameters that depend on original type variables, including
arrays, applications, unions, tuples and callable syntax. Only those wrapper
parameters receive context; non-generic annotations and explicit access are kept.
The hidden call still invokes the original generic template, whose solver checks
argument/result relations. This is concrete contextual specialization, not an
unconstrained generic placeholder or a weakening of ordinary Type::Void checking.

Captured generic names now use local_function_values wrappers during lexical
resolution, before escape analysis. Bare unannotated generic data wrappers retain
an explicit pending diagnostic. Capture-free template values are constructed from
a concrete expected callable in contextual_generic_value, using the same wrapper.
Its synthetic origin is separate from parsed closure IDs and keyed by source span.

closure recognizes actual generated template wrappers by their single return call
and shared call/callee/source spans. It supplies a concrete contextual result to
the original generic call, so result-only inference works, including () => int[]
for an empty generic array factory. Existing wrapper projection filtering preserves
callable source facts. Editor signature help, inferred result hover and exact
source declaration navigation have direct checks.

The original fixed-context test passes. Native controls also cover capture-free
string/array values, mutable array arguments, result-only inference, explicitly
returned independent mutable factories, array callback context, incompatible type
relations, transferred owned arguments, consuming captures and once invocation.
The prior capture suite now keeps an unannotated generic negative and has positive
contextual array-callback evidence in the new suite.

## Ownership lessons and failed evidence

The initial context witness used a bare string parameter in a fixed Function type.
Existing callable rules require explicit data access; returning that owner needs
Owned<string>. A removal function returns a scalar while changing its array, so
its fixed argument contract needs Mutable<int[]>, not Owned<int[]>. Both draft
failures are retained; corrected tests preserve the intended transfer/mutation.

The first matched workload used bare callable annotations and expected its outer
counter to change. Those annotations own their scalar captures under existing
rules, so native correctly kept private state. This was an invalid parity domain,
not a compiler defect. The corrected workload uses Mutable<Function<...>> for
borrowed capture state. A separate native control proves borrowed calls change the
outer counter while owned values keep private state. Do not introduce shared cells
or change owned callable semantics to make an invalid Node comparison pass.

## Verified checks and remaining scope

The focused generic suite has six passing contextual tests and three genuine red
polymorphic tests. The complete suite run records both; it is intentionally not an
all-green acceptance claim. The outstanding tests are unannotated capture-free and
captured polymorphic calls and inferred returned generic factory environments.
Keep them. Contextual support must not become a narrower substitute for their
required final behavior.

Existing capture/default/rest checks pass 27. All eight dispatch canaries pass 63.
The complete editor crate passes 282 (151 intelligence), supplemented by the
strengthened exact-definition check. Seven strict TypeScript/Node-after-tsc positive
references pass; the native owned-versus-borrowed counter control intentionally has
no shared-cell JavaScript equivalence. Guide output is exactly 3, 7, 3, text on
separate lines. Five matched native/Node cases pass with seven observations per
iteration; all verification is untimed.

The generic_function_contexts family is appended at manifest end as text. Matched
sources, independent oracle, source mutation guards, registration, native ABI and
oracle/run/profile/report hooks are complete. implementationAccepted remains false
until full gate173. No sampling protocols or measurement counts change.

Next implement deferred generic data specialization and one retained environment
across all methods, following the physical generic-value plan. Do not merely allow
unknown inferred arrows or create a new environment for each type. Audit ownership
mode differences across specializations, aliases, storage, callback-valued arguments,
rest/default signatures, generic recursion and exact disposal. Then finish the
wider inference/context audit and meaningful named-function family milestone.

Use CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target with
CARGO_INCREMENTAL=0 and offline locked Cargo. Node is
/Users/seancotter/.nvm/versions/node/v24.11.1/bin/node. Refresh five consumers
atomically via .refresh-tmp then rename. Use --verify for routine benchmark checks.
SQLite is canonical; import/read back original logs and actual inputs before
removing loose files. Preserve failures. No archives or routine whole compiler
matrix. Prune executable scratch while preserving libraries and five consumers.

Canonical untimed record: benchmarks/runtime/results/2026-10-06T21-37-51-886Z-verify.json
in benchmarks/records.sqlite. All 114 compiler source fingerprints and oracle input
hashes match current source. Sample arrays are empty. Imported and verified exact
bytes for all 22 original logs/results and eight actual reference/guide inputs,
then removed loose copies. Failed access drafts, invalid owned-capture parity check
and three pending polymorphic witnesses remain durable evidence. Pruned 52
obsolete executable test outputs and native/example scratch; libraries and five
consumers remain. Tests may relink.
