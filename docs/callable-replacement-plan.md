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

## Carrier primitive checkpoint

`src/polymorphic_callable_storage.rs` now implements canonical finite origin membership, declaration of an enum with independently typed environment payloads, and checked variant injection. Storage does not impose one invocation arity. Unknown origins, empty families, unmaterialized templates, and monomorphic descriptors are rejected. Source analysis and native declaration placement are not integrated yet; the module is deliberately test-build-only until those integration points exist.

Four focused tests pass after three initial red tests. Native Rust witnesses prove exactly-once expression evaluation, moving non-clone payloads into and between variants, saved-value independence, disposal of replaced storage exactly once, and explicit independent borrowed lifetimes with mutable access. This proves the carrier primitives, not source replacement or generic invocation dispatch. Exact red/green logs and the current source witness/reference are stored in SQLite under `runtime/diagnostics/polymorphic-storage-primitives-2026-10-07/`.

The saved-value source witness now explicitly reinitializes `action` after transferring it into `saved`. This respects the existing ownership model rather than relying on an implicit copy. Node still produces `7 10`; Tsrust rejects even the same-declaration reinitialization because it creates a distinct environment origin. The argument-time replacement rejection still passes. Formatting changes zero files and fingerprints.

Next, connect per-binding assignment-family discovery and stable origin IDs to these primitives, preserving point-sensitive call facts separately. Make the module available to production only when analysis and emitter consume it. Then add invocation adapters and pass both replacement witnesses before proceeding to mutable factory state. This is an implementation checkpoint within an open family, not a completed gate; full acceptance checks and workload/editor support remain owed.

## Replacement registry checkpoint

The carrier module now includes a monotone registry for storage connectivity. Assigning across origins joins their finite families transitively and deterministically without joining unrelated components. Membership does not claim that all stored origins reach every invocation. Compaction preserves separate origin identity and rejects missing mappings or accidental merges; it never substitutes an unknown variant for a lost environment.

Six focused tests pass, including the previous native ownership/lifetime tests and new transitive-connectivity and compaction controls. The exact log is in SQLite at `runtime/diagnostics/polymorphic-storage-registry-2026-10-07/tsrust-polymorphic-registry.log`. The registry is not yet connected to source assignment analysis or production emission. No source-level witness is claimed fixed. The next step remains recording checked assignment pairs, retaining and remapping their origins through analysis, and teaching emission to use the family while invocation checking preserves point-sensitive facts.

## Fresh declaration reassignment checkpoint

A fresh reference to the same capture-free generic declaration can now reinitialize an existing binding using its established empty native representation. This does not merge captured environments or convert an existing differently typed alias. The RHS is checked first and must be a fresh closure node; both source declarations must match and both checked capture lists and capture identities must be empty. Existing moves, invocation loans, and callback contracts still apply. No value or captured owner is cloned.

The native witness transfers `action` into `saved`, reconstructs `action` from `identity`, invokes it at integer and string specializations, replaces it again, and produces `3 7 owned` followed by `9`. The independent Node reference agrees. A captured generic declaration remains outside this representation reuse: its replacement is rejected with related environment origins. This checkpoint also fixes its previously misleading arrow-parameter inference error by retaining deferred generic checking when the contextual target is itself a deferred generic environment. A transferred saved alias remains rejected with `closure-consumed`.

The capture-effects integration suite now has 20 passing tests and the two retained distinct-declaration replacement witnesses still failing. All eight depth canaries pass. A new editor witness verifies diagnostics, integer signature help, string result hover, and the captured-origin rejection. The whole editor crate passes 340 tests; adjacent generic function-value and adapter suites pass 105 tests. Workspace all-targets checking reports zero compiler diagnostics. Exact logs are retained in SQLite under runtime/diagnostics/fresh-generic-reassignment-2026-10-07/. No whole compiler acceptance or performance measurement is claimed; this family remains open and its matched workload registration remains owed before acceptance. Project formatting changes zero files and fingerprints. Live consumers remain at the last accepted compiler checkpoint.

Next, use the family registry for distinct-declaration storage. Do not generalize empty-environment representation reuse to different function bodies or captured payloads. Point-sensitive calls and native adapters are still required for the two source witnesses; independent factory state remains a separate integration requirement within the same family.

## Production origin tracking checkpoint

The registry is now part of `Analyzer` and `CheckedProgram`. Concrete checked reassignments record their original deferred-generic environment pair before empty-environment representation reuse. Effects-only passes, result discovery, and symbolic template bodies do not publish provisional pairs. Closure compaction remaps the registry with the same origin map as expressions and signatures. Final analysis validates retained origins against checked polymorphic environment descriptors. Recording membership does not relax replacement, invocation, loan, or transfer rules.

Carrier declaration and injection remain test-only until native emission consumes them. Production tracking is verified by source tests that retain three fresh origins while the receiver keeps its established native type, exclude assignments from unused templates, and preserve concrete receiver membership through specialization and template compaction. The carrier suite passes nine tests; adjacent function-value/adapter suites pass 105 tests; the capture-effects suite retains 20 passes and the same two distinct-declaration failures. All eight depth canaries pass. Forty focused generic editor tests pass and all-targets workspace checking reports zero compiler diagnostics. Project formatting changes zero files and fingerprints. Exact checkpoint evidence is retained under `runtime/diagnostics/polymorphic-origin-analysis-2026-10-07/`.

Next, implement per-invocation reachable-origin planning and native family selection using this retained registry. A method index currently belongs to one closure origin; dispatch must retain that pairing rather than applying one variant's method number to another variant. Calls must preserve argument cardinality, access, results, failure conversion, and exactly-once receiver/argument evaluation. Do not enable distinct-origin assignments until the checked invocation plan and native representation agree. Family acceptance, matched workload support, complete compiler/editor regressions, measurements, and mutable factory state remain open.
