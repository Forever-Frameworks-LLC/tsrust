# B7 holder capture storage checkpoint

B7/gate 183 remains in progress. This checkpoint follows `1374ccb`. Read AGENTS.md,
docs/development-workflow.md, docs/caller-borrowed-record-stores-plan.md and
this handoff before continuing. Discriminated unions are a dedicated required D2
feature, with literal tags, narrowing, exhaustive checks and editor support; schema
validation builds on that later. No owner decision is pending.

## Changes

`native_capture_storage.rs` separates final native capture layouts and lifetime
counts from checked source point types. It propagates record storage through nested
captures without changing source binding types. `record_point_call_emit.rs` invokes
only source-permitted origins when a final native record carrier contains additional
origins installed later. Early readonly captures therefore keep readonly invocation
when a subsequent store introduces a mutable callback. Native payload planning now
retains singleton concrete environments, which otherwise left an undeclared generic
parameter in those earlier captures.

`record_parameter_borrows.rs` retains the installed callable invocation contract for
local record bindings. Subsequent captures see mutable callback capability. This is
a conservative join, not branch narrowing or exact replacement release. No implicit
clone, boxing, shared cell or lifetime cast was added.

Readonly, mutable and nested captures of replaced holders pass native assertions.
Capturing before later stores preserves the initial callback. An owned return cannot
hide local borrowing storage inside its captured holder.

## Evidence

Closed checks use `tsrust-b7-capture-storage-` keys in benchmarks/records.sqlite.
Failed intermediate checks are retained alongside the repaired checks.

- `single-environment.log`: all twelve replacement/representation witnesses pass.
- `single-controls.log`: seventeen caller-store and nine existing replacement checks pass.
- `single-unit.log`: source point types remain unchanged by final native layout planning.
- `layout-final.log`: four existing callable layout checks pass.
- `planner-final.log`: correlated capture vectors and recursive worklist termination pass.
- `editor-final.log`: installed mutable holder signatures, hover and capture-conflict pass.
- `registration.log`: matched workload source contracts and mutation controls pass.
- `format-final.log`: whole-project formatting twice changes no files or fingerprints.
- Earlier `depth.log`: all eight required depth canaries pass after the analyzer change.

`runtime/results/2026-10-09T18-18-00-805Z-verify.json` verifies five cases each on
native and Node-after-tsc. Fourteen observations and the independent arithmetic oracle
remain unchanged; observations now exercise captured holders. Execution, startup and
compilation samples are empty. Workload measurement and family acceptance remain false.
No complete regressions or timing ran in this slice. Installed consumers remain at B5.

## Next work

Continue the full B7 plan: direct callback stores without wrappers, nullable stores,
mutable data aliases, failure paths before and after stores, exact replacement and
clearing, branches, loops, early exits and disposal. Nested generic callback capture
environments and more complex recursive retained-store summaries remain unfinished.
Do not treat this checkpoint as family acceptance. Keep source point types separate
from final native carrier layouts when extending these cases.

Fifteen unrelated fixture edits remain uncommitted. Preserve their semantics and use
explicit Git paths. Reuse `/private/tmp/tsrust-gate100-editor-target`, disable Cargo
incremental compilation, and keep Node 24.11.1 first on PATH. Use
`/opt/homebrew/bin/python3` for the SQLite store. The previous cleanup removed obsolete
test binaries; unrelated suites may need relinking. Do not rebuild to refill caches.
