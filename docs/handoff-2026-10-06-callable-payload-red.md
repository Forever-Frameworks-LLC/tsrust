# Generic callable payload red checkpoint

Read AGENTS.md, development workflow and generic adapter plans first. Accepted compiler HEAD is 6f6885a. Preserve all four owner fixtures. No compiler source changed in this checkpoint.

New retained callable-payload native and strict TypeScript witnesses pass a function itself through identity<T>(value:T):T in one captured factory environment. Two adapters must preserve the same counter (output 1,2) while returning distinct callbacks whose results are 8 and 9. TypeScript compiles strictly and Node passes. Tsrust fails with Generic type constraints do not match the checked argument type at the generic result T. This is unaccepted behavior. Exact failure and positive reference are stored in SQLite with readback; loose logs and generated Node output are removed. All processes are terminal.

## Implementation plan

First trace inferred parameter and contextual result representations. Callback and declared callable result Type IDs can denote compatible contracts but are not equal IDs. Generic constraint solving must preserve full recursive invocation ownership, access, omission and rest rather than compare shape-only signatures or treat callable prototypes as wildcards. Use existing normal compatibility validation at the boundary; retain an exact concrete substitution and runtime origin for each selected callable.

Next provide full checked callable annotations for generic substitution. Types::annotation currently supports data only, while the existing callable_shapes map loses ownership and metadata. Reuse checked_value_annotations through a complete annotation map or an explicit reconstruction callback in generic signature/materialization APIs. Supply it in both value adapters and normal generic preparation, not only the first failing path. Keep unresolved deferred origins unknown and preserve borrowed/shared distinctions.

Extend generic specialization source keys to callable annotations. Current generic_functions::Key explicitly rejects Kind::Callable. Its recursive key must distinguish invocation ownership, parameter access, omission, rest, result and nominal identities; no source spans or irrelevant parameter spelling. Keep checked origin/capture cache identities and limits unchanged. Never clone environments to make generic storage work.

Then retained witness must pass native execution. Add readonly/mutable/single-use, capture disposal/failure and differing-signature cache controls. Complete editor hover/help/navigation, guide, matched scenario expansion and guard/ABI registration, bounded compiler/depth/whole-editor verification, exact SQLite records and explicit-path acceptance commit. Full Gate173 remains open; current five consumers and 93-observation benchmark family stay on accepted source.

Continuation prompt: Continue Tsrust from docs/handoff-2026-10-06-callable-payload-red.md. Preserve owner fixtures. Implement full generic callable payload contracts from retained red witness, with ownership-aware constraint solving, complete substitution and specialization keys. TDD, editor, matched support and SQLite evidence. Gate173 remains open.
