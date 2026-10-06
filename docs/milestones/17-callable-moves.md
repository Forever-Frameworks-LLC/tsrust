# Milestone 17: callable declaration moves

[Mutable callables](./16-mutable-callables.md) · [Callable captures](./09-callable-captures.md) · [Roadmap](../roadmap.md)

**Status:** Complete. [Milestone 18](./18-callback-moves.md) subsequently adds callback-parameter declaration moves; the scope below records the milestone 17 boundary. The full 217-group regression suite passes, including 10 callable declaration move groups. Formatting, Clippy, and local documentation links are verified.

## Declaration ownership

A `const` or `let` declaration can initialize from a named concrete callable. This transfers the environment into the new binding, preserving its exact concrete type, invocation access, argument contracts, and captured state. The source becomes unavailable even when the environment is reusable or captures nothing. No clone, shared alias, boxing, or dynamic dispatch is inserted.

```typescript
function make(text: string): () => string {
  const first = (): string => { return text.trim(); };
  const second = first;
  return second;
}
function main() {
  const original = make(" Pat ");
  const action = original;
  console.log(action());
  console.log(action());
}
```

This prints `Pat` twice. Using `original` after its transfer reports `closure-consumed`, with a related location identifying the move. Moving a single-use callable preserves single-use invocation; moving an already consumed callable fails.

The destination's declaration determines whether that binding can be replaced. A `let` source can be restored independently with a same-origin factory result; a `const` destination cannot be replaced. Mixed-origin replacement remains rejected even when signatures match.

## Captures and returned chains

The escape-origin pass follows declaration chains back to the original arrow expression. Returning the last binding selects owned capture storage when the original environment is created. This works through nested blocks and ordinary or arrow factory bodies. Signature and single-origin return checks remain unchanged.

Moving a callable inside a capturing wrapper consumes the captured environment and makes the wrapper single-use, even when the moved callable itself supports repeated calls. An escaping wrapper cannot retain an environment with borrowed captures. Local borrowed environments can move between bindings without weakening their original lexical loans. Moving into a shorter-lived block does not release the original capture loan early.

An outstanding wrapper loan prevents moving its callable. Mutable capture loans also continue to prevent access to the captured storage; shared read loans permit shared reads but prevent replacement. These checks happen in the frontend before Rust emission.

## Control flow and callback boundaries

Moves participate in the existing branch and loop availability analysis. A move on any continuing branch makes the source unavailable afterward unless restored. Terminated paths do not affect subsequent availability. A loop back edge must restore an initially available source after moving it; transfers on break exits still affect post-loop availability. Iteration-local environments can be moved afresh each iteration.

A moved concrete callable can be passed under the existing reusable, single-use, and owned callback contracts. Callback parameters themselves cannot initialize declaration moves: their abstract contracts and specialized representations still require an explicit storage design. They receive `callback-escape` as before. General callable fields, signature-only variable annotations, shared aliases, mixed environment types, and borrowed callable returns remain outside this gate.

## Evidence

The [callable move suite](../../crates/tsrust-compiler/tests/callable_moves.rs) covers declaration chains, unavailable-source diagnostics, source/destination mutability, consuming invocation, owned argument contracts, lexical loans, returned origins, consuming captures, branch/loop joins, callback forwarding, remaining storage restrictions, and backend-independent check/emit. Accepted programs compile generated Rust with the pinned backend and compare native output. Generated Rust is checked for absent repair clones and allocation wrappers.

Earlier closure suites now compile their formerly rejected concrete declaration-move cases. Callback-parameter escape rejections remain in place. The [returned chain fixture](../../crates/tsrust-compiler/tests/fixtures/callable-moves/returned-chain.tsr) prints `Pat` twice after moving through local bindings and returning an owned environment.

Collections and iteration contracts, mixed callable environment types, general callback storage, shared capture cells, async, modules, broader measurements, and the Rust language server remain separately scoped work.
