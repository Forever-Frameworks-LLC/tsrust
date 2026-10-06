# Milestone 04: synchronous reusable callbacks

[Closure milestone](./03-closures.md) · [Roadmap](../roadmap.md)

**Status:** All five exit gates are complete on September 30, 2026. Compiler behavior, acceptance evidence, diagnostics, and documentation are implemented together. The bounded rules remain provisional.

**Subsequent extension:** [Milestone 06](./06-closure-arguments.md) adds typed integer/boolean closure arguments. The zero-argument scope below records this milestone’s original gate.

## Bounded outcome

Support annotated zero-argument callback parameters in ordinary functions:

```ts
function twice(action: () => void) {
    action();
    action();
}
```

The parameter promises a synchronous, non-escaping, reusable callback. Arguments may be named local/returned closures, inline arrows, or factory results. Callback results may be any existing non-callable return type, including owned strings and records. Callback parameters may be forwarded through the existing acyclic call graph.

Every invocation requires exclusive temporary access to the callback. Functions infer `mutate` for an invoked/forwarded mutable callback parameter and `read` for an unused parameter. Read-only closure bodies also satisfy this reusable contract, but their callback argument is still borrowed exclusively when the callee invokes it. A single-use closure is rejected even if a particular function calls its parameter only once; the annotation promises reusability, not an invocation count.

This gate does not add callback arguments of their own, first-class named functions, consuming callback contracts, callback aliases, stored/returned callback parameters, callback capture, nested closure creation, retained callbacks, or async.

## Ownership and lifetime rules

- Passing a reusable closure temporarily borrows its environment; it does not transfer ownership. The caller can invoke or pass it again after the call.
- Named local closure capture loans retain the milestone 03 block lifetime. Passing a named closure does not shorten those loans.
- Inline closure captures live through evaluation of all subsequent arguments and the synchronous call. Their loans end when that call expression finishes. This permits owner access afterward.
- Arguments evaluate left to right. Active argument loans and existing capture loans constrain later argument evaluation and capture creation. Repeated mutable callback arguments are rejected.
- Callback parameters can only be invoked or forwarded to matching callback parameters. They cannot be returned, aliased, stored, or captured. These checks establish the no-escape promise within the supported language.
- Result types must match exactly. Callable return annotations do not erase callback reuse or lifetime restrictions.

The emitter uses a generated callback trait and generic function parameters. Concrete closure environments implement that trait only when reusable. Calls use static dispatch without inserted boxing, reference counting, or ownership-repair clones. Generic Rust code is an implementation detail, not new source generic syntax.

## Acceptance and exit gates

1. Parse callback annotations; reject unsupported callback shapes explicitly.
2. Check result types, reusable eligibility, direct invocation, transitive forwarding, and enclosing parameter access contracts.
3. Enforce non-escape, argument conflicts, temporary capture lifetimes, and existing block-scoped loans.
4. Generate Rust and compare native output for read, mutable, returned, inline, forwarded, and value-returning callbacks, including both branch outcomes.
5. Verify source diagnostics without a backend; pass the full existing suite, formatting, Clippy, and documentation checks.

No future language feature is a prerequisite for these gates. Retained callbacks remain a separate experiment. [Milestone 05](./05-single-use-callbacks.md) subsequently adds an explicit single-use parameter annotation; reusable annotations continue rejecting consuming invocation.

## Completion evidence

The [callback suite](../../crates/tsrust-compiler/tests/callbacks.rs) passes 13 test groups. Accepted programs compare in-process and CLI output, compile the generated Rust with the pinned backend, and match exact native output. They cover named read/mutable closures, inline arrows, returned environments, transitive forwarding, unused parameters, both branch outcomes, and all supported result types.

Rejected programs verify matching library/CLI diagnostics for consuming callbacks, escaped parameters, result mismatches, unsupported signatures, duplicate exclusive arguments, and conflicting capture access in both argument orders. Nested-call checks prove that temporary loan cleanup preserves outer-call and named-closure loans. Related spans identify capture creation or the earlier conflicting argument. With an empty `PATH`, valid checks/emission succeed and invalid source fails before backend lookup.

At completion of milestone 04, the complete suite passed 77 test groups, including all earlier source cases and independent Rust witnesses. Formatting and Clippy pass. Callback parameter access appears in the existing `contracts()` API and text/JSON checks; literal capture metadata remains available through `closures()`. The generated output uses concrete environments and generic trait bounds without inserted boxing, dynamic dispatch, reference counting, or repair cloning. These are bounded acceptance checks, not a general soundness proof or allocation benchmark.

Run from the repository root:

```sh
cargo test --offline
cargo fmt --all --check
cargo clippy --offline --all-targets -- -D warnings
cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/callbacks/inline_mutation.tsr
```

The [inline mutation example](../../crates/tsrust-compiler/tests/fixtures/callbacks/inline_mutation.tsr) prints `true`, `false`, `false`, `true`, `true`, `true`, and `true`. A callback toggles the record twice, the caller reads it after the inline loan ends, and another callback mutates it again. No open dependency remains in this gate.
