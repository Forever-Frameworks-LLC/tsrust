# Milestone 26: recursive functions

[Nested records](./25-nested-record-fields.md) · [Compiler workstream](../compiler-and-modules.md) · [Ownership workstream](../ownership-and-memory.md) · [Roadmap](../roadmap.md)

**Status:** Complete. All 325 regression groups pass, including 16 recursive-function groups. Formatting and Clippy pass.

## Named recursion and data contracts

Named functions support self and mutual recursion with scalar, string, record, and array parameters/results. All function names and declared types resolve before body analysis. Declaration order does not change inferred access. Scalar arguments copy; owned arguments infer read, mutate, or consume through the entire call cycle.

```typescript
type Node = { value: int; children: Node[]; };
function total(node: Node): int {
  let sum = node.value;
  for (const child of node.children) {
    sum = sum + total(child);
  }
  return sum;
}
```

`total.node` has a read contract, including recursive calls on borrowed children. A recursive updater using indexed child arguments receives a mutate contract. A recursive function returning an input owner receives consume access; forwarding transfers that owner into the next invocation and preserves owned results. No source-value clones, boxing, reference counting, or runtime dispatch are inserted.

A function returning a non-void value still requires an explicit result annotation. Arguments match exact types and arity. Return coverage follows existing source rules. A call to a result-returning recursive function supplies a declared result type even before its body has finished checking. Unused functions are checked too.

## Stable access inference and definitive checking

Acyclic programs retain the existing callee-before-caller path without speculative syntax copies. Encountering an eligible call cycle starts bounded inference. Owned parameter summaries begin at read and increase through mutate to consume. Copied scalar summaries remain copy. Completed callees contribute their current body effects; recursive back edges use the preceding pass's assumed contracts.

Each coordinate rises at most twice. After no summary increases, a fresh analysis checks the original program against the stable contracts. Only this definitive result is exposed by the library, CLI, or Rust emitter. Provisional syntax, closure IDs, specializations, and loans are discarded. Public checking and emission remain entirely in process and require no Rustup or rustc.

Inference passes retain name, type, shape, and source-structure validation. Effect-dependent availability, loan, invocation, callable-access, and capture-layout checks wait for stable summaries. For example, an initially shared local capture may become an owned consuming capture, allowing replacement of its original `let` binding after environment creation. Provisional borrowed lifetime counts are capped and cannot reject a final owned environment; definitive borrowed environments still enforce the 128-lifetime guard.

Permuting recursive arguments propagates effects to every reachable parameter position. The suite verifies both consuming and mutable nine-parameter rotations and an owned 130-capture environment, so inference cannot stop after merely finding a syntactic self-call or a single changed parameter.

## Existing ownership and runtime rules

Stable contracts enforce transfers, overlapping arguments, parent selection loans, capture conflicts, branch availability, loop restoration, and shared iteration. Recursive calls cannot consume borrowed iteration elements or mutate an owner during shared iteration. Lexical and returned closures can call eligible recursive functions; their capture access and invocation modes use the stable effects. Nonrecursive callback APIs can invoke or forward closures whose bodies call recursive functions, with exact declared argument access checked afterward.

Operands execute once in source order. Native recursive calls retain explicit checked integer arithmetic and bounds failures with source byte ranges, including optimized builds with backend overflow checks disabled. The compiler does not prove termination, insert a runtime recursion limit, or guarantee tail-call optimization. Runtime call-stack behavior follows the native backend.

Parser and combined analysis nesting retain their existing 128-level guards. Compiler inference does not recursively unfold runtime invocations. Large source dependency cycles can still exceed the combined compiler analysis guard; cycle membership does not exempt their source nesting. Deep record literals are verified through speculative syntax copies and final native compilation.

## Evidence and boundaries

The [recursive-function suite](../../crates/tsrust-compiler/tests/recursion.rs) covers self/mutual calls, declaration order, scalar recursion, recursive tree reading/mutation, owned string/record/nested-array results, permuted summaries, invalid transfers and overlaps, parent loans, lexical and owned closure captures, callback contracts, ownership restoration, shared iteration, source evaluation order, optimized arithmetic/bounds failures, source depth guards, deep literal processing, deterministic public metadata/Rust output, and CLI frontend operation with an empty executable search path.

The [tree fixture](../../crates/tsrust-compiler/tests/fixtures/recursion/tree.tsr) prints `6` and `9`: shared traversal sums three nodes, exclusive traversal increments each node, and the same owner remains available afterward. The [analysis measurements](../analysis-performance.md) include recursive tree and permuted-parameter workloads separately from native execution.

Named analysis dependency cycles containing declarations with callback parameters or callable results receive `unsupported` with a related cycle location. Their concrete environment identity, recursive type instantiation, and specialization behavior require a separate contract; this gate does not infer arbitrary recursive environment types. Calling the program entry point, local self-referential callable environments, borrowed results, partial field moves, shared ownership, generics, modules, async, and the Rust language server remain separately scoped work.

[Milestone 27](./27-recursive-callbacks.md) subsequently extends this inference and definitive checking to ordinary reusable and single-use callback parameters. Owned callback cycles and callable results retain their separate environment boundary.
