# Nested callable input red checkpoint

Read AGENTS.md, docs/development-workflow.md and docs/generic-value-adapters-plan.md.
Compiler source is unchanged from accepted checkpoint 3ccc6dc. Preserve four owner
fixture edits. New std:testing witness and strict TypeScript counterpart are ready.

## Nested callable input red checkpoint

The nested-callable-input fixture requires a fresh generic environment to acquire
Function<int, [action: Function<int, [value: int]>, value: int]>. It invokes the
same retained environment twice with a declared integer callback and observes
counter output 1,2 and return values 8,9. Strict TypeScript and Node pass. Tsrust
fails before invocation with unsupported: Generic argument is not a supported
value type, at make(). This is an unaccepted requirement, not working support.

The failure is in generic_inference::Types::annotation, which reconstructs data
annotations but has no callable branch. Fresh staging uses it for nested input
contracts, inferred result conversion, and selected factory signatures. Implement
one dedicated checked-type reconstruction helper shared by those paths. Preserve
full callable contracts: ownership and invocation kind, parameter names, access,
omission, rest, recursively nested results, nominal identities, arrays and unions.
The existing CallableShapes map only carries parameter/result types; it is not
sufficient to reconstruct contracts without losing metadata. Keep unresolved
deferred origins unknown. Preserve SharedRead as its shared wrapper rather than
silently converting it into an owned unique callable. Reuse existing callable
storage and normal compatibility checking; do not treat prototypes as wildcards
or introduce implicit clones.

Before acceptance, retain the original red witness, add access/ownership/default/
rest and nested-result controls, editor protocol facts and matched benchmark
support. Run affected checks and all depth canaries after dispatch changes, then
whole editor. Full Gate173 and the broader context audit remain open.

Exact red test and positive Node reference logs are imported into SQLite with
byte-exact readback; loose copies are removed. No source implementation or new
acceptance is claimed. All processes are terminal. Existing five consumers remain
on the accepted compiler source. Scratch Node output and obsolete test executables
were pruned. No timing or broad regression was run for this red-only checkpoint.

Continuation prompt: Continue Tsrust from
 docs/handoff-2026-10-06-nested-callable-input-red.md. Read AGENTS.md,
docs/development-workflow.md and docs/generic-value-adapters-plan.md first.
Keep the retained nested callable witness red until full checked contract
reconstruction is implemented. Preserve owner fixtures, ownership and prior cache/
editor/selection/inference repairs. Complete TDD, editor, matched support and exact
SQLite evidence. Full Gate173 remains open.
