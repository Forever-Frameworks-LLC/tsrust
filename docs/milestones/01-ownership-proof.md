# Milestone 01: ownership proof

[Roadmap](../roadmap.md) · [Ownership workstream](../guides/language/ownership-and-memory.md) · [Experiment](../experiments/ownership-01.md)

**Status:** Complete for the bounded subset, including the [integer amendment](./01-integer-subset.md). The experimental rules below are provisional and restricted to this milestone. They do not settle the full language design or satisfy the complete research program.

**Progress:** Gates 1–6 are complete for the bounded subset. The [Rust compiler](../../crates/tsrust-compiler/README.md) compiles and runs all eight accepted original source cases and the original numeric `User` example. All three invalid original source cases receive diagnostics before Rust invocation. The bounded evidence review is recorded below. This does not complete the broader core-proof research or imply general inference correctness.

## Question and bounded outcome

Can a small TypeScript-shaped program express read, mutate, and consume behavior without writing borrow syntax, while rejecting conflicting access and preserving explicit cloning?

The first deliverable is a precise subset with acceptance examples and safe Rust lowering witnesses. The subsequent compiler deliverable must parse those examples, derive their parameter access contracts, report invalid programs against source locations, generate Rust, and execute the accepted programs.

## Experimental priorities

For this experiment, memory safety, no tracing GC, and no inserted clones to repair ownership errors are hard constraints. Favor familiar syntax and readable diagnostics within those constraints. This gives the experiment a working rule while the project-wide priority conflict remains open.

The following scope records milestone 01. [Milestone 02](./02-control-flow.md) extends it with booleans, branches, block scope, and early returns.

## Supported subset

- One source file, named record types, owned string and copied integer fields, and explicit type annotations on parameters.
- Top-level named functions, direct calls, and an acyclic call graph. Declarations may appear in any order.
- Straight-line blocks with local `const` bindings, record literals, field replacement, and a final return.
- Owned record/string or copied integer return values; functions without a return produce no value.
- Reading string fields through `console.log`, `trim`, and `toLowerCase`; logging integer values.
- Whole-record moves and `clone(record)` for the experiment's records of strings and integers.
- Signed decimal `int` literals, locals, parameters, fields, and returns under the [integer amendment](./01-integer-subset.md).

The first compiler accepts this subset only. Unsupported constructs must produce a source diagnostic rather than falling through to arbitrary Rust or silently adopting another memory model. The original `id: int` example now runs under the integer amendment. The original handwritten witnesses still isolate ownership with a string-only record.

## Ownership and access rules

| Operation | Experimental meaning |
| --- | --- |
| Record literal or string literal | Produces a new owned value. |
| `const second = first` for an owned value | Transfers ownership. Later reads of `first` are rejected. There is no inferred-borrow exception for assignment. |
| `clone(record)` | Reads the input and returns an independent owned record, including independent owned strings. |
| Read-only parameter use | The call temporarily grants read access; the caller retains ownership. |
| Field replacement through a parameter | The call temporarily grants exclusive mutable access; the caller retains ownership. |
| Return or owned assignment of a parameter | The parameter consumes the caller's owned value. |
| Call to another function | Propagates that callee's parameter access requirement. |
| Repeated borrowed arguments in a single call | Multiple read accesses are allowed. Repeated binding access involving mutate or consume is rejected while an earlier loan is active. |
| Integer binding or field used as a value | Copies the integer and retains the source. A completed copy retains no argument loan. |

`const` prevents rebinding the source variable; it does not make the record's fields immutable. The emitter may use a Rust `mut` binding when field replacement requires it. This choice preserves the vision's ordinary record-mutation examples. Signal assignment is outside this experiment.

Access spans the call and ends on return. Borrowed values cannot escape through return values, storage, or closures in this subset. A return must produce an owned value or a copied integer. Returning a borrowed string field directly is unsupported; returning an independently usable string transform is supported.

Each binding is available or transferred. An owned assignment or consuming call marks the source binding transferred. Every later use must check that state, including uses inside the callee body. Choosing a consuming parameter contract does not make a use-after-transfer inside that function valid.

## Parameter contract inference

Resolve all function names first. Analyze the acyclic call graph from callees to callers, independent of source declaration order. Record a per-parameter access requirement:

- `copy`: an integer parameter passed by value; separate from the ordering of owned access requirements.
- `read`: used without changing or transferring the value; also the conservative default for an unused owned parameter in this experiment.
- `mutate`: needs exclusive temporary access, directly or through a call.
- `consume`: transfers ownership, directly or through a call.

For owned parameters, combine requirements with `consume` stronger than `mutate`, and `mutate` stronger than `read`. Separately record whether the function mutates a consuming parameter, so the Rust binding can be mutable. Validate statement order and binding availability after computing the contracts.

These names are compiler metadata, not proposed source keywords. General higher-order inference, recursive constraints, and borrowed-result relationships are deliberately excluded.

The compiler prints an inspectable contract summary through `check` and `check --json`. In a later module system, exported contracts must be part of the checked public interface; changing a parameter from read to consume cannot silently preserve the same API contract. The representation, versioning policy, and optional source annotations for that interface remain research work.

## String and output behavior

For the experiment, `trim()` and `toLowerCase()` read their inputs and return independently usable string values. The original lowering and handwritten witnesses materialize owned buffers and may allocate intermediates. The later [primitive optimization](../measurements/primitive-performance.md) preserves source ownership while sharing immutable static literal bytes and precomputed normalization results. Dynamic owned inputs still produce independent buffers; allocation is a representation choice, not a promise of the source API.

The original ownership fixtures validate ASCII string transforms. Primitive storage regressions additionally preserve Rust Unicode trim/lower behavior, including expanding case mappings; this does not establish full JavaScript Unicode compatibility. Literal escaping and UTF-8 diagnostic positions have separate test coverage. [Milestone 33](./33-native-utf8-strings.md) now uses native Rust UTF-8 byte length/slicing and scalar charAt, with Rust whitespace. Case conversion keeps the pinned Unicode 17 tables; complete contextual conversion equivalence across Unicode versions and bracket indexing remain separately scoped. `console.log(string)` reads the value and appends a newline.

## Lowering obligations

- Read parameters lower to immutable references, mutate parameters to mutable references, and consume parameters to owned Rust values. Integer copy parameters lower to `i64` values.
- Preserve source statement order and the contract of each call.
- Insert no `clone()` to repair an ownership error. Lower explicit source cloning directly.
- Use source diagnostics for invalid source programs before Rust compilation. Rust rejection remains a defensive backend check.
- Preserve source ranges for both the transfer/conflicting argument and the later invalid use.

The experiment uses no reference counting, locks, unsafe blocks, or external runtime. That describes the current witnesses, not a policy ruling out those mechanisms for future workstreams.

## Acceptance corpus

The [typed case inventory](../../crates/tsrust-compiler/tests/support/corpus.rs) contains 11 cases: eight accepted programs and three rejected programs. The `.tsr` files in `crates/tsrust-compiler/tests/fixtures/ownership/` are compiler acceptance inputs; paired `.rs` files remain handwritten reference lowerings.

For source-level rejection, use diagnostics equivalent to:

- **Transferred value:** “`user` is no longer available because its value was transferred.” Show the transfer and invalid use.
- **Conflicting access:** “This call cannot change `user` while another argument reads it.” Show both arguments.

The words may evolve; the cause, consequence, and source locations are required. Do not suggest `Shared(user)` until the shared-handle semantics are defined.

## Completion gates

1. **Groundwork:** Define the subset and accepted/rejected source examples.
2. **Lowering feasibility:** Compile and execute handwritten Rust witnesses; verify expected backend rejections.
3. **Frontend:** Parse the supported source, resolve names, and reject type/unsupported-syntax errors with source ranges.
4. **Ownership inference:** Derive the expected contracts and reject the three invalid source programs before invoking Rust.
5. **Native compiler:** Generate Rust for all accepted cases and match expected output. Include the original numeric `User` example once the integer subset is defined.
6. **Evidence review:** Record what the compiler proves and what it cannot yet handle. Scope closures, control flow, and async as separate follow-up experiments.

Passing gate 2 does not imply gates 3–5 are complete. No performance or general inference claim follows from this corpus alone.

## Bounded evidence review

On September 30, 2026, the source compiler suite passed 17 test groups using `rustc 1.96.1 (31fca3adb 2026-06-26)`. These include the 11 original cases and additional checks for invalid types, names, syntax, returns, transitive contracts, nested alias conflicts, literal escaping, and CLI behavior.

The accepted corpus is compiled from `.tsr` inputs into generated Rust and native executables. Parameter contracts are compared with the typed case inventory. The three rejected cases produce source diagnostics with both relevant locations. A subprocess check removes Rust from `PATH` to establish that frontend success/rejection does not rely on rustc. No compiler-inserted ownership-repair clones are present in the accepted corpus.

This supports the feasibility of inferred argument access for direct acyclic calls and straight-line owned values. It does not establish general inference soundness, control-flow behavior, closure capture semantics, allocation cost, or competitive performance.

The compiler and verification suites are Rust, organized under `crates/tsrust-compiler/`. The 17 original acceptance groups are preserved. With backend pinning checks and the integer amendment, the Cargo suite reached 37 groups at completion of this milestone: 19 original acceptance, eight integer acceptance, eight analysis/pin, and two witness/inventory. Original corpus assertions use one typed case inventory. Rust 1.96.1 is [pinned for both compilation stages](../guides/tooling/rust-toolchain-policy.md). The future language server will share the Rust analysis library. See the [analysis performance baseline](../measurements/analysis-performance.md) for a bounded measurement, not a claim of production readiness.

The [integer acceptance evidence](./01-integer-subset.md#acceptance-evidence) completes the original numeric example and verifies copy behavior and field access. [Milestone 02](./02-control-flow.md) now implements the scoped branch experiment and brings the suite to 48 groups. Next scope closure captures and escaping lifetimes with new accepted/rejected examples before implementation. Async follows once those lifetime contracts are understood.

## Deferred work

Shared/weak ownership, graph cycles, field moves, borrowed returns, loops, branches, recursion, generics, interfaces, classes, arrays, arithmetic and broader numeric semantics, closures, async, modules, npm, HTTP, JSX, and desktop implementation remain outside the current witness experiment. Research on their constraints can continue without expanding this first implementation slice.
