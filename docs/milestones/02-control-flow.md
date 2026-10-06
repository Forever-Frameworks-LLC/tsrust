# Milestone 02: ownership across branches

[Roadmap](../roadmap.md) · [Ownership rules](./01-ownership-proof.md)

**Status:** Implemented and tested on September 30, 2026. These rules are provisional and extend the first ownership experiment.

## Outcome and boundaries

Support `if (condition) { ... }` with an optional `else { ... }`, including nested branches and early returns. Conditions require `boolean`; there is no truthiness conversion. Boolean literals, locals, parameters, fields, returns, and logging use the same scalar copy rules as integers. Boolean spelling is `boolean`, with literals `true` and `false`.

Both arms require braces. There is no semicolon after the conditional statement. `else if`, comparisons, logical operators, loops, reassignment, closures, and async remain outside this slice. Use a nested `if` inside an `else` block when necessary.

## Ownership rules

1. Evaluate the condition once before analyzing either arm's entry state. Ownership effects of calls in the condition apply to both arms.
2. Analyze both arms from that same availability state. Consuming a value in one arm does not prevent its independent use in the other arm.
3. At a join, a value is available only if it is available on every path that reaches the join. A missing `else` is an unchanged path.
4. An arm that always returns does not contribute availability to the continuation. Its accesses still contribute to the function's parameter contract.
5. Parameter contracts combine the requirements of both arms. If either arm consumes a parameter, every caller must treat that parameter as consuming, regardless of the supplied condition.
6. Locals declared in an arm end at that arm's closing brace. Sibling arms may reuse names. Shadowing an active outer binding remains unsupported.
7. Both arms are checked even for literal conditions. The analyzer does not use constant propagation or correlations between repeated conditions to relax ownership checks.
8. A function with a result must return on every path. Statements after an unconditional return or an `if` whose arms both return are rejected as unreachable.

The implementation retains one checked syntax tree. Branch analysis snapshots availability for visible bindings; it does not duplicate syntax trees or enumerate all execution paths. Inferred access and mutation requirements accumulate across arms. Recursive parsing and combined analysis remain bounded by the prototype nesting limit.

## Acceptance examples

Accept independent consumption in mutually exclusive arms:

```ts
function choose(user: User, first: boolean): User {
    if (first) { return user; } else { return user; }
}
```

Accept a move on an exiting path, followed by a read on the surviving path:

```ts
function display(user: User, stop: boolean) {
    if (stop) { const saved = user; return; }
    console.log(user.name);
}
```

The caller still sees `display.user: consume` because one path consumes it.

Reject a move on a path that reaches a later read:

```ts
function invalid(user: User, take: boolean) {
    if (take) { const saved = user; }
    console.log(user.name);
}
```

The diagnostic must identify the later read and a transfer on a path reaching it. Also reject a branch-local name outside its block, non-boolean conditions, missing return paths, and conflicting call arguments within branches.

## Verification and completion

Compile accepted source through generated Rust and compare native output for both true and false inputs. Compare inferred contracts, verify transfer/conflict spans on rejected source, and preserve the previous integer and ownership suites. Cover nested joins, absent `else`, returning arms, condition effects, branch scopes, and bounded nesting.

The [branch acceptance suite](../../crates/tsrust-compiler/tests/control_flow.rs) passes 11 test groups, including a 16-case join matrix. Accepted programs compile through the pinned Rust backend and produce exact expected native output. At completion of this slice, the full suite passed 48 test groups, preserving the earlier ownership and integer cases. Formatting and Clippy are checked separately.

Run from the repository root:

```sh
cargo test --offline --test control_flow
cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/control-flow/branch_reuse.tsr
```

The [branch example](../../crates/tsrust-compiler/tests/fixtures/control-flow/branch_reuse.tsr) prints `Pat`, `Pat`, `Sam`, and `Sam`. It demonstrates that the caller retains the record after either a read or mutation path.

This slice does not establish loop analysis, path-sensitive condition reasoning, or closure lifetime safety. [Milestone 03](./03-closures.md) now scopes captures and escaping lifetimes with separate source proposals and verified handwritten Rust witnesses. Its local and returned closure compiler packages are now complete.
