# Handoff: nested function capture lowering

Continue Tsrust at `/Users/seancotter/Desktop/dev/learning/tsrust`. Read `AGENTS.md`,
`docs/development-workflow.md`, `docs/plans/nested-function-declarations-plan.md`,
`docs/milestones/171-nested-function-captures.md` (the last section, "Native lowering
checkpoint", describes the current model) and this file before edits. Inspect Git
status and log. One agent works in this tree. `docs/handoffs/handoff-2026-10-06-gate171-layout.md`
remains valid for scope, owner decisions, worktree rules and execution rules; this
file replaces its "Next implementation" and evidence sections.

## What changed

Capturing nested functions execute. The model is lambda lifting, not an environment
value: captures are hidden native parameters after the source parameters, and each
direct call passes them from the calling body with a borrow that lasts for the call.
Recursive groups forward the same hidden parameters by reborrow. See the milestone
section for consequences and checks.

Code map:

- `named_capture_lowering.rs`: model documentation and the validation that every use
  of a capturing function is a direct call with recorded slots; marks mutated scalar
  captures `capture_reference`; runs the exception validation that capture programs
  deferred.
- `analysis.rs`: `declared_named_call` records caller slots in `Expr::capture_slots`
  and calls `named_capture_arguments`, which checks hidden captures after the written
  arguments with their retained loans. `capturing_function_is_unused` orders checking
  through the declaring function and stubs functions that no call reaches.
  `finish_analysis` calls the lowering instead of returning the pending diagnostic.
- `emit.rs`: hidden parameters in the function signature loop (`hidden`), and
  `Emitter::named_capture_arguments` at `declared_function_call`.
- `fixed_storage.rs`: a body that calls a capturing function keeps ordinary array
  storage.
- `syntax.rs`: `Expr::capture_slots`.

## Pending for gate 171, in suggested order

1. Function value of a capturing function (`values.map(add)`, `const run = tick`,
   passing to a callback parameter). Create the wrapper arrow
   `(parameters) => name(parameters)` in the lexical resolver (`local_functions.rs`),
   after capture planning and before analysis, so the escape pre-pass in
   `analysis.rs` (the `escaping` set, built from closure syntax ids) sees it. A
   rewrite during analysis is unsound: the wrapper would count as non-escaping and a
   borrowing closure could be returned. The wrapper then follows existing closure
   loan rules; a returned wrapper is gate 172 (owned transfer). Until then the use is
   an `unsupported` error that names the arrow form. The witness in
   `nested_function_captures::capture_borrows_follow_argument_and_view_rules` expects
   that error today and must change to a positive native witness.
2. Type-check capturing functions that no call reaches, against the final context of
   the declaring function. Today they are stubbed as templates and not checked.
3. Capturing functions with rest parameters or type parameters (pending diagnostic).
4. Group-level requirements that make an outer member of a recursive group require a
   capture owned by a nested member (pending diagnostic). Use the layout module's
   invocation sources.
5. Precise fixed-array storage: only captured slots lose a fixed shape.
6. Guide (`docs/` page for nested functions with captures), editor examples and
   checks for hover and diagnostics inside capturing bodies, and a matched capture
   workload with oracle, source contract, registration, hooks and manifest row.
   Then accept gate 171 with records, coverage, roadmap and SQLite evidence.

Gate 172 (escaping, owned transfer) and gate 173 (signatures, generics, result
inference) follow. The two failing tests in `nested_function_declarations`
(`..._transfer_escaping_environments`, `..._infer_nonrecursive_result_when_omitted`)
are their open witnesses; do not weaken them.

## Evidence at this checkpoint

- `nested_function_captures`: 13/13, including the four native witnesses, the new
  `borrowed-calls.tsr` assertion fixture (passes the pinned Node reference) and the
  rule witnesses for conflicts and accepted reads.
- Batch of 14 compiler suites with all eight depth canaries: 124 pass; the 2 failures
  are the future-stage witnesses named above.
- Compiler unit tests 308/308. Four unit tests that asserted the removed barrier now
  assert that the same programs check.
- Editor crate 275/275.
- `nested_function_declarations` matched workload verifies native/Node-after-tsc
  untimed: `benchmarks/runtime/results/2026-10-06T19-21-04-566Z-verify.json`.
- Eight logs and that result are imported into `benchmarks/records.sqlite` and read
  back byte for byte; loose copies are removed. `target/editor/gate171.log` is the
  concise index.
- Five consumers are rebuilt and replaced by atomic rename (hashes in the SQLite
  record `target/editor/gate171-lowering-consumers.log`).
- No timing and no full regression. Gate 171 is not accepted.

The four unrelated fixture edits stay uncommitted (see the layout handoff). Use
explicit Git paths; never `git add -A crates` or `git commit -a`.
