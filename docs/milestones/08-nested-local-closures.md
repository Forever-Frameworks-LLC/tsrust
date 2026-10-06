# Milestone 08: nested local closures

[Owned closure arguments](./07-owned-closure-arguments.md) · [Roadmap](../roadmap.md)

**Status:** All five exit gates are complete on September 30, 2026. Implementation, diagnostics, acceptance evidence, and documentation cover the bounded provisional rules below.

**Subsequent extensions:** [Milestone 09](./09-callable-captures.md), [milestone 10](./10-callable-results.md), and [milestone 11](./11-retained-callbacks.md) complete callable captures, callable-valued arrow results, and owned retained callbacks. The boundaries below record this gate's original scope.

## Bounded outcome

Allow arrow bodies to create and invoke local closures, pass them to synchronous reusable/single-use callback parameters, and create local owned environments through ordinary factory calls. Nested closures can capture enclosing scalar values, strings, records, and parameters. Multiple lexical levels are supported within the existing analysis/parser depth limits.

```ts
const outer = (user: User): void => {
    const rename = (): void => { user.name = "Sam"; };
    rename();
};
```

A local closure binding belongs to the body that declares it. Invoking or forwarding it is permitted. Invoking or forwarding a closure/callback binding inherited from an enclosing body would capture a callable value and remains rejected. Closure aliases and callable-valued closure results remain unsupported. In particular, this gate does not return a borrowed nested environment or add retained callbacks.

## Ownership rules

- Inner capture creation applies effects immediately to the enclosing body, even if the inner closure is never invoked.
- Those effects propagate through all enclosing captures and parameter contracts. Reading/mutating an ancestor capture requires corresponding outer access; consuming it makes each affected outer environment single-use.
- Consuming an enclosing invocation's fresh argument does not consume the outer environment. Repeated outer calls may supply fresh arguments.
- Local inner capture loans end at their declaration block's exit. A narrower branch block permits later enclosing access. Inline callback loans end at call completion. Calling or consuming a named inner closure does not shorten its block loan.
- Each invocation constructs fresh local environments. Captures of owned returned environments are reborrowed only for that invocation; they cannot escape it.
- Calls to ordinary factories inside closure bodies apply their ordinary inferred access. A factory can consume a capture, consume a fresh argument, or borrow for explicit cloning. Its result can be invoked or passed locally.
- Existing left-to-right argument evaluation, whole-binding loans, branch joins, and single-use transfer checks apply at every level.

Concrete environment emission and static dispatch remain in use. Closure metadata gains a one-based lexical `depth`; IDs remain deterministic within one analysis but should not be persisted across edits.

## Exit gates

1. Distinguish local callable bindings from inherited bindings while allowing nested creation and local factory results.
2. Propagate read/mutate/consume capture and argument requirements through nested bodies.
3. Preserve lexical/temporary loans, transfers, branch joins, callback non-escape, and bounded recursion.
4. Compile generated Rust and compare exact native output for nested local, returned-outer, callback, factory, and multiple-level examples.
5. Verify metadata and backend-independent diagnostics; pass all existing tests, formatting, Clippy, and documentation checks.

Nested callable returns, captures of callable values, retained callbacks, and async are separate gates. None blocks this local nesting gate.

## Completion evidence

The [nested closure suite](../../crates/tsrust-compiler/tests/nested_closures.rs) passes 13 test groups. Accepted cases compile generated Rust with the pinned backend and compare exact native output for nested mutation, typed inner arguments, multiple ancestor captures, returned outer environments, fresh argument consumption, local reusable/single-use callbacks, inline loans, and ordinary factory results. Repeated outer calls construct fresh locals while retaining the intended owned capture state.

Rejected cases verify matching library/CLI diagnostics for capture conflicts, transitive transfers, repeated consuming invocations, invalid branch continuations, callable capture, aliases, invalid results, and out-of-scope local names. Tests verify both argument orders when an outer loan conflicts with nested capture creation. Related spans identify the capture or prior conflicting access.

Twelve lexical levels compile and execute; metadata reports depths 1–12, while excessive nesting returns a bounded source diagnostic. Empty-`PATH` checking and emission succeed; invalid native builds fail at source analysis before backend lookup. Existing unsupported-nesting guards now cover still-unsupported callable aliases rather than rejecting the newly supported constructs.

At completion of this gate, the full suite passed 127 test groups. Formatting and Clippy pass. The existing concrete environment emitter handles nested bodies without inserted boxing, dynamic dispatch, reference counting, or ownership-repair cloning. These checks provide bounded acceptance evidence, not general soundness or performance guarantees.

Run from the repository root:

```sh
cargo test --offline
cargo fmt --all --check
cargo clippy --offline --all-targets -- -D warnings
cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/callbacks/nested_local.tsr
```

The [nested local example](../../crates/tsrust-compiler/tests/fixtures/callbacks/nested_local.tsr) prints `Sam` three times. An inner closure mutates its parent's record parameter, a branch block ends that loan, and the enclosing body and caller can then read the record. No dependency remains open in this gate.
