# Milestone 25: nested record fields

[Array fields](./24-array-record-fields.md) · [Types workstream](../type-system.md) · [Ownership workstream](../ownership-and-memory.md) · [Roadmap](../roadmap.md)

**Status:** Complete. All 309 regression groups pass, including 14 nested-record groups. Formatting and Clippy pass.

## Named fields, ownership, and representation

Records can own other named records directly. Forward references resolve after all record names are collected. Field types match exact nominal identity; structurally identical records with different names are not interchangeable. Context from a field annotation permits nested record literals and empty array literals.

```typescript
type User = { profile: Profile; active: boolean; };
type Profile = { name: string; scores: int[]; };
function main() {
  const user: User = {
    profile: { name: "Pat", scores: [1] },
    active: true
  };
  const saved = clone(user);
  user.profile.scores.push(7);
  user.profile.name = "Sam";
  console.log(saved.profile.name);
  console.log(user.profile.scores[1]);
}
```

This prints `Pat` and `7`. A direct record field lowers to a concrete inline native record. No boxing, reference counting, or implicit cloning is introduced. Strings and vectors retain their ordinary native allocation behavior. Construction and field replacement move owned values into fields. Whole-owner moves transfer every nested field. Explicit cloning recursively creates independent owned storage.

Nested record field access borrows. Read and mutable arguments can refer to a nested record without cloning; argument loans protect the entire root owner. Owned field extraction through declarations, consuming arguments, or returns still requires explicit cloning. Partial moves and borrowed local declarations/results remain separate work.

## Layout validation

A direct cycle such as `type Node = { next: Node; };` has no finite inline layout and receives `recursive-layout` before any backend invocation. The primary location identifies the field closing the cycle; a related location identifies the repeated record type. Unused records are checked too.

Only direct record fields add edges to the inline layout graph. Array indirection breaks a layout cycle, so `A` owning `B` directly while `B` owns an `A[]` is supported. Values remain owned trees without shared aliases or reference cycles. Native recursive cloning/destruction of array-backed trees does not promise constant-stack runtime traversal.

Validation traverses the graph iteratively and caches each record's depth and storage bound. Shared dependencies are checked once; their layouts are not expanded into exponentially repeated compiler metadata. Direct inline nesting supports at most 128 record levels. A conservative inline-storage estimate supports at most 1 GiB: each field is rounded to the largest supported primitive alignment before summing cached child bounds. This is a prototype guard, not a stable ABI or an exact record-size promise. Excessive depth or storage receives `unsupported` at the contributing field annotation.

Expression, combined analysis, and array-rank guards retain their existing 128-level limits. Parser, analyzer, and emitter process recursive record literals through small frames, so supported deep inputs do not retain the larger general-expression frames.

## Paths, loans, and evaluation order

Mixed field/index paths support reads, scalar copies, record and array replacement, length, push, shared iteration, and read/mutable arguments. They work through local owners, parameters, lexical captures, returned owned environments, and callbacks. Existing nominal matching, availability joins, ownership restoration, and single-use contracts remain enforced.

Shared loans to different fields can coexist. Exclusive field access conflicts with other access to the same root owner; field names and constant indices do not prove disjoint storage. Iteration over nested array fields borrows the root owner for the entire loop. Borrowed record iteration bindings cannot mutate, transfer, rebind, or escape through retention. Clone an owned element before retaining it in an escaped environment.

Index expressions evaluate once in source order. Earlier parent selections stay stable while later operands evaluate. `box.child.n = mutate(box)` conflicts with the selected child loan. Read-only replacements such as `box.left.n = box.right.n` work. Direct final-field replacement `box.child = replacement(box)` preserves its previous right-hand-side-before-final-selection behavior.

A field immediately after an index retains replacement-before-final-bounds-check behavior: `boxes[1].child = make()` runs `make()` before checking the final record slot. Another field selection turns that record slot into an earlier parent: `boxes[1].child.n = mark(7)` fails on the invalid parent before `mark` runs. Indexed writes still check their final slot after the replacement; indexed push checks its receiver before its argument. Source byte ranges and failure order remain correct in optimized builds with backend overflow checks disabled.

## Evidence and remaining scope

The [nested-record suite](../../crates/tsrust-compiler/tests/nested_records.rs) covers contextual construction, nominal mismatches, missing/duplicate fields, nested replacement, moves and owner reinitialization, clone independence, borrowed arguments, sibling loans, iteration, captures, retained callbacks, direct cycles, array-indirected recursion, shared and empty layouts, layout limits, source operand order, optimized bounds failure timing, 100-level native literals/reads/writes/cloning, and the 128-level direct layout boundary. CLI checking and emission are verified with an empty executable search path.

The [profile fixture](../../crates/tsrust-compiler/tests/fixtures/nested-records/profile.tsr) prints `Pat`, `7`, `Sam`, and `9`, combining forward field types, string transformation, vector mutation, independent cloning, record replacement, and nested iteration.

Owned field extraction and partial moves, borrowed local declarations/results, temporary storage receivers, callable fields/elements, disjoint field loans, shared ownership, modules, async, generics, and the Rust language server remain separately scoped work.

Milestone 31 subsequently adds [direct named record field moves and restoration](./31-record-field-moves.md), tracking partial availability through joins and loops. Indexed and iteration element fields remain borrowed; conservative root loans and whole environment capture storage remain in force.

## Foundation follow-up

The subsequent [foundation audit](../foundation-audit.md) replaces automatic recursive clone/destruction with iterative work stacks within each recursive record component. A 100,000-node chain is verified in debug and optimized builds. Records remain movable by field because cleanup belongs to recursive array wrappers. User-written recursive traversal functions still have ordinary native call-stack limits. The original gate evidence above describes its implementation at completion.
