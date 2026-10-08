# Handoff: named declaration family accepted

Source checkpoint 15af854 implements inline small scalar rest snapshots; 118346d publishes accepted measurement metadata and registration controls. This handoff commit closes publication. The fifteen owner fixture edits remain uncommitted and are excluded from staging.

## Accepted behavior and evidence

The family contract audit, context witnesses, editor coverage and twelve matched workloads are accepted. See milestones/named-declaration-family.md for the requirement table and performance review. Existing capture ownership rules remain unchanged.

Reconciled complete compiler milestone evidence covers 2,328 unique passing checks: 2,325 original checks plus two context tests and one allocation test. This is preserved full evidence with affected supplements, not a new complete compiler run after optimization. Current affected project and generic adapter groups pass 89 and 88 tests; spread/rest groups pass 47. Whole current editor passes 359. Twelve workload registrations pass. Eight unchanged parser/analyzer depth canaries retain their accepted evidence.

The family preserves 2,232 original timing samples and nine original allocation samples. Only rest and generic adapters change emitted programs; their remeasurement adds 372 timing samples and three allocation samples. Ten other generated programs are identical and retain original measurements. All original protocols and samples remain in SQLite. Optimized rest removes 10,240 snapshot allocations and lowers native median execution time about 20.4%, while still trailing compiled Node. Generic adapters show a roughly 2.6% increase in native time; it is preserved as a small cross-run signal, not an established regression.

All five consumers are atomically refreshed from 118346d and pass combined CLI/native/LSP checks for path/OS exports, independent callable state and scalar rest forwarding. Consumer hashes, build logs, smoke source and results are preserved under tsrust-named-family-consumer-* records. Closed evidence is imported, read back byte for byte and removed from scratch. Closed test executables are pruned; the cleanup receipt is tsrust-named-family-cleanup.json. No cache-warming rebuild follows cleanup.

## Remaining work

A large generic adapter compilation hit the 120-second observation bound in debug and during concurrent checks; a serial release comparison completed. The preserved live sample points to specialization worklist, guard partitions and state joins. This is a compiler-efficiency investigation, not a resolved cause. Records: tsrust-rest-snapshot-compiler-sample.txt, tsrust-rest-snapshot-comparison-review.json and tsrust-rest-snapshot-affected-programs-final.json. Preserve rejected comparison evidence and use trimmed stdout exactly as the benchmark runner does.

An unrefined caught runtime-error payload passed directly to generic std:testing.strictEqual reports an internal unsupported type. Boolean comparison works. Repair this with a focused witness rather than treating it as named-declaration parity. Scoped aggregate composition, fallible borrowed callback fields and named function expressions remain separate language work.

The parallel stdlib worker has an authorized std:path.relative plan in parallel-path-relative-plan.md. It must finish isolated branch acceptance, then obtain exclusive main ownership, integrate the latest committed main, publish source and SQLite evidence, refresh consumers and return ownership. Branch-only acceptance is insufficient. Do not concurrently edit main or overwrite its SQLite database.

## Continuation prompt

Continue Tsrust from this handoff. Read AGENTS.md and docs/development-workflow.md first. Preserve all fifteen owner fixture edits and stage explicit paths only. Named declaration family acceptance is complete; do not repeat its full matrix or timings. Next investigate the caught runtime-error assertion diagnostic with TDD, or the sampled specialization joins with a bounded witness and preserved performance controls. Coordinate std:path.relative integration through exclusive main ownership. Use Node 24.11.1 first on PATH, /opt/homebrew/bin/python3 for SQLite tools, /private/tmp/tsrust-gate100-editor-target and CARGO_INCREMENTAL=0. Give a physical handoff at the next checkpoint.
