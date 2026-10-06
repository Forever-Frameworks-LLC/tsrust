# Milestone 18: callback-parameter moves

[Concrete callable moves](./17-callable-moves.md) · [Retained callbacks](./11-retained-callbacks.md) · [Roadmap](../roadmap.md)

**Status:** Complete. [Milestone 19](./19-callback-returns.md) subsequently adds direct owned-callback returns; the scope below records the milestone 18 boundary. The full 227-group regression suite passes, including 10 callback move groups. Formatting, Clippy, and local documentation links are verified.

## Moving a callback into a local binding

All four callback parameter contracts support declaration moves into `const` or `let`. The destination preserves the source contract, and the source becomes unavailable. Chained moves preserve the same rules.

```typescript
function use(action: () => int) {
  const next = action;
  console.log(next());
  console.log(next());
}
function main() {
  const action = (): int => { return 7; };
  use(action);
  console.log(action());
}
```

This prints `7` three times. A reusable parameter holds a borrow from the caller. Moving it transfers that local borrow; the caller retains ownership and can invoke its environment after the call returns. The function's inferred access remains exclusive borrowing rather than consuming ownership. Reading the moved source reports `callback-consumed` with the transfer location.

| Parameter contract | What moves | Destination invocation | Capture permission |
| --- | --- | --- | --- |
| Reusable | The local exclusive borrow | Repeated | No capture |
| `once` | The received environment | At most once | No capture |
| `owned` | The received environment | Repeated | Existing owned capture rules |
| `owned once` | The received environment | At most once | Existing owned capture rules |

The `once` contract remains single-use even when the concrete argument supports repeated invocation. Owned specialization preserves that declared restriction across moves and captures. Argument access contracts also remain unchanged, including read, mutate, and consume access to strings and records.

## Forwarding, captures, and replacement

Moved callbacks can be forwarded through matching parameter contracts. Ownership kinds remain exact: moving a reusable borrow does not turn it into an owned callback. Simultaneous exclusive arguments still conflict. No shared alias, clone, allocation wrapper, or dynamic dispatch is inserted.

Owned callback locals can be captured by returned wrappers. Repeated wrapper invocation preserves retained state when the contract is reusable. Moving a captured callback inside a wrapper makes that wrapper single-use. Non-owned callback locals cannot be captured, including after a chain of moves; their existing synchronous boundaries remain enforced.

A `let` callback local can receive a value derived from the same original parameter. This allows moving a local to temporary storage and restoring it, or self-assignment while available. A different parameter or a fresh concrete environment produces `callable-origin`, even with a matching signature. Separate generic parameters can represent different concrete environments, so signature matching alone does not justify replacement. Original parameters and `const` destinations remain non-rebindable. A consumed single-use value cannot restore itself.

## Control flow and lowering

Moves participate in branch availability joins, including the exclusion of terminated paths. Loop back edges require restoration of initially available callback locals. Break exits contribute transferred state to post-loop availability. Moving an original parameter inside a repeating loop fails because parameters cannot be restored by assignment.

Reusable and ordinary single-use callbacks retain generic Rust parameters. Their moved local types are inferred from those parameters. Reusable locals retain reference representation for invocation and forwarding. Owned templates still validate abstract contracts before generating cached concrete specializations; local move metadata retains their declared invocation contract. Frontend checking and Rust emission require no backend process.

## Evidence and remaining scope

The [callback move suite](../../crates/tsrust-compiler/tests/callback_moves.rs) covers caller ownership, source diagnostics, inferred access, owned arguments, mutation, once invocation, owned retention, captures, replacement identity, branches, loops, forwarding conflicts, specialization, and backend-independent analysis. Accepted programs compile with the pinned Rust backend and compare native output. Earlier callback suites now accept their formerly rejected declaration moves while retaining invalid return and capture checks.

The [retained counter fixture](../../crates/tsrust-compiler/tests/fixtures/callback-moves/retained.tsr) moves an owned parameter into a local captured by a returned wrapper. Independent counters print `1`, `2`, and `1`.

Direct callback-parameter returns, shared aliases, callable fields/globals, signature-only callable variable annotations, non-owned callback captures, mixed environment types, borrowed callable returns, collections, async, modules, and the language server remain separate scopes.
