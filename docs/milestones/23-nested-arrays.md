# Milestone 23: nested arrays

[Owned array elements](./22-owned-array-elements.md) · [Borrowed iteration](./21-for-of.md) · [Collections workstream](../guides/language/primitives-and-collections.md) · [Roadmap](../roadmap.md)

**Status:** Complete. The full 283-group regression suite passes, including 12 nested-array groups. Formatting, Clippy, and local documentation links are verified.

## Exact nested types and ownership

Arrays can contain other arrays with the same exact element type. `int[][]`, `boolean[][]`, `string[][][]`, and `User[][]` lower to nested native vectors. Array rank and the nominal leaf type are part of type identity. No flattening, implicit conversion, reference counting, or cloning is added.

```typescript
function total(rows: int[][]): int {
  let sum = 0;
  for (const row of rows) {
    for (const value of row) {
      sum = sum + value;
    }
  }
  return sum;
}
function main() {
  const rows = [[1, 2], [3]];
  rows[1].push(4);
  rows[0][0] = 7;
  console.log(total(rows));
}
```

This prints `16`. The outer array owns each row. Literal construction, push, and row replacement move owned rows into their destination. Moving the outer array transfers all nested storage. Explicit cloning recursively creates independent vectors and owned leaves.

Nested literals infer their rank and leaf type from the first element. Empty literals need type context at the point they are analyzed. `[[1], []]` works; `[[], [1]]` still needs an annotation because inference does not look ahead to a later element. Function parameters, returns, assignment, callbacks, and local annotations supply nested context. Record literals still need a named expected record type.

## Borrowed access, updates, and iteration

`rows[0]` borrows the owned row. Declaration moves, owned returns, and consuming arguments cannot extract it; use `clone(rows[0])` for independent ownership. `rows[0][1]` copies an integer leaf. Owned string/record leaves borrow under the same rules as the preceding gate.

Nested array paths support read-only length, push, element replacement, and record-field access or replacement. Direct calls and callable arguments infer or require read/mutate/consume access under existing contracts. A mutable row or record argument protects the entire outer array during the call; shared arguments may coexist. Different constant indices do not prove disjoint mutable access.

For-of borrows owned rows, and another for-of can traverse each row's copied scalars or borrowed owned elements. Shared row bindings cannot be moved, mutated, rebound, or retained by an escaping environment. Their lexical read captures remain valid. Clone a row before changing it or capturing it in an owned retained callback. Iterating `rows[0]` holds a source loan on the outer array for the entire loop; iterating `clone(rows[0])` permits independent mutation of the original.

Existing break/continue, ownership joins, loop restoration, whole-array capture loans, returned owned environments, callback specialization, and single-use contracts remain enforced.

## Operand evaluation and parent stability

An index expression evaluates once before selecting its element. A nested path selects and bounds-checks each parent before evaluating the next index. After a parent has been selected, later indices and operands may read the outer array, but cannot mutate, replace, or transfer its storage. Conflicting access receives `conflicting-access` with the parent selection location.

For example, `rows[0][rows[1][0]]` can read an index from another row. `rows[0][grow(rows)]` fails when `grow` mutates the outer array. A first index such as `rows[grow(rows)]` retains the existing flat-array behavior: its mutation runs before any row is selected.

A nested indexed update selects its parents and evaluates the final index, then evaluates the replacement, then checks and updates the final slot. Thus an invalid parent fails before the replacement runs; an invalid final index fails after it runs. Later operands can read the source, so `rows[0][0] = rows[1][0]` is supported. They cannot invalidate the selected parent. Flat row replacement such as `rows[0] = grow(rows)` keeps the existing rule that the replacement can mutate the receiver before its final indexed access.

Indexed push evaluates and checks its row selection before evaluating its owned argument. Integer rows allow `rows[0].push(rows[1].length)`; owned-leaf rows allow `rows[0].push(clone(rows[0][0]))`. An argument that mutates or transfers the outer array conflicts with the selected row loan. Push on a named array keeps its existing argument-before-mutation contract.

Generated code stores index results and takes final exclusive access after operand evaluation. Mutable paths recheck their already validated parents when obtaining native mutable references. These checks have no source side effects, and the enforced parent loan keeps the selected storage stable. No source index or replacement expression is evaluated twice.

## Bounds, depth, and representation

Every failed index reports the source byte range of the selection that fails, including intermediate parents. Negative and out-of-range checks remain active in optimized native builds with backend overflow checks disabled. Allocation and deterministic destruction follow native vector behavior at each level.

Array types support at most 128 dimensions. Excessive annotated or inferred rank receives a source diagnostic. Expression and combined analysis nesting retain their existing 128-level guards. Arrays use compact copyable leaf-and-depth type metadata; nested types need no heap-allocated compiler type chain. The emitter dispatches recursive array expressions through small frames. Generated Rust sets a bounded trait recursion limit sufficient for the supported source rank.

## Evidence and remaining scope

The [nested-array suite](../../crates/tsrust-compiler/tests/nested_arrays.rs) verifies rank/nominal matching, empty contexts, row transfers and cloning, indexed reads and updates, mutable child arguments, shared iteration and captures, callbacks and owned results, source evaluation order, parent stability, ownership restoration, runtime bounds and failure timing under two optimization profiles, backend-independent analysis, 100-level native reads and writes, and native cloning at the 128-dimension type boundary.

The [matrix fixture](../../crates/tsrust-compiler/tests/fixtures/nested-arrays/matrix.tsr) prints `16`, `7`, and `9`, demonstrating nested mutation, shared iteration, and clone independence. Earlier scalar and owned-element suites retain their original coverage while now accepting nested array construction.

[Milestone 24](./24-array-record-fields.md) subsequently adds arrays in record fields and mixed field/index storage paths. Callable elements, borrowed local declarations or results, slices, extraction/removal, mutable or consuming iteration, disjoint-index proofs, iterator pipelines, shared ownership, async, modules, and the Rust language server remain separately scoped work. At this gate, indexing and methods require named or indexed storage; arbitrary temporary receivers require a separate lifetime contract.
