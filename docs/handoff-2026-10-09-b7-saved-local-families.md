# B7 saved-local family checkpoint

B7/gate 183 remains open. Conditional callback families can now be saved, invoked locally, moved into another local and stored through record parameters. D2 discriminated unions remain a dedicated feature. No owner decision is pending.

## Implementation

Retained-borrow summaries now receive the checked storage registry and collect all origins of a saved borrowed family. The original red failed with undeclared capture lifetimes because the summary omitted those origins. Repairing provenance exposed the separate native-carrier defect: a generic local initializer tried to construct a family already expanded into every concrete caller owner or callback capture type.

`local_borrowed_family_emit.rs` owns local representation and conversion. Each local family has one untagged payload per checked source origin. The shared borrowed-family emitter supplies dispatch and typed results for both local and final representations, emitting local definitions only when needed by a local binding. Record storage matches the existing local value, moves the selected payload and attaches an owner marker when needed. Nullable and plain fields use their correct destination types. Replacement bounds expand source origins when either owner or callback-capture alternatives require it. No initializer is replayed, capture cloned, source contract widened or lifetime cast added.

The generic-input probe also exposed expansion without owner markers; it is covered by the same separation. Its first log used a test name mentioning typed failure, but the actual source checked generic inputs and a local move. The test was renamed accurately. Do not cite that probe as new fallible-local-family coverage.

## Verified

All 90 integration cases across eight affected suites pass on the final source. All 38 borrowed/provenance unit checks, the saved-family editor witness and eight depth canaries pass. Four new tests cover readonly and mutable invocation before storage, nullable and plain destinations, exact branch effects, a local move, generic callback inputs, old-owner reuse and new-owner rejection. The existing callee-local escape test also checks its saved-family form. Formatter round trips remain in the shared helpers.

The matched workload retains 37 observations. Its conditional installer now invokes the selected local before storing it and adds that result to the effect observation. The independent expected total is `329 + 2 * value`. The source contract and mutation guards require local invocation and storage. A readonly local family uses a matching readonly destination; the earlier mutable destination was rejected by the existing declared-contract rule. Mutable local families are covered separately without changing that rule.

`runtime/results/2026-10-09T20-37-33-937Z-verify.json` verifies five cases on native and Node-after-tsc with empty timing arrays. SQLite preserves the result, 566 exact source/input records under `runtime/source-evidence/saved-local-families/` and 17 logs prefixed `tsrust-b7-local-family-`. Every import was read back byte for byte. Compiler input hashes and unmodified/instrumented emitted hashes match. Original failed checks remain preserved. Five disposable workload artifacts totaling 887374 bytes were removed after preservation. Whole-project formatting changed one workload/fingerprint, then zero; the final formatting passes changed zero. No whole-crate regression or timing ran.

## Next work

Continue the B7 plan with closure-body caller provenance and installer function values. Those contexts still suppress owner specialization when concrete provenance is incomplete. Probe each with focused source/native witnesses rather than assuming local-family support proves it. Then cover field/index inputs, computed owner temporaries, retained loops, early exits, disposal, recursive stores and nested generic environments. Fallible, consuming and more complex local-family composition need dedicated witnesses before making broader claims.

Keep the source ownership guards, old-owner release and retained-new-owner rejection controls. B7 family acceptance still requires full regressions, matched timing review and consumer refresh. Five installed consumers remain at B5. Preserve the fifteen unrelated fixtures, use explicit Git paths, one Cargo target, disabled incremental compilation, Node 24.11.1 first on PATH and Homebrew Python for SQLite. No process remains live after this checkpoint.

## Continuation prompt

Continue Tsrust from this checkpoint. Read AGENTS.md, docs/development-workflow.md and docs/caller-borrowed-record-stores-plan.md. B7 saved-local conditional families now preserve precise local payloads separately from final record carriers, including generic callback inputs and local moves. Add focused witnesses for closure-body callers and installer function values, then implement complete concrete provenance without weakening source ownership. Continue the full B7 plan before its family milestone. Keep D2 separate, consumers at B5 and the full compiler goal active.
