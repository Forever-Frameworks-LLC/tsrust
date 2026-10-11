# Gate 189 native Result view provenance

Read AGENTS.md, docs/development-workflow.md, docs/plans/resources-bytes-plan.md and
docs/native-byte-views-implementation.md first. This checkpoint follows 8e1db16e.
Gate 189 and S07 remain open. No native view factory is published.

## Implementation

Native loans carry value-relative field/element paths and variant guards. The new
native_view_projections module owns wrapping, selection, narrowing, flow feasibility
and normalization. Result Err projections discard success loans; Ok projections
satisfy the guard and payload selection preserves the loan. Current flow facts can
release a guarded loan on Err paths without releasing an unconditional sibling loan.
When the guard disappears after a branch, the possible success loan still applies.

Native factory checking records the checked Result union and success record in an
optional Context map. This is nominal identity, not display-name matching. The value
walker uses an iterative task stack and preserves paths through records, arrays,
injections, conditionals and projections. Record siblings stay separate. Array
indices remain conservative. Availability joins normalize by owner, access, storage
path and variant guards, retaining every reaching alternative.

Two implementation defects were found and repaired: aggregate task boundaries wrapped
previous siblings together, and field wrapping initially repeated the same mistake.
Original failing outputs remain in SQLite, along with a test setup quoting error.

## Verification and limits

Seventeen private loan tests pass. They include parsed, formatted Result match bodies
through the actual analyzer: Err permits owner replacement; Ok rejects it. Tests also
cover success payload transport, field selection, joined availability and guarded
versus unconditional loans to one owner. Native loans are injected into the starting
analyzer context; these are not public factory or native execution witnesses.
Ten depth-canary suites pass 89 tests without failures or aborts. Fourteen native
editor project checks pass. Whole-project formatting changes zero files. No full
regression or timing ran. Exact record:
runtime/results/native-view-result-paths-2026-10-10-checks.json in records.sqlite.

Keep native_binding_types' borrowed-signature guard. Owner conflicts still use whole
binding roots. Field-sensitive owner conflicts, projection moves/replacement, source
escapes, return inference and explicit backend source lifetimes remain open. So do
forwarding/stored factories, recursive wrappers, callback/view ABI composition,
captures, all exceptional/loop exits and last-use release. Do not infer completion
from the private Result checks.

## Next work

Connect precise source places and projected moves/replacements, then infer returned
source relationships and native function lifetimes. Reject returning views of local
owners or consuming parameters and dangling temporary sources. Retain exact roots
through forwarding functions and stored factories. Publish factories and byte
contracts only when source checking and backend agree. Complete IntelliSense,
source/native examples and matched view benchmark support before gate acceptance.
Full regressions and timing remain the completed S07 family milestone.

Installed consumers remain f569190a; no public source behavior changed. The shared
debug compiler is rebuilt for formatting. Reuse /private/tmp/tsrust-gate100-editor-target
with CARGO_INCREMENTAL=0, Node 24.11.1 first on PATH and /opt/homebrew/bin/python3
for store.py. No unrelated owner edits remain.

## Continuation prompt

Continue gate 189 from docs/handoff-2026-10-10-native-view-result-paths.md. Read
AGENTS.md and docs/development-workflow.md first. Implement precise owner places,
projected loan moves/replacement and returned lifetime inference. Finish source,
native and editor view execution with ready matched benchmark support. Preserve
actual borrowed slices, no implicit copy or hidden shared owner. Keep public factories
guarded until their analyzer and emitter paths agree. Gate 189 remains open.
