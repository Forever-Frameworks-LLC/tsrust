# Milestone 31: record field moves and restoration

[Roadmap](../roadmap.md) · [Nested records](./25-nested-record-fields.md) · [Array removal](./30-array-removal.md)

**Status:** Complete. All 404 regression groups pass, including 18 record-field-move groups. Formatting and Clippy pass.

## Ownership of individual fields

Moving a string, record, or array field transfers that field without cloning. Direct named field paths such as `profile.details.name` support moves into declarations, assignments, calls, aggregate literals, and returns. Exact nominal/rank types still apply. Scalar fields copy and do not become missing.

Siblings remain available after a move. Reading, borrowing, cloning, capturing, or transferring the whole record or a partially missing parent fails with `partial-transfer`. The diagnostic identifies the missing field and related original transfer. Writing the exact missing field restores it after successful right-hand evaluation and type checking. Replacing a parent restores all of its descendants. A missing parent cannot be reconstructed by writing one of its children. Initialized local `let` records may also be replaced in full under the existing rebinding rules.

```typescript
type Profile = { name: string; tags: string[]; };
function main() {
  const profile: Profile = { name: "Pat", tags: ["admin"] };
  const name = profile.name;
  console.log(profile.tags[0]);
  profile.name = "Sam";
  const restored = profile;
}
```

The record retains ownership of its other fields while `name` owns the detached string. Leaving a record partially moved is valid at scope exit; native Rust drops only remaining initialized fields. Restoration requires a direct assignment place and does not borrow uninitialized field storage.

## Joins, loops, and contracts

Continuing branch paths join their missing-field sets. Returned/broken/continued paths retain the established exit rules. Every loop back edge must preserve the availability present at entry; a field moved within an iteration must be restored before that edge unless it was already missing at entry. Zero-iteration and break paths contribute their availability to the post-loop join. Existing `loop-transfer` diagnostics apply to new missing fields.

Moving a parameter's field infers consuming access to the parameter. The caller transfers the whole record to that function; the callee may drop its remaining fields or return a restored record. Callable argument annotations must match this consuming access exactly. Recursive effect inference and owned callback specialization preserve the same rules.

Capture storage still operates on whole root bindings. A consuming field use makes invocation single-use; creation moves the whole captured record into the environment. A partially missing outer record cannot be captured, including for a sibling-only body. Move or explicitly clone a complete field into a separate local before capturing it. Existing root-owner argument, capture, iteration, and selection loans remain conservative across sibling fields.

An owned field argument has detached storage after evaluation and does not retain a root loan through later arguments. Borrowed field arguments retain their root loans. Evaluation order remains left to right.

## Native lowering and analysis

Availability combines whole-binding transfer state with deterministic missing field paths. Ordinary fully initialized field reads do not allocate a projection path. Partial-record access validates the complete selected path rather than borrowing a missing parent. Indexed paths validate the first complete array field; indexed elements cannot themselves be partially moved.

The emitter constructs field places directly and creates references only for selected complete storage. Mixed field/index paths preserve saved indices, stable parent selection, and existing bounds failure timing. Moves and restoration use native Rust places and drop flags; no runtime field-state map, repair cloning, boxing, unsafe code, or optional field representation is introduced.

## Evidence and boundaries

The [fixture](../../crates/tsrust-compiler/tests/fixtures/field-moves/profile.tsr) detaches a nested name, reads its array/scalar siblings, restores the missing field, transfers the whole restored record, and returns the detached string. It prints `admin`, `7`, `New`, and `Pat` without generated cloning, boxing, or unsafe code.

The [18-group suite](../../crates/tsrust-compiler/tests/field_moves.rs) verifies exact string/record/array moves, scalar copies, sibling access, repeated/overlapping/whole-record rejection, related source transfers, parent restoration, missing-parent writes, right-hand evaluation, whole let replacement, branch and short-circuit joins, loop back/break/continue states, preexisting missing fields, whole capture storage, consuming invocation, all callback contracts, mutual recursion and owned specialization, recursive returned factories, detached argument loans, direct aggregate construction, indexed/iteration restrictions, active loans, stable mixed paths, deterministic metadata, and backend-independent CLI operation.

A 100-level native record path moves and restores its deepest string while reading the remaining sibling. Bounds failures after a sibling transfer run at optimization levels 0 and 3 with overflow checks disabled, preserving prior stdout and exact source ranges. Existing storage suites retain their deep/layout/rank limits, operand-order checks, and native witnesses. [Frontend measurements](../analysis-performance.md#gate-31-measurements) keep partial-state analysis separate from emission/native compilation.

Indexing and for-of still borrow owned elements; moving their fields fails with `borrowed-element`. Remove an element first or explicitly clone its field. Borrowed results, disjoint field loan inference, partial environment captures, callable fields, user-defined drop behavior, mutable/consuming iteration, modules, and async remain separately scoped.

## Runtime comparison follow-up

The [runtime comparison ledger](../runtime-comparisons.md) adds the requested catch-up measurements after this compiler gate. `field_restore` compares detached named fields, sibling reads, and field/whole-owner restoration against independently authored TypeScript with the same observed numeric result. JavaScript does not enforce partial-move availability. The ledger also records previous observable operations, compiler/startup costs, and separate primitive allocation probes. Future gates require an accurate counterpart or an explicit non-comparability note before advancing.
