# Nested callable input implementation progress

Read AGENTS.md, docs/development-workflow.md and docs/plans/generic-value-adapters-plan.md first. HEAD remains the red checkpoint 01f4c54. Implementation is uncommitted and unaccepted. Preserve all four owner fixture edits.

The retained nested-callable-input witness now passes generated native execution. All 29 generic_value_adapters tests pass. Exact failed diagnostic traces and passing focused logs are stored in benchmarks/records.sqlite under their original /private/tmp paths with verified byte-exact readback. Loose copies are removed.

A dedicated checked_value_annotations.rs reconstructs nested callable contracts for fresh staging and selected factories. Synthetic function parameters no longer receive duplicate data access wrappers. A second defect was exposed: resolved_call can select another concrete body for callable arguments. deferred_generic_call now checks that selected body before retaining its invocation method. Temporary backtrace instrumentation was removed.

Acceptance remains open. Audit full ownership and shared callable representation; SharedRead currently reports unsupported rather than silently becoming uniquely owned. Add controls for access, omission, rest, nested results and ownership. Complete editor checks, documentation, matched benchmark support, depth canaries and whole-editor checks before acceptance. Current executable consumers remain unchanged. No timing or broad regression was run. No process remains live.

Continuation prompt: Continue Tsrust from docs/handoff-2026-10-06-nested-callable-input-progress.md. Inspect current uncommitted implementation and preserve owner fixtures. Finish full checked callable reconstruction and acceptance controls without weakening ownership. Complete editor, matched benchmark support and SQLite evidence. Gate 173 remains open.

## Additional contract controls

Two retained tests exposed and now verify additional fixes. Void callable results are reconstructed as named void annotations, and generic constraint matching recognizes an exact void result. Optional callback parameters contribute their declared payload rather than the omission-added undefined member when inferring a single payload type. Normal callback compatibility remains responsible for omission and ownership validation. Nested optional/default and homogeneous rest parameters now execute correctly through the retained generic environment.

All 31 generic adapter tests and the eight depth canary suites pass on current source. Passing controls and original failures are preserved in SQLite with exact readback. Whole-editor checks, shared ownership representation, further controls, matched benchmark support and acceptance records remain outstanding. Source remains uncommitted and unaccepted; no executable consumers were refreshed.

## Shared contracts and editor verification

A retained failing unit witness exposed SharedRead being converted into Owned by callback annotation conversion. CallableOwnership now carries an internal SharedRead marker. Callback and callable result annotation round trips, checked reconstruction, project facts and editor type rendering retain that marker. This does not add a new source ownership wrapper. The result round-trip unit witness passes. The editor gained a protocol witness for nested callable argument help and factory navigation. The entire editor crate passes (305 tests). Original failure, compile repair and passing logs are preserved exactly in SQLite. All processes are terminal.

Acceptance remains open: finish runtime shared/access/ownership and nested-result controls, matched benchmark expansion, guide and acceptance records. Recheck affected compiler suites after the shared annotation changes. Preserve owner fixtures. All implementation remains uncommitted, and the five executable consumers have not been refreshed.

## Nested callable result boundary remains red

Two additional native controls were retained. Borrowed readonly callback inputs pass and remain callable by the outer owner after repeated adapted invocations. Nested callable results remain red. The original failure was Cannot infer type parameter T because adapter inference supplied no callable shape metadata. The new checked_inference_callable_shapes helper collects concrete nested argument and result shapes without evaluating expressions. check_generic_adapter now supplies those shapes to constraint solving. This resolves inference, but the retained witness now fails with Expected closure, found closure at make(). Do not weaken compatibility or the witness to bypass the callable-result origin mismatch.

The focused five-control run reports four passes and one retained failure. Both original inference failure and subsequent origin failure logs are stored exactly in SQLite. Whole-editor 305-test evidence predates this latest inference-helper change and does not prove it. All processes are terminal. Matched benchmark expansion has not started; existing matched family remains 84 observations. Continue by tracing the result origin compatibility boundary, then complete ownership controls, Node references, editor, matched support and records. All implementation is still uncommitted and unaccepted.

## Nested result control passes; matched support expanded

Trace identified actual Type::Callable against expected Type::Callback, rather than two incompatible closure identities. callable_compatibility now compares that declared result contract using its structural invocation kind and existing exact argument/result checks. Temporary tracing was removed. Retained nested-result native execution passes, and a new negative witness rejects a single-use result in a reusable callback contract. All 34 adapter tests pass.

Matched generic_value_adapters workload sources now append four observations for nested callable inputs: value+33, value+181, value+34, value+182. Previous 84 observations are preserved. Independent oracle and literal registration expectations now contain 88 observations, source contract capture update count is nine, and manifest fingerprints were updated as text. Registration test passes. Matched native/Node execution has NOT yet run on these expanded sources. Extend semantic mutation guards for the new scenario before acceptance. Exact trace, focused/suite checks and registration evidence are stored in SQLite. All processes are terminal.

Remaining work: native shared/ownership disposal controls, final Node references, editor refresh/checks after latest changes, matched untimed verification, guide, milestone/coverage/roadmap records, atomic five-consumer refresh and explicit-path acceptance commit. Owner fixtures remain untouched. HEAD stays 01f4c54; implementation remains uncommitted and unaccepted.

## Final verification pass and consumer refresh

Source guards now require both nested callback invocations and the callback-derived result; registration mutation controls reject changing those operations after updating the fingerprint. Registration passes. Compiler and editor binaries were rebuilt, then all five consumers were copied to sibling .refresh-tmp paths and atomically renamed, with SHA256 equality verified. Consumers now reflect current uncommitted source.

Matched native versus Node-after-tsc verification passes all five cases with 88 observations. Exact canonical record benchmarks/runtime/results/2026-10-07T01-20-05-313Z-verify.json is imported into SQLite, read back byte-exactly, and its loose copy removed. No timing was run. All 34 adapter tests, four shared callback runtime tests and eight depth canary suites pass on final source. Whole editor passes all 305 tests on final source. Logs are stored exactly in SQLite. All processes are terminal. The compiler README now explains nested generic factory contracts.

Acceptance still requires the remaining semantic audit and records/commit. Inspect shared reconstruction and concrete closure ownership carefully; add any missing relevant controls rather than treating these checks as full Gate173 acceptance. Finish milestone/coverage/roadmap updates and cleanup before handoff. Preserve four owner fixtures and commit only explicit implementation paths. Full Gate173 remains open.
