# Milestone 06: scalar closure arguments

[Single-use callbacks](./05-single-use-callbacks.md) · [Roadmap](../roadmap.md)

**Status:** All five exit gates are complete on September 30, 2026. The compiler and acceptance suite implement the bounded provisional rules below.

**Subsequent extension:** [Milestone 07](./07-owned-closure-arguments.md) adds owned argument access contracts. The scalar-only scope below records this milestone’s original gate.

## Bounded outcome

Allow zero or more explicitly typed `int` and `boolean` parameters on arrow functions, callback parameter annotations, and callable factory results:

```ts
function apply(action: (value: int, enabled: boolean) => int): int {
    return action(7, true);
}
function main() {
    const choose = (value: int, enabled: boolean): int => {
        if (enabled) { return value; }
        return 0;
    };
    console.log(apply(choose));
}
```

The same argument syntax applies to `once (value: int) => Result` and factory return annotations. Existing zero-argument programs retain their behavior. Callback parameter names describe the signature; matching depends on ordered types and arity, not names. All existing non-callable result types remain supported.

Arguments are copied. Closure parameters do not become captures or affect the environment's read/mutate/consume invocation contract. Parameter bindings cannot be rebound or shadow an existing visible binding, matching the prototype's existing declaration rule. Duplicate and reserved parameter names are rejected, including in callable annotations.

## Ownership and lowering

- The closure receiver is accessed first, followed by arguments from left to right. Argument evaluation applies ordinary ownership effects.
- Shared receiver access permits nested shared invocation. An exclusive receiver loan prevents overlapping calls or transfers while arguments are evaluated. A consuming receiver is unavailable during subsequent argument evaluation.
- Scalar argument values do not retain loans used to compute them. Existing capture loans and outer argument loans still constrain nested evaluation.
- Reusable and single-use callback contracts, synchronous non-escape, named capture block lifetimes, inline capture call lifetimes, and single-origin factory returns retain their existing rules.
- Generated concrete `call` methods take scalar arguments by value. Generic callback traits carry a typed argument tuple and use static dispatch without boxing or repair clones.

Strings, records, callable values, and `void` as closure argument types are outside this gate. Owned arguments need a public read/mutate/consume parameter contract before higher-order calls can preserve inferred access consistently. Callback arguments with their own callbacks, nested closures, retained callbacks, optional/default/rest arguments, and source generics remain separate work. No such feature blocks this scalar gate.

## Exit gates

1. Parse typed arrow parameters, callback annotations, and factory signatures while preserving grouping expressions and existing identifiers.
2. Check parameter declarations, ordered signature compatibility, argument types, arity, and result types.
3. Preserve receiver/argument evaluation order, ownership conflicts, capture inference, and both callback invocation contracts.
4. Compile generated Rust and compare native output for local, returned, inline, forwarded, reusable, and single-use closures with multiple arguments.
5. Verify backend-independent diagnostics, full regressions, formatting, Clippy, and documentation links.

## Completion evidence

The [closure argument suite](../../crates/tsrust-compiler/tests/closure_arguments.rs) passes 12 test groups. Accepted programs compare exact native output after compiling generated Rust with the pinned backend. Cases cover direct, inline, returned, forwarded, reusable, and single-use closures; multiple arguments; mutable/owned environments; both branch outcomes; and grouped expressions. Capture metadata confirms that closure parameters are not environment captures.

Rejected programs verify matching library/CLI diagnostics for argument count/type errors, mismatched callback and factory signatures, duplicate/reserved/shadowing parameters, unsupported argument types, escaped callbacks, repeated single-use invocation, and overlapping receiver access. Related spans identify conflicting receiver access or prior transfer. Argument-order checks observe mutation output and prove that computed scalar values do not retain input loans. Existing zero-argument suites remain passing.

At completion of milestone 06, the full suite passed 100 test groups. Formatting and Clippy pass. Empty-`PATH` tests verify backend-independent checking/emission and source rejection before native build lookup. Emitted callback traits use typed tuples with static dispatch; acceptance checks verify no inserted boxing, dynamic dispatch, reference counting, or repair clones. This bounded evidence is not a general soundness or performance claim.

Run from the repository root:

```sh
cargo test --offline
cargo fmt --all --check
cargo clippy --offline --all-targets -- -D warnings
cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/callbacks/scalar_arguments.tsr
```

The [scalar argument example](../../crates/tsrust-compiler/tests/fixtures/callbacks/scalar_arguments.tsr) prints `7` and `0`, exercising both branches through a callback and a direct call. No dependency remains open in this gate.
