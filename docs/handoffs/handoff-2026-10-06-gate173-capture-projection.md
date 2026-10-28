# Handoff: generic lexical capture projection

Read AGENTS.md, docs/development-workflow.md, the previous
handoff-2026-10-06-gate173-deferred-capture-free.md and
plans/generic-function-values-plan.md. The previous implementation checkpoint is
0704364. Gate173 and the full language objective remain incomplete.

This checkpoint isolates named_capture_context::Contexts::source_context. Given
an original emitted declaration and defining Context, it returns typed captured
bindings and lexical BindingIds without allocating instances or adding loans.
Both that entry point and ordinary prepare use project_request. Keep ordinary
prepare indexed by its stored Request: specialization names do not necessarily
appear in the declared-name map. A focused control covers this invariant.

The new test was red because the API did not exist. Five focused projection and
named-capture tests now pass. No parser/analyzer dispatch behavior or supported
language surface changes. No new benchmark or timing is claimed; editor binaries
remain at the previous checkpoint. Four exact red/green/control logs are stored
and verified in benchmarks/records.sqlite under their original paths:
/private/tmp/tsrust-generic-source-context-{red,green,final,controls}.log (stored
paths omit /private/tmp/). The loose files are removed after readback.

Next implement captured deferred origins using this projection, then concrete
method replay with one retained environment and per-method read/mutate/consume
access. Do not select a dummy T, clone captures or introduce shared cells.
The five original captured native witnesses remain pending. Continue the previous
handoff's result-context, callback/storage, editor and wider gate173 audit scope.
No feature acceptance or completed family milestone is claimed by this checkpoint.

The four owner fixtures remain untouched and uncommitted. All tool handles are
terminal at handoff. Only the newly built unit-test executable is pruned; library
caches and the five existing executable consumers are retained. Hourly checks
remain paused.

## Continuation prompt

Continue Tsrust from docs/handoffs/handoff-2026-10-06-gate173-capture-projection.md.
Read AGENTS.md, docs/development-workflow.md and the generic function values plan.
Implement captured deferred generic values, preserving one environment, lexical
identity and existing ownership rules. Retain all five red native witnesses and
complete editor and benchmark support before acceptance. Preserve the four owner
fixture edits. Use explicit Git paths, bounded checks and the single Cargo target.
