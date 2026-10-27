# Milestone 24: arrays in record fields

[Nested arrays](./23-nested-arrays.md) · [Types workstream](../guides/language/type-system.md) · [Ownership workstream](../guides/language/ownership-and-memory.md) · [Roadmap](../roadmap.md)

**Status:** Complete. All 295 regression groups pass, including 12 array-field groups. Formatting and Clippy pass.

## Owned fields and exact types

Named records can own arrays of any supported leaf type and rank. Scalar fields retain their existing semantics. Array fields can contain strings, named records, or other arrays; their exact rank and nominal leaf identity remain part of type identity. All record names are collected before resolving field annotations, so forward references work.

```typescript
type Node = { values: int[]; children: Node[]; };
function main() {
  const root: Node = { values: [1], children: [] };
  root.children.push({ values: [2], children: [] });
  root.children[0].values.push(7);
  const saved = clone(root);
  root.children[0].values[0] = 9;
  console.log(saved.children[0].values[0]);
}
```

This prints `2`. Self-recursive and mutually recursive array fields have finite native layouts through vector indirection. Values form owned trees, with no shared aliases or reference cycles. Clone and destruction follow native recursive behavior; this gate does not promise constant-stack traversal of arbitrarily deep runtime trees.

Construction and field replacement supply expected types to empty array and record literals. Owned arrays move into fields; the source binding becomes unavailable. Whole-record moves transfer all field storage. Explicit cloning recursively creates independent records, vectors, and strings. No hidden cloning or reference counting is inserted.

Reading an owned field borrows it. Extracting it through a declaration, consuming argument, or owned return still requires explicit cloning. Bare by-value record fields and callable fields remain outside this gate.

## Storage paths and ownership

Mixed field/index paths rooted in named storage support reading, array length, push, element and row replacement, record-field replacement, iteration, and read/mutable arguments. Scalar leaves copy; owned leaves borrow. The same paths work through parameters and captured owners, including returned owned closure environments and specialized callback arguments.

Source loans protect the whole root owner. Shared field arguments can coexist. An exclusive argument to one field conflicts with access to another field of the same owner; separate field names do not prove disjoint access. For-of over a field array holds a shared loan on the root record throughout the loop, blocking mutation, replacement, and transfer of that owner. Iteration over an explicit clone permits independent mutation of the original.

Borrowed iteration bindings retain the previous restrictions on mutation, moves, rebinding, and escaped retention. Local shared captures are lexical. Clone owned elements before retaining them in returned environments. Existing single-use callback contracts, ownership reinitialization, and branch/loop joins remain enforced.

## Evaluation order and failure timing

Every source index and replacement expression evaluates once in source order. A selected parent field or indexed element remains stable while later operands evaluate. Later operands may read its owner, but cannot mutate, replace, or transfer it. For example, `box.items[grow(box)]` and `box.items.push(grow(box))` receive `conflicting-access` when `grow` mutates `box`.

Direct replacement such as `box.items = replacement(box)` evaluates its right-hand side before selecting the final field and can therefore mutate the owner while computing the replacement. Indexed record-field replacement such as `boxes[index()].items = replacement(boxes)` similarly saves the index and evaluates the replacement before checking the final record slot. Earlier selected parents remain protected.

An array element replacement selects its parents, evaluates the final index and right-hand side, and then checks the final slot. An invalid parent fails before later operands; an invalid final slot fails after the right-hand side. Push checks the selected field/index receiver before evaluating its owned argument. Generated exclusive paths reuse saved indices; their native parent checks have no repeated source effects.

Negative and out-of-range failures retain the byte range of the selection that fails under both unoptimized and optimized native builds with backend overflow checks disabled.

## Evidence and boundaries

The [array-field suite](../../crates/tsrust-compiler/tests/array_fields.rs) covers field context and nominal identity, moves and clone independence, root-owner parameter contracts, iteration and argument loans, capture lifetimes, retained callbacks, owned environments, source evaluation order, bounds failure timing, forward and mutually recursive types, recursive owned trees, deep mixed paths, existing depth guards, and backend-independent CLI analysis.

The [catalog fixture](../../crates/tsrust-compiler/tests/fixtures/array-fields/catalog.tsr) prints `Pat`, `admin`, `Sam`, `new`, `7`, and `admin`. It combines record arrays, string array fields, nested scalar arrays, iteration, mutation, and explicit clone independence.

[Milestone 25](./25-nested-record-fields.md) subsequently adds direct named-record fields. Callable fields/elements, owned field extraction, borrowed local declarations/results, temporary storage receivers, slices/removal, mutable or consuming iteration, disjoint-access proofs, shared ownership, async, modules, and the Rust language server remain separately scoped work. Array rank and expression/analysis nesting retain their existing 128-level guards.

Milestone 31 subsequently adds [direct named record field moves and restoration](./31-record-field-moves.md), tracking partial availability through joins and loops. Indexed and iteration element fields remain borrowed; conservative root loans and whole environment capture storage remain in force.

## Foundation follow-up

The subsequent [foundation audit](../plans/foundation-audit.md) replaces automatic recursive clone/destruction with iterative work stacks within each recursive record component. A 100,000-node chain is verified in debug and optimized builds. Records remain movable by field because cleanup belongs to recursive array wrappers. User-written recursive traversal functions still have ordinary native call-stack limits. The original gate evidence above describes its implementation at completion.
