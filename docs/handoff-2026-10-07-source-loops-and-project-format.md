# Handoff: source loops and automatic project formatting

Read AGENTS.md, docs/development-workflow.md, docs/nested-function-declarations-plan.md and docs/unused-generic-call-validation-plan.md first. The compiler source-loop checkpoint is 502457c. The following accepted checkpoint records the automatic project formatting foundation and its evidence; use Git HEAD as the authoritative revision.

## Accepted behavior

See docs/milestones/project-formatting-and-source-loops.md for acceptance scope and counts. Source-loop analysis uses an iterative control-flow graph in generic_call_flow_graph.rs. It handles updates, labeled control, abrupt completion and literal iteration bounds without changing runtime callable compatibility or introducing implicit clone. Full Gate173 remains open.

Run `node tools/format-project.mjs` before every source checkpoint. Set TSRUST to `/private/tmp/tsrust-gate100-editor-target/debug/tsrust` and use Node `/Users/seancotter/.nvm/versions/node/v24.11.1/bin/node`. The command keeps pre-format pinned sources in memory and updates only proven layout-only manifest hashes. Benchmark scenario guards compare tokens, but literal contents, mutation rejection and raw evidence hashes remain exact. Never silently accept semantic workload changes.

## Owner changes

Fifteen protected fixture changes remain unstaged and must not be committed. Their explicit paths are retained in the SQLite record `/private/tmp/tsrust-project-format-proof.json`. Read that record from SQLite if needed; there are no loose source snapshots. The unused-alias-contracts owned-transfer example repair was authorized earlier and remains an owner edit. Use explicit Git paths and inspect status before edits.

## Evidence and next work

Closed source-loop and project-formatting logs, the token proof and both untimed generic adapter verify records are stored in benchmarks/records.sqlite. Read back and decompress records when investigating; do not discard original failed evidence. No complete compiler regression or performance timing was run. Whole editor acceptance preserves passing groups and repairs/resumes only affected checks.

Continue the source-validation audit with focused red witnesses for exception/finally flow, then expression evaluation order and capture writes. Finish stored fields, concrete environment replacement and lifetime/context coverage before accepting Gate173. Keep existing ownership rules, no implicit cloning and separate implementation acceptance from pending milestone measurements.

Use CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target and CARGO_INCREMENTAL=0, offline locked Cargo. Refresh the five executable consumers by sibling .refresh-tmp copy and atomic replacement. Preserve bounded checks and all eight depth canaries after parser/analyzer dispatch changes. Keep Node-only references as .ts. No archives, source snapshots or routine timing.

## Generic closer formatting repair

A later focused repair separates the generic closing bracket from adjacent
assignment in headers such as `type Box<T>=...` and defaulted wrapper annotations.
The formatter splits combined lexer tokens only within parsed type spans, leaving
comparison and shift operators intact. Compact and previously formatted `T >=`
headers now become `T> =`. The focused native witness prints 7; 25 formatter tests,
three affected editor tests, the fixture corpus and the data-union registration
checks pass. Project formatting updates seven files and one proven layout-only
benchmark hash. Exact failed and repaired logs are stored in SQLite under
`/private/tmp/tsrust-generic-closer-*.log`. Restart the language server to load
the refreshed binary. This repair does not change Gate173 acceptance scope.
