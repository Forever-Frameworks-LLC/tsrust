# Milestone 14: mutable local bindings

[While loops](./13-while-loops.md) · [Ownership workstream](../ownership-and-memory.md) · [Roadmap](../roadmap.md)

**Status:** Complete. All 187 regression groups pass, including 11 mutable binding groups. Formatting, Clippy, and local documentation links are verified. This gate adds initialized `let` bindings for integers, booleans, strings, and nominal records, with exact-type replacement and ownership reinitialization.

**Subsequent extension:** [Milestone 15](./15-capture-replacement.md) adds inherited `let` replacement inside closures. Writable local scalar captures borrow exclusively, and escaped writers modify private copied fields. The boundaries below record this gate's original scope.

## Declaration and replacement

```typescript
function take(text: string): string { return text; }
function main() {
  let count = 0;
  let text: string = "Pat";
  while (count < 3) {
    console.log(take(text));
    text = "Sam";
    count = count + 1;
  }
  console.log(text);
}
```

This prints `Pat`, `Sam`, `Sam`, and `Sam`. Each iteration transfers the current string, then initializes its binding with another string before repeating.

Like `const`, `let` requires an initializer and may infer its type. A record literal needs a named expected type. The binding's type never changes: assignments require the same scalar type or nominal record identity. Duplicate names and shadowing remain rejected.

`const` bindings and function/arrow parameters cannot be rebound. Field mutation remains distinct from replacement of the whole binding: both `const` and `let` records permit field mutation under the existing ownership rules. A closure may reassign a `let` declared in its own body.

The right-hand side evaluates once before replacement commits. It may read or consume the old value, so `text = text.trim()` and `text = take(text)` work. A prior transfer prevents reading the old value, but a fresh right-hand side can reinitialize the unavailable binding. Assignment itself does not read the old value. Replacing an initialized owned value drops that old value using normal Rust ownership behavior; no implicit clone is inserted.

## Flow and iteration ownership

Reinitialization clears the binding's transfer state only after the right-hand side has been checked. Branch joins require initialization on every continuing path before subsequent use. A branch that returns does not contribute to later availability. A transfer on one branch remains an error if another branch alone restores the binding.

At each loop back edge, every binding available at loop entry must again be available. A consume-then-replace sequence can therefore repeat, including through `continue`. Missing replacement on even one continuing branch produces `loop-transfer`. Break and return paths retain the existing exit rules.

A loop may execute zero times. A binding transferred before the loop does not become available afterward merely because the body restores it. Condition effects and break exits still determine post-loop availability. A previously transferred binding may be assigned fresh data before use within each iteration; the analysis does not assume initialization inherited from an earlier iteration.

These rules preserve the entry invariant without general fixed-point inference: analysis requires all initially available bindings to remain available on every back edge and makes no availability assumptions about initially transferred bindings. Arbitrary state-dependent initialization and broader control-flow inference remain separate work.

## Capture boundary

Replacing a binding cannot invalidate a live closure capture loan, even after that closure's last invocation. Loan lifetime remains lexical. Replacement is allowed after the borrowing block ends or after the old value has moved into an owned environment.

Existing scalar captures remain copied snapshots. For example, a closure created while `let count = 1` retains `1` after the outer binding is assigned `2`. This follows the existing inspectable `copy` capture contract; it is not a live reference to the mutable variable.

Replacement of an inherited captured binding from inside a closure remains unsupported. Mutate a captured record's fields for shared mutable state. Mutable callable bindings also remain unsupported; use `const` for callable environments. These boundaries avoid silently changing capture storage or introducing mixed environment types.

## Rust lowering and evidence

The analyzer tracks source-level replacement permission separately from inferred Rust mutability. Locals lower to ordinary Rust variables; only actual updates or existing mutation requirements introduce `mut`. Reassignment emits a direct assignment after ownership checking. The backend provides normal destruction of replaced strings/records.

The [mutable binding suite](../../crates/tsrust-compiler/tests/mutable_bindings.rs) contains 11 groups covering:

- Scalar loop updates and repeated owned-value replacement.
- Right-hand-side order, self-transfer/reinitialization, and invalid reads after transfer.
- Branch restoration, loop back edges, condition effects, breaks, and zero-iteration joins.
- Capture conflicts, lexical loan release, owned capture independence, and scalar snapshots.
- Fresh closure locals, exact types, const/parameter boundaries, and backend-independent checks.

Exit gates require native-output acceptance, matching library/CLI source diagnostics, full regressions, formatting, Clippy, and updated documentation. A [replacement fixture](../../crates/tsrust-compiler/tests/fixtures/mutable-bindings/reinitialize.tsr) demonstrates repeated ownership transfer and reinitialization.

Uninitialized declarations, assignment expressions, compound updates, captured-binding replacement, mutable callable bindings, collections, and general type/initialization inference remain outside this gate.
