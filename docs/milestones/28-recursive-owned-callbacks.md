# Milestone 28: recursive owned callbacks

[Recursive callbacks](./27-recursive-callbacks.md) · [Owned callback contracts](./11-retained-callbacks.md) · [Roadmap](../roadmap.md)

**Status:** Complete. All 354 regression groups pass, including 15 recursive-owned-callback groups. Formatting and Clippy pass.

## Owned environments through recursive calls

All four callback parameter contracts support self and mutual named-function recursion with non-callable results. `owned` and `owned once` parameters always transfer their environment, including when unused. Declaration aliases preserve that transfer and the annotation’s invocation promise. Abstract source templates are checked even when no concrete caller instantiates them.

```typescript
function countdown(action: owned () => int, remaining: int): int {
  if (remaining === 0) {
    return action();
  }
  const value = action();
  const next = action;
  return value + countdown(next, remaining - 1);
}
```

The same concrete environment moves into each invocation. A reusable owned callback retains mutable state across those moves; invoking it does not consume the environment. `owned once` can forward through aliases until one base-case invocation consumes it. Invocation or forwarding after transfer remains an error. Callers cannot reuse a transferred named callback.

Owned parameters keep their existing capture rights. A local wrapper can retain and invoke the callback, or invoke the recursive function with the retained callback. Owned single-use invocation remains single-use even when the concrete argument happens to be reusable. Named borrowed environments cannot satisfy an owned annotation; inline owned arguments own their captures before creation. Capture conflicts, argument evaluation order, loans, branch joins, loops, and shared iteration retain their existing rules.

Exact callback argument/result/access contracts remain required. Owned callbacks can read or mutate borrowed record arguments and consume fresh owners. Recursive permutation of data arguments still infers stable access before definitive checking. Ordinary borrowed and single-use parameters can accompany owned parameters without changing their own transfer, lifetime, or invocation contracts.

## Iterative specialization and stable arrow types

The existing specialization cache uses the source declaration and ordered owned environment types. Each entry is registered before body checking, so a recursive back edge reuses the active or pending concrete native function. Non-callable results already have complete declared types; their concrete bodies drain iteratively after source checking. Native type contexts therefore do not unfold on the compiler’s source-analysis stack. Callable-result specializations still require eager checking to resolve their returned environment.

A source arrow’s concrete type is reused when its template/escape classification, lexical depth, result/invocation/argument contracts, and ordered capture types/storage/access match. Capture binding positions and callback invocation/reference contracts also participate. Unused outer owned parameter types do not produce different environments. Captured callable types do, preserving environment identity when a wrapper actually contains a different callback type.

Every construction still evaluates fresh captured values and applies its transfer effects and loans. Only the concrete type and fully checked body are reused. Different source arrow origins remain distinct. Fresh inline arrows and factory results can replace recursive arguments without falsely growing identical types; nested owned wrappers whose captured callable types keep changing remain distinct.

The global 128-specialization bound remains unchanged. Exactly 128 concrete instances compile; a 129th receives `unsupported` before emission or backend execution. The suite also runs all 120 permutations of five distinct owned types. Growing inline and factory-produced wrappers reach a source resource diagnostic rather than causing unbounded monomorphization. Actual source nesting keeps the existing 128-level guards. Concrete owned specializations with ordinary callback parameters inherit milestone 27’s separate per-function 128-vector bound. Pure owned recursion skips ordinary vector traversal because its concrete specialization cache already supplies the bound.

No source-value clones, boxing, type erasure, reference counting, or runtime dispatch are introduced. Template syntax copies, inference passes, and cached concrete bodies remain compiler costs. [Measurements](../measurements/analysis-performance.md) separate these frontend costs from native compilation.

## Evidence and remaining boundaries

The [owned-recursion suite](../../crates/tsrust-compiler/tests/recursive_owned_callbacks.rs) verifies self/mutual calls and declaration order, aliases, retained mutable state, owned single-use captures, local wrappers calling recursion, exact read/mutate/consume arguments, stable data effects, capture conflicts, mixed ordinary/owned parameters, shared iteration and loop joins, fresh factory/inline replacement, canonical environment metadata, specialization reuse and limits, growing environments, inherited ordinary vector limits, source guards, deterministic public metadata/Rust output, and backend-independent CLI checking/emission/rejection.

The [counter fixture](../../crates/tsrust-compiler/tests/fixtures/recursive-owned-callbacks/countdown.tsr) prints `10` and `23` from two independent counter environments of the same native type. Both calls share one countdown specialization. Public source contracts contain only the original declarations; specialization entries remain internal.

Named analysis cycles containing callable results still require a separate environment/result contract. Local self-referential environments, mixed callable origins, shared aliases, callable fields, callable-valued callback signature results, partial field moves, shared ownership, source generics, modules, async, and the Rust language server remain separately scoped. Termination, constant-stack execution, and tail-call optimization are not guaranteed.

[Milestone 29](./29-recursive-callable-results.md) subsequently resolves single-origin callable-result cycles. Its discovery pass matches capture binding names rather than provisional binding positions, then refreshes definitive body and capture indices before emission.
