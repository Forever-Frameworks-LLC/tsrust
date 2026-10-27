# Optional and selected generic calls checkpoint

Read AGENTS.md, docs/development-workflow.md and both generic value plans first.
This accepts focused optional/selected generic invocation after red 3b0ff7f and
preparatory 17f6bdc. Full Gate173 and the broader thread objective remain open.

## Behavior and implementation

Deferred generic call checking accepts either a named binding or a selected
expression receiver. Both paths retain the same original declaration, projected
capture context, checked methods and finite cache limits. Selected expressions use
normal receiver expression checking and real loan roots instead of a fabricated
binding. Already checked fresh receivers retain their checked type without
rechecking transferred factory arguments. Method receiver access remains separate
from runtime environment identity.

Conditional call dispatch now forwards expected result context. Optional result-only
inference keeps the complete nullable expectation until optional_chain supplies its
present branch's payload. A dedicated fresh nullable factory hint helper prepares
omitted named results before conditional hint joining; it does not execute source
arguments or construct runtime environments. Void remains a genuine unsupported
union member. Parser and source metadata fixes from 17f6bdc preserve ?. and explicit
arguments throughout this process.

Dynamic absent calls skip argument effects. Present calls retain counters across
explicit applications. Failures preserve reusable state at the point reached.
Fresh consuming environments can transfer captures, while named optional selections
still reject consuming borrowed owners. Explicit type/arity and nested receiver loan
violations remain errors, including typechecking absent branches.

Compiler expression facts report the complete optional result. Editor generic method
metadata described only the present method and conflicted with that complete type at
one source span. Initializer inference now gives the checked expression precedence;
parameter help keeps the method contract. Original failing editor/trace evidence is
retained, and all temporary tracing is removed. Do not discard same-span facts
indiscriminately; both facts have distinct responsibilities.

## Verification and evidence

The original aliases/fresh witnesses pass, with the retained fixture upgraded to
function-selected dynamic presence. Strict TypeScript/Node passes 1,2 and assertions.
Additional red controls exposed transferred-argument rechecking and lost result-only
context; repaired controls pass. Runtime failure/reuse, type/arity/loan, monomorphic
explicit rejection and consuming ownership controls pass. O0/O3 disposal observes
three original records, exact count 3 and identity mask 14, across success/failure.

All 58 adapter tests pass in the bounded batch. The added consuming-capture control
passes separately, covering 59 unique tests; the expanded monomorphic negative
control also passes separately. Seventeen generic value and two signature controls
pass. All 63 depth canaries pass. Whole editor crate passes 310 tests. No whole
compiler regression or timing was run.

Matched sources now observe 106 values per scenario, preserving the previous 101.
Independent oracle, hashes, semantic guards and registration pass. All five native/
Node-after-tsc cases pass untimed. Exact record
benchmarks/runtime/results/2026-10-07T02-10-49-964Z-verify.json and all failed/repaired
logs under /private/tmp/tsrust-optional-selected-*.log are imported and verified
byte-for-byte in benchmarks/records.sqlite. Loose records and generated Node/matched
outputs are removed. SQLite quick_check passes. Earlier original red evidence under
/private/tmp/tsrust-optional-explicit-*.log remains in SQLite.

Five executable consumers are refreshed atomically with SHA256 verification. Keep
CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target and CARGO_INCREMENTAL=0.
Obsolete test executables are pruned; retained libraries and five consumers remain.
No process is live at checkpoint.

## Remaining audit and continuation

Full Gate173 is not accepted. Continue parameter-pattern contexts, nested callable
optional/default/rest payloads, alias/editor contexts, borrowed result escapes and
unused generic source-validation controls. Field explicit applications remain a
separate syntax/type-contract boundary: do not silently apply type arguments to a
monomorphic stored callback. Keep one environment per factory and explicit ownership.
The broad generic workload implementationAccepted flag remains false.

Four owner fixture edits remain uncommitted and untouched: comma-contexts/guards.tsr,
interfaces/shapes.tsr, map-for-each/values.tsr and map-group-by/documented.tsr. Never
stage them. Use explicit Git paths.

Continue Tsrust from docs/handoffs/handoff-2026-10-06-optional-generic-calls-accepted.md. Read
AGENTS.md, workflow and both generic value plans. Optional/selected generic calls are
accepted; full Gate173 remains open. Audit remaining patterns, nested callable
contracts, alias/editor and borrowed escape/source-validation requirements with red
witnesses. Preserve owner fixtures, environment identity and specialization bounds.
Complete native/Node/editor/matched support, bounded checks and SQLite evidence.
