# Handoff: generic value adapter witnesses

Read AGENTS.md, docs/development-workflow.md, both function plans and
docs/plans/generic-value-adapters-plan.md. Previous checkpoint: 0b11cf4. Gate173 remains
open. This checkpoint changes tests and planning only, not compiler implementation.

generic_value_adapters.rs has three genuine red native witnesses with strict
TypeScript counterparts. Explicit borrowed/owned adaptation and record storage
fail at deferred_generic_requirement. Map callback adaptation fails because
materialize_closure_method cannot replay a deferred prototype. Return values are
asserted through std:testing; captured counters observe environment identity.

TypeScript references pass after strict tsc compilation with ES2022, CommonJS
and the installed node type roots. contracts and callbacks print exactly 11, 12,
13; storage prints 11, 21, 22. The native three-test command fails all three at the
expected boundaries. Original logs are retained in benchmarks/records.sqlite as
tsrust-generic-adapters-red.log and tsrust-generic-adapters-reference.log, with
exact-byte readback before removing loose copies. Generated reference JavaScript
and obsolete native test executables are removed. All processes are terminal.

Implement adapters around the existing value environment. The dedicated plan
identifies expression/compatibility dispatch, source solving, synthetic escape
classification, callback replay and native emission seams. Borrowed adapters keep
existing loans; owned adapters transfer once. Do not rebuild the original capture
environment, clone owners or use void as a compatibility wildcard. Keep the current
guard until an actual adapter replaces its rejected path.

Next pass the concrete contract witness plus ownership rejection controls, then
the callback and storage witnesses, defaults/rest/patterns, caught failures and
exact disposal. Complete editor/guide and matched benchmark support with bounded
checks and SQLite evidence. Dispatch changes require the eight depth canaries;
acceptance requires the whole editor crate. Preserve the wider gate173 audit.

Four unrelated owner fixtures remain untouched and uncommitted. Five consumers
remain from 0b11cf4; no implementation refresh is required for this red checkpoint.
Hourly checks remain paused.

Continuation prompt: Continue Tsrust from this handoff and plans/generic-value-adapters-plan.md.
Implement real adapters from existing generic value environments, preserving
borrowing, transfer and shared state. Make the three red witnesses pass, add
ownership/disposal/editor/benchmark evidence, then continue the full gate173 audit.
Preserve owner fixtures and explicit Git paths. Do not mark gate173 complete early.
