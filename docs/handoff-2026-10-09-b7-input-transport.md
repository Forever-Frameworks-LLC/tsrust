# B7 input transport checkpoint

B5 remains accepted at `00214a11`. B7/gate 183 is in progress, not accepted.
Discriminated unions remain a dedicated required D2 feature; see
`docs/discriminated-unions-plan.md`. There is no new owner decision pending.

Read AGENTS.md, docs/development-workflow.md, the B5 acceptance handoff and
`docs/caller-borrowed-record-stores-plan.md` before continuing. Preserve the fifteen
unrelated fixture edits. Use explicit Git paths and the shared Cargo target with
incremental compilation disabled. Node 24.11.1 is first on PATH.

## Implemented slice

`record_parameter_borrows.rs` collects retained input provenance and maps it through
named relay arguments. Named calls attach the resulting capture loans to holder
paths. Scope-exit validation rejects an inner owner retained by an outer holder.
The carrier eligibility check admits direct borrowed input captures while retaining
callee-local rejection. CheckedProgram carries the final borrow summaries.

`record_parameter_borrow_emit.rs` supplies explicit source-input lifetimes in native
function parameters and replacement bounds. Replacement bounds select the stored
origin and destination path together. Without that path, two calls to the same
installer incorrectly demanded that independent source lifetimes outlive each other.
The focused red witness and repaired run are preserved.

## Evidence

SQLite contains exact logs under `tsrust-b7-` paths. Every closed imported log was
read back byte for byte before its loose copy was deleted.

- `destination-lifetimes.log`: seven focused compiler/native tests pass. These cover
  readonly and mutable array inputs, reordered relay parameters, independent calls,
  repeated origin at distinct destinations, mutation/move conflicts, callee-local
  rejection and an inner owner retained by an outer holder.
- `scope-controls.log`: five earlier focused tests and nine replacement tests pass.
- `composition-regression.log`: all 201 existing composition tests pass. This run
  precedes the final destination-specific bound correction; source scope validation
  was included. Do not claim a second run or a complete compiler regression.
- `depth-canaries.log`: all eight depth canaries pass after the analysis dispatch changes.
- `editor-inputs.log`: focused diagnostics, hover and signature-help test passes.
- `inputs-node.log`: independent readonly relay and mutable-input reference assertions pass.
- `repeated-origin-red.log` and `short-owner-red.log` preserve discovered defects.

No B7 workload measurement or family acceptance has occurred. Installed consumers
remain the last accepted B5 binaries; do not describe them as refreshed B7 consumers.

## Next work

Continue the whole B7 plan. First add nested record and multiple-capture witnesses,
then repeated writes to one destination with different inputs. The emitter currently
chooses the last matching source lifetime for that destination, so this needs a real
representation/bound solution rather than a diagnostic or a lifetime cast.

Input aliases need owner provenance, not a binding.parameter shortcut. Direct
borrowed callback stores and nullable stores retain their earlier rejection guards.
The direct collector currently recognizes Closure values, not BorrowedCallable
families. Cover these intentionally rather than removing the guards blindly.

Relay summary propagation repeatedly appends destination paths. Establish finite
behavior for recursive/nested relays; do not accept an unbounded fixed point. Current
loan transport retains possible stores conservatively. Exceptional paths, exact
replacement/clearing, loops, early exits and disposal still require evidence and
integration with record_call_capture_release. Add matched workload support before
closing the family; run heavy regression/timing only at that family milestone.
