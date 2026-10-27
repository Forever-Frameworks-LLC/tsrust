# Gate 189 owner places and record loan lifecycle

Read AGENTS.md, docs/development-workflow.md, docs/plans/resources-bytes-plan.md and
docs/milestones/native-byte-views-implementation.md first. This checkpoint follows c5a7ccb9.
Gate 189 and S07 remain open. Public native view signatures are still guarded.

## Implementation

Each loan now stores both its owner's field path and its holder's value-relative
storage path. Owner overlap is based on path prefixes. Borrowing a field blocks its
ancestors and descendants, while unrelated sibling fields remain accessible. Whole
owner replacement checks the empty path explicitly. Native view functions retain
field access footprints before their first partial move.

The new native_view_places module handles owner footprints, conditional sources,
overlap, projected consumption and direct record-field replacement. Conditional sources
retain multiple fields of the same owner rather than deduplicating only by root.
Borrowing an indexed source conservatively retains its containing array footprint.
Normalization and Availability joins distinguish owner paths as well as holder paths
and variant guards.

A projected move deactivates only the selected old holder loans. Initializing the moved
value retains the selected origins. Direct field replacement removes old origins under
that field and installs prefixed replacement origins; sibling origins remain intact.
These paths are connected to the ordinary analyzer access and assignment dispatch.

## Evidence and limits

The focused tests inject native loans into real analyzer contexts. Parsed and formatted
source bodies check sibling mutation, field moves and replacement. The moved value
keeps its owner blocked until the moved value leaves scope. Conditional sources preserve
distinct fields of one owner. These are private analyzer checks, not public source
factory or native execution acceptance.

Exact original red, repair and final command outputs are in records.sqlite under
runtime/results/native-view-owner-fields-2026-10-10-checks.json. The final record owns
measured test counts; no timing or whole-crate regression is claimed. Installed
consumers remain f569190a because no published source behavior changed. The warm debug
compiler is rebuilt for whole-project formatting.

Keep native_binding_types' borrowed-signature guard. Dynamic indexed stores and
collection removal/clear do not yet transport native view provenance. Return source
relationships and backend function lifetimes, local-owner escape rejection, captures,
stored/forwarding factories, recursive wrappers and callback/view composition remain
open. The existing source read.tsr probe still fails at its missing view export.

## Next work

Infer returned source places and function lifetime relationships, retaining multiple
borrowed parameter origins without arbitrarily unifying unrelated input lifetimes.
Reject local-owner and consuming-parameter escapes. Complete indexed collection loan
transport and exceptional/loop exits before enabling source factories. Then add full
IntelliSense, actual source/native pointer and allocation controls, and matched view
benchmark support. Gate acceptance requires the whole view matrix; full regressions
and timing remain the completed S07 family milestone.

Use /private/tmp/tsrust-gate100-editor-target with CARGO_INCREMENTAL=0, Node 24.11.1
first on PATH and /opt/homebrew/bin/python3 for store.py. One agent works in this tree.
No unrelated owner edits remain.

## Continuation prompt

Continue gate 189 from docs/handoffs/handoff-2026-10-10-native-view-owner-places.md. Read
AGENTS.md and docs/development-workflow.md first. Implement precise returned source
places, lifetime inference and source escape diagnostics, then finish collection and
control-flow loan transport. Preserve actual borrowed slices, no implicit copy or
hidden shared owner. Keep public factories guarded until source checking and emission
agree. Finish source/native/editor witnesses and matched benchmarks before acceptance.
