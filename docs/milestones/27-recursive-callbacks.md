# Milestone 27: recursive callback parameters

[Recursive data functions](./26-recursive-functions.md) · [Callback contracts](./04-synchronous-callbacks.md) · [Single-use callbacks](./05-single-use-callbacks.md) · [Compiler workstream](../guides/language/compiler-and-modules.md) · [Roadmap](../roadmap.md)

**Status:** Complete. All 339 regression groups pass, including 14 recursive-callback groups. Formatting and Clippy pass.

## Named recursion with callbacks

Self and mutual named-function recursion supports reusable and ordinary single-use callback parameters with non-callable results. Declaration order does not change the contracts. Exact callback argument types, result types, arity, argument access, and invocation kind remain required. Existing recursive data ownership inference resolves all read/mutate/consume effects before definitive validation.

```typescript
type Node = { value: int; children: Node[]; };
function visit(node: Node, action: (value: int) => void) {
  action(node.value);
  for (const child of node.children) {
    visit(child, action);
  }
}
```

`visit.node` has a read contract and `visit.action` a mutate contract. A callback can update borrowed outer state while traversing the owned tree. The [runnable fixture](../../crates/tsrust-compiler/tests/fixtures/recursive-callbacks/visit.tsr) prints `6`, then releases its block-scoped callback loan. The caller retains both the tree and callback environment.

Unused reusable parameters infer read; invocation or forwarding to an invoking function propagates mutate. Declaration aliases such as `const next = action` transfer the local callback borrow and preserve its source contract and concrete native type. Permuting callback positions propagates invocation effects across all reachable positions. Owned data parameters may simultaneously permute and acquire consuming contracts through the same fixed-point analysis.

Ordinary `once` annotations always consume the parameter, even when unused. A recursive path may forward that environment until a base case invokes it. Branch joins and loops still enforce at most one invocation or forwarding per path. Concrete owned or borrowed environments can satisfy ordinary single-use contracts when they do not escape. Borrowed environments retain their caller loans until the synchronous call ends; named capture loans retain their existing lexical scope. Reusable and single-use parameter annotations remain distinct and cannot substitute for each other.

## Finite native type instantiation

Callbacks lower through the existing generic `Callback` and `OnceCallback` traits with static dispatch. Borrowed owned arguments retain independently quantified native lifetimes. No source-value clones, boxing, type erasure, reference counting, or new callback wrappers are inserted for recursion.

Fresh closed arrows and ordinary factory results can replace forwarded callback values. Non-owned callback parameters still cannot be captured, including by a local wrapper; that restriction prevents recursive forwarding from building environments containing their own generic parameter types. Concrete type vectors therefore contain flat environment identities. Runtime capture values and erased borrow lifetimes do not add types.

Flat types can still produce many native function instances when distinct callbacks exchange positions. After definitive ownership checking, an iterative traversal records typed calls and propagates concrete callback vectors through native functions. Every nongeneric named function and concrete closure method is an entry context, including unused concrete declarations. An unused generic declaration with no concrete entry remains type/ownership checked but contributes no instantiated vector. Literal conditions do not prune possible typed calls.

At most 128 distinct callback type vectors may instantiate each recursive callback function. Excess produces `unsupported` at the call with a related function declaration location, before Rust emission or backend invocation. This bound is per recursive function; the existing 128-entry owned-specialization bound remains separate. Ordinary acyclic/data-only programs avoid this traversal. Inference passes do not enforce the vector bound on provisional closure metadata.

The suite compiles and runs exactly 128 distinct callback environments, rejects the 129th, accepts 24- and 120-vector permutations, and rejects excessive permutations of seven distinct types. Seven values from the same factory environment type count once. Excessive calls inside a concrete closure method are checked even when that closure is never invoked. Uninstantiated generic declarations remain accepted.

## Evidence and remaining boundaries

The [recursive-callback suite](../../crates/tsrust-compiler/tests/recursive_callbacks.rs) verifies tree traversal, reusable state and operand order, self/mutual forwarding and aliases, single-use owned/borrowed environments, exact scalar/read/mutate/consume arguments, stable data transfer effects, later-operand overlaps, capture conflicts, shared iteration, fresh arrow/factory replacement, type-vector limits, source depth guards, deterministic public metadata and Rust output, and CLI checking/emission with an empty executable search path. Existing suites continue checking arithmetic, bounds, transfers, and native behavior. [Analysis measurements](../measurements/analysis-performance.md) record callback workloads separately from native compilation.

Cycles containing owned callback parameters or callable results still receive `unsupported` with a related cycle location. Recursive retention and growing owned environment types require a separate contract. Non-owned callback captures, local self-referential environments, callable-valued callback signature results, shared aliases, partial field moves, shared ownership, source generics, modules, async, and the Rust language server remain separately scoped. The compiler does not prove termination, enforce a runtime recursion depth, or guarantee tail-call optimization.

[Milestone 28](./28-recursive-owned-callbacks.md) subsequently extends recursive parameters to owned reusable/single-use contracts. It adds iterative data-result specialization and canonical source arrow layouts while preserving this gate’s borrowed type-vector guard. Callable-result cycles retain their separate boundary.
