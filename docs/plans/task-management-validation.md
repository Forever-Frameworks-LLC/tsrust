# Compiler performance and Worklist acceptance

Owner goal, 2026-11-14. Optimize compiler/LSP and especially dev operations, then
validate the platform through a complete native task-management product. Finish
local acceptance before pushing and guiding a PR to main. Repair OS-specific CI
failures locally before another push. Existing Studio scope remains retained.

## Performance acceptance

Measure Tsrust end-to-end build overhead against a direct Rust build of the same
generated source, matching profiles, optimization flags, runtime dependencies,
toolchain and cache state. The target is less than 10 percent overhead. Report
cold, warm, leaf-edit, shared-edit and configuration-edit observations separately.
A cache hit is not evidence for an edited build's overhead. Record phase timings
and observable output, preserve baseline samples, and ingest comparable workloads
into benchmarks/measurements.sqlite. Use established protocols before introducing
new harnesses. LSP measurements separately cover foreground diagnostics, edits,
navigation availability, stale-result rejection and retained memory.

## Product acceptance

Worklist lives in apps/tasks with separate desktop and API roots and shared
Tsrust contracts. Use .tsr/.tsrx for application and orchestration logic and the
existing checked native bindings. Two views provide a main task workspace and
task details. The main workspace supports virtualized table and card modes,
sorting, filtering, search, sticky columns and pagination. Details support
creating, editing and deleting tasks with explicit conflict/error feedback.
Choose coherent native product styling, consistent zoom, compact controls and
accessible names. Validate actual interactions and resize/overflow behavior.

The API uses Docker Postgres through std:sql and std:http. Desktop keeps a
persistent offline cache and applies resumable CDC batches, including external
database changes and delete tombstones. Mutations serialize sequence allocation
with commit ordering. Snapshot data and its high-water cursor use one consistent
database snapshot. Cache rows and applied cursor commit atomically. Test crash
replay, duplicates, rollback gaps, concurrent commits, disconnect/reconnect and
optimistic version conflicts. The exact delivery transport remains an
implementation choice.

Use std:testing suites and tsrust test for runtime acceptance. Verify database,
API, desktop, offline cache and worker boundaries, including simultaneous
API/desktop dev sessions. Repair every discovered compiler/LSP/runtime issue and
keep focused regression witnesses. Verify external editor diagnostics and matched
compiler results. Final acceptance includes all requested product behavior,
the performance comparison, complete local regressions, supported-platform CI
and the merged PR. Partial application scaffolding does not satisfy this goal.

## Initial state

The established dev-loop harness benchmarks/dev/baseline.py is being sampled
with its original three-sample, 4/128-leaf module protocol. Docker Desktop is
available. The UI design uses a compact work-management layout and shared text
sizing. Existing Studio edits and acceptance evidence remain preserved.
