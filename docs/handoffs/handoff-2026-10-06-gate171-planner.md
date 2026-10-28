# Handoff: gate 171 capture planner checkpoint

/caveman Continue tsrust at /Users/seancotter/Desktop/dev/learning/tsrust. Read AGENTS.md, docs/development-workflow.md, docs/plans/nested-function-declarations-plan.md and docs/milestones/171-nested-function-captures.md first. Inspect Git status/current checkpoint.

Gate170 accepted bcd7e88; interface editor repair ba34914; gate171 red witnesses 54f178e. Gate171 is still not accepted. This chunk introduces local_capture_plan.rs and integrates collection into local_functions.rs. Captures use BindingId(owner, declaration_start), preserve source spans and spelling, and are separate from local callable dependency edges. Iterative Kosaraju groups recursion; iterative on-demand requirements follow condensed dependencies. No recursive graph frames, implicit clone, shared cells or access-mode guesses. Capture-free declarations skip graph solving. The no-local-declaration fast path remains. Captured programs still reject explicitly after planning; five native capture/readiness positives remain pending.

Next retain plans and captured AST through checked contexts. Existing resolver statically lifts before binding types exist, while Analyzer::closure borrows at closure creation. Hoisted declarations need types/access analysis separated from environment materialization and readiness at reachable calls/value uses. Reuse existing capture/loan/layout contracts under the settled owner decision. Partition group-external capture slots from member invocation-local parameter/slot dependencies using BindingId.owner. Do not put sibling function values inside the recursive environment. Preserve finite group dispatch, source/editor identity, default/generic contracts and block/loop lifetime. Never replace this with eager const-arrow lowering, hidden untyped parameters or shared mutable cells. Owned escape gate172; result inference gate173.

The planner has focused red-to-green tests for recursive deduplication/forwarding, shadowed binding identities and a 4,096-node chain. Resolver diagnostic/source identity integration and accepted lexical behavior are checked. All eight required canary suites run after resolver changes. Full editor checked before this checkpoint; see milestone171 and SQLite logs for final counts. No timing or new runtime capability/benchmark acceptance is claimed. Original failures are retained in SQLite, exact readback before deleting loose logs. Five consumers are atomically refreshed/hash verified.

Preserve all four unstaged fixture edits: comma-contexts/guards.tsr, map-for-each/values.tsr, map-group-by/documented.tsr and interfaces/shapes.tsr. The interface fixture changed again during this chunk; its origin is unproven; leave it untouched and never commit it. One agent, explicit git add paths only. Same Cargo target /private/tmp/tsrust-gate100-editor-target, CARGO_INCREMENTAL=0; pinned Node /Users/seancotter/.nvm/versions/node/v24.11.1/bin/node. No archives/full compiler regression/routine timing. Broader goal active; hourly checks paused.

Final evidence: library14, lexical5, canaries63 and editor272 pass. Capture suite1
reference passes and5 feature witnesses remain red. Existing lexical retained-result
benchmark passes5 matched native/node-tsc cases untimed; runtime-result path in SQLite
is benchmarks/runtime/results/2026-10-06T17-05-38-605Z-verify.json. Graph/library/editor
logs are target/editor/nested-plan-* gate-log records, with exact readback verification.
Generated examples, native comparison outputs and scratch executable tests are pruned;
keep libraries and five consumer executables. Gate171 remains incomplete.
