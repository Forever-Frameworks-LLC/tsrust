# Milestone 11: retained owned callbacks

[Callable captures](./09-callable-captures.md) · [Callable results](./10-callable-results.md) · [Roadmap](../roadmap.md)

**Status:** Complete. The five exit gates below are resolved.

## Contract and syntax

`action: owned (value: read User) => string` transfers a fully owned callable environment to the receiving function and permits repeated invocation. `action: owned once (value: consume User) => string` transfers the environment and permits at most one invocation or forwarding per continuing path. Both annotations expose a consuming parameter contract even when unused. `owned` and `once` are contextual; existing record and value names remain valid.

An owned callback may be captured by a local or returned wrapper. Returned wrappers own that environment, so they remain valid after the receiving function returns. Reusable wrappers preserve mutable state across invocations. A wrapper invoking an `owned once` parameter remains single-use even when the concrete argument happens to be reusable.

```typescript
function retain(action: owned () => string): () => string {
  return (): string => { return action(); };
}
function main() {
  const text = " Pat ";
  const callback = retain((): string => { return text.trim(); });
  console.log(callback());
  console.log(callback());
}
```

Passing an inline arrow to an owned parameter classifies its captures as owned before creation. It transfers owned captures and copies scalars. Passing a named environment requires that it already contain no borrowed captures, recursively. A borrowed named closure is rejected rather than silently changing its creation effects. Capture conflicts, left-to-right argument evaluation, branch joins, and lexical loan lifetimes continue to apply.

All callback annotations require exact argument types, access modes, result types, ownership, and invocation kinds. Ordinary reusable and `once` parameters retain their synchronous non-escape rules. There is no implicit conversion between parameter contracts; a caller can explicitly construct an appropriate wrapper when its body satisfies the destination contract.

## Analysis and Rust lowering

The frontend checks every owned-callback function as a template, including unused declarations. Calls with concrete owned environments specialize the checked function by its ordered owned-callback argument types. Specializations are cached within one analysis and limited to 128 across the source file. Exceeding the bound produces a source diagnostic. Templates preserve the declared invocation contract during specialization.

Only concrete specializations are emitted. Captures remain concrete Rust struct fields, and calls use static dispatch. No boxing, dynamic dispatch, reference counting, or ownership-repair cloning is inserted. Ordinary function bodies still move into analysis; only owned-callback templates retain syntax copies for specialization. This is a bounded implementation strategy, not source-level generics or a performance guarantee.

`ClosureInfo.template` and JSON `closures[].template` distinguish checked template closures from concrete environments. Text checks also label templates. IDs remain deterministic within an analysis. `contracts()` reports the source declaration once, without duplicate specialization entries.

## Completed exit gates

1. Parse reusable and single-use owned callback annotations, preserving contextual identifiers and exact signature checks.
2. Enforce ownership transfer, non-borrowed environment storage, retained capture behavior, branch availability, and single-use invocation.
3. Generate cached concrete specializations, preserve declared contracts, and diagnose excessive specialization before backend execution.
4. Compare native output and library/CLI diagnostics for stateful callbacks, forwarding, owned arguments, multiple environments, local loans, invalid captures, unused declarations, and specialization boundaries. Empty-`PATH` checks prove frontend independence.
5. Pass the full 149-group regression suite, formatting, Clippy, and documentation checks. Ten groups live in [retained callback tests](../../crates/tsrust-compiler/tests/retained_callbacks.rs).

## Remaining boundaries

Retention is demonstrated through returned owned wrappers. Callback aliases, direct callback-parameter returns, callable fields/globals, callable-valued arrow parameters, and callable-valued callback signature results remain unsupported. Callable-valued ordinary-function and arrow results are supported by milestone 10, with one concrete origin per result.

Borrowed results, mixed closure origins, shared ownership, cross-thread use, library callback registries, UI handlers, async tasks, cancellation, and module contracts require separate scopes. None blocks the completed owned-retention gate.
