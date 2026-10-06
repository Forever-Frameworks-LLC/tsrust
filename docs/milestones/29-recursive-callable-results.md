# Milestone 29: recursive callable results

[Recursive owned callbacks](./28-recursive-owned-callbacks.md) · [Callable results](./10-callable-results.md) · [Roadmap](../roadmap.md)

**Status:** Complete. All 370 regression groups pass, including 16 recursive-callable-result groups. Formatting and Clippy pass.

## One concrete result origin

Self and mutual named-function cycles can return callable values when every return resolves to one concrete environment origin. Branch and declaration order do not change acceptance. Direct returns, declaration aliases, forwarding through other functions, owned callback parameter returns, retained wrappers, and nested callable result annotations preserve their existing contracts.

```typescript
function counter(start: int, depth: int): () => int {
  if (depth > 0) {
    return counter(start, depth - 1);
  }
  let value = start;
  return (): int => {
    value = value + 1;
    return value;
  };
}
```

The returned environment owns its scalar counter and retains mutation across calls. The [fixture](../../crates/tsrust-compiler/tests/fixtures/recursive-callable-results/counter.tsr) uses a named declaration move and prints `2` and `3`. Owned record/string captures transfer into returned environments; consuming invocation remains single-use. Exact argument/result/access matching applies recursively to callable signatures.

Returned closures may call their own factory or mutually recursive factories during later invocation. Lazy nested factories also work when a concrete inner origin is established. Runtime evaluation remains in source order; discovery does not execute source or reorder emitted statements. No source-value clones, boxing, type erasure, reference counting, or runtime unions are introduced.

## Origin discovery and definitive validation

Acyclic programs retain their existing checking path. Recursive analysis first explores callable-return source scopes for concrete seeds. Known results propagate through named calls and aliases, including the existing substitution rules for owned callback parameter returns. Concrete owned factory specializations are discovered with their actual environment types; the global 128-specialization bound remains enforced.

Declared callable shapes permit provisional local invocation before a factory's origin is known. A shape cannot supply a concrete returned origin or become an environment capture. Within a callable-valued arrow body, concrete return seeds can establish the result independently of unresolved forwarding arms. A cycle with no concrete seed, or one requiring recursively embedded inline environment storage, receives `unsupported` at its declaration with a related unresolved call location when available.

Discovery uses speculative syntax and effect checking. It explores source scopes independently of conditions and defers origin-dependent declarations when necessary. It does not establish final availability, loans, invocation rights, or return coverage. Existing monotone data summaries stabilize before a fresh definitive pass. Provisional changes in capture access cannot cause a final mixed-origin rejection; final checking still rejects genuinely different environments.

Normal checking validates every source path and refreshes cached environment bodies, captures, and parameter indices. Source arrow type identity uses captured binding names/types/storage/access and callable contracts, rather than provisional local binding positions. This preserves type identity when discovery defers an earlier local declaration; fresh construction still applies current values, transfers, and loans.

Before public metadata or emission, an iterative pass removes discovery-only environments and remaps definitive closure references. Unresolved references cannot become a successful analysis. Cleanup is skipped when every environment is already definitive; data-only recursion avoids unnecessary tree rewriting. Public closure IDs remain deterministic within an analysis and contain no declared-shape artifacts.

## Preserved ownership and limits

All four callback contracts may participate in callable-return cycles. Ordinary callbacks remain synchronous and non-escaping. Owned parameters retain capture/return rights; owned single-use returns preserve consuming invocation even for reusable concrete arguments. Capture conflicts, transferred values, repeated consumption, exact owned argument access, source guards, and native specialization limits remain enforced.

The suite compiles 128 distinct owned factory specializations and rejects a 129th. Growing return environments fail in frontend checking. Twenty-function result cycles run natively; oversized source dependency cycles retain their existing 128-level diagnostics. Termination, constant-stack execution, and tail-call optimization are not guaranteed.

## Evidence and remaining boundaries

The [recursive-result suite](../../crates/tsrust-compiler/tests/recursive_callable_results.rs) covers eager/lazy and mutual factories, branch/declaration order, aliases, retained mutation, owned data capture, consuming invocation, ordinary and owned callbacks, direct parameter returns, retained wrappers, scalar/read/mutate/consume arguments, nested eager/lazy results, stable capture effects, caller loans, signature errors, mixed/missing origins, recursive environment storage, source/specialization guards, deferred local binding positions, deterministic clean metadata, and frontend-only CLI operation. [Measurements](../analysis-performance.md) separate frontend result discovery from native compilation.

Mixed callable origins, borrowed results, shared aliases, local self-referential environments, recursive inline environment storage, callable fields, callable-valued callback signature results, partial field moves, shared ownership, source generics, modules, async, and the Rust language server remain separately scoped.

Milestone 31 subsequently adds [direct named record field moves and restoration](./31-record-field-moves.md), tracking partial availability through joins and loops. Indexed and iteration element fields remain borrowed; conservative root loans and whole environment capture storage remain in force.
