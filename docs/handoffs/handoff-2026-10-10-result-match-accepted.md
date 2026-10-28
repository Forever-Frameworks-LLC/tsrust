# Handoff: B10 Result and statement match accepted

B10/S06 is accepted at gate 186. Read AGENTS.md and `docs/development-workflow.md`
first, then `docs/plans/compiler-completion-plan.md`, `docs/milestones/186-result-match.md`,
`docs/milestones/b10-result-match-acceptance-map.md` and `docs/plans/standard-library-roadmap.md`.
The larger compiler completion goal remains open. One agent works in this tree.
No protected owner files or unrelated modifications remain.

## Source checkpoints

- `ae5374fe`: cleanup/failure witnesses and final seven-observation matched workload.
- `8d431eed`: optional type-application regression repair, complete Result coloring,
  catalog witness registration and acceptance map. All executable source is here.
- The following publication commit records gate 186, measured coverage and this
  handoff. Inspect Git HEAD rather than assuming a hash for that documentation commit.

## Accepted behavior

Contextual and explicit Ok/Err construct `Result<T, E>`. Exhaustive statement match
has block arms, optional commas, arm-local const payload bindings, one scrutinee
evaluation and explicit return propagation. Owned, ReadOnly and Mutable matches
preserve ownership, scalar copying and scoped loans. Captured reusable/Once payloads,
replacement, nested Results, early returns, throws, finally and loop exits have native
and disposal witnesses. Project and single-file editor assistance preserve checked
payload identity and imported aliases; incomplete headers recover lexical assistance
without guessed types. Formatter and TextMate support are complete.

Keep these decisions: no implicit clone; no shared JavaScript capture cells; matching
an owned Result transfers it; a borrowed payload cannot escape; mutable payload views
exclude access through the owner until the view ends. Zero-argument constructors
represent void alternatives, which use discard patterns. Result errors do not throw.

## Final evidence

Compiler: 2,754 passed, zero failed, three intentional ignores, no aborted suite.
Editor: 472 passed, zero failed, one intentional ignore. Complete coverage combines
preserved preceding suites with resumed/unrun and affected suites, not a repeated
full run. All ten depth canaries pass on normal worker stacks. Compiler doctests,
extension compilation, grammar and final manifest registration pass.

All eighteen Result integration tests pass. Cleanup restores exact live bytes over
32 source-body invocations at optimization levels zero and three. Initialize the
process entry hook only once: repeatedly invoking process setup chains panic hooks
and is not a valid Result cleanup witness. Independent TypeScript reference passes.
Six catalog examples execute after idempotent formatting with exact output.

The final matched workload verifies five cases on both native and Node-after-tsc.
Uncontended execution-only timing keeps eight warmups and three processes with 31
samples each. Median batch time: native 0.111625 ms, Node-after-tsc 0.628750 ms,
Node/native ratio 5.63. This is one mixed-workload baseline, not isolated operation
costs. The earlier overlapping measurement is retained and explicitly excluded.
The milestone lists every final SQLite record path. Original failed runs are retained;
all closed loose records are imported and read back byte-for-byte before deletion.

Five consumers were refreshed atomically from `8d431eed`; three compiler paths pass
the cleanup fixture check. Installed TextMate grammar matches source. Restart the
Tsrust server and reload VS Code for running processes to pick up these updates.
Obsolete executable variants and temporary workload outputs are pruned after checks;
the cleanup receipt is retained in SQLite. The shared target remains partially warm.

## Lessons from the milestone

An absent optional call still needs static type-argument validation. Run preparation
before absence lowering, which can erase the application. The existing rejection
witness caught this; it now prints its source on failure. Do not weaken it. The editor
has a matching formatted diagnostic test. When adding catalog globals, also register
their examples in the exhaustive catalog witness dispatcher.

Count aborted suites and missing result lines, not just successful summaries. The
first failed compiler record had 102 started suites and 102 summaries; the resumed
compiler record has 150 each. The complete editor record has 41 each. Preserve failed
records, resume unrun suites and repeat only affected checks after repairs.

## Remaining scope

`?`, match expressions, guards and nested destructuring patterns are separate work.
General discriminated unions remain D2, using this finite-variant foundation. Direct
borrowed callable payload storage is rejected. Context-free inline callback generic
inference can require explicit type arguments. Do not claim these forms are supported.

Next dependency row is B11/S07: resources and bytes. Write a concrete plan and gates
before source work. Audit existing bytes/opaque native support first. Define owned
handles, explicit close plus deterministic drop, use-after-close diagnostics, byte
buffer/view bounds and loans, failure/early-return cleanup, Result APIs, editor help
and matched benchmark support together. Keep async/tasks and native threads separate
until their prerequisites land. Use small modules and focused red witnesses; complete
regressions and timing belong at the resource/bytes family milestone.

## Environment

Node 24.11.1 first on PATH; `/opt/homebrew/bin/python3` for the analytics store.
`CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target`, `CARGO_INCREMENTAL=0`.
Format the whole project with the current verified TSRUST before every source
checkpoint. Use explicit Git paths. Never create archives or snapshots. Do not replay
B10 full checks or timing merely to restate this handoff.

## Continuation prompt

Continue Tsrust from docs/handoffs/handoff-2026-10-10-result-match-accepted.md. Read AGENTS.md
and docs/development-workflow.md first. B10/S06 is accepted at gate 186; source closes
at 8d431eed and the latest commit publishes its records. Compiler 2,754 and editor
472 pass with no failures or aborts; all ten depth controls pass. SQLite retains exact
failed, resumed, cleanup and measurement evidence. Five consumers and installed
syntax grammar are current. No protected owner edits remain. Continue the overall
compiler completion goal by auditing and planning B11/S07 resources and bytes, then
implement coherent TDD increments with ownership/drop, Result, editor and benchmark
support. Do not redo accepted generics or Result. Keep D2, match expressions, guards,
nested patterns and ? separate. Preserve bounded checks, explicit paths and whole-
project formatting; use the shared target and disable incremental compilation.
