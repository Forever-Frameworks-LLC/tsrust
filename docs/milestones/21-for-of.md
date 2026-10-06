# Milestone 21: borrowed scalar array iteration

[Scalar arrays](./20-scalar-arrays.md) · [While loops](./13-while-loops.md) · [Roadmap](../roadmap.md)

**Status:** Complete. [Milestone 22](./22-owned-array-elements.md) subsequently extends arrays to owned strings and records; the scope below records this gate's original boundary. The full 259-group regression suite passes, including 10 for-of groups. Formatting, Clippy, and local documentation links are verified.

## Iteration contract

`for (const value of values) { ... }` and `for (let value of values) { ... }` iterate `int[]` and `boolean[]` arrays in element order. The iterable expression evaluates once. Each iteration binds a fresh copied scalar; assigning a `let` iteration variable does not modify the array. A `const` iteration variable cannot be reassigned. The binding's scope ends with the loop, and the existing no-shadowing rule applies.

```typescript
function sum(values: int[]): int {
  let total = 0;
  for (const value of values) {
    total = total + value;
  }
  return total;
}
function main() {
  const values = [1, 2, 3];
  console.log(sum(values));
  console.log(values.length);
}
```

This prints `6` and `3`. Iterating a named array borrows it for the loop and leaves ownership with its binding. An ordinary parameter used only for iteration infers read access. Generated Rust uses `.iter().copied()` with no hidden clone, allocation, or index counter.

Array literals and array-producing calls are valid iterable expressions. Temporary array storage remains alive through iteration and is released afterward. `clone(values)` explicitly selects an independent array; the original may then be mutated during the loop. An empty literal without element-type context still fails; a typed empty array produces zero iterations.

## Active iteration loans

A named source array holds a shared iteration loan for the entire loop. Reading it, indexing it, taking its length, and nested read iteration are allowed. Mutating, pushing, replacing, or transferring it is rejected with `iteration-conflict` and the source location where iteration borrowed the array. This includes indirect calls and closures that request incompatible capture access.

The loan ends on loop exit, including break and return. Mutation after the loop is allowed. These conservative rules do not shorten the loan merely because a move would be followed by break. Existing closure capture loans remain independently enforced: an exclusive capture prevents iteration, while compatible shared captures can coexist.

## Control flow and captures

Break and continue target the nearest loop, including mixtures of while and for-of. They cannot cross a closure boundary. Statements after an unconditional exit remain rejected.

Existing ownership invariants apply to other values visible before the loop. A value moved on a repeating path must be restored before the back edge. Break paths contribute transferred state after the loop; function-return paths do not. Iterable evaluation effects occur before the zero-iteration baseline, so moving an array into an array-producing call consumes it even when that call returns an empty array.

For-of does not prove that a function returns, even for a visibly nonempty literal. A fallback return is required. Existing single-origin callable result rules also remain in force.

Closures may capture copied iteration values. Local writers of `let` iteration bindings borrow that iteration's scalar storage; owned escaping callbacks capture private scalar values under the existing rules. Each iteration creates fresh storage. Arrays captured by an owned returned closure can themselves be iterated repeatedly without consuming the closure.

## Evidence and remaining scope

The [for-of suite](../../crates/tsrust-compiler/tests/for_of.rs) checks element order, caller ownership, inferred access, temporary arrays, one-time iterable evaluation, explicit cloning, fresh bindings, lexical scope, nested break/continue, active loan conflicts, captures, ownership restoration, zero-iteration behavior, return coverage, unsupported syntax, depth limits, and backend-independent analysis. Accepted programs compile generated Rust and compare native output. A 100-level valid loop nest checks and emits successfully; a 200-level nest receives the existing depth diagnostic. Recursive block parsing uses small dedicated statement frames so the 128-level guard runs before stack exhaustion, including earlier branch tests.

The [iteration fixture](../../crates/tsrust-compiler/tests/fixtures/for-of/sum.tsr) skips `2`, stops before `5`, and prints `8`, the unchanged array length `5`, then `true` and `false`.

C-style for loops, for-in, labels, async iteration, mutation through iteration variables, owned element arrays, slices, iterator adapters, consuming iteration syntax, and borrowed iterator values remain separate scopes. This gate defines stable borrowed-array iteration rather than JavaScript's behavior when an array changes during traversal.
