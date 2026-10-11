# Handoff: recursive views and finally cleanup

Current at the commit carrying this note. The original October 10 handoff is
superseded; prior checkpoint evidence remains in Git, SQLite and
`docs/b11-resources-bytes-acceptance-map.md`. S07 is not accepted or measured.

Read `AGENTS.md`, `docs/development-workflow.md`, the acceptance map,
`docs/resources-bytes-plan.md`, `docs/recursive-view-storage-plan.md` and
`docs/standard-library-roadmap.md`. All decisions below are already approved.

## Approved scope and completed work

- Nullable recursive links, including mutual cycles, use unique heap ownership.
  Nonrecursive layouts stay unchanged. No shared ownership or implicit byte copies.
- Recursive records containing views share a real borrow lifetime. Array recursion,
  nullable links, provenance, iterative link destruction/clone and public fixtures
  are implemented. Views remain non-clonable. Recursive records storing callback
  fields still have the explicit callable-layout diagnostic and remain pending.
- View-capturing callbacks can return argument views or views of borrowed Bytes
  arguments. Terminal-branch provenance and temporary record calls are repaired.
- The installed CLI/editor consumers were refreshed from `4367f4cc` with executable
  fixture and LSP smoke checks. Later source work is not installed yet.
- Resource close consumes ownership even on Err. Later use and double close are
  static errors. Drop is a nonthrowing fallback. Bytes and views need no close.
- Whole-owner finally returns were repaired in `ed469b32` using initialized stack
  slots. Partial-field direct cleanup routes landed in `eba57bd2`, preserving
  returned allocations and sibling liveness without cloning or placeholders.

## Current checkpoint: lexical disposal through partial returns

Map callback return projections and cleanup carriers were repaired in `4d887f9d`.
Whole-owner completion scopes were restored in `a26bda43`. This checkpoint extends
lexical disposal to ordinary partial-field returns: Normal completions carry the
surviving fields, while Return carries the returned value. Body locals leave their
real Rust scope before cleanup. Normal fields are restored before cleanup and
survivors restored again directly in the final Normal dispatch arm.

Checked ownership snapshots now distinguish Body, Handled and Cleaned phases.
Separate input/output field sets support conditional cleanup returns and fields
consumed during cleanup. Original storage retains untouched siblings and their
borrows. No cloning, dummy values or manual whole-record drops are introduced.
Parameters and locals receive native mutability only where restoration needs it.

Focused native probes cover conditional/unconditional partial returns, nested
cleanup, eligible catch returns, local record partial moves, allocation identity
and exact disposal at O0/O3. Terminal return/throw bodies need no Normal field
carrier and retain their lexical boundary directly. Escaping break/continue paths
are excluded from that shortcut. Dead catches are not emitted by field transport;
catch transfer guards include nested expression spans, not only statement spans
(which can cover just an if/while keyword).

The original ordinary and Map partial-return disposal witnesses are enabled and
pass. Map completion bodies now carry normal fields beside independently typed Map
owners. Cleanup carries its surviving fields back to the normal continuation.
Eligible catches carry body-normal fields only after selecting the normal branch;
moving those fields into the body's completion prematurely breaks catch ownership
correlation. Body/catch locals leave real Rust scopes before cleanup. Terminal body
returns and unconditional cleanup replacement need no surviving normal fields.

Focused O0/O3 probes cover Map body/catch partial returns, terminal returns,
unconditional cleanup replacement, pointer identity and exact local disposal.
A separate callback-environment probe verifies exact disposal and allocation
identity. Integration cases verify conditional cleanup returns and terminal
branches with different callback origins. See the acceptance map for final counts.

Ordinary field-return loops now carry a separate field tuple for every escaping
break/continue destination. Handled ownership snapshots select fields entering
cleanup; Cleaned snapshots select survivors for the eventual loop exit. Body scopes
end before cleanup. Cleanup can replace a pending return with break. Normal branches
that have no checked completion diverge instead of inventing field initialization.
Nested cleanup, continue/update, labeled exits, normal fallthrough and a Map-backed
loop disposal probe pass. Empty native ownership payloads now render the valid unit
tuple. The original loop-control witness is enabled.

Remaining ignored native witness: `body_field_return_and_catch_keep_lexical_disposal`.
One body branch returns a field; another throws to an empty catch. The body local
still survives into cleanup on the return path. Field ownership must travel on the
throw/error completion and be restored before catch, independently of normal and
return payloads. `completion_field_plan` currently rejects body-field transfers when
a reachable catch exists. The new `field_control_transport.rs` also leaves catches
to existing routes. Specialized/captured record layouts still need coverage. Do not
accept resource cleanup yet; preserve real scopes without manual drops or clones.

Public fixtures and matched `exception_completion` now exercise loop field returns,
break, continue and normal fallthrough with cleanup trace. Matched TypeScript,
independent closed-form oracle, mutation guards and hashes are current. Exact red
and repaired records for this checkpoint use SQLite prefix `tsrust-field-control-`.
See the acceptance map for final counts and untimed verification. No timing or full
family regression ran. Installed CLI/editor remain `4367f4cc`; scratch release is
rebuilt from this checkpoint before acceptance.

Then finish recursive stored callback fields, gate 190's resource backend and
synchronous file proof, and S07 family acceptance: complete compiler/editor
regressions, matched measurements, regression review, justified optimization and
consumer refresh. Continue through the existing S08–S21 roadmap after S07. Preserve
completed evidence; rerun affected checks and resume unrun measurement groups.

## Working environment

One agent in this tree. `CARGO_INCREMENTAL=0`,
`CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target`, cargo `--offline --locked`.
Node 24.11.1 is first on PATH; use `/opt/homebrew/bin/python3` for SQLite blob imports.
Format the whole project before every source checkpoint with
`TSRUST=/private/tmp/tsrust-gate100-editor-target/debug/tsrust node tools/format-project.mjs`.
Keep focused TDD and milestone-only heavy checks. Import/read back exact closed
records before deleting loose copies. Prune obsolete native/test outputs and retain
current executable consumers. Do not create archive directories or snapshots.
