# Nested selected generic factories checkpoint

Read AGENTS.md, docs/development-workflow.md and docs/generic-value-adapters-plan.md.
Preserve four owner fixture edits. Stage explicit paths. Full Gate173 and adapter
acceptance remain open.

Selected factory lowering now walks nested callees iteratively. A candidate AST
is distributed into conditional calls; every leaf must resolve to a deferred
generic factory before the original expression is replaced. Traversal obtains its
concrete contract from a factory leaf and checks every selected branch against it.
Predicates retain order and short-circuit behavior. Only selected invocation
arguments execute, once. No recursive lowering frame or relaxed callable origin
requirement was added. Prior owning storage, captured-type cache, partial editor
metadata and temporary staging-loan repairs remain intact.

Twenty-five adapter tests and all eight depth canaries (63 tests) pass on this
source. Whole editor passes 303 tests; its selected factory protocol control now
checks both the original and nested fixtures. Strict TypeScript and Node preserve
the original output and nested evaluation assertions. Matched workload expansion
has seventy-six observations; registration mutations and five native/Node-after-tsc
cases pass untimed. Canonical SQLite record:
benchmarks/runtime/results/2026-10-07T00-50-57-473Z-verify.json.
Original red and intermediate compile failures remain exact SQLite records. Five
consumers were refreshed atomically and SHA-256 verified. No timing or full Gate173
acceptance is claimed.

Next audit heterogeneous inferred result joins and fresh aggregate destinations.
Continue omitted generic results, nested callable contracts, parameter patterns
and remaining ownership/nullable contexts in the full plan. Treat observed passing
cases as evidence, not proof of the full audit. Start defects with focused red
witnesses and complete matched support as they land. Defer timing until the family
scope is complete. No live processes remain. SQLite records were read back before
loose cleanup; obsolete test/native outputs were pruned while consumers remain.

Continuation prompt: Continue Tsrust from
 docs/handoff-2026-10-06-nested-selected-generic-factories.md. Read AGENTS.md,
docs/development-workflow.md and docs/generic-value-adapters-plan.md first.
Preserve owner fixtures, prior ownership/cache/editor fixes and nested selection
ordering. Audit remaining adapter contexts with TDD, std:testing, editor and matched
untimed support. Keep exact SQLite evidence. Full Gate173 remains open.
