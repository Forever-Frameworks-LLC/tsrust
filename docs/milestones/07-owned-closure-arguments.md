# Milestone 07: owned closure arguments and access contracts

[Scalar closure arguments](./06-closure-arguments.md) · [Roadmap](../roadmap.md)

**Status:** All five exit gates are complete on September 30, 2026. Compiler behavior, acceptance evidence, diagnostics, and documentation implement the bounded provisional rules below.

**Subsequent extension:** [Milestone 08](./08-nested-local-closures.md) adds local nested closure creation and invocation. The exclusions below describe this milestone’s original scope.

## Bounded outcome

Allow string and nominal record arguments in closure bodies. Infer their read, mutate, or consume access exactly as for ordinary function parameters. Callable annotations expose that access explicitly:

```ts
function apply(action: (user: mutate User) => void, user: User) {
    action(user);
}
const rename = (user: User): void => { user.name = "Sam"; };
```

Owned parameters in callback and factory-result signatures require a contextual `read`, `mutate`, or `consume` modifier. Scalars retain implicit copy access and reject access modifiers. Arrow body parameters retain inference and reject explicit modifiers. Access words remain usable as ordinary identifiers and nominal type names.

Signatures match arity, ordered nominal types, and exact access modes. There is no access weakening, implicit adapter, or ownership-repair clone. Parameter names do not affect compatibility. An unused owned closure parameter infers `read`.

## Ownership rules

- A read argument borrows; a mutable argument borrows exclusively; a consuming argument transfers ownership. Scalar arguments copy.
- Receiver access begins before left-to-right argument evaluation. Borrowed arguments remain active through later argument evaluation and invocation. A fully computed owned result does not retain its input loans.
- Consuming a fresh argument does not consume the closure environment. A reusable closure can consume a different owned argument on each call. Consuming a capture still requires the single-use environment contract.
- Enclosing function contracts include callback argument requirements, including requirements propagated through forwarding and both branch arms.
- Owned results remain required. The gate adds no borrowed returns, owned field moves, callback-valued arguments, nested closures, retained callbacks, or async.
- Existing named capture block lifetimes and inline capture call lifetimes remain in force. Argument access cannot bypass capture loans.

## Lowering and inspection

Concrete methods use the inferred Rust reference/value parameter forms. Callback trait bounds quantify borrowed argument lifetimes independently of the closure environment and of each other. This permits repeated calls with different temporary inputs and prevents an input lifetime from becoming part of an owned result. Generic dispatch remains static.

Closure metadata exposes each argument's name and inferred access in the library, JSON, and text checks, separately from captures and environment invocation access.

## Exit gates

1. Parse contextual access annotations while preserving existing scalar syntax and identifiers.
2. Infer owned closure argument access; validate exact callback/factory contracts and result types.
3. Enforce transfers, whole-binding loans, receiver/argument ordering, capture conflicts, and branch joins.
4. Compile generated Rust for local, returned, reusable, single-use, inline, forwarded, and mixed-access argument lists, with exact native output.
5. Verify metadata and backend-independent diagnostics; pass all regressions, formatting, Clippy, and documentation checks.

No future language feature blocks this gate.

## Completion evidence

The [owned argument suite](../../crates/tsrust-compiler/tests/owned_closure_arguments.rs) passes 14 test groups. Accepted cases compile emitted Rust with the pinned backend and compare exact native output for direct and higher-order read, mutate, and consume arguments. Cases cover borrowed captures, owned factory environments, multiple independent argument lifetimes, mixed scalar/owned arguments, repeated fresh-argument consumption, forwarding, and both branch outcomes.

Rejected cases compare library and CLI diagnostics for every incompatible pair of access modes, nominal type mismatches, missing/invalid annotations, ownership transfer, conflicting argument loans, receiver overlap, capture conflicts, and unsupported escape routes. Related spans identify conflicting access or prior transfer. Tests verify argument access through the library, JSON, and text checks. Empty-`PATH` checking/emission succeeds, while invalid builds report source errors before backend lookup.

At completion of milestone 07, all 114 test groups passed, including the earlier scalar, callback, closure, branch, integer, ownership, and independent witness suites. Formatting and Clippy pass. Generic Rust bounds and native-output checks validate the implemented lifetime lowering. No boxing, dynamic dispatch, reference counting, or ownership-repair clones are inserted. This is bounded acceptance evidence, not a general soundness proof or an allocation benchmark.

Run from the repository root:

```sh
cargo test --offline
cargo fmt --all --check
cargo clippy --offline --all-targets -- -D warnings
cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/callbacks/owned_arguments.tsr
```

The [owned argument example](../../crates/tsrust-compiler/tests/fixtures/callbacks/owned_arguments.tsr) prints `Sam`, `Sam`, and `Jo`. A callback mutates a borrowed record, then a reusable closure consumes and returns two separate records. No dependency remains open in this gate.
