# Gate 189 native view loan joins

Read AGENTS.md, docs/development-workflow.md, docs/plans/resources-bytes-plan.md and
docs/native-byte-views-implementation.md first. This checkpoint follows d418d468.
Gate 189 and S07 remain open. No native view factories are public.

## Changes

Availability now carries native view origins and retained state through snapshots,
restoration and joins. Whole-binding consumption makes the old holder's loans dormant
while preserving provenance for a destination. Initializing the destination retains
its loans. Joining two paths keeps any loan retained on either path, independently of
the binding's ordinary transferred diagnostic. Two dormant paths remain dormant.

The previous holder index became stale after state restoration. Native view functions
now scan visible binding identities without allocating a holder list. Functions that
have never encountered native loans take the empty fast path. Source-loan identity is
still a checked binding index, never a span or spelling alone.

Eleven focused loan tests pass, including parsed if statements through Analyzer::block
with injected private loan contexts. The first red batch demonstrated both restoration
loss and incorrect release after a transfer on one branch. Public factories remain
guarded, so these are not end-to-end source-view acceptance tests.

## Next work

Implement Result Ok/Err projections and path-specific source-loan release. Preserve
all roots when success is possible; Err must retain no success payload loan. Propagate
precise aggregate field provenance rather than treating every field as the entire
owner. Infer returned source relationships and emit explicit lifetimes where elision
is ambiguous. Cover borrowing parameters, local-owner rejection, forwarding/stored
factories, reborrows, captures and all control-flow exits.

Do not remove native_binding_types' borrowed-signature guard until source checking
and backend agree. Complete source examples, IntelliSense and matched benchmark
support before accepting gate 189. Full regression and timing remain the completed
S07 family milestone, not this private checkpoint.

## Evidence and environment

SQLite record runtime/results/native-view-branches-2026-10-10-checks.json retains
original red and green output plus final bounded checks. No timing ran. Installed
consumers remain f569190a because no public source behavior changed. Reuse the warm
/private/tmp/tsrust-gate100-editor-target with CARGO_INCREMENTAL=0. Use Node 24.11.1
first on PATH and /opt/homebrew/bin/python3 for store.py. No owner edits remain.

## Continuation prompt

Continue gate 189 from docs/handoff-2026-10-10-native-view-branches.md. Read AGENTS.md
and docs/development-workflow.md first. Implement Result-path native view provenance,
precise field roots and returned lifetime inference, then source/native/editor
execution and matched workload support. Preserve safe slice lifetimes, no implicit
copy or hidden shared owner. Gate 189 remains open; keep public factories guarded
until their analyzer and emitter paths agree.
