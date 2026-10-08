# Handoff: callable replacement family

Compiler, editor, documentation, benchmark support and performance review pass. Consumer publication remains pending until the final verification section is completed. Read AGENTS.md, docs/development-workflow.md and docs/milestones/callable-replacement-family.md first. Preserve all fifteen owner fixture edits; stage explicit paths only.

## Accepted implementation scope

Generic callable values use finite storage families and point-specific checked invocation alternatives. Different declarations and arities retain independent bodies, defaults/rest packing, source access, result/error conversion and saved values. Replacing the callee during argument evaluation remains rejected. Native witnesses verify ownership and exactly-once disposal without implicit clones.

Owned mutable factory state is point-sensitive. Reusable callbacks restore transferred capture slots on every proved completion. Instances remain independent through direct calls, repeated producing contexts, loop-created array members, removal, reinsertion and nested owned child captures. Array provenance keeps an ordered prefix and repeated tail. Graph transfers are iterative, preserve cycles and shared edges, retain borrowed lexical identities and commit outgoing state only after validation. Unknown or consumed children remain conservative.

The milestone found one analysis nontermination defect: recursive mutable-array requests generated growing exact shape keys. A sampled live worker confirmed worklist evaluation and shape hashing rather than backend work. The interrupted run and completed groups are preserved. Dedicated recursive input widening bounds caller, capture and argument shapes before caching recursive requests. Nonrecursive requests keep exact precision. The original native witness passes; convergence and request-count controls pass.

## Milestone evidence

All 232 compiler integration targets are covered. Reconciliation yields 2,323 unique passing tests, zero failures and two intentional subprocess-worker ignores. The complete editor crate passes 357 checks. All eight normal-stack depth canaries pass on repaired source. No old baseline failure is retained as an accepted failure.

Exact compiler receipt: tsrust-callable-family-compiler-coverage.json. Original interrupted evidence: tsrust-callable-family-compiler-milestone.log. Resumed and affected checks: tsrust-callable-family-compiler-resume.log and tsrust-callable-family-compiler-affected.log. Editor evidence: tsrust-callable-family-editor-milestone.log. All are in benchmarks/records.sqlite with exact readback verified before loose copies were removed.

The current generic_callable_replacement workload has nineteen observations per scenario and 2,048 scenarios per batch. Five native/Node-after-tsc correctness cases pass. Execution-only measurement uses three processes per engine, eight warmups and 31 samples per process: 186 original timing samples. Median batch times are 0.157334 ms native and 1.575708 ms Node-after-tsc, approximately 10.02 times faster native for this workload on this machine. Earlier observation counts are different cost profiles and are not historical regression baselines. No allocation profile was submitted.

Raw measurement: runtime/results/2026-10-08T07-43-42-877Z.json. Review: tsrust-callable-family-performance-review.json. Normalized analytics preserve 93 samples and three process identities per engine. Proof: runtime/diagnostics/callable-replacement-family-2026-10-08/measurement-acceptance.json. Source acceptance metadata and measurement status are explicit; other generic-value workload measurements keep their separate status.

## Remaining work outside this family

The broader named-declaration milestone still requires its accumulated workload review; this family measurement does not measure every earlier callback workload. General scoped aggregate composition, broader borrowed callback carriers and other language backlog items retain their existing plans. Use current evidence to choose the next complete family. Do not restart already completed regressions or retime unrelated workloads for presentation.

The stdlib worker may finish path.relative in its isolated tree. Main integration is required but needs exclusive ownership transfer. Merge the latest committed main into the worker branch and verify combined behavior before publication. Do not merge through active uncommitted source, stash owner edits or replace main SQLite.

## Environment and continuation prompt

Use /private/tmp/tsrust-gate100-editor-target, CARGO_INCREMENTAL=0 and offline/locked Cargo. Put Node 24.11.1 first on PATH. Use /opt/homebrew/bin/python3 for storage tools. Format the entire project before every source checkpoint. Refresh all five consumers with sibling .refresh-tmp files and atomic rename. Preserve exact SQLite history and prune obsolete generated outputs before handoff.

Continue Tsrust from docs/handoff-2026-10-08-callable-replacement-family.md. Read AGENTS.md and docs/development-workflow.md first. Verify the consumer publication section before treating the callable replacement family as closed. Preserve fifteen owner fixture edits and use explicit Git paths. Next, finish the named-declaration family milestone or its planned scoped aggregate continuation using the current acceptance matrix. Coordinate any stdlib integration through an exclusive main ownership handoff. Keep TDD, complete editor behavior, formatting, matched workloads and exact SQLite evidence together; retain completed milestone evidence and original protocols.

## Observer-call publication repair

The initial five-consumer smoke failed because imported assertions made contextual factory proofs opaque. A stronger project witness also exposed lost native identity through a stored path alias. Explicitly listed observer calls now preserve captures on normal and possible throwing exits, and native symbols retain identity through aliases. Callback runners, registration and callback-sharing intrinsics remain conservative. The combined std:testing/native-alias witness passes at optimization levels 0 and 3. A checked project editor witness preserves separate one-argument and two-argument signatures. Affected acceptance passes 535 unit tests, 88 project tests, 358 editor tests and all eight rebuilt default-stack canaries. Whole-project formatting changes no files. Consumer rebuild and combined smoke remain pending. Exact red, repaired and boundary records are stored under tsrust-callable-observer and tsrust-callable-imported-assertion identities in SQLite.
