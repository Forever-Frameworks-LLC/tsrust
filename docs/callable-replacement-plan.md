# Callable replacement implementation plan

## Current evidence

Continue from the gate 174 handoff and warning-cleanup checkpoint `5f7649e`. Preserve fifteen owner fixture edits. This family is not accepted. The existing native literal-guard witness fails in analysis with `callable-origin`, before Rust emission; it is not currently a generated-Rust type mismatch. A new runtime-selection/saved-snapshot witness has the same failure. Its independent Node reference produces `7 10`. Replacing a callee during its argument evaluation remains correctly rejected with `conflicting-access`, because invocation already borrows that storage. Node allows that expression; the accepted Tsrust ownership rules intentionally reject it.

The other original red witness concerns point-sensitive mutable factory environments. Its prototype state helpers were restricted to test builds during warning cleanup because production did not call them. Restore those helpers to production when integrating them, and remove the narrow dead-code expectation once production creates live states. Do not mistake cleanup for implementation of the pending behavior.

## Required implementation chunks

1. Define finite generic-origin storage independently of call signatures. Origins may have different arities; an invocation uses the point-sensitive reachable origin set and validates each reachable declaration. Keep saved values independent of later replacement. Use explicit variants with owned or borrowed environment payloads; matching signatures alone do not identify an environment. Retain source origins and concrete capture contracts through compaction and specialization.
2. Integrate assignment checking and final storage planning. Preserve existing loan, transferred-value, invocation-mode, capture, and disposal checks. Collect the full assignment family for native storage even when a branch is unreachable, while only reachable alternatives determine invocation validity. Do not replace this work with dead-code pruning. Avoid widening all calls to all historical assignments.
3. Emit finite native carriers and concrete invocation adapters. Each reachable alternative gets the checked specialization it needs. Inactive variants still have valid storage layouts even when their different-arity bodies have no invocation specialization. Preserve exactly-once receiver/argument evaluation, saved snapshots, old-environment disposal, result/error conversion, explicit clones, and no hidden shared cells. Isolate carrier planning and emission from recursive dispatch.
4. Integrate point-sensitive factory state. Allocation identity remains finite and stable; mutable capture values live in flow state, not mutable global allocation records. Seed new owned environments from checked capture facts, project updated state through invocation frames, preserve separate instances, and join uncertain branches conservatively. Include sequential invocation, recursion, loops, failure, consumption, and disposal. Missing ownership proof remains unknown rather than authorizing stale snapshots.
5. Finish editor, documentation and matched benchmark support, then accept the family. Add direct native assertions, independent Node references for equivalent cases, ownership rejection controls, full editor checks, whole compiler acceptance per owner request, all eight depth canaries after analyzer dispatch edits, and scoped benchmark verification. Append manifest rows at the end and retain exact SQLite evidence. Record measurement and platform limits separately.

## Acceptance matrix

- Literal false replacement and literal true replacement execute the original kept witness.
- Runtime branch replacement supports reachable different-origin calls with their correct argument counts and preserves the saved original value (`7 10`).
- Invalid reachable argument counts still produce source diagnostics with related origins.
- Mutating an actively borrowed callee during argument evaluation remains a source rejection.
- Captured callable replacement preserves borrowing/mutation/once contracts; old and saved environments dispose correctly without implicit clone.
- Two owned factory instances retain independent mutable state across repeated calls, restoration, failure, and recursive requests.
- Existing capture-write, factory-guard and recursion suites retain their intentional uncertainty boundaries or are amended only after stronger proof and native behavior are verified.
- Formatter round trips, editor assistance, complete executable documentation and benchmark integration agree with compiler behavior.

## Next step and handoff

Implement chunk 1 with a focused carrier-planning test before changing shared assignment dispatch. Inspect `deferred_generic_values.rs`, `closure_methods.rs`, callable storage/dispatch, and the reassign path in `analysis.rs`. The source-level cardinality engine already handles many captured writes; its facts must guide calls without erasing the full storage family. Integrate the mutable environment engine only after the origin/storage contract is explicit.

The added tests live in `tests/generic_call_capture_effects.rs`. One new positive test is intentionally red and one new loan rejection control passes. This document and those witnesses are a red checkpoint, not gate acceptance. No runtime consumers should be refreshed from an unaccepted compiler change.
