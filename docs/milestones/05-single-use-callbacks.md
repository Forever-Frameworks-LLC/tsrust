# Milestone 05: synchronous single-use callbacks

[Reusable callbacks](./04-synchronous-callbacks.md) · [Roadmap](../roadmap.md)

**Status:** All five exit gates are complete on September 30, 2026. The compiler, acceptance checks, diagnostics, and documentation implement the bounded provisional rules below.

**Subsequent extension:** [Milestone 06](./06-closure-arguments.md) adds typed integer/boolean closure arguments. The zero-argument scope below records this milestone’s original gate.

## Bounded outcome

Add a contextual `once` modifier to zero-argument callback parameter annotations:

```ts
function take(action: once () => string): string {
    return action();
}
```

Passing an argument transfers the closure environment to the callee, even if the callee never invokes it. The callee may invoke or forward the callback at most once on each execution path. “Once” means at most once, not guaranteed execution. The callback remains synchronous and non-escaping.

Concrete read, mutable, and consuming closures satisfy this annotation when their result types match. Named closures, inline arrows, and factory results are accepted. Invocation may return an owned capture. A reusable callback parameter cannot be transferred to a single-use parameter because it only borrows the caller's environment. A single-use parameter cannot satisfy a reusable annotation. Forwarding between matching single-use parameters transfers ownership.

`once` is contextual before `()` in a parameter annotation; existing values, functions, and record types named `once` remain valid. This gate adds no source generics, callback arguments of their own, aliases, callback capture, callback returns, retained callbacks, or async.

## Ownership and lowering

- Single-use parameters expose a `consume` contract, including unused parameters.
- Invoking or forwarding a single-use parameter transfers its binding. Subsequent uses report the previous transfer location.
- Branches use the existing ownership join: transfers on returning paths do not invalidate continuing paths; transfers on any continuing path do.
- Passing a concrete closure consumes its binding regardless of its own read/mutate/consume invocation contract. A later caller-side use is invalid.
- Capture creation still applies immediately. Moving an owned capture makes its original unavailable; borrowed captures retain their existing lifetime rules.
- Inline capture loans last through subsequent argument evaluation and the call. Named local capture loans keep block lifetime, even after the environment is consumed.
- The generated Rust uses a generic callback trait with an owning receiver. Concrete environments require no inserted boxing, reference counting, dynamic dispatch, or repair clones.

## Acceptance and exit gates

1. Parse the annotation and preserve existing identifier behavior.
2. Enforce ownership transfer, result types, callback compatibility, and synchronous non-escape.
3. Verify direct invocation, forwarding, unused parameters, caller reuse errors, and branch joins.
4. Compile generated Rust and compare native output for read, mutable, consuming, returned, and inline callbacks.
5. Pass backend-independent diagnostic checks, the full regression suite, formatting, Clippy, and documentation checks.

No future language feature is required to complete these gates.

## Completion evidence

The [single-use callback suite](../../crates/tsrust-compiler/tests/single_use_callbacks.rs) passes 11 test groups. Accepted cases compile emitted Rust with the pinned backend and compare exact native output for read, mutable, consuming, inline, named, and returned environments. Forwarding consumes the parameter, unused callbacks remain uninvoked, and both branch outcomes verify the ownership join rules.

Rejected cases check matching library/CLI diagnostics for repeated invocation or forwarding, caller-side reuse, incompatible callback contracts, aliases, captures, returns, result mismatches, and conflicting arguments. Transfer diagnostics retain both the later-use and previous-transfer spans. Contextual `once` preserves existing record, function, and value identifiers. Empty-`PATH` checks and emission work without the backend; invalid builds reject source before backend lookup.

At completion of milestone 05, the full suite passed 88 test groups. Formatting and Clippy pass. `contracts()` and text/JSON checks expose `consume` for single-use parameters. Generated Rust uses concrete environments and generic `OnceCallback` bounds with an owning receiver. Acceptance checks confirm no inserted boxing, dynamic dispatch, reference counting, or repair clones; this does not constitute a general soundness proof or an allocation benchmark.

Run from the repository root:

```sh
cargo test --offline
cargo fmt --all --check
cargo clippy --offline --all-targets -- -D warnings
cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/callbacks/single_use.tsr
```

The [single-use example](../../crates/tsrust-compiler/tests/fixtures/callbacks/single_use.tsr) prints `Pat`. A callback consumes a captured record and returns it through a synchronous forwarding function. Retained callbacks, nested closure creation, and async remain separately scoped; no open dependency remains in this gate.
