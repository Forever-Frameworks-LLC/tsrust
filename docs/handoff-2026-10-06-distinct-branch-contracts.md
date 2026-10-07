# Distinct branch callable contracts

Read AGENTS.md, docs/development-workflow.md, the generic function value plans and
docs/unused-generic-call-validation-plan.md first. The formatter checkpoint is
8698c99; distinct contract red witnesses are checkpoint e0d6413. Full Gate173 and
the broader language goal remain active and incomplete.

The new generic_call_contract_sets module retains finite sets of source signatures.
Branch joins, snapshots and repeated call facts preserve different generic/value
argument cardinalities. Equal shapes merge declaration provenance in canonical
source order. Every known reachable signature must accept a source call. Definite
assignment narrows the current set without changing saved aliases. Unknown target
paths remain conservative. These are metadata checks; runtime environment transport
is unchanged.

The initial editor run exposed lost equal-shape alternative declaration links.
The resolver now groups equal shapes and preserves both locations. Retain the
original failed evidence and the focused repair; completed editor groups are not
restarted. Remaining editor suites are resumed separately.

## Next checkpoint

Implement source loop fixed points with break/continue targets, update edges and
post-test ordering. Begin with genuinely failing loop-carried call witnesses and
positive zero-iteration, definite overwrite, saved-alias and shadowing controls.
Use finite contract sets, lexical identities and explicit completion edges. Do not
approximate a later textual write as a runtime sequence. Then cover exceptions and
finally, expression evaluation order and captured write effects. Stored field checks,
compatible concrete environment replacement and the full lifetime/context audit
remain required. Do not mark Gate173 accepted before auditing its full plan.

Use the single warm Cargo target with CARGO_INCREMENTAL=0. Keep formatted fixture
execution and all eight depth canaries for dispatch changes. Preserve every owner
fixture edit; stage explicit paths only. Five consumers are refreshed atomically.
Closed original logs and untimed verification records belong in benchmarks/records.sqlite,
verified byte-for-byte before their loose copies are removed. No timing was performed.

## Acceptance evidence

Focused checks pass 88 adapter tests, 17 generic function value tests and all 63
depth canaries: 168 unique compiler tests. Whole editor coverage is 318 passing
tests, combining the original passed groups and 179 intelligence tests, the focused
13-test source diagnostic repair and the resumed 115 tests. The new editor witness
also passes separately. Existing branch fixture passes strict TypeScript and Node.
Five matched native/Node correctness cases pass untimed; the final record is
benchmarks/runtime/results/2026-10-07T03-46-46-164Z-verify.json, stored exactly in
SQLite with current compiler source hashes. The previous passed record is retained
as separate evidence predating the provenance repair. All closed original logs are
verified in SQLite. Consumers and the installed CLI are refreshed; Gate173 is open.
