# Block and branch call contracts

Read AGENTS.md, docs/development-workflow.md, both generic value plans and
unused-generic-call-validation-plan.md first. This checkpoint follows 147371e and
red commit 74b5ecd. Full source validation, Gate173 and the broad goal remain open.

## Implementation

The dedicated generic_call_branch_flow module builds and propagates a source graph
iteratively. Blocks and if statements retain lexical binding identities, initializer
snapshots, definite direct assignments and abrupt completion. Branch-local calls
see their branch state; completing arms join common cardinality and declaration
provenance. Supported call facts are computed by graph propagation, avoiding the
per-call assignment reconstruction used by the earlier linear path.

This is metadata validation, not callable environment transport or runtime execution.
Distinct joined cardinalities currently become unknown. Loops, exceptional regions,
expression assignments, closures and lifted capture writes retain conservative
validation. These are remaining requirements, not intentional scope exclusions.

## Verification and owner edits

Three red cases now reject definite block replacement, branch-local invalid calls
and both-arm replacement. Self-assignment, lexical shadowing and early returns pass.
The separate branch-alias-contracts native/strict TypeScript pair asserts both branch
outcomes with integers and owned strings. The new editor test verifies source call
and current declaration locations.

Bounded evidence passes 84 adapter tests and 17 generic value tests, plus the new
paired native test separately and all 63 depth canaries: 165 unique compiler tests.
The whole editor run completes every suite: one Set documentation test initially
fails after owner formatting; its four-test suite then passes after canonical
format comparison. Aggregate editor coverage is 316 passing tests, retaining the
original failed log and supplemental repair. No routine whole compiler run or
performance timing was performed.

The owner authorized repairing only the ownership example in the newly edited
unused-alias-contracts.tsr. It now self-assigns before transferring into saved and
returns saved's result. Owner formatting remains; do not stage that file. Unrelated fixture edits remain dirty; their current inventory is Git status. Inspect Git status and preserve all
unrelated edits, including the previously clean fixture now edited by the owner.

## Remaining work and continuation prompt

Continue source-call validation from this checkpoint. Retain distinct candidate
contracts through joins instead of collapsing differing shapes to unknown. Validate
calls against reachable target contracts without fabricating generic identities.
Then implement loop fixed points with break/continue and post-test ordering,
exceptional exits/finally, expression evaluation order and captured write effects.
Finish stored-field validation, compatible concrete callable environment transport,
lifetime/context auditing and full Gate173 acceptance. Keep ownership rules and
saved callable snapshots intact. Use TDD, native disposal/assertion controls, strict
Node references, matched support, bounded checks and durable SQLite evidence.

Use the single warm target /private/tmp/tsrust-gate100-editor-target with
CARGO_INCREMENTAL=0. Refresh the five consumers atomically. Preserve owner edits and
stage explicit paths only. Keep the full goal active until all requirements are
proven, not merely until these source cardinality witnesses pass.

## Durable evidence and consumers

All five existing matched native/Node cases pass untimed with the same
116-observation workload family. Record
benchmarks/runtime/results/2026-10-07T03-19-54-872Z-verify.json and all closed
failed/repaired tsrust-callable-branch logs are imported into benchmarks/records.sqlite,
verified byte-for-byte and removed as loose files. SQLite quick_check passes.
Five executable consumers were atomically refreshed and SHA256-verified. Node and
matched scratch plus obsolete test executables were pruned. No process remains live.
