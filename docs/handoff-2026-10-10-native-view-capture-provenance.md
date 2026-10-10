# Native view capture provenance groundwork

Read AGENTS.md, docs/development-workflow.md, docs/native-byte-views-implementation.md
and the preceding closures handoff. Gate 189 and S07 remain open.

The previously retained red witness now passes analysis. Returned call summaries
carry an explicit argument-or-capture source instead of overloading a binding ID as
an argument position. Closure and specialized method summaries map capture slots
through their checked capture lists. The iterative loan walker projects captured
carried origins from their lexical binding. A formatted and idempotent source test
returns an alias of the second enclosing parameter through a zero-argument closure.

This is not native capture acceptance. The generated closure environment still
needs named lifetime slots for captured native views, and returned lifetimes must
be bound to those slots or the receiver as appropriate. Captured owner storage
continues to receive a receiver-contract diagnostic. Escaped environments, nested
closure rebinding and arbitrary stored callable families need explicit witnesses;
do not assume outer binding indices remain valid across those transitions.

The public borrowed-signature guard remains enabled. No public factory, ownership
policy or shared-owner fallback was added. Installed consumers remain f569190a;
the warm CLI was rebuilt for project formatting only.

Focused checks: 49 private view tests, nine registered callable signature controls,
89 depth canaries across all ten required suites and 16 native editor tests (163
total). Project formatting changed zero files and source
fingerprints. No full regression or timing run. Original logs and this increment's
record are retained in SQLite under
`runtime/results/native-view-capture-provenance-2026-10-10.json`.

## Continuation prompt

Continue gate 189 from this handoff. Finish native captured-view layout and returned
lifetime emission, starting with the passing analyzer witness. Add generated Rust
O0/O3 pointer-identity controls and a negative owner-escape control before claiming
support. Verify exclusive and consuming captures under the existing ownership
rules. Then cover specialized callbacks, stored families, collections and completion
paths in the implementation plan. Keep public factories guarded until the required
matrix agrees. Finish editor/docs and matched benchmark support before acceptance;
full regressions and timing belong to S07 completion.
