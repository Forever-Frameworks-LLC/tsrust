# Generic parameter pattern audit checkpoint

Accepted behavior remains at 08c867a. Red audit witnesses are committed at c770b76.
Full Gate173 and the broader objective remain open.

Four unrelated owner fixture edits remain untouched: comma-contexts/guards.tsr,
interfaces/shapes.tsr, map-for-each/values.tsr and map-group-by/documented.tsr.
Never stage these files. Use explicit Git paths.

The audit adds native and strict TypeScript counterparts for callable payload
optional/default/rest contracts and owned array/record parameter patterns. The
callable payload witness passes, including default effects and spread forwarding.
The unused generic alias unknown-type control passes. Both pattern witnesses
originally fail generic solving because an unexpected void result constraint
conflicts with the correctly inferred int result.

A captured stack identifies initialize_parameter_pattern as the source. Synthetic
forwarding wrappers retain the source pattern# parameter name and replace its span
with the entire value span. The pattern initializer then mistakes the wrapper return
for destructuring setup and checks it using Type::Void. local_function_values.rs now
renames synthetic forwarding parameters to forwarded-pattern# while preserving
source pattern names in the actual function. No constraints are weakened. Temporary
tracing in generic_functions.rs and generic_inference.rs is removed.

This patch is not accepted. The focused record witness now passes. The array witness
progresses to a borrowed-place rejection when returning the string element from an
Owned<T[]> destructuring parameter. Investigate whether ordinary named owned array
patterns also have this defect. Preserve explicit ownership and do not introduce
implicit cloning or weaken the witness. The focused repaired run contains one pass
and one failure; no full adapter, editor, depth or benchmark run was performed.

All closed audit logs, including original reds, trace compilation failure, precise
stack trace and repaired pattern run, are imported into benchmarks/records.sqlite
and verified byte-for-byte before loose removal. SQLite quick_check passes. The warm
Cargo compiler binary contains the current unaccepted patch without tracing; the
five workspace consumers remain at accepted baseline. No process is live. Node
scratch directories may still exist at /private/tmp/tsrust-generic-pattern-node and
/private/tmp/tsrust-callable-default-rest-node. Reuse the single documented target
and CARGO_INCREMENTAL=0. Do not refresh consumers until the relevant build finishes.

Continue by repairing owned array destructuring transfer semantics with focused
controls, then cover editor signatures, matched workload/oracle/source registration,
guide and gate records. Run bounded adapter/generic suites, all eight depth canaries
and whole editor before acceptance. Do not claim full Gate173 completion: remaining
alias/editor, borrowed-result escape and unused-template validation audits still apply.

## Implementation checkpoint after the original red audit

The ordinary named Owned<string[]> pattern also fails, confirming a general array
pattern defect rather than an adapter-only defect. Its red witness is committed at
0be26f1. Current implementation introduces owned_array_patterns.rs, used only for
explicit consuming array parameters. Direct positions lower to remove with offsets
adjusted for previous extracted positions; skipped positions remain skipped. Default
positions guard removal with length and yield undefined when missing, retaining lazy
default evaluation. Explicit rest retains its existing independent-copy contract and
uses the adjusted offset. Nested extracted array holders track their own offsets.
Readonly parameters and ordinary array indexing retain existing borrow behavior.

Both adapter pattern witnesses now pass. The bounded batch passes 151 tests: all 63
adapter tests, destructuring/default suites and all eight depth suites. Additional
arrow, readonly rejection, transferred-source and nested array controls pass. Final
focused pattern run passes five tests. Whole editor regression passes before the
nested holder tracking extension; this is supporting evidence rather than acceptance
of the final source. No process remains live. All original red, compile-failure,
repaired, bounded and editor logs under /private/tmp/tsrust-owned-array-pattern-*.log
are imported and verified in SQLite, then removed. Five consumers are unchanged.

The implementation remains provisional. Complete disposal and failure controls,
record-to-array and rest-to-nested-pattern contexts, dedicated editor checks, strict
TypeScript references for the added ordinary witnesses, matched workload/oracle/hash
support and normal gate records. Recheck affected suites and whole editor on final
source before acceptance. Sequential remove can shift remaining elements; benchmark
support must expose that cost instead of claiming measured performance. No timing
was performed. Full Gate173 remains open.

Continuation prompt: Continue from this implementation checkpoint. Read AGENTS.md,
workflow and generic value plans. Finish owned parameter-pattern semantic coverage,
disposal/editor/Node/matched support, then bounded acceptance. Preserve the four
owner fixture edits and keep source acceptance separate from unmeasured workloads.
