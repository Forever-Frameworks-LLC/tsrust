# Milestone 03: closure captures and escaping lifetimes

[Ownership workstream](../ownership-and-memory.md#core-42) · [Branch rules](./02-control-flow.md) · [Lowering experiment](../experiments/closures-01.md)

**Status:** Gates 03a, 03b, and 03c are complete on September 30, 2026. The source compiler implements the bounded subset below; its rules and syntax remain provisional. No upcoming feature blocks this milestone.

**Subsequent extension:** [Milestone 06](./06-closure-arguments.md) adds typed integer/boolean closure arguments. The zero-argument scope below records this milestone’s original gate. [Milestone 08](./08-nested-local-closures.md) subsequently adds local nested creation and invocation; [milestone 09](./09-callable-captures.md) adds callable captures, [milestone 10](./10-callable-results.md) adds callable-valued arrow results, and [milestone 11](./11-retained-callbacks.md) adds owned retained callbacks.

## Outcome

Allow a closure to read, mutate, or consume captured values without source borrow annotations. Permit a returned closure to own its environment safely. Keep ownership transfers, repeated-call restrictions, and explicit cloning inspectable through source diagnostics and compiler metadata.

The important distinction is between **capture storage** and **invocation access**. Moving a record into a returned closure does not necessarily make the closure single-use. A closure that owns a record and only reads it can be called repeatedly. A closure that returns that record by ownership can be called only once.

## First source subset

Use zero-argument arrow functions with block bodies and explicit result types:

```ts
const show = (): void => { console.log(user.name); };
const get = (): User => { return user; };
```

Local closure bindings use inferred concrete types. A factory can declare a callable result, for example `function make(user: User): () => string`. The callable syntax describes arguments and result; it does not erase the inferred invocation contract or environment identity. Call sites in this single-file experiment retain that information.

Support direct calls by binding name, captures of existing strings, records, integers, and booleans, and the existing branch/return subset inside bodies. Captures refer to the whole binding, including when the body mentions only one field. No partial-record capture is inferred.

This milestone initially rejects closure parameters and passing closures as arguments; [milestone 04](./04-synchronous-callbacks.md) subsequently adds synchronous reusable callback parameters. Still reject closure-valued fields, reassignment or aliasing of closure bindings, capture of other closures, recursive closures, nested closure creation inside closure bodies, borrowed results, and async. Each factory has one closure-expression origin; returning different closure expressions from different paths is deferred. Do not insert boxing or a runtime variant type merely to unify them.

## Capture storage rules

| Captured value and use | Local closure | Returned closure |
| --- | --- | --- |
| Integer or boolean | Copy at creation | Copy at creation |
| Owned value only read by the body | Borrow for the closure's block scope | Move into the environment at creation |
| Owned value mutated by the body | Exclusive borrow for the closure's block scope | Move into the environment at creation |
| Owned value consumed by any body path | Move at creation | Move at creation |

These are choices for this experiment, not Rust's source-language capture inference rules. Determine whether the closure escapes before applying creation effects. Returning a named closure upgrades its owned captures to ownership at its original creation site; it must not first borrow and later silently change into a move. A closure returned on any path is classified as escaping on all paths for that closure origin.

Creation applies effects immediately, even if the closure is never called. Moving a captured record makes the original unavailable. Copying a scalar leaves the original available. A read loan permits other reads; a mutable loan excludes other access. An outstanding incompatible loan prevents capture creation, and an outstanding capture loan prevents a conflicting ordinary access.

Local capture loans last until the closing brace of the block containing the closure binding, including after its last call. This conservative first rule avoids adding last-use analysis in the same change. It intentionally rejects some programs Rust accepts. A nested branch block can delimit the loan; there is no new `drop` source primitive. Loans also end when execution returns from that scope. Consuming a closure early does not shorten these prototype loan bounds.

No capture is repaired by implicit cloning. To retain an owned original when a closure escapes, explicitly clone into a separate binding and capture that binding. `make(user)` can then remain a read of the caller's record if `make` only uses it to create that explicit clone.

## Invocation and enclosing-function contracts

| Invocation contract | Body behavior | Call effect |
| --- | --- | --- |
| `read` | Reads or copies captures | Reusable with read access to the closure |
| `mutate` | Mutates captures without consuming them | Reusable with exclusive access to the closure |
| `consume` | Moves an owned capture on any path | Consumes the closure; a second call is invalid |

These contracts are compiler metadata, not source keywords. Join requirements across body branches as in milestone 02. Scalar captures remain copies. Record/string captures remain owned or borrowed according to the separate storage classification.

Enclosing-function contracts must include capture creation effects. Returning a read-only closure that owns a parameter makes that parameter `consume`. Returning a closure over a separately cloned record can keep the original parameter `read`. A local mutable closure makes the enclosing parameter `mutate`, even when its source binding uses `const`.

Closure invocation analyzes against its captured environment, not the now-transferred outer binding. A body may use its owned capture normally until it transfers that capture. Mutating a moved capture requires mutable storage in the generated environment. Ordinary source `const` continues to prevent rebinding, not mutation through exclusive access.

## Escape and lifetime boundary

Returned environments contain only owned values and scalar copies in this slice. Borrowed environments cannot escape. There is no public borrowed-closure return contract yet, and no implicit reference counting or tracing GC.

The factory's result must retain its invocation contract. An owned read closure may be reused after its factory returns; a returned consuming closure may be invoked once. A factory must not claim to borrow a parameter when its returned environment actually owns it.

This avoids returning references to destroyed locals. It does not solve callbacks retained by libraries, UI event handlers, async tasks, shared captures, cancellation, or cross-thread use. Those require separate escape and public-interface contracts.

## Analysis and lowering

1. Parse callable result types and arrow bodies, resolving lexical names to binding IDs. Preserve declaration, capture-use, call, and return spans.
2. Determine the single closure origin behind each callable binding/result. Classify local versus returned closure before applying ownership effects. Reject unsupported escape routes explicitly.
3. Infer body access to each captured binding and the invocation contract. Propagate enclosing-function capture effects in the existing acyclic call graph.
4. Apply creation effects to the enclosing flow state. Retain capture loans in the enclosing scope. Check each body in a separate environment context, then validate its captures against outer availability and loans; outside aliases receive no exemption.
5. At calls, check closure availability and invocation access. Merge transferred closure/value states across continuing branches. Remove scope-owned loans on block exit and early return.
6. Emit concrete closure environments with static calls. Return a single concrete environment per factory; avoid mandatory heap allocation and dynamic dispatch for closure storage. Existing string construction, transforms, and explicit cloning still have their ordinary costs.

For Rust witnesses, read/mutate/consume invocation maps to `Fn`/`FnMut`/`FnOnce`; ownership capture and invocation are separate. The Rust Reference explicitly distinguishes these concepts. [Rust closure call traits](https://doc.rust-lang.org/reference/types/closure.html#call-traits-and-coercions).

The witnesses use unboxed `impl Fn...` returns. The clone factory uses `use<>` to avoid an unnecessary input lifetime in its opaque return type under edition 2024. The compiler uses concrete environment structs instead, preserving independence because returned environment fields contain no borrowed captures. [Rust opaque return types and precise captures](https://doc.rust-lang.org/reference/types/impl-trait.html#precise-capturing).

## Work packages and exit gates

**03a — scope and lowering witnesses: complete.** Fifteen source cases and sixteen handwritten Rust witnesses cover eight accepted source programs, seven rejected source programs, and one backend-only dangling-borrow control. Source and backend outcomes remain distinct, including Rust accepting the stricter source lexical-loan rejection.

**03b — local closure compiler: complete.** Arrow parsing, binding resolution, capture metadata, lexical loans, invocation contracts, source diagnostics, and concrete Rust environments are implemented. Semantic acceptance tests replace the unsupported-source guard. Tests include conditional consumption, uncalled closures, transitive calls, mixed captures, nested scopes, early returns, and use after a consuming invocation. Earlier branch/integer/ownership tests pass.

**03c — returned closure compiler: complete.** Callable results, single-origin escape classification, concrete owned environments, enclosing parameter contracts, and repeated versus consuming calls after return are implemented. Tests cover named escapes classified at creation, forwarding factories, retained mutable state, explicit-clone independence, mixed-origin rejection, and capture conflicts when an existing loan prevents ownership escape. The invalid borrowed Rust lowering remains an independent backend control.

## Completion evidence

The [closure compiler suite](../../crates/tsrust-compiler/tests/closures.rs) passes 14 test groups. Every accepted original source case compiles through generated Rust and matches exact native output; rejected cases produce the expected source error and related location. Additional tests cover both local and returned gates. `check` succeeds or rejects with an empty `PATH`, proving that analysis does not depend on Rust invocation.

At completion of milestone 03, the complete Cargo suite passed 64 test groups, including independent handwritten witnesses and all earlier milestones. Formatting and Clippy also pass. `Analysis::closures()`, text `check`, and the JSON `closures` array expose creation spans, escape classification, invocation contracts, and each capture's storage/access modes.

Run from the repository root:

```sh
cargo test --offline
cargo fmt --all --check
cargo clippy --offline --all-targets -- -D warnings
cargo run --offline --bin tsrust -- check crates/tsrust-compiler/tests/fixtures/closures/returned_clone_independent.tsr --json
cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/closures/returned_clone_independent.tsr
```

The last command prints `Pat` and `Sam`: the returned closure owns the explicit clone while the original remains independently mutable. Performance measurements and richer callback APIs remain separate work, not dependencies of these completed gates.
