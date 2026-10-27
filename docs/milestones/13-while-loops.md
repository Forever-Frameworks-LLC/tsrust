# Milestone 13: while loops and iteration ownership

[Scalar operators](./12-scalar-operators.md) · [Ownership workstream](../guides/language/ownership-and-memory.md) · [Roadmap](../roadmap.md)

**Status:** Complete. All 176 regression groups pass, including 14 loop groups. Formatting, Clippy, and local documentation links are verified. This gate covers braced `while` loops, unlabelled `break` and `continue`, and ownership across repeated iterations.

**Subsequent extension:** [Milestone 14](./14-mutable-bindings.md) adds local `let` replacement and reinitialization before loop back edges. The rules below record the original immutable-binding scope.

## Syntax and execution

```typescript
type Counter = { value: int; };
function main() {
  const counter: Counter = { value: 0 };
  while (counter.value < 5) {
    counter.value = counter.value + 1;
    if (counter.value === 2) { continue; }
    if (counter.value === 4) { break; }
    console.log(counter.value);
  }
  console.log(counter.value);
}
```

This prints `1`, `3`, and `4`. The boolean condition runs before each body execution, including after `continue`. A false condition exits without running the body. `break` exits the nearest loop; `continue` starts its next condition evaluation. Both require semicolons and an enclosing loop in the same function or closure. Closures never inherit a surrounding loop's control target.

Bodies require braces and have lexical scope. Locals are recreated on each iteration; their moves and capture loans end with that iteration. Nested loops have independent targets. `return` leaves the current function or closure. Statements after every preceding path has exited the current block are rejected as unreachable.

A loop is conservatively allowed to execute zero times, even with literal `true`. A non-void function needs a return after the loop or another path that establishes complete return coverage. Literal conditions do not remove ownership checks from a body, and termination is not proven.

## Ownership across iterations

Bindings declared before a loop must retain their ownership on every path that reaches another iteration. Moving one in a condition or body and then reaching `continue` or the end of the body produces `loop-transfer`. The diagnostic identifies the transfer and the loop. This applies to strings, records, concrete environments, single-use callbacks, and captures transferred into an owned environment.

A transfer may occur on a path that immediately or eventually exits through `break` or `return`, provided no continuing path reaches another iteration with that transfer. Other branches may continue without transferring. Newly created iteration-local values can be consumed on repeating paths because each iteration recreates them.

After the loop, availability joins the state after condition evaluation with every `break` exit. A transfer on any such path makes the binding unavailable afterward. Function-return paths do not reach this join. The condition's effects apply even when the body executes zero times. Existing short-circuit rules conservatively include any possible condition transfer.

```typescript
function choose(text: string, flag: boolean): string {
  while (flag) { return text; }
  return text; // The loop's return path cannot reach this statement.
}
```

Reads and mutations can repeat. Inferred parameter and capture contracts include all body paths, including paths that return. A closure consuming a capture on an exiting path remains single-use; a loop cannot repeatedly invoke a single-use closure or callback. Reusable callbacks and retained mutable environments can run across iterations.

Existing loans remain effective. Body-local closure loans end on normal body completion, `break`, `continue`, or `return`; an outer closure's loan still prevents conflicting loop access. An inner `break` does not leave an outer loop: a transfer escaping the inner loop must still be valid on the outer repeating path.

## Analysis and lowering

A loop frame records the visible bindings and their availability before condition evaluation. The analyzer checks each reachable back edge against that entry and accumulates transfer states for `break` exits. Branch joins exclude paths that exit the current block. Returned closure origin discovery also traverses conditions and loop bodies.

The current subset has immutable local bindings and no ownership reinitialization. Availability therefore only loses ownership along a path, so one body analysis plus explicit checks of continuing edges is sufficient for this bounded gate. This is not a general fixed-point algorithm for mutable rebinding or arbitrary control-flow graphs. The compiler does not reparse or duplicate a body to approximate multiple iterations.

Emission uses ordinary Rust `while`, `break`, and `continue`. No ownership-repair clones, boxing, or runtime ownership tracking are inserted. Existing parser and combined-analysis nesting bounds remain active.

## Exit evidence

- Exact native output for zero/multiple iterations, condition effects, field updates, nested targets, loop-local values and loans, and callback/closure integration.
- Source diagnostics for repeated transfers, invalid control targets, unreachable statements, scope errors, condition types, labels, and incomplete returns.
- A 36-cell branch matrix distinguishes repeating paths, breaks, function returns, and their ownership joins; accepted functions also compile through Rust.
- Backend-independent checks, nesting guards, related source locations, and earlier milestone regressions.
- Formatting, Clippy, current documentation, and local documentation links.

The [loop suite](../../crates/tsrust-compiler/tests/loops.rs) has 14 groups. The [counter fixture](../../crates/tsrust-compiler/tests/fixtures/loops/count.tsr) prints `5` and `9` while demonstrating both `continue` and `break`.

## Remaining scope

`for`, `for...of`, `do...while`, labelled control flow, value-producing loops, mutable local rebinding/reinitialization, collections and iterators, and termination analysis remain separate gates. Existing unsupported features such as async, modules, mixed callable origins, and borrowed results are unchanged. None blocks this while-loop gate.
