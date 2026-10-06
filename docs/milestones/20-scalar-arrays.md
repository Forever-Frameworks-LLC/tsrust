# Milestone 20: owned scalar arrays

[Scalar operators](./12-scalar-operators.md) · [While loops](./13-while-loops.md) · [Collections workstream](../primitives-and-collections.md) · [Roadmap](../roadmap.md)

**Status:** Complete. [Milestone 22](./22-owned-array-elements.md) subsequently extends arrays to owned strings and records; the scope below records this gate's original boundary. [Milestone 21](./21-for-of.md) subsequently adds borrowed for-of iteration; the scope below records the milestone 20 boundary. The full 249-group regression suite passes, including 12 scalar array groups. Formatting, Clippy, and local documentation links are verified.

## Types and operations

The first collection gate supports owned `int[]` and `boolean[]` values, lowered to Rust `Vec<i64>` and `Vec<bool>`. This is a realized, growable collection with normal vector allocation and deterministic cleanup. Elements copy; the array itself transfers ownership. No garbage collector, reference counting, implicit cloning, or lazy iterator is introduced.

```typescript
function sum(values: int[]): int {
  let total = 0;
  let index = 0;
  while (index < values.length) {
    total = total + values[index];
    index = index + 1;
  }
  return total;
}
function main() {
  const values = [1, 2, 3];
  values.push(4);
  values[0] = 7;
  console.log(sum(values));
}
```

This prints `16`. A nonempty literal infers its element type from the first element and requires every subsequent element to match. An empty literal needs an annotation, call argument, assignment, or return context specifying `int[]` or `boolean[]`. Trailing commas are accepted; holes are rejected.

| Operation | Contract |
| --- | --- |
| `[1, 2]`, `[true, false]` | Construct a realized owned array |
| `const values: int[] = []` | Construct an empty array with a known element type |
| `values.length` | Read-only `int` length |
| `values[index]` | Bounds-checked copied element; index must be `int` |
| `values[index] = value` | Replace an existing element of the exact type |
| `values.push(value)` | Append one exact-type element; returns `void` |
| `clone(values)` | Explicit independent copy of the array |

Indexing, length, and push require a named array binding. Index writes never extend the array. `const` prevents rebinding but permits element updates and push, consistent with existing record mutation. `let` additionally permits exact-type replacement and ownership restoration.

## Ownership, captures, and callbacks

Named assignment, declaration moves, and owned returns transfer the array. Use after transfer reports the existing `transferred` diagnostic. Explicit cloning leaves the source available and produces independent storage. Ordinary parameters infer read access for indexing/length, mutable access for element writes/push, and consuming access for transfers.

Existing capture rules apply to whole arrays. Local readers borrow, local writers borrow exclusively, and returned environments own their captured arrays. Lexical loans prevent conflicting reads, mutation, replacement, or moves. Returning a captured array consumes its environment; repeatedly mutating an owned captured array remains supported.

Arrow bodies infer array argument access. Public callback and callable-result argument annotations require `read`, `mutate`, or `consume`, for example `(values: mutate int[]) => int`. Array results work in functions, arrows, callbacks, and direct owned-callback returns. Existing branch joins, loop restoration requirements, and single-use invocation restrictions remain enforced.

## Evaluation order and failures

Literal elements evaluate once from left to right. An indexed read evaluates its index before accessing the element. An indexed update evaluates its index, then the replacement value, then performs the checked mutation. Push evaluates its element before mutating the array. These rules permit `values.push(values.length)` and `values[0] = values[1]` without overlapping native borrows. A nested operation may mutate the same array during operand evaluation; a transfer that makes the receiver unavailable is rejected before emission.

Negative or out-of-range indices terminate execution with the original index expression's source byte range. Index conversion and bounds checks remain active in optimized builds with backend overflow checks disabled. Bounds checking for updates occurs after both operands are evaluated. Length conversion is checked against the signed `int` range. Allocation failure follows Rust's native allocation behavior; recoverable allocation and bounds errors remain outside this gate.

## Evidence and remaining scope

The [array suite](../../crates/tsrust-compiler/tests/arrays.rs) covers inference, empty contexts, reads/writes, iteration, push, moves, clone independence, inferred parameter access, capture loans, returned environments, callback contracts, evaluation order, receiver transfers, branch/loop joins, diagnostics, and backend-independent analysis. Bounds failures are verified in unoptimized and optimized native builds. Deep literal and index nesting produces source diagnostics instead of overflowing the parser stack.

The [sum fixture](../../crates/tsrust-compiler/tests/fixtures/arrays/sum.tsr) prints `10`, `4`, `true`, and `false`. Accepted programs compile generated Rust with the pinned backend and compare exact native output.

Owned elements such as strings and records, nested arrays, arrays in record fields, slices, sparse arrays, array equality/logging, pop/removal, destructuring, for-of iteration, iterator pipelines, shared ownership, async, modules, and the language server remain separately scoped. This gate does not adopt JavaScript array coercions, holes, or push's numeric return value.
