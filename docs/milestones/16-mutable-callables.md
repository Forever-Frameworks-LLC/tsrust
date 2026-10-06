# Milestone 16: mutable callable bindings

[Capture replacement](./15-capture-replacement.md) · [Mutable locals](./14-mutable-bindings.md) · [Roadmap](../roadmap.md)

**Status:** Complete. [Milestone 17](./17-callable-moves.md) subsequently adds concrete callable declaration moves; the scope below records the milestone 16 boundary. The full 207-group regression suite passes, including 10 mutable callable groups. Formatting, Clippy, and local documentation links are verified.

## Concrete environment identity

`let` can bind a concrete closure and replace it with another value of exactly the same environment type. The binding infers that type from its initializer. Calling the same factory again produces a new value of the same type, with independent capture state.

```typescript
function make(text: string): () => string {
  return (): string => { return text; };
}
function main() {
  let action = make("Pat");
  console.log(action());
  action = make("Sam");
  console.log(action());
}
```

This prints `Pat` and `Sam`. Each environment is single-use; the assignment restores the binding with a fresh environment. Calls to a callable-valued arrow factory use the same rule.

Matching argument and result signatures alone are insufficient. Separate arrow expressions or factory origins have different concrete types, even when their signatures match. Such replacement produces `callable-origin` with related locations for both environment definitions. No runtime union, boxing, type erasure, or implicit adapter is added. Specializations retain their existing concrete identities.

## Replacement and transfer

Replacement evaluates its right-hand side before committing. It can invoke the old environment to compute factory arguments, including consuming it: `action = make(action())` works when `make` returns the same environment type. Reading an environment already consumed by an earlier statement still fails.

`first = second` moves a same-origin named environment into an existing mutable binding and makes `second` unavailable. It does not create an alias. Self-assignment moves and restores an available binding. The existing restriction on initializing an ordinary callable alias (`let second = first`) remains outside this gate, as do direct callback-parameter returns and general callback storage. `const` and callback parameter bindings cannot be replaced.

Replacing an available owned environment drops its previous value with normal Rust assignment semantics. Replacing a consumed environment restores availability. Invocation access and argument contracts remain unchanged because the concrete type is unchanged.

## Loans, captures, and loops

A wrapper borrowing a callable prevents replacement of that callable for the loan's lexical lifetime. Borrowed callable environments retain their original capture loans through replacement; this gate does not shorten loan lifetimes or permit borrowed returns.

Closures can replace captured `let` callable bindings using the existing mutable-capture rules. A local wrapper borrows the callable exclusively; an escaping wrapper owns it and can preserve replacements across repeated invocations. A right-hand side that consumes a captured environment still makes the wrapper single-use under the established capture contract.

Branch joins require restoration on every continuing path before use. Loops can consume a callable and replace it before repeating; omitting restoration on any back edge fails. Post-loop availability still includes the zero-iteration path, condition effects, and all break exits. Return paths are excluded as before.

## Evidence and remaining scope

The [mutable callable suite](../../crates/tsrust-compiler/tests/mutable_callables.rs) contains 10 groups covering native replacement output, arrow/ordinary factory identity, single-use reinitialization, forwarding transfers, right-hand-side invocation, named moves, self-assignment, borrowed loans, branch/loop joins, captured replacements, origin/type diagnostics, and backend-independent analysis. Generated output is checked for absent boxing, trait objects, reference counting, and repair clones.

The [reinitialization fixture](../../crates/tsrust-compiler/tests/fixtures/mutable-callables/reinitialize.tsr) prints `Pat` followed by `Sam` three times. The full suite retains all earlier ownership and closure evidence.

Mixed environment types, signature-only callable variable annotations, callable aliases/declaration moves, generic callback storage, shared capture cells, collections, async, modules, and the language server remain separate scopes. None blocks this concrete mutable-callable gate.
