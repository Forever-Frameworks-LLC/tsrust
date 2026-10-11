# B7 forwarded installer source-effects checkpoint

B7/gate 183 remains open. This checkpoint repairs source analysis and editor diagnostics for named installers invoked through function values. It does not accept their native implementation. The previous saved-local-family checkpoint remains accepted at `8f9a7c34`. D2 remains a separate dedicated feature. No owner decision is pending.

## Source repair

`forward_call` checked and specialized arguments but omitted the retained-borrow and complete-replacement effects applied by named invocation. It now releases overwritten captures and retains the new caller-owned captures after specialization. Source borrow summaries, record replacement propagation and release replay follow `ForwardFunction` edges alongside `Function` edges. This also covers a named relay whose body invokes a saved installer.

The focused source witness checks both direct forwarded invocation and a relay containing it. The old owner is reusable after replacement. Conflicting access to the replacement owner is rejected while the holder remains live. Temporarily removing forwarded retention makes the direct case fail because analysis accepts the conflicting access; the original source was restored in memory before continuing. The editor witness verifies the same diagnostics and nullable readonly hover.

## Verification and kept reds

The eight affected integration suites pass 91 cases with precisely the two newly added native witnesses excluded. The editor witness and all eight depth canaries pass. Whole-project formatting changes zero files on both passes. No whole-crate regression, workload verification or timing ran in this source-only chunk. New matched workload support remains owed with native implementation.

The two executable witnesses remain red on current source:

- `closure_body_installer_releases_independent_input_owners`: a concrete arrow wrapper calls the named installer. Its generated method lacks the record replacement `From<Closure0<...>>` bound and takes a concrete record instead of the caller's record view.
- `installer_function_value_releases_independent_input_owners`: a saved named installer has the same missing replacement bound in its forwarded method. Caller analysis now knows the correct retained payload; generated storage still holds the old owner lifetime and Rust rejects its reuse. This is a native representation defect, not permission to weaken release analysis.

Logs and a concise evidence record are preserved in SQLite, along with exact changed inputs under `runtime/source-evidence/forwarded-installer-source/`. Keep the failed native evidence. No consumer refresh occurred; five consumers remain at B5. Fifteen unrelated fixtures remain uncommitted and unchanged. No process remains live at this checkpoint.

## Next implementation

Complete the native forwarded-call interface before enabling concrete owner specialization for function values. Named function headers already assemble record views, replacement bounds, retained-input lifetimes, generic callback bounds and owner marker parameters in `emit.rs` around the function-plan loop. Extract this into a reusable, isolated signature builder rather than independently recreating the rules in closure forwarding. Forwarded method declarations must carry the same obligations. `forward_function_call` must preserve receiver and argument evaluation, pass owner markers and reify final record carriers after a proved replacement, including typed failure paths.

`borrowed_owner_provenance.rs` currently suppresses any retained origin called by closure bodies or represented by `ClosureForward::Function`. Add concrete checked `ForwardFunction` call edges and complete owner transport. Preserve suppression for genuinely unresolved abstract uses; merely removing it is unsound. Closure-body callers need explicit parameter/capture mapping and the same native record-view and lifetime obligations. Their written parameter indices come from `CheckedClosure.parameters`, not a presumed zero-based function parameter vector. The current `Owner.function` namespace names ordinary checked functions; closure-local owners require an explicit body identity or complete mapping to outer owners.

Keep both executable witnesses until they pass, add retained-new-owner rejection and independent nullable/plain/mutable/failure controls, then extend the matched workload, oracle, guards and registrations. Continue the remaining B7 plan before its family milestone. No implicit clones, shared cells, lifetime casts or changed source callable kinds are authorized by this repair.

## Continuation prompt

Continue Tsrust from this checkpoint. Read AGENTS.md, docs/development-workflow.md and docs/plans/caller-borrowed-record-stores-plan.md. B7 forwarded installer source effects and editor diagnostics now pass, but two new executable witnesses remain red. Implement the forwarded native record/lifetime/replacement/owner interface and complete concrete provenance; then handle closure-body callers. Preserve source release and retention proofs, the fifteen unrelated fixtures, B5 consumers and SQLite evidence. Run focused checks and eight depth canaries after shared dispatch changes. Keep the full compiler goal active and D2 separate.
