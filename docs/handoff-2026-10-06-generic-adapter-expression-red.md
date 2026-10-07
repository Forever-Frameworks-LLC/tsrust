# Fresh generic expression adapter red checkpoint

Read AGENTS.md, docs/development-workflow.md and generic-value-adapters-plan.md.
Current accepted support is described in the owned-rest handoff. Gate173 and full
adapter acceptance remain open. Preserve four unrelated owner fixture edits.

A new native witness proves result-only and nullable named adapters already work:
a borrowed integer-array callable, direct string-array invocation, then owned
nullable callable all reuse one captured counter. No compiler repair was needed.

Fresh factory expression adaptation remains red. factory-expression.tsr requires
Function<int> = make(10) and map(make(20)); the original compatibility guard rejects
the first factory call result. Strict TypeScript output is 10,11,12,20,21,22. Each
factory must evaluate once and retain one environment across subsequent calls.
No compiler source changed. Original red and reference evidence is retained.

Next implement checked expression staging as described in the adapter plan. Bind
the evaluated environment temporarily, adapt its real origin with the existing
solver/wrapper machinery, and emit one scoped evaluation. Owned results transfer
that environment; traversal borrows last through the synchronous call; ephemeral
borrowed escapes must fail. Do not replay source checking/effects or reconstruct
factories. Audit expression traversal, loans, facts and emission. Add failure and
exact disposal controls, editor coverage, matched benchmark support and bounded
canaries/editor checks before acceptance.

Continuation prompt: Continue from this red checkpoint. Implement real staging
for fresh generic expressions, preserving single evaluation, capture identity and
ownership. Keep original factory-expression witness intact. Full Gate173 remains
open; preserve owner fixtures and exact SQLite evidence.
