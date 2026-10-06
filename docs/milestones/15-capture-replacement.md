# Milestone 15: replacing captured bindings

[Mutable local bindings](./14-mutable-bindings.md) · [Closure captures](./09-callable-captures.md) · [Roadmap](../roadmap.md)

**Status:** Complete. All 197 regression groups pass, including 10 capture replacement groups. Formatting, Clippy, and local documentation links are verified. This gate permits a closure to replace inherited `let` bindings while preserving explicit capture storage and invocation contracts.

**Subsequent extension:** [Milestone 16](./16-mutable-callables.md) adds mutable callable bindings with exact concrete environment identity. The boundaries below record this gate's original scope.

## Local captures

An assignment to an inherited `let` binding counts as mutable capture access, including assignments whose right-hand side never reads the old value. Type identity stays exact. Captured `const` bindings and parameters remain non-rebindable.

```typescript
function main() {
  let count = 0;
  if (true) {
    const increment = (): int => {
      count = count + 1;
      return count;
    };
    console.log(increment());
    console.log(increment());
  }
  console.log(count);
}
```

This prints `1`, `2`, and `2`. A local closure that writes a scalar now holds exclusive access to the original storage, just as a mutable record capture does. Its contract reports `storage: mutate` and `access: mutate`. Calls update the original binding. The lexical loan prevents conflicting outer reads, writes, and captures until the enclosing block ends. Inline callback loans end with the call under the existing rules.

Read-only scalar captures continue to copy snapshots. An existing copied snapshot does not observe later assignments to the original binding. Introducing a writer does not silently convert other captures into shared cells.

Strings and records can be replaced through a mutable capture when the right-hand side does not consume the old captured value. Examples include `text = text.trim()` and `user = { name: "Sam" }`. Normal Rust assignment drops the replaced owned value. Existing inner capture loans still prevent conflicting replacement.

## Escaping environments

Escaping closures own string/record captures and copy scalar captures. A write to a copied scalar modifies that private environment field, and repeated calls preserve its state. Updating the original scalar afterward does not change the escaped environment. Separately created environments remain independent.

```typescript
function makeCounter(): (step: int) => int {
  let count = 0;
  return (step: int): int => {
    count = count + step;
    return count;
  };
}
```

The contract reports copied storage with mutable access and mutable invocation. No shared ownership or implicit allocation is added. Owned retained callbacks follow the same rules. Nested local closures can borrow the enclosing environment's writable field, and their effects propagate to ancestor capture contracts.

## Consuming replacement and control flow

A right-hand side that consumes the old capture, such as `text = take(text)`, keeps the closure's invocation contract consuming, even if the binding is restored before return. Its environment owns the capture and may be invoked only once. This gate does not insert `mem::replace`, a sentinel value, or an ownership-repair clone to make consuming captures reusable.

Whole-binding replacement updates availability after the right-hand side completes. Branch joins and loop back-edge checks remain unchanged. A loop inside a closure may repeatedly mutate or replace borrowed storage. An outer loop may repeatedly create a fresh local writer; the capture loan ends on each iteration exit. It cannot repeatedly invoke an inherited consuming closure.

## Rust lowering and evidence

The checked representation records whether each capture is accessed through a reference. Scalar loads copy through that reference; whole-binding stores dereference it. Local writable scalar fields lower to mutable references with explicit environment lifetimes. Escaping writable scalar fields are ordinary owned integer/boolean fields borrowed from the mutable invocation receiver. A consuming invocation can instead take its fields by value.

The [capture replacement suite](../../crates/tsrust-compiler/tests/capture_replacement.rs) contains 10 groups. It verifies exact native output and source diagnostics for scalar/string/record replacement, write-only captures, local/owned/retained environments, nested propagation, consuming right-hand sides, loan conflicts, loop behavior, and backend independence. The [counter fixture](../../crates/tsrust-compiler/tests/fixtures/capture-replacement/counter.tsr) prints `2`, `5`, and `1` from independent returned environments.

Exit gates include all earlier regressions, formatting, Clippy, current documentation, and local documentation link validation. Mutable callable bindings, mixed callable environment types, shared capture cells, last-use loan shortening, and reusable consuming replacement remain separate scopes.
